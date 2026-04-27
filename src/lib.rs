//! sema — the record store.
//!
//! redb-backed slot → bytes table plus a meta table holding the
//! slot-allocation counter. Records arrive as opaque rkyv-encoded
//! bytes from criome (sema doesn't depend on signal — the type
//! discipline lives one level up); sema just allocates a slot and
//! persists.
//!
//! Slot allocation is monotone. The seed range `[0, 1024)` is
//! reserved per [criome/ARCHITECTURE.md §10
//! ](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md#10--project-wide-rules)
//! so the counter starts at 1024 on first open. The counter is
//! persisted in the `meta` table and restored across reopens.
//!
//! M0 scope: store + get + slot allocation. The full sema design
//! (per-kind tables, change-log, SlotBinding, bitemporal queries)
//! lands as kinds beyond Node/Edge/Graph come online.

use std::path::Path;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use thiserror::Error;

/// Slot identity — matches `signal::Slot` semantics. Sema doesn't
/// depend on signal directly; criome bridges the types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Slot(pub u64);

const RECORDS: TableDefinition<u64, &[u8]> = TableDefinition::new("records");
const META: TableDefinition<&str, u64> = TableDefinition::new("meta");
const NEXT_SLOT_KEY: &str = "next_slot";

/// First user-allocatable slot. Slots `[0, SEED_RANGE_END)` are
/// reserved for genesis / built-in records (see criome arch §10).
const SEED_RANGE_END: u64 = 1024;

#[derive(Debug, Error)]
pub enum Error {
    #[error("redb database: {0}")]
    Database(#[from] redb::DatabaseError),
    #[error("redb storage: {0}")]
    Storage(#[from] redb::StorageError),
    #[error("redb transaction: {0}")]
    Transaction(#[from] redb::TransactionError),
    #[error("redb table: {0}")]
    Table(#[from] redb::TableError),
    #[error("redb commit: {0}")]
    Commit(#[from] redb::CommitError),
    #[error("meta table missing slot counter — sema file may be corrupt")]
    MissingSlotCounter,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Sema {
    db: Database,
}

impl Sema {
    /// Open or create a sema database at `path`. Initialises the
    /// slot counter to `SEED_RANGE_END` on first open; subsequent
    /// opens preserve whatever counter value is on disk.
    pub fn open(path: &Path) -> Result<Self> {
        let db = Database::create(path)?;
        let txn = db.begin_write()?;
        {
            let mut meta = txn.open_table(META)?;
            if meta.get(NEXT_SLOT_KEY)?.is_none() {
                meta.insert(NEXT_SLOT_KEY, SEED_RANGE_END)?;
            }
            // Touch records table to ensure it exists.
            let _ = txn.open_table(RECORDS)?;
        }
        txn.commit()?;
        Ok(Sema { db })
    }

    /// Allocate the next slot, persist `record_bytes` at that slot,
    /// return the assigned slot.
    pub fn store(&self, record_bytes: &[u8]) -> Result<Slot> {
        let txn = self.db.begin_write()?;
        let slot_value;
        {
            let mut meta = txn.open_table(META)?;
            slot_value = meta
                .get(NEXT_SLOT_KEY)?
                .ok_or(Error::MissingSlotCounter)?
                .value();
            meta.insert(NEXT_SLOT_KEY, slot_value + 1)?;

            let mut records = txn.open_table(RECORDS)?;
            records.insert(slot_value, record_bytes)?;
        }
        txn.commit()?;
        Ok(Slot(slot_value))
    }

    /// Fetch the record bytes at `slot`, if present.
    pub fn get(&self, slot: Slot) -> Result<Option<Vec<u8>>> {
        let txn = self.db.begin_read()?;
        let records = txn.open_table(RECORDS)?;
        match records.get(slot.0)? {
            Some(guard) => Ok(Some(guard.value().to_vec())),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        p.push(format!("sema_test_{}_{}.redb", std::process::id(), n));
        let _ = std::fs::remove_file(&p);
        p
    }

    struct TempSema {
        sema: Sema,
        path: PathBuf,
    }

    impl Drop for TempSema {
        fn drop(&mut self) {
            let _ = std::fs::remove_file(&self.path);
        }
    }

    fn fresh() -> TempSema {
        let path = temp_path();
        let sema = Sema::open(&path).unwrap();
        TempSema { sema, path }
    }

    #[test]
    fn first_slot_is_seed_range_end() {
        let s = fresh();
        let slot = s.sema.store(b"first").unwrap();
        assert_eq!(slot, Slot(SEED_RANGE_END));
    }

    #[test]
    fn slots_are_monotone() {
        let s = fresh();
        let s1 = s.sema.store(b"a").unwrap();
        let s2 = s.sema.store(b"b").unwrap();
        let s3 = s.sema.store(b"c").unwrap();
        assert_eq!(s1.0 + 1, s2.0);
        assert_eq!(s2.0 + 1, s3.0);
    }

    #[test]
    fn get_returns_stored_bytes() {
        let s = fresh();
        let slot = s.sema.store(b"hello world").unwrap();
        assert_eq!(s.sema.get(slot).unwrap(), Some(b"hello world".to_vec()));
    }

    #[test]
    fn get_missing_slot_returns_none() {
        let s = fresh();
        assert_eq!(s.sema.get(Slot(999_999)).unwrap(), None);
    }

    #[test]
    fn empty_record_bytes_are_stored_and_retrieved() {
        let s = fresh();
        let slot = s.sema.store(b"").unwrap();
        assert_eq!(s.sema.get(slot).unwrap(), Some(Vec::<u8>::new()));
    }

    #[test]
    fn slot_counter_persists_across_reopens() {
        let path = temp_path();
        {
            let sema = Sema::open(&path).unwrap();
            let _ = sema.store(b"a").unwrap();
            let _ = sema.store(b"b").unwrap();
        }
        let sema = Sema::open(&path).unwrap();
        let s = sema.store(b"c").unwrap();
        assert_eq!(s, Slot(SEED_RANGE_END + 2));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn records_persist_across_reopens() {
        let path = temp_path();
        let slot;
        {
            let sema = Sema::open(&path).unwrap();
            slot = sema.store(b"durable").unwrap();
        }
        let sema = Sema::open(&path).unwrap();
        assert_eq!(sema.get(slot).unwrap(), Some(b"durable".to_vec()));
        let _ = std::fs::remove_file(&path);
    }
}
