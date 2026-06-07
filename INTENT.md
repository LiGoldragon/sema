# INTENT — sema

*What the psyche has explicitly intended for this project. Synthesised
from psyche statements and the applicable workspace constraints; not
embellished. Maintenance: `primary/skills/repo-intent.md`.*

`sema` is today's typed storage kernel: a Rust library over redb and
rkyv that opens a database file with an explicit schema, guards the
rkyv database format, and provides typed table operations inside
closure-scoped transactions. It is to state what `signal-core` is to
the wire — the small kernel of primitives higher layers build on. It is
not a daemon, not shared storage, and not the full database engine;
full database-operation execution lives in `sema-engine`.

## Repo-scope only

This file carries kernel-side intent for `sema`. Engine execution
(Signal-verb execution, query/mutation plans, operation log, snapshots,
subscriptions) is `sema-engine`'s. Workspace-shape intent stays in
`primary/INTENT.md`.

## Goals

- Own the redb file lifecycle, the rkyv database-format guard, the
  schema-version guard, closure-scoped read/write transactions, and
  `Table<K, V>` typed storage that hides rkyv encode/decode at the table
  boundary and returns owned rows.
- Publicly name the closure-scoped transaction types it passes into
  consumers, so higher layers can type local table reducers without
  adding their own redb dependency.
- Stay a clean kernel: respond to engine-side discoveries that require
  kernel changes, while most active development lives in `sema-engine`.

## Constraints

- **One-way dependency direction.** `sema-engine` may depend on `sema`;
  `sema` must not depend on `sema-engine`, Signal contracts, Kameo,
  tokio, NOTA, Nexus, Persona, or Criome.
- **No schema-less open and no raw byte store.** Durable state opens
  only through `Sema::open_with_schema(path, schema)`; there is no
  schema-less public open path and no raw-byte store API. The retired
  raw-byte append path is intentionally absent.
- **Schema and format mismatches hard-fail at open.** The schema guard
  writes the schema on first open and hard-fails on mismatch; the
  format-identity guard refuses incompatible rkyv builds.
- **Table layout belongs to the consuming component.** Record Rust types
  live in Signal contract crates or component domain crates, not in
  `sema`; runtime ordering, actor mailboxes, commit-before-effect
  policy, and subscriptions belong to the consumer or to `sema-engine`.
  Internal kernel tables use the `__sema_` prefix and are limited to
  kernel metadata and database headers.
- **Removal is irreversible at the kernel layer.** redb's copy-on-write
  page reuse means a removed record's bytes linger only until a later
  write reclaims the pages; there is no undelete, tombstone, or history.
  A consumer that may need a removed record back must capture it before
  `remove`. `sema-engine`'s `Retract` inherits this; recoverable
  deletion is a deliberate higher-layer feature.
- **Backward compatibility is not a constraint for this kernel.** `sema`
  may break to make the engine substrate beautiful.

## Today and eventually

Today's `sema` is the Rust-on-redb storage kernel and a realization
step. The eventually-self-hosting `Sema` substrate — the universal
medium for meaning, Sema-on-Sema, content-addressed schema identity and
reducer-based migration — is a different artifact at a different layer.
Today's manual `SchemaVersion` mechanic is the realization step for the
eventual content-addressed model, which is future work, not
first-prototype work.

## See also

- `ARCHITECTURE.md` — role, boundary, public surface, deletion
  durability, versioning today-and-eventually.
- `../sema-engine/ARCHITECTURE.md` — the full database engine over this
  kernel.
- `primary/ESSENCE.md` §"Today and eventually" — the scope boundary.
