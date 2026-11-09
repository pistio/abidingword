# Phase A1.2: same results on all 113 editions

A1.2 (`e9c406a`) was run on all 113 disclosed editions with the
structure-aware EPUB reading (`AW_PIPELINE=structural`), on the
checksummed inputs (`CORPUS_OK`, `SETG_OK`).

## Edition reports

Compared field by field with the A1.1 structural run, ignoring only the
four time and memory fields: **113 of 113 identical.** That covers every
verified, wrong, missing, refused and not-in-edition verse, every
difference listed and every unverified reason.

## Structural (provenance) reports

These hold each verse's status, its number of spans and excluded ranges,
its id check, and full provenance (character and byte offsets, excluded
ranges) for every verse that is not exact, has removed ranges, is gated,
or is among the first 25.

* **101 of 113 identical** to the A1.1 structural run.
* **The other 12 are the development corpus's byte-order-marked EPUBs.**
  There every byte offset differs from that run by the BOM's 3 bytes.
  That run was made before A1.1's own correction (`47a4309`, "byte-order
  marks counted in byte ranges"; `docs/scripture-recognition-a1.1.md`
  §2.3).
* Run with A1.1's final code (`0e4c08e`), the same 12 reports are
  **identical** to A1.2's.
* A1.2's offsets are the right ones. Example: `kjv-epub` 1CH 10:1 starts
  at byte 75 622 of `OEBPS/1CH.xhtml`, the verse's first letter, in the
  stored entry (SHA-256 `c7f15e3e…`). The earlier run's 75 619 points at
  `an>`, three bytes before.

## Every verse's provenance

The structural report keeps full provenance for only some verses. For all
of them, the 24 EPUB editions were read by both builds with a digest of
the whole provenance report added (`performance.md`): **identical for all
525 033 verses.**

## Unit tests

* `epub_source::compact_tests`: `Runs` and `ByteRanges` give exactly the
  dense tables they replace. This covers ASCII and longer characters,
  entities, line-break tags and long white space.
* `provenance::nonspace_tests`: `rank` and `select` give exactly the list
  of positions.
* `tests/recognition_a1_structure.rs` now reads each verse's bytes back
  from the EPUB's stored entry. It checks that entry's SHA-256 and length
  against the provenance, where the reader's own copy of the entry is no
  longer kept.
