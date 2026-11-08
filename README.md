# Sema

Sema version 0 reads Markdown into a typed document tree and prints Markdown from that tree. This repository begins with the reviewed contract only; independent implementations start from separate branches.

Vision source: `psyche-belief` revision `738727d23ec9e9cc6f37b018f428eee5c6cee057`, `vision/sema-v0.md`.

## Status: prototype, incomplete

- `print.[ Markdown ]` is declared total, and this implementation is not: a
  Document holding a `Target.Place` stops with `unimplemented!`, because the
  Place-to-anchor conversion is not designed. The contract test is kept,
  ignored, in `tests/round_trip.rs`.
- Markdown link targets, anchors included, are read and printed as
  `Target.Url` (Fable ruling, inferred, pending the living).
- A module's front matter is refused; `Module.{ Front Document }` is pending
  the living. `sema body <path>` sets front matter aside as a test harness only.
- `Structural` implements `code`, `references` and `search` by hand.
  `outline` is not implemented: its root, for a document of other than one
  top-level section, is not stated. `edit` is not implemented: its `Refusal`
  has no declaration.
- No generated code yet: Ethos 19.3.1 refuses `document.ethos` (TraitWanted
  Depth) and `markdown.ethos` (Undeclared Error); those questions are routed.
