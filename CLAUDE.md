# sema

Sema is the universal typed binary format — the thing.
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
corec       — .core → Rust with rkyv derives (bootstrap tool)
synth-core  — grammar .core + corec → Rust rkyv types (askicc↔askic contract)
aski-core   — parse tree .core + corec → Rust rkyv types (askic↔veric↔semac contract)
veri-core   — veric-output .core + corec → Rust rkyv types (veric↔semac contract)
askicc      — source/<surface>/*.synth → dsls.rkyv (all 4 DSLs combined)
askic       — reads source + dsls.rkyv → per-module rkyv (aski-core types)
veric       — per-module rkyv → program.rkyv (verified, linked)
domainc     — program.rkyv → Rust domain types (proc macro, compile-time)
semac       — program.rkyv + domain types → .sema (pure binary)
rsc         — .sema + domain types → .rs (Rust projection)
askid       — .sema + domain types + names → .aski (canonical text)
```

Only corec and semac (via rsc) generate Rust. Everything between
is rkyv. Only semac produces true sema.

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
