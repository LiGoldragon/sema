# Skill — working in sema

*What an agent needs to know to be effective in this repo.*

---

## What sema is

Sema is the **workspace's typed-database kernel**. redb-backed;
values are rkyv-archived; tables are typed and version-guarded.
Each ecosystem layers its own typed tables atop sema in a
`<consumer>-sema` crate (criome's records today, `persona-sema`
in flight; future `forge-sema`, etc.).

Sema is to **state** what `signal-core` is to **wire**: the
kernel of typed primitives every consumer's typed layer
depends on.

Read `ARCHITECTURE.md` for the role/boundaries summary: what
sema (the kernel) owns, what each `<consumer>-sema` layer
owns, and the surface each side has.

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

- **One redb file per consumer.** Each `<consumer>-sema`
  layer opens its own file; cross-consumer sharing is not a
  thing. The kernel doesn't care which file; the consumer
  decides.
- **Values are rkyv-archived.** No JSON, no string-tagged
  blobs, no untyped bytes. The kernel's `Table<K, V: Archive>`
  enforces this.
- **Schema version is checked at open.** The kernel writes
  the consumer's `Schema::version` on first open and refuses
  to open a file whose stored version doesn't match. Schema
  changes are coordinated upgrades, not silent migrations.
- **Database format is checked at open.** The kernel persists a
  `DatabaseHeader` naming Sema's rkyv format identity
  (little-endian, pointer-width-32, unaligned, bytecheck) and
  refuses to open a database whose stored header mismatches this
  build.
- **Internal tables are namespaced.** Sema-owned redb tables use
  the `__sema_` prefix (`__sema_headers`, `__sema_meta`,
  `__sema_records`). Consumer table names must not use that
  prefix.
- **Record types live in `signal-<consumer>`, not in
  `<consumer>-sema`.** The consumer's typed-storage crate
  references the wire crate's records as values; it owns
  the table layout, not the records.
- **Sema's slot allocation is a utility, not a policy.**
  Append-only stores can use `Slot(u64)` + the monotone
  counter; non-append-only stores ignore them entirely.
- **Typed table scans return owned keys.** redb yields borrowed
  keys for borrowed key types (`&str`, `&[u8]`). Sema's
  `Table::iter` and `Table::range` eagerly collect rows and
  return `OwnedTableKey::Owned`, so callers never hold redb
  guards across the transaction boundary.

---

## What this repo is canonical for

Sema (the kernel) owns:

- The redb file lifecycle (open-or-create + parent mkdir +
  ensure_tables).
- Closure-scoped txn helpers (`store.read(|txn| ...)`,
  `store.write(|txn| ...)`).
- The typed `Table<K, V: Archive>` wrapper — hides rkyv
  encode/decode at the table boundary.
- `Table::ensure` for explicit typed-table materialization in
  consumer schema open paths.
- The standard `Error` enum (5 redb-error variants +
  rkyv + io + schema-version mismatch).
- The version-skew guard and database-format guard.
- The `Slot(u64)` newtype + monotone slot counter +
  `iter()` snapshot — utility for append-only stores.

## Test command surface

Use the repo-local scripts through Nix:

```sh
nix run .#test
nix run .#test-kernel-surface
nix run .#test-legacy-slot-store
nix run .#test-doc
```

`nix flake check` remains the pre-commit gate. The scripts are
for named inner-loop surfaces and are exposed as flake apps so
the pinned Rust toolchain is used.

Sema does **not** own:

- Per-consumer schema, table layouts, or migration
  helpers — those live in the consumer's
  `<consumer>-sema` crate.
- Record types — those live in `signal-<consumer>`.
- Validator pipelines — those live in the consumer's
  daemon (criome, persona-router, etc.).
- Wire format — `signal-core` + `signal-<consumer>`.
- Artifact bytes — `arca`.

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
