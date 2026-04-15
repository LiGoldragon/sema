# sema

Universal typed binary format. Domain ordinals ARE the bytes.
No strings. Zero-copy, mmap-ready, deterministic.

Sema is the core. Aski is the stepping stone. The criome is the endgoal.

## What This Repo Is

The top-level aggregator for the compiler pipeline. `nix flake check`
here runs all checks across all three compiler stages:

```
nix flake check        — build + test synthc, askic, semac
nix develop            — shell with all three compilers + synth dialect
```

The three stages are separate repos wired as flake inputs:
- **synthc** — Stage 1: .synth + .aski → data-tree + derived enums
- **askic** — Stage 2: data-tree + .aski bodies → typed parse tree
- **semac** — Stage 3: parse tree → .sema binary + codegen

The `follows` chains ensure all three build against the same
toolchain and nixpkgs.

## What Sema Is

Sema is the universal typed binary format. Not a library, not a
framework — the format. Everything serializes into sema. rkyv is
the encoding. Domain ordinals are the bytes. Inter-linking is
content-addressed. Zero-copy, mmap-ready, deterministic.

The four prime generators (2, 3, 5, 7) produce every possible
meaning through fractal composition. Natural language text is a
lossy projection of a fully enumerable domain tree. The tree IS
the meaning.

## Reference

`v015_reference/kernel.sema` — old v0.15 compiled kernel (rkyv binary).

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
