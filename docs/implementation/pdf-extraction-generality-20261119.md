# PDF extraction generality audit

19 November 2026. Audit baseline:
`0b6ffa76ac4e3528ea83d54c86d0e6588eb22924` (PR #54), plus the invented
regressions in `src-tauri/tests/pdf_extraction_variants.rs`.

## Finding

The PDF correction is independent of the owner's supplied document. Review of
the generated-heading, native-outline, printed-contents and stored-structure
code found no branch keyed by that document's filename, hash, byte size,
destination pages or font family. The regression fixtures introduced here are
generated from invented text; they do not contain the owner's PDF, extracted
text, page renders or opening-page map.

Recovery uses evidence that other PDFs can provide: a heading's size relative
to body text, actual PDF destinations and page labels, and a printed-page offset
corroborated by real openings. A printed label can differ from the drawn title
only when the supplied or corroborated physical page has a suitable unambiguous
opening. It does not infer missing destinations from Bible-book order.

This audit adds tests and documentation only. It found no demonstrated
production defect in the additional layouts tested here, so it does not expand
the extraction algorithm or introduce OCR.

## Additional invented regressions

The five tests in `pdf_extraction_variants.rs` exercise these independent cases:

| Case | Evidence required by the regression |
| --- | --- |
| Roman front matter followed by restarted decimal page labels | Native labels `iii`, `1` and `2` resolve to physical pages 3, 4 and 5. The latter two drawn titles differ from their printed labels. All three destinations retain actual text anchors. |
| Duplicate native page labels | Two physical pages both labelled `1` do not make the printed entry choose the first one. Independent typography can retain a real heading without falsely labelling it as printed-contents evidence. |
| Two-column printed contents | The extractor first proves that the fixture has two columns. Twelve entries drawn across rows are recovered in column reading order, with correct physical targets; the right column is not treated as nested indentation. |
| Font families, body sizes and large covers | Times Roman at 8.5 pt, Helvetica at 11 pt and Courier at 14 pt retain genuine 1.4-times-body book headings despite three larger cover sizes. No printed contents or bookmark is supplied. |
| A destination with no extractable text | Three real openings corroborate a nonzero printed-page offset. A fourth entry targeting an empty appendix page appears in Contents as a page target, without a fabricated heading, text anchor or stored segment. |

Focused validation command:

```sh
cargo test --manifest-path src-tauri/Cargo.toml --locked \
  --test pdf_extraction_variants -- --nocapture
```

Result: **5 passed, 0 failed**. The font test covers three separate font/body-size
combinations. These are native-library integration tests through staged PDF
import, structure storage and Contents construction on Linux; they do not claim
Windows desktop or installer acceptance.

## Existing complementary coverage

| Area | Existing fixture suite and checked behaviour |
| --- | --- |
| Printed contents | `pdf_printed_contents.rs`: corroborated physical-page offsets, differing titles, repeated overprinting, oversized covers, user corrections and legacy saved structures. Negative cases refuse an uncorroborated title mismatch and a numbered outline inside a book. |
| Embedded destinations | `pdf_outline_destinations.rs`: named destinations in both destination stores, page-only targets with different drawn titles, hierarchy and printed labels different from physical pages. Parallel-column ambiguity is refused. |
| Text, coordinates and structure | `pdf_structure.rs`: tagged and layout-only headings, Korean Unicode in multiple columns, ambiguous precise coordinates, targetless hierarchy groups, preserved text/IDs and source-file identity checks. |
| Missing structure | `generated_contents.rs` and `pdf_structure.rs`: uniform prose does not acquire invented headings. `library.rs`: a PDF without usable text is reported as needing OCR instead of importing invented text. |
| Persistence and bounded work | `pdf_build_processing.rs`: a generated 400-page document, page-boundary cancellation, rollback, responsiveness, saved analysis and explicit retry. |

The actual-file verification recorded in
[the source-navigation report](source-navigation-20261119.md) is additional
evidence for the reported failure. The invented variants above test different
layout and metadata choices; the real document is not an executable fixture or
a special case in production.

## Conservative limits

PDF text and navigation quality vary. The passing cases establish coverage for
the tested inputs, not every PDF encoding or layout.

- A printed listing page needs at least three distinct entries. Without unique
  native page labels, a numeric offset requires at least three distinct titles
  corroborated against actual opening pages. A very short or inconsistent
  printed contents may therefore remain unresolved.
- Roman printed labels can resolve through unique native page labels. They do
  not acquire an inferred Arabic offset when that metadata is absent.
- A duplicate label, ambiguous coordinate or ambiguous opening is not enough to
  invent an anchored destination. A verified page target can still be useful
  without claiming a text mapping.
- Heading recovery still requires usable text and evidence of section
  structure. Image-only pages, absent or broken text mappings, and typography
  indistinguishable from ordinary prose are not made reliable by a font-size
  correction. OCR is outside this change.
- Column recognition remains geometry-dependent. This suite covers a genuine
  two-column listing alongside the existing multilingual column regression;
  it does not establish general reading order for arbitrary magazine layouts,
  intersecting tables or decorative text.

These limits preserve honest destinations and make the fallback behaviour
explicit. Further production changes should be driven by a reproducible input
and an invented regression that demonstrates the failure, while keeping private
source documents out of the repository.
