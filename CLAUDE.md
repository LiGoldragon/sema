# sema

Sema is a universal typed binary format — the thing.
Domain variants ARE the bytes. No strings. No unsized data.
Zero-copy, mmap-ready, deterministic.

**Only semac produces sema.** Everything upstream is rkyv —
serialized data that still has strings. It becomes sema only
when semac resolves all strings to domain variants.

Everything else exists to serve sema:
- Aski is one text notation for specifying sema (a frontend)
- The criome is the runtime that hosts sema worlds (the endgoal)
- Rust is the current compilation target

## What This Repo Is

The top-level Nix aggregator for the sema engine.

```
nix flake check        — build + test everything
nix develop            — shell with all compilers + data
```

## The Pipeline

```
cc       — .aski → Rust types (bootstrap seed)
askicc   — .synth → rkyv domain-data-tree (embedded in askic)
askic    — reads rkyv data-tree → dialect state machine → rkyv parse tree
semac    — reads rkyv → produces sema + Rust
```

Four separate binaries. Only cc and semac generate Rust.
Only semac produces true sema.

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
