# The Markdown codec under the Document, measured

Every belief module of psyche-belief `35d2be5` is read into the Document, printed back, and read again: 77 modules, of which vision holds 56, intent 17, spirit 3 and knowledge 1. Byte-identical compares the printed body with the body as written. The body is the text after the front matter; reading and printing the front matter is sema's own work and the same under every codec. Document-equal means the printed text reads back to the same Document. A module holding a construct the Document has no place for is counted unread and is not printed. The measure is made with `measure/`: `nix build 'git+file://<lane>?dir=measure'`, then `sema-codec-measure <module.md>...`.

| codec | byte-identical | Document-equal | modules with an unread construct |
| --- | --- | --- | --- |
| hand-written reader and printer, `ab1a90c` | 21 | 77 | 0 |
| comrak 0.59.0, the lane codec | 21 | 77 | 0 |
| markdown 1.0.0 with mdast_util_to_markdown 0.0.2 | 21 | 74 | 0 |

Over the 54 vision modules at `b8e84e9`, comrak and the hand codec each give 11 byte-identical and 54 Document-equal; markdown-rs gives 11 and 52. markdown-rs reads every module, but mdast_util_to_markdown 0.0.2 has no printer for a Table, so it cannot print the three modules that hold one. comrak leaves no construct unread, so pulldown-cmark was not measured. Over all 77 modules, comrak prints byte for byte what the hand printer printed. A module that is not byte-identical differs at the line of its first heading written at `##`, since the canonical print puts a top section at `#`, or at a place where the print's spacing is canonical.

| module | hand, `ab1a90c` | comrak | markdown-rs |
| --- | --- | --- | --- |
| vision/account.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/anatomy.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/aspects.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/belief-modules.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/belief-nexus-v1.md | Document-equal, bytes differ from body line 6 | Document-equal, bytes differ from body line 6 | Document-equal, bytes differ from body line 6 |
| vision/book.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/chronos.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/clojure.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/collaboration.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/committing.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/complex-v1.md | byte-identical | byte-identical | byte-identical |
| vision/criomos.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/crystallization.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/datom.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/deployment.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/energy-nexus-v1.md | byte-identical | byte-identical | byte-identical |
| vision/ethos.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/far-flow-nexus.md | byte-identical | byte-identical | byte-identical |
| vision/far-sema.md | byte-identical | byte-identical | byte-identical |
| vision/far-versioning.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/field-aspect.md | byte-identical | byte-identical | byte-identical |
| vision/flow-nexus.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/flow-nexus-v1.md | Document-equal, bytes differ from body line 12 | Document-equal, bytes differ from body line 12 | Document-equal, bytes differ from body line 12 |
| vision/git-server.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/herdr-nexus-v1.md | Document-equal, bytes differ from body line 6 | Document-equal, bytes differ from body line 6 | Document-equal, bytes differ from body line 6 |
| vision/high-level-view.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/horizon.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/long-model-roles.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/long-nexus.md | byte-identical | byte-identical | byte-identical |
| vision/long-sema.md | byte-identical | byte-identical | byte-identical |
| vision/memory.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/mind-aspect.md | byte-identical | byte-identical | byte-identical |
| vision/model-roles.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | print failed: no Table printer |
| vision/nexus.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/nexus-v1.md | byte-identical | byte-identical | byte-identical |
| vision/nix-builds.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/orchestrate.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/persona.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/presentation.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/proposal.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/protos.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | print failed: no Table printer |
| vision/psyche-aspect.md | Document-equal, bytes differ from body line 5 | Document-equal, bytes differ from body line 5 | Document-equal, bytes differ from body line 5 |
| vision/psyche-fable.md | byte-identical | byte-identical | byte-identical |
| vision/psyche.md | Document-equal, bytes differ from body line 12 | Document-equal, bytes differ from body line 12 | Document-equal, bytes differ from body line 12 |
| vision/remembering.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/review.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/sema.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/sema-v0.md | byte-identical | byte-identical | byte-identical |
| vision/short-ethos.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/short-sema.md | byte-identical | byte-identical | byte-identical |
| vision/signal.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/subflows.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/system-one.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/testing.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/versioning.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| vision/x11.md | byte-identical | byte-identical | byte-identical |
| intent/anatomy.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/belief-modules.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/configuration.md | byte-identical | byte-identical | byte-identical |
| intent/context.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/conversion.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/design.md | Document-equal, bytes differ from body line 9 | Document-equal, bytes differ from body line 9 | Document-equal, bytes differ from body line 9 |
| intent/energy.md | byte-identical | byte-identical | byte-identical |
| intent/fable.md | byte-identical | byte-identical | byte-identical |
| intent/living-interaction.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/main-flow.md | byte-identical | byte-identical | byte-identical |
| intent/models.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/opus.md | byte-identical | byte-identical | byte-identical |
| intent/protos-parsing.md | byte-identical | byte-identical | byte-identical |
| intent/psyche.md | Document-equal, bytes differ from body line 3 | Document-equal, bytes differ from body line 3 | Document-equal, bytes differ from body line 3 |
| intent/scratch.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/startup-prompt.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| intent/testing.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
| knowledge/flow-messaging.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | print failed: no Table printer |
| spirit/core.md | byte-identical | byte-identical | byte-identical |
| spirit/ghost.md | byte-identical | byte-identical | byte-identical |
| spirit/psyche.md | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 | Document-equal, bytes differ from body line 1 |
