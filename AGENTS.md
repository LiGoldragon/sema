# Agent instructions — sema

You **MUST** read AGENTS.md at `github:ligoldragon/lore` — the workspace contract.

## Repo role

The workspace's typed storage kernel. Sema opens redb files with an
explicit schema, guards the rkyv database format, and provides typed
`Table<K, V>` access over rkyv-archived values.

Sema is not Criome's records engine and does not own Persona runtime
state. Full database-operation execution lives in `sema-engine`.

---

## Carve-outs worth knowing

- **No schema-less open.** Durable state opens through
  `Sema::open_with_schema(path, schema)`.
- **No raw slot store.** The retired `Slot` / raw bytes / monotone
  counter surface must not reappear in this crate.
- **No Criome read-pool config.** Criome runtime tuning belongs to
  Criome; `reader_count` is not a sema kernel concept.
- **rkyv feature-set** must match the project pin per
  lore/rust/rkyv.md.
