# Phase A1.1 evidence

All on **disclosed** documents (Sets B–G, recon and development corpora). It
is regression evidence, not independent evaluation. The corpus checksums
(`docs/audit/v0.24.7/corpus-checksums.sha256`) and the Set G seal were
verified before every run. Set G's blind record, the v0.24.7 baseline and
every earlier evaluation record are unchanged.

| File | What it holds |
|---|---|
| `cases.md` | The 8 formerly wrong verses, the 16 formerly refused ones, what the new rules refuse, and the faults found on the way |
| `regression-tables.md` | Frozen, A1 and A1.1 on every EPUB-family edition: printed, verified, wrong, not in edition, trace status, withdrawals by cause |
| `performance.md` | Runtime and memory, frozen against structure-aware, in the optimised build, one process per edition |
| `removed-ranges.md` | Every reference removed from a verified verse, as character, byte and element ranges, each checked against the stored file |

## Runs

| Run | Binary | Editions | What it shows |
|---|---|---|---|
| Frozen, all disclosed editions | `real_document_editions` release build at `e96d241` | 113 | The frozen reading is unchanged by A1.1. |
| Structure-aware, all disclosed editions | release build after the second-round fixes, before the byte-order-mark fix (which changes byte numbers only) | 113 | Which verses are verified, refused and withdrawn. |
| Frozen and structure-aware in turn | release build at `47a4309` | 34 run, of which the 24 EPUB-family editions are tabled (the other 10, Set G PDFs and documents, are read by the frozen path in both settings and are identical) | The tables in this folder, and runtime and memory. |

## The frozen reading is unchanged

The frozen reading at `e96d241` was compared with the v0.24.7 reports
(`c96756d`) on all 113 disclosed editions. Seconds, peak memory and the
temporary reference file name were ignored. Every field is identical in 104
editions. In the other 9, the only difference is the **apparatus piece
count**: pieces the recogniser removes from inside a verse now carry their
position (`Apparatus::within_verse`) and are no longer merged.

| Edition | Apparatus pieces v0.24.7 → A1.1 |
|---|---|
| engwebu-rl2col | 2 197 → 2 203 |
| engbsb-rwpdf2col | 4 961 → 4 964 |
| spavbl-rlofodt | 1 986 → 1 988 |
| deutkw-docx | 705 → 706 |
| deutkw-lopdf | 894 → 895 |
| deutkw-pdf | 2 206 → 2 207 |
| engwebp-epub | 1 199 → 1 207 |
| porbr2018-epub | 758 → 827 |
| porbr2018-pdf | 1 753 → 1 762 |

None of the commits after `e96d241` touch the frozen reading's code.

## Editions not read by the EPUB reader

Under the structure-aware setting, the 79 editions that are not EPUB-family
(PDF, DOCX, ODT, Markdown, text, USFM) are read by the frozen path. Their
reports are identical to v0.24.7 apart from the same apparatus count.

The disclosed corpora still hold **20 verified verses with wrong text**, all
on the frozen path in non-EPUB editions, all from before A1, and unchanged:

* `spablm-docx`: 9;
* `deutkw-pdf`: 5;
* `deutkw-lopdf`: 2;
* `porbr2018-pdf`, `engasv-rrlab1col`: 1 each;
* `spav1602p-lodocx`: 2 (and 2 more verified verses that the edition does not have).

They are why no release candidate can be frozen from this state.
