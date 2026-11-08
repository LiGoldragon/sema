# Sema

Sema version 0 reads Markdown into a typed document tree and prints Markdown from that tree. This repository begins with the reviewed contract only; independent implementations start from separate branches.

Vision source: `psyche-belief` revision `738727d23ec9e9cc6f37b018f428eee5c6cee057`, `vision/sema-v0.md`.

## Status: prototype, incomplete

- `print.[ Markdown ]` is declared total, and this implementation is not: a
  Document holding a `Target.Place` stops with `unimplemented!`, because the
  Place-to-anchor conversion is not designed. The contract test is kept,
  ignored, in `tests/round_trip.rs`.
- The Markdown codec is comrak 0.59.0, both ways (Fable order). comrak reads
  the text into its CommonMark tree, `src/compose.rs` walks that tree into the
  Document, and `src/print.rs` builds the tree from a Document for comrak to
  print; `src/commonmark.rs` configures both directions in one place. No comrak
  type is in the schema. What the Document has a place for is read whichever
  way it is written: a setext heading, a `*` list, a tilde or indented fence.
  What it has no place for is `Unsupported`: an ordered list, a thematic break,
  an HTML block, an image, a hard line break, a link title, an aligned table,
  a list item of more than one block, a heading inside a list or a quote. A
  tag written in prose, `vision:<topic>`, is words and prints as written. The
  libraries measured against the belief corpus, and the per-module table, are
  in `measure/round-trip.md`.
- Depth is the nesting, read relative: the first heading's level is the top
  and each deeper level a child; a heading above the top, or one skipping a
  level, is `Unsupported`. The print is canonical, top sections at `#`, so a
  module opening at `##` prints back one level up (Fable ruling, inferred,
  pending the living).
- Fence spellings are the tags the modules write: `ethos`, `datom`, `rust`,
  `nix`, `sh` (Shell), `ini`, `toml`, `mermaid`, and no tag for Plain.
- Markdown link targets, anchors included, are read and printed as
  `Target.Url` (Fable ruling, inferred, pending the living).
- A whole module is a `ModuleText`, composed into `Module.{ FrontMatter Document }`
  and printed back into a `ModuleText` (`sema module <path>`; Fable rulings,
  inferred, pending the living). The front matter is read as the three
  fields of FrontMatter, in any order, and printed canonically in the type's
  order; the text after the closing delimiter line is handed to Markdown. A
  description may be plain, double-quoted or single-quoted, and prints plain
  when the plain form reads back the same, double-quoted otherwise. A
  missing or empty field is `Missing`; any other front-matter line is
  `Unsupported`, carrying that line. Source bytes are kept only when the
  source is already canonical.
- Reading refuses with `ReadRefusal.{ Line Problem }` as ruled:
  `UnclosedFence`, `Unsupported` carrying the line as written, or `Missing`
  a front field. A Document
  alone still refuses front matter.
- `Structural` implements `outline`, `code`, `references` and `search` by hand.
  `edit` is not implemented: `Mismatch.{ Place Expected.Block Found.Block }` names
  a Found block, and no rule yet says which block is found when no block at
  the place equals Expected.
- No generated code yet. ethos-zero 20.0.0 checks every ethos file of the lane
  in the flake check.
