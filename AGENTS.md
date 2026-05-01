# Agent instructions — sema

You **MUST** read AGENTS.md at `github:ligoldragon/lore` — the workspace contract.

## Repo role

Criome's **records database** — content-addressed records keyed by blake3 of their canonical rkyv encoding, redb-backed. Owned by criome.

---

## Carve-outs worth knowing

- **`Slot(u64)` has a private field.** Construct via `Slot::from(value)`; extract via `let value: u64 = slot.into()`. Same pattern for `Revision`.
- **`reader_count()` / `set_reader_count()`** persist the read-pool size in sema's redb meta table. Default `DEFAULT_READER_COUNT = 4` if unset. criome reads this at daemon startup to size its `Reader` actor pool.
- **rkyv feature-set** must match the project pin per [lore/rust/rkyv.md](https://github.com/LiGoldragon/lore/blob/main/rust/rkyv.md).
