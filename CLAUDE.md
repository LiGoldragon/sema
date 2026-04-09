# sema

Application-layer sema types. The programming-logic domains (Op, Expr,
Control, Ownership, etc.) built on top of sema-core's prime generators.

## What This Repo Provides

The domain vocabulary that replaces strings. When a samskara Rule has a
Body, that Body is a composition of sema domain terms — not text. When a
nexus message asserts a fact, the content fields are trees of sema
references that resolve to deterministic ordinals.

sema-core gives the generators (2, 3, 5, 7). This repo gives the
application-layer compositions: operations, expressions, control flow,
ownership patterns, type kinds — the domains needed to specify programs,
knowledge, and eventually all media.

## Translation Tables

Domain trees are language-independent meaning. Translation tables render
them into surface forms:

- English prose
- Chinese
- Sign language
- Color mappings
- Sound mappings
- Any sensory modality

The same sema object can be rendered as text, as color, as sound, as
spatial arrangement. The rendering is a projection — the domain tree is
the truth.

## Current State

v1 branch has CozoScript seed data. To be rewritten as aski type
definitions once the nexus engine is operational.

## VCS

Jujutsu (`jj`) is mandatory. Always pass `-m`.
