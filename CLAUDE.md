# sema

Sema is a universal typed binary format — the thing.
Domain variants ARE the bytes. No strings. Zero-copy,
mmap-ready, deterministic.

Everything else exists to serve sema:
- Aski is one text notation for specifying sema (a frontend)
- The criome is the runtime that hosts sema worlds (the endgoal)
- Rust is the current compilation target

Aski will eventually be replaced by better ways to represent
sema for human consumption. The .sema format is permanent.

## What This Repo Is

The top-level Nix aggregator for the sema engine.

```
nix flake check        — build + test everything
nix develop            — shell with all compilers + data
```

## The Two Compilers

```
askic (frontend)   .aski → .sema     reads text, produces binary
semac (backend)    .sema → .rs       reads binary, produces code
```

askic and semac are independent. Multiple frontends can produce
.sema. semac is permanent.

askic internally contains three layers:
```
cc      (aski-core crate)  — .aski → Rust types
askicc  (askicc crate)     — uses cc + .synth → scoped types + dialects
askic   (askic crate)      — uses askicc → parser, data-tree, .sema output
```

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
