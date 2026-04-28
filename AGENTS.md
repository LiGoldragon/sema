# Agent instructions

Repo role: criome's **records database** — content-addressed records keyed by blake3 of their canonical rkyv encoding, redb-backed. Owned by criome.

Read [ARCHITECTURE.md](ARCHITECTURE.md) for the storage shape.

Workspace conventions live in [mentci/AGENTS.md](https://github.com/LiGoldragon/mentci/blob/main/AGENTS.md).

**`Slot(u64)` has a private field.** Construct via `Slot::from(value)`; extract via `let value: u64 = slot.into()`. Same pattern for `Revision`.

**`reader_count()` / `set_reader_count()`** persist the read-pool size in sema's redb meta table. Default `DEFAULT_READER_COUNT = 4` if unset. criome reads this at daemon startup to size its `Reader` actor pool.

**rkyv feature-set** must match the project pin per [tools-documentation/rust/rkyv.md](https://github.com/LiGoldragon/tools-documentation/blob/main/rust/rkyv.md).
