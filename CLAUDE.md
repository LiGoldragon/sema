# sema

Universal typed binary format. Domain ordinals ARE the bytes.
No strings. Zero-copy, mmap-ready, deterministic.

Sema is the core. Aski is the stepping stone. The criome is the endgoal.

## What This Repo Is

The top-level aggregator for the sema engine. `nix flake check`
here runs all checks across the entire pipeline:

```
nix flake check        — build + test askicc, askic, semac
nix develop            — shell with all compilers + data
```

## The Sema Engine

```
aski-core  →  askicc  →  askic  →  semac
(anatomy)    (bootstrap)  (compiler)  (sema gen)
```

All wired as flake inputs with `follows` chains for shared toolchain.

## Reference

`v015_reference/kernel.sema` — old v0.15 compiled kernel (rkyv binary).

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
