//! sema — the record store.
//!
//! redb-backed slot → bytes table plus a meta table holding the
//! slot-allocation counter. Records arrive as opaque rkyv-encoded
//! bytes from criome (sema doesn't depend on signal — the type
//! discipline lives one level up); sema just allocates a slot and
//! persists.
//!
//! Slot allocation is monotone — the counter starts at 0 on first
//! open, hands out 0, 1, 2, ... in assert order, and is persisted
//! in the `meta` table across reopens. Genesis records are simply
//! the records asserted first; they get the lowest slot values by
//! virtue of order, not by reservation.
//!
//! Scope: store + get + iter + slot allocation. The full sema
//! design (per-kind tables, change-log, SlotBinding, bitemporal
//! queries) lands as kinds beyond Node/Edge/Graph come online.

use std::path::Path;

use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use thiserror::Error;

/// Slot identity — a u64 newtype matching `signal::Slot` semantics
/// at the type-system layer (sema and signal are independent;
/// criome bridges the two `Slot` types). Construct via
/// [`Slot::from(value)`]; read out via `let value: u64 = slot.into()`.
/// The wrapped field is private to keep callers honest about going
/// through the conversion traits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slot(u64);

impl From<u64> for Slot {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<Slot> for u64 {
    fn from(slot: Slot) -> u64 {
        slot.0
    }
}

const RECORDS: TableDefinition<u64, &[u8]> = TableDefinition::new("records");
const META: TableDefinition<&str, u64> = TableDefinition::new("meta");
const NEXT_SLOT_KEY: &str = "next_slot";
const READER_COUNT_KEY: &str = "reader_count";

/// Default size of the read-pool when the `reader_count` meta
/// entry has never been set. criome's `Engine` actor spawns
/// this many `Reader` actors at startup.
pub const DEFAULT_READER_COUNT: u32 = 4;

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
    database: Database,
}

impl Sema {
    /// Open or create a sema database at `path`. Initialises the
    /// slot counter to 0 on first open; subsequent opens preserve
    /// whatever counter value is on disk.
    pub fn open(path: &Path) -> Result<Self> {
        let database = Database::create(path)?;
        let transaction = database.begin_write()?;
        {
            let mut meta = transaction.open_table(META)?;
            if meta.get(NEXT_SLOT_KEY)?.is_none() {
                meta.insert(NEXT_SLOT_KEY, 0u64)?;
            }
            // Touch records table to ensure it exists.
            let _ = transaction.open_table(RECORDS)?;
        }
        transaction.commit()?;
        Ok(Sema { database })
    }

    /// Allocate the next slot, persist `record_bytes` at that
    /// slot, return the assigned slot.
    pub fn store(&self, record_bytes: &[u8]) -> Result<Slot> {
        let transaction = self.database.begin_write()?;
        let slot_value;
        {
            let mut meta = transaction.open_table(META)?;
            slot_value = meta
                .get(NEXT_SLOT_KEY)?
                .ok_or(Error::MissingSlotCounter)?
                .value();
            meta.insert(NEXT_SLOT_KEY, slot_value + 1)?;

            let mut records = transaction.open_table(RECORDS)?;
            records.insert(slot_value, record_bytes)?;
        }
        transaction.commit()?;
        Ok(Slot::from(slot_value))
    }

    /// Fetch the record bytes at `slot`, if present.
    pub fn get(&self, slot: Slot) -> Result<Option<Vec<u8>>> {
        let transaction = self.database.begin_read()?;
        let records = transaction.open_table(RECORDS)?;
        match records.get(u64::from(slot))? {
            Some(guard) => Ok(Some(guard.value().to_vec())),
            None => Ok(None),
        }
    }

    /// Snapshot every record in the store as `(Slot, Vec<u8>)`
    /// pairs. Eagerly collected — the redb transaction closes
    /// before the result is returned. Order is by slot value.
    ///
    /// M0 query path: criome calls this to scan-and-try-decode
    /// each record against the requested kind. Per-kind tables
    /// (which would let criome iterate just one kind) are an
    /// M1+ sema concern; for M0 the scan-everything cost is
    /// acceptable at our record volume.
    pub fn iter(&self) -> Result<Vec<(Slot, Vec<u8>)>> {
        let transaction = self.database.begin_read()?;
        let records = transaction.open_table(RECORDS)?;
        let mut all = Vec::new();
        for entry in records.iter()? {
            let (slot_guard, bytes_guard) = entry?;
            all.push((Slot::from(slot_guard.value()), bytes_guard.value().to_vec()));
        }
        Ok(all)
    }

    /// Read the configured size of the criome read-pool. Returns
    /// [`DEFAULT_READER_COUNT`] if the meta entry has never been
    /// set. Intended to be called once at criome-daemon startup
    /// so the `Engine` actor knows how many `Reader` actors to
    /// spawn.
    ///
    /// Lives in the redb meta table for now (instance-level
    /// infrastructure config). M2+ migrates this onto a typed
    /// `CriomeInstance` record when the multi-instance kind
    /// machinery lands.
    pub fn reader_count(&self) -> Result<u32> {
        let transaction = self.database.begin_read()?;
        let meta = transaction.open_table(META)?;
        match meta.get(READER_COUNT_KEY)? {
            Some(guard) => Ok(guard.value() as u32),
            None => Ok(DEFAULT_READER_COUNT),
        }
    }

    /// Persist the read-pool size to the meta table. Takes
    /// effect on the next [`Sema::reader_count`] call (and
    /// therefore the next daemon startup).
    pub fn set_reader_count(&self, count: u32) -> Result<()> {
        let transaction = self.database.begin_write()?;
        {
            let mut meta = transaction.open_table(META)?;
            meta.insert(READER_COUNT_KEY, count as u64)?;
        }
        transaction.commit()?;
        Ok(())
    }
}
