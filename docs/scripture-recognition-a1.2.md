# Scripture recognition, Phase A1.2: the structure-aware EPUB reading in less memory

**Status:** candidate for the owner's review, on branch
`claude/recognition-a1.2-memory`, from A1.1 (`5b2bbdd`). Results are
unchanged. There is no migration, no change to `Cargo.lock`, no interface
change and no production integration.

All editions are disclosed (the development corpus and Sets B–G). These
are regression measurements, not independent evaluation.

## 1. Commits

| SHA | What |
|---|---|
| `0e4c08e` | Harness: time and peak memory of the reading alone (`read_seconds`, `read_peak_memory_mb`), measured before the report is compared and written |
| `4473951` | Smaller per-character data: link marks as runs, removal marks only where needed, innermost elements computed once, byte ranges compact, the entry's stored bytes not kept |
| `e9c406a` | Byte ranges by anchors; non-space positions by bitmap with rank and select |
| the commit after it | Evidence (`docs/audit/a1.2/`) and this report |

## 2. Result

On the 24 EPUB editions (`docs/audit/a1.2/performance.md`):

| Measure, the reading alone | A1.1 | **A1.2** |
|---|---:|---:|
| Peak memory, largest (`latVUC-epub`) | 599 MB | **219 MB** (−63 %) |
| Peak memory, highest of the 24 | 599 MB | **248 MB** (`G01`) |
| Peak memory, median | 278 MB | **120 MB** (−57 %) |
| Time, all 24 | 396 s | 409 s (+3 %) |
| Every verse's provenance (525 033 verses) | — | **identical** |

* Every edition's peak falls, by 30 % to 63 %.
* Per edition, the reading takes between 6 % less and 13 % more time.
  The extra comes from the run-length and rank/select lookups.
* The harness's whole-run peak is often higher than the reading's. That
  comes from comparing every verse with the edition and writing the
  report, which is not app memory.

**Same results.** All 113 edition reports are identical to A1.1's. The
provenance reports are identical to A1.1's final code. Details, including
why an earlier A1.1 run differs by 3 bytes on byte-order-marked EPUBs:
`docs/audit/a1.2/equivalence.md`.

## 3. What changed

Before A1.2, the structure-aware reader kept several tables with an
entry for every character of the EPUB's text, for the whole book. Now:

| Per-character data | A1.1 | A1.2 |
|---|---|---|
| Link mark and role | a dense table | runs of equal marks (`epub_source::Runs`) in one u32 with the role in the top bits |
| Removal marks | a dense table | only for an entry that has a removal |
| Innermost element | computed twice, dense | computed once, as runs |
| Byte range | two u32 | an anchor every 64 characters or at a break; ranges that are not the character's own bytes (entities, `<br>`) kept as exceptions (`ByteRanges`) |
| Non-space positions | a list of positions | a bitmap with counts per 64 and every 64th position sampled (`NonSpace::rank` / `select`) |
| The entry's stored text | kept | not kept: each span points into the stored entry, identified by SHA-256 (`provenance::original_bytes`) |

Each compact structure gives exactly what the dense table gave, and unit
tests check this. Nothing is approximated. The verification decisions,
source spans, excluded ranges, byte offsets and refusal reasons are
computed from the same values as before.

## 4. Checks

At `e9c406a`:

* `cargo fmt -- --check`: clean.
* `cargo clippy --all-targets --locked -- -D warnings`: clean.
* `cargo test --release --locked`: 1 080 passed, 0 failed, 8 ignored (the
  harnesses), in 79 binaries.
* No interface file changed. The interface checks pass on A1.3, which
  contains A1.2 (`docs/recognition-a1.3-non-epub-safety.md`).

## 5. What is left

* The reading still holds a whole EPUB's text and per-element data at
  once, about 120 MB for a typical Bible. Incremental reading (entry by
  entry, releasing each after its verses are assembled) would lower that
  further. It changes the order in which the reader sees the book, so it
  would need the same equivalence evidence. It was not attempted.
* The time cost (+3 %) comes from lookups that could be cached per text
  node. They were left simple.
