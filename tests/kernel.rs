//! Kernel-mode tests — exercise the Schema / Table&lt;K, V&gt; /
//! version-guard surface introduced for `<consumer>-sema` crates.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rkyv::{Archive, Deserialize, Serialize};
use sema::{Error, Schema, SchemaVersion, Sema, Table};

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
    path.push(format!(
        "sema_kernel_test_{}_{}.redb",
        std::process::id(),
        counter
    ));
    let _ = std::fs::remove_file(&path);
    path
}

// A small typed record to round-trip through Table<K, V>.
#[derive(Archive, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[rkyv(derive(Debug))]
struct ToyRecord {
    name: String,
    value: u32,
}

const SCHEMA_V1: Schema = Schema {
    version: SchemaVersion::new(1),
};

const SCHEMA_V2: Schema = Schema {
    version: SchemaVersion::new(2),
};

const TOYS: Table<&str, ToyRecord> = Table::new("toys");
const KEYED_BY_U64: Table<u64, ToyRecord> = Table::new("keyed_by_u64");

#[test]
fn open_with_schema_writes_version_on_first_open() {
    let path = temp_path();
    {
        let _sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    }
    // re-open with same schema — should succeed
    let _sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    let _ = std::fs::remove_file(&path);
}

#[test]
fn open_with_mismatched_schema_version_hard_fails() {
    let path = temp_path();
    {
        let _sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    }
    let result = Sema::open_with_schema(&path, &SCHEMA_V2);
    match result {
        Err(Error::SchemaVersionMismatch { expected, found }) => {
            assert_eq!(expected, SchemaVersion::new(2));
            assert_eq!(found, SchemaVersion::new(1));
        }
        Err(other) => panic!("expected SchemaVersionMismatch, got {other:?}"),
        Ok(_) => panic!("expected SchemaVersionMismatch, got Ok(...)"),
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn open_with_schema_refuses_legacy_file_lacking_schema_version() {
    // The version-skew guard MUST refuse to retro-stamp an
    // existing file that was created in legacy mode (no
    // schema_version stored). Silent acceptance was the bug
    // designer/66 §1.5 (Issue A) named.
    let path = temp_path();
    {
        let legacy = Sema::open(&path).unwrap();
        let _ = legacy.store(b"legacy bytes").unwrap();
    }
    let result = Sema::open_with_schema(&path, &SCHEMA_V1);
    match result {
        Err(Error::LegacyFileLacksSchema { expected, .. }) => {
            assert_eq!(expected, SchemaVersion::new(1));
        }
        Err(other) => panic!("expected LegacyFileLacksSchema, got {other:?}"),
        Ok(_) => panic!("expected LegacyFileLacksSchema, got Ok(...)"),
    }
    let _ = std::fs::remove_file(&path);
}

#[test]
fn typed_tables_can_use_arbitrary_key_types() {
    // Issue L: pre-creating tables with hard-coded K=&str
    // locked out tables with other key types. After dropping
    // the pre-create step, a Table<u64, V> works alongside a
    // Table<&str, V> in the same file.
    let path = temp_path();
    let sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    let toy = ToyRecord {
        name: "u64-keyed".to_string(),
        value: 100,
    };
    sema.write(|txn| KEYED_BY_U64.insert(txn, 42u64, &toy))
        .unwrap();
    let read_back = sema
        .read(|txn| KEYED_BY_U64.get(txn, 42u64))
        .unwrap()
        .expect("u64-keyed value present");
    assert_eq!(read_back, toy);
    // and the &str-keyed table still works too
    let other = ToyRecord {
        name: "str-keyed".to_string(),
        value: 200,
    };
    sema.write(|txn| TOYS.insert(txn, "k", &other)).unwrap();
    assert_eq!(sema.read(|txn| TOYS.get(txn, "k")).unwrap().unwrap(), other);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn typed_table_round_trips_value() {
    let path = temp_path();
    let sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    let original = ToyRecord {
        name: "first".to_string(),
        value: 42,
    };
    sema.write(|txn| TOYS.insert(txn, "k1", &original)).unwrap();
    let read_back = sema
        .read(|txn| TOYS.get(txn, "k1"))
        .unwrap()
        .expect("value present");
    assert_eq!(read_back, original);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn typed_table_get_returns_none_for_missing_key() {
    let path = temp_path();
    let sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    let result = sema.read(|txn| TOYS.get(txn, "nonexistent")).unwrap();
    assert!(result.is_none());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn typed_table_remove_works() {
    let path = temp_path();
    let sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    let toy = ToyRecord {
        name: "doomed".to_string(),
        value: 1,
    };
    sema.write(|txn| TOYS.insert(txn, "k", &toy)).unwrap();
    let removed = sema.write(|txn| TOYS.remove(txn, "k")).unwrap();
    assert!(removed);
    let after = sema.read(|txn| TOYS.get(txn, "k")).unwrap();
    assert!(after.is_none());
    let _ = std::fs::remove_file(&path);
}

#[test]
fn write_closure_rolls_back_on_error() {
    let path = temp_path();
    let sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    let toy = ToyRecord {
        name: "ghost".to_string(),
        value: 7,
    };
    let result = sema.write(|txn| -> sema::Result<()> {
        TOYS.insert(txn, "k", &toy)?;
        Err(Error::MissingSlotCounter)
    });
    assert!(matches!(result, Err(Error::MissingSlotCounter)));
    // The insert should have been rolled back when the txn was dropped without commit.
    let after = sema.read(|txn| TOYS.get(txn, "k")).unwrap();
    assert!(after.is_none(), "rolled-back insert should not persist");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn legacy_slot_store_coexists_with_typed_tables() {
    let path = temp_path();
    let sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    // typed table use
    let toy = ToyRecord {
        name: "mix".to_string(),
        value: 99,
    };
    sema.write(|txn| TOYS.insert(txn, "k", &toy)).unwrap();
    // legacy slot use
    let slot = sema.store(b"raw bytes").unwrap();
    // both readable
    let typed = sema.read(|txn| TOYS.get(txn, "k")).unwrap().unwrap();
    assert_eq!(typed, toy);
    assert_eq!(sema.get(slot).unwrap(), Some(b"raw bytes".to_vec()));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn schema_path_creates_parent_directories() {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "sema_kernel_subdir_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    path.push("nested");
    path.push("subdir");
    path.push("kernel.redb");
    let sema = Sema::open_with_schema(&path, &SCHEMA_V1).unwrap();
    assert_eq!(sema.path(), path);
    let _ = std::fs::remove_dir_all(path.parent().unwrap().parent().unwrap().parent().unwrap());
}
