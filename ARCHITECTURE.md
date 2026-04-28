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
- The validator pipeline (that's criome).
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
└── lib.rs    — Sema struct (open/store/get) + Slot newtype + Error;
                redb tables (records, meta) defined inline; tests
                cover persistence + slot-allocation invariants
```

The longer-term split into `tables.rs` / `reader.rs` /
`writer.rs` lands when behaviour grows beyond M0's
slot-counter + bytes-by-slot pair.

## Status

**Working M0 core.** `Sema::open`, `Sema::store(&[u8]) → Slot`,
`Sema::get(Slot) → Option<Vec<u8>>`, `Sema::iter`,
`Sema::reader_count`, `Sema::set_reader_count` implemented and
tested (12 tests cover monotone slot allocation starting at
`SEED_RANGE_END = 1024`, persistence across reopens, empty-
record round-trip, missing-slot returns None, and
`reader_count` persistence with `DEFAULT_READER_COUNT = 4`).

The `reader_count` API persists the read-pool size in sema's
redb meta table — criome-daemon reads it at startup to size
its `Reader` actor pool.

Per-kind tables, change-log, `SlotBinding`, and bitemporal
queries land as kinds beyond Node/Edge/Graph come online (M1+).

## Cross-cutting context

- Two-stores model (sema + lojix-store):
  [criome/ARCHITECTURE.md §5](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
- Per-kind change-log discipline:
  [criome/ARCHITECTURE.md §5](https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md)
