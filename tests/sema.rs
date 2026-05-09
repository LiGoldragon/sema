//! Integration tests for the sema record store. Live in
//! `tests/` per the rust style rule that tests exercise the
//! public API rather than internal items.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use sema::{Sema, Slot};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!("sema_test_{}_{}.redb", std::process::id(), counter));
    let _ = std::fs::remove_file(&path);
    path
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
fn first_slot_is_zero() {
    let temp = fresh();
    let slot = temp.sema.store(b"first").unwrap();
    assert_eq!(slot, Slot::from(0u64));
}

#[test]
fn slots_are_monotone() {
    let temp = fresh();
    let slot_1 = temp.sema.store(b"a").unwrap();
    let slot_2 = temp.sema.store(b"b").unwrap();
    let slot_3 = temp.sema.store(b"c").unwrap();
    assert_eq!(u64::from(slot_1) + 1, u64::from(slot_2));
    assert_eq!(u64::from(slot_2) + 1, u64::from(slot_3));
}

#[test]
fn get_returns_stored_bytes() {
    let temp = fresh();
    let slot = temp.sema.store(b"hello world").unwrap();
    assert_eq!(temp.sema.get(slot).unwrap(), Some(b"hello world".to_vec()));
}

#[test]
fn get_missing_slot_returns_none() {
    let temp = fresh();
    assert_eq!(temp.sema.get(Slot::from(999_999u64)).unwrap(), None);
}

#[test]
fn empty_record_bytes_are_stored_and_retrieved() {
    let temp = fresh();
    let slot = temp.sema.store(b"").unwrap();
    assert_eq!(temp.sema.get(slot).unwrap(), Some(Vec::<u8>::new()));
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
    let slot = sema.store(b"c").unwrap();
    assert_eq!(slot, Slot::from(2u64));
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

#[test]
fn iter_returns_empty_for_fresh_store() {
    let temp = fresh();
    let all = temp.sema.iter().unwrap();
    assert!(all.is_empty());
}

#[test]
fn iter_yields_every_record_in_slot_order() {
    let temp = fresh();
    let slot_1 = temp.sema.store(b"first").unwrap();
    let slot_2 = temp.sema.store(b"second").unwrap();
    let slot_3 = temp.sema.store(b"third").unwrap();
    let all = temp.sema.iter().unwrap();
    assert_eq!(
        all,
        vec![
            (slot_1, b"first".to_vec()),
            (slot_2, b"second".to_vec()),
            (slot_3, b"third".to_vec()),
        ]
    );
}

#[test]
fn iter_survives_across_reopens() {
    let path = temp_path();
    {
        let sema = Sema::open(&path).unwrap();
        let _ = sema.store(b"persists").unwrap();
    }
    let sema = Sema::open(&path).unwrap();
    let all = sema.iter().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].1, b"persists".to_vec());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn reader_count_defaults_when_unset() {
    let path = temp_path();
    let sema = Sema::open(&path).unwrap();
    assert_eq!(sema.reader_count().unwrap(), sema::DEFAULT_READER_COUNT);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn reader_count_persists_across_reopens() {
    let path = temp_path();
    {
        let sema = Sema::open(&path).unwrap();
        sema.set_reader_count(8).unwrap();
        assert_eq!(sema.reader_count().unwrap(), 8);
    }
    let sema = Sema::open(&path).unwrap();
    assert_eq!(sema.reader_count().unwrap(), 8);
    let _ = std::fs::remove_file(&path);
}
