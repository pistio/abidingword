# Windows owner check — Stage 2 remains unaccepted

Use a separately built, pinned Stage 2 executable and a disposable profile. No paid workflow or installer publication is authorized by this document. The Stage 1 schema-26 workflow does **not** build Stage 2. Record the exact application source SHA, installer/executable hash, Windows version and display scaling with the evidence. Do not accept a build solely by its displayed application version.

1. Close Abiding Word. Make a full backup with the existing application, retain its matching executable, then use a copied/disposable profile for this check. Stage 2 upgrades the copy from schema 26 to 27 and creates the normal pre-migration backup. Never point an older executable at that upgraded database.
2. Import a legally owned copy of the **original** John MacArthur *Basisinformationen zur Bibel* PDF. Record edition, file SHA-256, page count and any printed pagination. The actual file was unavailable during cloud development. Do not upload or redistribute it without the necessary rights.
3. For an existing Source import, open **Document structure → Build structure**. For a new import, structure is collected during import. Check the source's language. Original PDF bookmarks remain available separately; do not mistake an unaligned page bookmark for a recognised text section. Inspect structure evidence and uncertainty warnings; compare representative headings, levels, physical pages and printed labels against Original.
4. Select **Acts 6** in Scripture, then inspect its Source result. Record the first location, its reason/evidence, displayed heading and physical PDF page. Open it. It must show the relevant dedicated passage/chapter section when one is reliably identified, or the biblical-book section as an explicitly broader fallback. An incidental mention near Acts 7 must not displace a grounded Acts 6 section. A mention-only result must remain labelled as a mention. If no reliable section is detected, record that as a failure/limitation rather than assuming success.
5. Open hierarchical Contents, navigate to the expected section in Reading and Original, switch both ways, use Back/Forward, open the full reader and return to the pane. Record screenshots of the whole workspace and close-ups showing the target heading, physical page, printed label and Original location. Check 900/1100/1440 widths and Windows display scaling, including 125%/150%.
6. Before rebuilding, create an exact quotation/citation in a disposable Study, a highlight, a note/bookmark, and a reading position. Change a section title or mapping in Document structure. Check the changed result/Contents, Undo, Reset and Undo Reset. Rebuild. The citation's words, source identity and original place, annotation anchors, Study content and reader navigation must survive.
7. Save a full `.awbackup`, restore it to another disposable profile, and confirm the correction and Undo history are present. Verify the managed original file hash is unchanged. Do not exercise permanent deletion on real work.
8. Repeat representative English and Korean documents, multicolumn pages, footnotes with unrelated references, running headers and a PDF without reliable structure. Ordinary text must remain readable when inference cannot align a heading.

Return a pass/fail table with exact references, expected heading/page, actual first result/page, mode-switch results and screenshot paths. Keep the original-file defect **OPEN / NOT ACCEPTED** until this evidence passes. If manual correction is required, distinguish that result from successful automatic detection.

## Reproducing cloud synthetic evidence

With the documented native development environment active:

```sh
AW_PDF_REVIEW_DIR=/tmp/aw-pdf-review cargo test --manifest-path src-tauri/Cargo.toml --locked --test pdf_structure --test migration_v27
```

The test writes an invented `structure-review.pdf` and serialized actual Rust import/index/reader metadata in that directory. Copy those two files alongside `web.json` in the screenshot harness's public data folder. Start its Vite config with `SCREENSHOT_DATA` pointing there; run `tests/screenshots/milestone-a3-pdf.mjs` with `SCREENSHOT_URL`, `PLAYWRIGHT_MODULE` and `CHROMIUM_PATH` as appropriate. This uses real PDF.js for Original, the actual interface, and serialized backend results. It is synthetic browser evidence, not a native Windows or MacArthur acceptance test.
