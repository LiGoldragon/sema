# Sema — Future State

Sema is the universal typed binary format. Domain variants ARE the
bytes. The pipeline (corec, askicc, askic, veric, domainc, semac)
lands a program as a sema file; the criome hosts it. This document
captures what that endgame looks like — the properties sema commits
to, the runtime that receives it, the verification layers that will
grow around it, and the tools still to come.

This is the aspirational counterpart to `CLAUDE.md` (current state).
When something written here becomes reality, it moves out of this
doc into the operational specs.


## Format invariants

- **Pure binary.** Domain variants ARE discriminant bytes. No strings
  in sema itself; no unsized data; no pointer tags. What bytes a
  program occupies at rest is exactly what it means.
- **Zero-copy, mmap-ready, deterministic.** Layouts are structural;
  two programs that define the same domains produce byte-identical
  sema for equivalent source.
- **No strings in code — strings live in translation tables.** Sema
  carries domain identities as bytes. Strings live in external
  translation tables indexed by those identities. One table per
  (language, dialect, style modifier) — English-formal,
  English-casual, Spanish-Rioplatense, Japanese-keigo, and so on.
  At render time the criome consults the right table; the sema
  binary itself is language-agnostic.
- **Bidirectional round-trip.** `.aski → askic → semac → .sema →
  askid → .aski` is lossless for canonical-equivalent programs.
  askid (the deparser) is the structural proof that sema is
  complete.


## The criome as runtime

The criome is the endgoal. It is to agents what an operating system
is to processes: a substrate that hosts sema worlds without
dictating what runs on them.

- **World hosting.** Each agent owns its world; the criome provides
  content-addressed storage, identity, persistence, and navigation.
- **Semantic-level editing.** As sema enumerates audio, video, and
  spatial composition as domain variants, the criome becomes the
  runtime for experiences that today require ad-hoc file formats
  and interpretation pipelines. Edit a chord, a color, a camera
  move at the variant level.
- **Zero-copy structural sharing.** Versions of a world share
  unchanged subtrees; forks, histories, and merges are pointer
  operations, not byte copies.
- **Meaning independent of natural language.** A program, a text,
  a score — all three are sema identities at the criome layer.
  Which human language renders them is a translation-table choice,
  not a property of the stored form.


## Verification goals

Current veric performs structural checks — types, scopes, trait
completeness at a surface level. Future layers extend this toward
checking that sema programs are *correct by their domain's rules*,
not merely parseable.

- **Origin enforcement.** Place-based origins (`'Place`,
  `'self.Field`, `'(A B)`) currently parse into the tree without
  semantic checking. When Rust's Polonius model stabilizes, veric
  (and rsc) will verify that borrows live within their origin's
  scope, across function boundaries.
- **View-type exactness.** The `{| Field ... |}` view-type
  annotations will be verified: a borrow with view X can touch
  exactly those fields; concurrent disjoint-view borrows must
  type-check with disjoint field sets.
- **Mutation within representation.** A mutable value may only
  transition to other values its type structurally represents. The
  type's declared shape bounds every legal post-mutation state.
  Verifier-enforced before compilation. Far-future.
- **Other checks on the roadmap** (promised in architecture.md,
  pending implementation): circular import detection, visibility
  enforcement across modules, trait-completeness detail, literal
  range checks at const-eval time.


## Tools to come

- **rsc — Rust projector.** Mechanical transformation from `.sema`
  to `.rs`. One domain variant → one codegen pattern. No semantic
  analysis; verification has already happened upstream. Will emit
  Rust origins once Polonius lands.
- **askid — aski deparser.** Reads `.sema + .aski-table.sema +
  domain types → canonical .aski text`. Proves bidirectional
  round-trip. Canonical: formatting is determined by the tree, not
  by the original source; running askid twice is idempotent.
- **lojix — build DSL.** Sketch only. A future dialect of the aski
  family dedicated to build orchestration, replacing Nix flakes in
  the sema ecosystem.


## Language evolution

- **Rust self-hosted in aski.** The bootstrap pipeline is written in
  Rust today; eventually every component (corec, askicc, askic,
  veric, domainc, semac, rsc, askid) rewrites itself in aski and
  compiles via the sema pipeline.
- **Grammar bidirectionality.** Every synth rule parses and
  deparses with the same structural definition. Adding a syntactic
  construct adds both directions simultaneously, by construction.
- **nexus protocol.** The wire protocol for agents communicating
  with criome worlds. Named; detail pending.
- **Operator and type-system completeness.** Gaps (division,
  bitwise, dyn dispatch, const expressions, and so on) land
  incrementally. See `Mentci/bridge/` for the current scheduling.


## Ontology expansion

Sema's expressive range grows with its ontology. Every domain that
gets enumerated as variants loses the need for a string placeholder
in the stored form (translation tables still carry the human
rendering).

- **Today.** Core types (enums, structs, numerics, strings as
  placeholders). Astrological ontology (signs, planets, houses,
  dignities) partially enumerated.
- **Near.** Scheduling, geography, music intervals, color,
  typography.
- **Later.** Audio composition, video composition, spatial layout,
  natural-language structure itself.
- **Principle.** Every string in the system today is a placeholder
  for a domain composition not yet specified. As the ontology
  grows, placeholders collapse into variants. Translation tables
  absorb the human-rendering job.


---

## Companion docs

- `sema/CLAUDE.md` — current state + pipeline.
- `criome/CLAUDE.md` — world-hosting model.
- `aski-core/spec/design.md` — language design constraints.
- `aski-core/spec/architecture.md` — pipeline architecture.
- `Mentci/bridge/` — current aski syntax evolution.
