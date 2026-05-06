# Skill — working in sema

*What an agent needs to know to be effective in this repo.*

---

## What sema is

Sema is the **records database** at the centre of the engine.
redb-backed; content-addressed by blake3 of canonical rkyv
encoding. Owned exclusively by criome — no other process opens
this file. Every concept the engine reasons about (code, schema,
rules, plans, authz, history, world data) is expressed as a
record here. Everything else orbits sema.

Read `ARCHITECTURE.md` for the role/boundaries summary: what
sema owns, what it doesn't, and the per-kind table layout.

---

## Intent

**Criome and sema are meant to be eventually impossible to
improve.**

> *"I am much more interested in a good design than in producing
> it quickly — criome and sema are meant to be eventually
> impossible to improve, so I value clarity, correctness, and
> introspection above production volume, speed, and time to
> market."*
>
> — Li, 2026-04-29

For sema specifically, this commits the project to:

- **The on-disk format reads cleanly.** A future engineer (or
  agent) opening the redb file and the per-kind table layout
  understands what each table is, why each index exists, what
  invariants the change-log preserves.
- **Every wire shape is typed.** No string-tagged columns, no
  opaque blobs the engine treats as untyped. If a value flows
  through sema, it has a closed Rust type with a derived rkyv
  encoding.
- **Content-addressing is non-negotiable.** Identity is the
  blake3 of canonical rkyv encoding. The hash is what the rest
  of the engine references; slots are the mutable handle on top
  of immutable identity.
- **Bitemporal correctness.** Slot reuse is safe for historical
  queries because the per-kind change-log carries the ground
  truth. The current-state tables are derivable.

When a design choice trades clarity for speed of writing, intent
wins. The right format now is worth more than a wrong format
sooner.

---

## Hard invariants for an agent working here

- **One redb file per criome instance.** Never open it from
  another process; never stand up a parallel store.
- **Per-kind change-log is ground truth.** Primary tables and
  index tables are derivable; if they disagree with the log,
  the log wins and the others are rebuilt.
- **Slot range `[0, 1024)` is reserved for seed.** Don't allocate
  application-level slots in this range.
- **Slot allocation policy is criome's, not sema's.** Sema
  exposes the binding tables; criome decides which slot a new
  binding lands in.
- **Record types live in signal, not sema.** Sema knows the
  schema as data (so prism can read it and emit Rust source);
  the Rust types themselves are signal's.

---

## What this repo is canonical for

Sema owns:

- The redb file (per criome instance).
- The `SlotBinding` table: slot → current content-hash +
  display-name.
- Per-kind change-log tables: keyed by `(Slot, seq)`, carrying
  `ChangeLogEntry` records (rev, op, content hashes,
  principal, sig-proof).
- Per-kind primary tables — current state of each record kind.
- Per-kind index tables and the global revision index.

Sema does **not** own:

- The Rust types of records (signal).
- The validator pipeline (criome).
- The signal envelope or wire format (signal).
- The artifact bytes (arca; sema records reference by hash).

---

## See also

- `ARCHITECTURE.md` — sema's role and boundaries.
- `AGENTS.md` — repo-specific carve-outs.
- criome's `skills.md` — the engine that owns sema.
- signal's `skills.md` — the rkyv types of records.
- arca's `skills.md` — the content-addressed artifact store.
- prism's `skills.md` — sema → Rust projector.
- lore's `programming/abstractions.md`,
  `programming/beauty.md`,
  `programming/push-not-pull.md` — cross-language discipline.
- this workspace's `skills/skill-editor.md` — how to edit and
  cross-reference skills.
