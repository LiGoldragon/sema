# ARCHITECTURE — sema

The workspace's typed-database substrate. redb-backed; values
are rkyv-archived; tables are typed and version-guarded. Sema is
the **kernel** for every sema-flavored store in the workspace:
criome's records, Persona's state, and other ecosystem stores.

## Role

Sema is to **state** what `signal-core` is to **wire**: the
kernel of typed primitives every consumer's typed layer
depends on. Each ecosystem layers its own typed tables atop
sema:

```
signal-core             sema
  ├─ signal-persona       ├─ persona-sema
  ├─ signal-forge         ├─ forge-sema  (future)
  └─ signal-arca          └─ ...
```

The kernel owns:

- The redb file lifecycle (open-or-create, ensure tables,
  parent-dir mkdir).
- The typed `Table<K, V: Archive>` wrapper that hides rkyv
  encode/decode at the table boundary.
- The closure-scoped txn helpers (`store.read(|txn| ...)`,
  `store.write(|txn| ...)`).
- The standard `Error` enum (5 redb-error variants + rkyv +
  io + schema-version-mismatch).
- The version-skew guard (per `~/primary/skills/rust-discipline.md`
  §"Schema discipline" — known-slot record carrying the
  schema version, hard-fail on mismatch).
- The slot-allocation utility (`Slot(u64)` + monotone
  counter + `iter()` snapshot) — generally useful for any
  append-only store.

Each consumer's typed layer (a separate crate, named
`<consumer>-sema` per the signal-family naming convention)
owns:

- Its `Schema` constant declaring the table list + schema
  version.
- Its typed table layouts (one table per record kind).
- Its convenience open methods (canonical path discovery,
  default schema registration).
- Its own migration helpers when needed.

The records' Rust types live in the matching `signal-<consumer>`
contract crate, not in `<consumer>-sema`. The wire side
defines the records; the storage side persists them.

Runtime write ordering is a consumer concern. In Persona, the
store actor owns the mailbox, transaction order, and commit
visibility; `persona-sema` owns only the table layout and schema
guard over this kernel.

## Boundaries

Sema (kernel) owns:

- The redb file lifecycle (open-or-create, parent mkdir,
  ensure_tables).
- Closure-scoped txn helpers.
- Typed `Table<K, V: Archive>` wrapper; rkyv encode/decode
  at the table boundary.
- The standard `Error` enum (typed `#[from]` for redb's 5
  error types + rkyv + io + schema-version mismatch).
- Version-skew guard — known-slot record carrying schema
  version, checked at open, hard-fail on mismatch.
- The `Slot(u64)` newtype + monotone slot counter + `iter()`
  snapshot — utility for append-only stores.

Each consumer's typed layer (`<consumer>-sema`) owns:

- Its `Schema` constant (table list + schema version).
- Its typed table layouts.
- Its open conventions (path discovery, schema registration).
- Its migration helpers.

Each consumer's runtime store actor owns:

- The mailbox into the database.
- Transaction sequencing.
- Commit-before-effect ordering.
- Subscription events emitted after durable state changes.

Sema does **not** own:

- Record Rust types — those live in the matching
  `signal-<consumer>` contract crate.
- Per-ecosystem table layouts — those live in
  `<consumer>-sema`.
- Runtime write ordering or actor mailboxes — those live in the
  consumer's daemon or dedicated store actor.
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

## Code map

```
src/
└── lib.rs    — Sema struct (open + read/write txn helpers) +
                Table<K, V: Archive> typed wrapper +
                Slot newtype + slot counter + iter +
                Error + version-skew guard
```

Files split (`store.rs`, `table.rs`, `error.rs`, `version.rs`)
land when the kernel grows past ~300 LoC.

## Status

**Kernel role.** Sema is the shared database kernel. Consumer
layers such as `persona-sema` define typed table layouts over it;
runtime store actors own sequencing and external effects.

## Cross-cutting context

- Sema-as-kernel design: `~/primary/reports/designer/63-sema-as-workspace-database-library.md`
- Persona's typed wire records (the values persona-sema
  persists): `signal-persona/`
- Persona's store layer over this kernel:
  `persona-sema/ARCHITECTURE.md`
- Persona's channel choreography:
  `~/primary/reports/designer/72-harmonized-implementation-plan.md`
- Two-stores model (sema + arca): `criome/ARCHITECTURE.md` §5
- Per-kind change-log discipline (criome's specific layer):
  `criome/ARCHITECTURE.md` §5
