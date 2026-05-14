# ARCHITECTURE — sema

Sema is the workspace's typed storage kernel. It is a Rust
library over redb and rkyv. It opens a database file with an
explicit schema, guards the rkyv database format, and provides
typed table operations inside closure-scoped transactions.

Sema is to state what `signal-core` is to wire: the small kernel of
primitives that higher layers build on. Full database-operation
execution lives in `sema-engine`, not in this crate.

## Role

The kernel owns:

- The redb file lifecycle: create parent directories, open or create
  the file, create sema-owned internal tables.
- The database header guard: the file records Sema's rkyv format
  identity and refuses incompatible builds.
- The schema-version guard: `Sema::open_with_schema(path, schema)`
  writes the schema on first open and hard-fails on mismatch.
- Closure-scoped transactions: `read(|transaction| ...)` and
  `write(|transaction| ...)`.
- Typed tables: `Table<K, V>` hides rkyv encode/decode at the table
  boundary and returns owned rows from scans.
- The crate `Error` enum for kernel failures.

Consumers own:

- Record Rust types, usually in Signal contract crates or component
  domain crates.
- Table layouts and schema constants.
- Runtime ordering, actors, authorization, validation, subscriptions,
  and commit-before-effect policy.
- Database-operation execution through `sema-engine`.

## Boundary

```text
component daemon
  owns actors, policy, validation, subscriptions, sockets
  |
  v
sema-engine
  owns Signal verb execution, query/mutation plans, catalog,
  operation log, snapshots, subscription delivery
  |
  v
sema
  owns redb/rkyv typed table storage, schema guard,
  database-format guard, transaction helpers
  |
  v
component.redb
```

The dependency direction is one-way. `sema-engine` may depend on
`sema`. `sema` must not depend on `sema-engine`, Signal contracts,
Kameo, tokio, NOTA, Nexus, Persona, or Criome.

## Non-Goals

Sema does not own:

- Signal verbs or request routing.
- Query planning, mutation planning, validation, operation logs,
  snapshots, or subscriptions.
- Component actors or mailboxes.
- Runtime configuration for any component.
- Raw untyped record storage.
- Peer inspection or daemon sockets.

The retired raw-byte append path is intentionally absent. If a future
engine needs append-only identity or sequence allocation, that lands
as a typed `sema-engine` primitive with its own records and witnesses,
not as a raw storage surface in `sema`.

## Constraints

- Sema opens durable state only through
  `Sema::open_with_schema(path, schema)`.
- Sema stores typed rkyv values through typed tables.
- Sema has no schema-less public open path.
- Sema has no raw byte store API.
- Sema has no Criome read-pool configuration API.
- Sema internal table names use the `__sema_` prefix.
- Sema-owned internal tables are limited to kernel metadata and
  database headers.
- Component table names must not use the `__sema_` prefix.
- Table layout belongs to the component that owns the state.
- Record Rust types live in Signal contract crates or component
  domain crates, not in sema.
- Runtime ordering, actor mailboxes, commit-before-effect policy, and
  subscriptions belong to the consuming component or to
  `sema-engine`.

## Public Surface

```rust
pub struct Sema;

impl Sema {
    pub fn open_with_schema(path: &Path, schema: &Schema) -> Result<Self>;
    pub fn read<R>(&self, body: impl FnOnce(&ReadTransaction) -> Result<R>) -> Result<R>;
    pub fn write<R>(&self, body: impl FnOnce(&WriteTransaction) -> Result<R>) -> Result<R>;
    pub fn path(&self) -> &Path;
}

pub struct Table<K, V> { ... }

impl<K, V> Table<K, V> {
    pub const fn new(name: &'static str) -> Self;
    pub fn ensure(&self, transaction: &WriteTransaction) -> Result<()>;
    pub fn get(&self, transaction: &ReadTransaction, key: K) -> Result<Option<V>>;
    pub fn insert(&self, transaction: &WriteTransaction, key: K, value: &V) -> Result<()>;
    pub fn remove(&self, transaction: &WriteTransaction, key: K) -> Result<bool>;
    pub fn iter(&self, transaction: &ReadTransaction) -> Result<Vec<(K::Owned, V)>>;
    pub fn range<R>(&self, transaction: &ReadTransaction, range: R) -> Result<Vec<(K::Owned, V)>>;
}
```

## Code Map

```text
src/
└── lib.rs    — Sema handle, schema/database header guards,
                closure-scoped transactions, Table<K, V>,
                OwnedTableKey, Error
```

Internal Sema tables:

- `__sema_headers`
- `__sema_meta`

## Tests

Named Nix surfaces:

```sh
nix run .#test
nix run .#test-kernel-surface
nix run .#test-no-legacy-surface
nix run .#test-doc
nix flake check
```

Load-bearing witnesses:

- `sema_does_not_export_slot`
- `sema_does_not_export_legacy_slot_store`
- `sema_does_not_export_reader_count`
- schema mismatch hard-fails at open
- database format mismatch hard-fails at open
- typed table scans return owned keys and values
- write transactions roll back on typed errors

## Status

Package A of the sema / sema-engine split has **landed**: this crate
is the cleaned storage kernel (no `Slot`, no legacy raw-byte store,
no `reader_count`, no schema-less open). The structural witnesses
for those deletions exist. `sema-engine` has been created as a
sibling library-only repository and is in active development; the
first consumer migration (persona-mind) is in flight on operator
track `[primary-5ir2]`.

Ongoing work for this crate is bounded: respond to engine-side
discoveries that require kernel changes (per ESSENCE §"Backward
compatibility is not a constraint" — `sema` may break to make the
engine substrate beautiful). Most active development lives in
`sema-engine`.

Canonical handoff:

- `~/primary/reports/operator/115-sema-engine-split-implementation-investigation.md`
- `~/primary/reports/designer/158-sema-kernel-and-sema-engine-two-interfaces.md`
- `~/primary/reports/designer/159-reply-to-operator-115-sema-engine-split.md`
- `~/primary/reports/designer-assistant/49-sema-engine-state-and-introspect-readiness.md`
