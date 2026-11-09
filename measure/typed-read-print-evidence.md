# Typed Document read/print evidence, Opus lane

Lane commit bec116b (sema-v0-opus, pushed), gate `checks.default` green on prometheus.
Corpus: psyche-belief 2c84339, 79 Markdown files, each read by `sema module <path>` as
`Module.{ FrontMatter Document }`, printed back, and the printed text read again.

- 78 of 78 modules: the printed text reads back to an equal tree.
- 21 of 78 print byte-identical; the other 57 differ only by the canonical print
  (first case read: vision/account.md opens at `##`, printed at `#`, the ruled relative depth).
- README.md has no front matter: as a Module it is refused `Unsupported «# psyche-belief»` at line 1;
  as a Document (`sema document`) it is byte-identical and reads back equal.
- vision/sema-v0.md itself: 3018 bytes read, 3018 printed, byte-identical, tree equal.
  `sema outline vision/sema-v0.md 2` → `[ { «Markdown, read structurally» [] } ]`;
  `sema code … [ «Markdown, read structurally» ]` yields its Ethos and datom fences typed.

Per-file results: corpus.tsv beside this file.
