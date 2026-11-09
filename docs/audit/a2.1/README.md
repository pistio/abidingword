# Phase A2.1 evidence

A2's 57 disclosed, checksummed PDFs (`docs/audit/a2/`), read by the
current backend and by PDFium glyphs through the app's own layout code.
These are regression measurements on known documents, not independent
evaluation. The report is `docs/recognition-a2.1-pdfium.md`.

| File | What it holds |
|---|---|
| `measurements.md` | Per document, grouped as asked (G05, Hebrew, Korean, Greek, multi-column) and all 57: text coverage, reading order, verified, wrong, not in edition, time and peak memory for both paths and the helper alone |
| `glyph-fidelity.md` | The seven differences between PDFium's text page and the glyphs the layout needs, found by glyph-by-glyph comparison, and what remains (word spacing) |
| `isolation-packaging-licensing.md` | Helper process, watchdog, memory cap, tests; Windows cross-build; licences; offline operation |
| `open-questions.md` | Before any further integration: word spacing (`Tw`), the Windows sandbox requirements (Job Object, `CREATE_NO_WINDOW`, least privilege, signing), the security review |
| `data/fidelity.json` | Scores per document and engine (`a21_measure.py`) |
| `data/recognition.json` | Verified, wrong and not-in-edition verses per document and engine (harness) |
| `data/clean-timing.json` | Wall time, CPU time and peak memory per reading (`a21_clean_timing.py`) |

Helper binary measured: `aw-pdfium-glyphs` at `a26af11`, Linux x86-64,
SHA-256 `6efa6e8ebdd3f6f28f48896e778d5d0e9008ef9931626f3692d1d1bf5ee80b2e`.
The library is pinned in `tools/pdfium-glyphs/PDFIUM.lock`.
