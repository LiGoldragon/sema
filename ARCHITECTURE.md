# ARCHITECTURE — sema

The records database. redb-backed; content-addressed by blake3
of canonical rkyv encoding. Owned exclusively by **criome** —
no other process opens this file.

## Role

Sema is the centre of the engine. Every concept the engine
reasons about (code, schema, rules, plans, authz, history,
world data) is expressed as records here. Everything else
exists to serve sema:

- nexus is text → criome writes records here.
- signal is the rkyv envelope nexus uses to send criome
  edits to apply to records here.
- lojix-store holds the actual artifact bytes; sema records
  reference lojix-store by hash.
- rsc projects records here → Rust source for nix to compile.

> **Sema is all we are concerned with** (per
> [criome/ARCHITECTURE.md §1](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)).

## Boundaries

Owns:

- The redb file (one per criome instance).
- Slot allocation: counter-minted by criome, freelist-reuse,
  range `[0, 1024)` reserved for seed.
- `SlotBinding` table — slot → current content-hash + display-
  name. Bitemporal; slot-reuse is safe for historical queries.
- Per-kind change-log tables — keyed by `(Slot, seq)`,
  carrying `ChangeLogEntry` records (rev, op, content hashes,
  principal, sig-proof). Per-kind logs are ground truth.
- Per-kind primary tables — current state of each record kind.
- Per-kind index tables and a global revision index — derivable
  views.

Does not own:

- The Rust types of records (those live in
  [signal](https://github.com/LiGoldragon/signal); the former
  nexus-schema crate was absorbed there).
- The validator pipeline (that's criome; CANON-MISSING).
- Signal envelope or wire format (that's
  [signal](https://github.com/LiGoldragon/signal)).
- Artifact bytes (those live in
  [lojix-store](https://github.com/LiGoldragon/lojix-store);
  sema records reference by hash).

## Identity model

Records use **slot-refs** (`Slot(u64)`), not content hashes,
for cross-record references. Sema's index maps each slot to
its current content hash plus a bitemporal display-name
binding. Content edits update the slot's current-hash without
rippling rehashes through dependents. Renames update the
slot's display-name without rewriting any records anywhere.

Display-name is global — one name per slot, globally
consistent. rsc projections pick it up everywhere.

Slots are **global**, not opus-scoped.

## Stored by precise kind

Sema is the storage end of the project's [perfect-specificity
invariant](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md#invariant-d).
Every record stored here belongs to a specific kind declared
by a `KindDecl` record (which is itself a kind, stored
self-describingly). There is no untyped-blob pool, no
"miscellaneous record" table, no fallback storage path for
records that don't fit a known kind — a record without a
KindDecl cannot land. Kind growth happens via the criome
self-host loop: assert a new KindDecl, recompile, the new
kind has its own storage discipline (per-kind change-log,
per-kind primary table, per-kind indexes).

The schema isn't a separate concern bolted onto storage; it
*is* records here. Bitemporal queries over `KindDecl`
naturally answer "what was the schema yesterday."

## Code map

```
src/
├── lib.rs    — module entry; opens / closes the redb file
├── tables.rs — table definitions, key/value codecs
├── reader.rs — read paths
└── writer.rs — write paths (called only from criome's
                validator pipeline)
```

(Currently `todo!()` skeleton.)

## Status

**Skeleton-as-design**, day-one canonical. Backing types are in
signal. Behavior fills as criome scaffolds.

## Cross-cutting context

- Two-stores model (sema + lojix-store):
  [criome/ARCHITECTURE.md §5](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
- Per-kind change-log discipline:
  [criome/ARCHITECTURE.md §5](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
