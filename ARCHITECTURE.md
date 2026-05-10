# ARCHITECTURE — sema

The workspace's typed-database substrate. redb-backed; values
are rkyv-archived; tables are typed and version-guarded. Sema is
the **kernel** for every sema-flavored store in the workspace:
criome's records, Persona's state, and other ecosystem stores.

## Role

Sema is to **state** what `signal-core` is to **wire**: the
kernel of typed primitives every consumer's typed layer
depends on. Each state-bearing component layers its own typed
tables atop sema:

```
signal-core             sema
  ├─ signal-persona       ├─ persona-mind tables
  ├─ signal-forge         ├─ criome tables
  └─ signal-arca          └─ future component tables
```

The kernel owns:

- The redb file lifecycle (open-or-create, ensure tables,
  parent-dir mkdir).
- The typed `Table<K, V: Archive>` wrapper that hides rkyv
  encode/decode at the table boundary, can materialize
  typed tables explicitly, and snapshots typed rows with
  owned keys.
- The closure-scoped txn helpers (`store.read(|txn| ...)`,
  `store.write(|txn| ...)`).
- The standard `Error` enum (5 redb-error variants + rkyv +
  io + schema-version-mismatch + database-format-mismatch).
- The version-skew guard (per `~/primary/skills/rust-discipline.md`
  §"Schema discipline" — known-slot record carrying the
  schema version, hard-fail on mismatch).
- The database header guard naming Sema's rkyv format identity:
  little-endian, pointer-width-32, unaligned archives, and
  bytecheck validation.
- The slot-allocation utility (`Slot(u64)` + monotone
  counter + `iter()` snapshot) — generally useful for any
  append-only store.

Each consumer's typed layer owns:

- Its `Schema` constant declaring the schema version.
- Its typed table constants.
- Its typed table layouts (one table per record kind).
- Its convenience open methods (canonical path discovery,
  default schema registration).
- Its own migration helpers when needed.

That typed layer normally lives inside the state-owning component
repo. A dedicated companion crate is only warranted when multiple
components genuinely share the same table layer. Do not create
`persona-sema` or another broad shared Persona database component by
default.

The records' Rust types live in the matching `signal-<consumer>`
contract crate, not in sema. The wire side defines the records; the
storage side persists them.

Runtime write ordering is a consumer concern. In Persona, each
state-bearing component actor owns the mailbox, transaction order,
and commit visibility for its own database; its local typed Sema
layer owns only the table layout and schema guard over this kernel.

## Boundaries

Sema (kernel) owns:

- The redb file lifecycle (open-or-create, parent mkdir,
  ensure_tables).
- Closure-scoped txn helpers.
- Typed `Table<K, V: Archive>` wrapper; rkyv encode/decode
  at the table boundary; `ensure`, `get`, `insert`,
  `remove`, `iter`, and `range` table affordances.
- The standard `Error` enum (typed `#[from]` for redb's 5
  error types + rkyv + io + schema-version mismatch).
- Version-skew guard — known-slot record carrying schema
  version, checked at open, hard-fail on mismatch.
- Database header guard — rkyv format identity stored in
  `__sema_headers`, checked at open, hard-fail on mismatch.
- The `Slot(u64)` newtype + monotone slot counter + `iter()`
  snapshot — utility for append-only stores.

Each consumer's typed layer owns:

- Its `Schema` constant (schema version).
- Its typed table layouts and explicit table-materialization
  path.
- Its open conventions (path discovery, schema registration).
- Its migration helpers.

Each consumer's runtime actor owns:

- The mailbox into the database.
- Transaction sequencing.
- Commit-before-effect ordering.
- Subscription events emitted after durable state changes.

Sema does **not** own:

- Record Rust types — those live in the matching
  `signal-<consumer>` contract crate.
- Per-ecosystem table layouts — those live in the state-owning
  component's typed Sema layer.
- Runtime write ordering or actor mailboxes — those live in the
  consumer's daemon actor.
- The validator pipeline — that's the consumer's daemon
  (criome, persona-router, etc.).
- Wire format — that's `signal-core` + `signal-<consumer>`.
- Artifact bytes — those live in `arca`; sema records
  reference by hash.

Criome owns its runtime-specific configuration:

- `reader_count` / `set_reader_count` — criome-specific
  read-pool config; lives in criome.

## Identity model

Records use **slot-refs** (`Slot(u64)`), not content hashes,
for cross-record references. Sema's index maps each slot to
its current content hash plus a bitemporal display-name
binding. Content edits update the slot's current-hash without
rippling rehashes through dependents. Renames update the
slot's display-name without rewriting any records anywhere.

Display-name is global — one name per slot, globally
consistent. prism projections pick it up everywhere.

Slots are **global**, not graph-scoped.

## Stored by precise kind

Sema is the storage end of the project's perfect-specificity
invariant.
Every record stored here belongs to a specific kind defined
in a signal-family closed Rust type — for criome that is this
repo's companion `signal` crate; for Persona that is
`signal-persona` plus the relevant channel contract. There is no
untyped-blob pool, no "miscellaneous record" table, no fallback
storage path for records that don't fit a known kind.

## Constraints

- Sema opens, creates, versions, and guards redb files; it does not own any
  Persona runtime state.
- Sema stores typed rkyv values through typed tables; it does not accept
  untyped blob tables as a fallback path.
- Sema's API uses closure-scoped transactions so callers cannot leak redb
  transaction lifetimes across actor boundaries.
- Table layout belongs to the component that owns the state.
- Record Rust types live in Signal contract crates or component domain crates,
  not in sema.
- Runtime ordering, actor mailboxes, commit-before-effect policy, and
  subscriptions belong to the consuming component.
- A broad shared Persona database layer is not part of the current
  architecture.

## Code map

```
src/
└── lib.rs    — Sema struct (open + read/write txn helpers) +
                Table<K, V: Archive> typed wrapper +
                OwnedTableKey for iterator snapshots +
                DatabaseHeader rkyv-format guard +
                Slot newtype + slot counter + iter +
                Error + version-skew guard
```

Internal Sema tables are namespaced with `__sema_`:
`__sema_headers`, `__sema_meta`, and `__sema_records`.

The remaining hygiene split (`store.rs`, `table.rs`,
`error.rs`, `version.rs`) is tracked separately from the
header/namespacing change.

## Status

**Kernel role.** Sema is the shared database kernel. State-owning
components define their typed table layouts over it; consumer runtime
actors own sequencing and external effects.

## Cross-cutting context

- Sema-as-kernel design: `~/primary/reports/designer/63-sema-as-workspace-database-library.md`
- Persona's typed wire records: `signal-persona/` and
  `signal-persona-*` contract repos.
- Persona state components' local typed Sema layers, beginning with
  `persona-mind/`.
- Persona's channel choreography:
  `~/primary/reports/designer/72-harmonized-implementation-plan.md`
- Two-stores model (sema + arca): `criome/ARCHITECTURE.md` §5
- Per-kind change-log discipline (criome's specific layer):
  `criome/ARCHITECTURE.md` §5
