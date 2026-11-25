# Structured PDF Reading — separate feature after A.1

Status: outstanding functional feature, independently reproduced by the owner. A.1 does not implement it. Start only after the immediate correction review; keep its implementation and tests separate from Milestone B and Claude's Bible recognition/provisional-import work.

## Contract

Original bytes remain immutable. Every displayed text span retains its source ID, physical page, original text offsets and bounding coordinates. Original mode and citations remain available for every Reading location. Printed page labels are metadata, never physical PDF page indices. Neither a guessed heading nor a guessed biblical quotation becomes authoritative text.

## Inspect and reuse

Inspect PDF.js extraction and the native PDF ingestion pipeline, stored segments and citation navigation before adding representations. Prefer tagged-PDF structure trees and marked-content associations. Reuse existing EPUB/DOCX headings and Contents infrastructure rather than flattening their structure. Measure extraction on licensed/disposable fixtures before choosing storage changes; never edit historical migrations 0001–0026.

## Incremental implementation

1. Collect a provenance-preserving layout representation: page, text items, font/size, bounding boxes, marked-content IDs and available structure-tree roles. Retain extraction order and exact source text separately from reading order.
2. Render trustworthy existing tags as titles, headings, paragraphs, quotations, lists and notes. Keep semantic reading order distinct from geometric placement.
3. For untagged documents, propose paragraph boundaries and hierarchy from repeatable typography and spatial evidence. Record the rule, confidence and whether structure was provided, inferred or corrected by the user. A large bold line alone cannot establish theological relevance.
4. Identify repeated running headers/footers and candidate footnotes conservatively using page position, repetition, size and explicit markers. Keep all original text reachable; uncertain classifications stay visible and correctable. Handle multi-column pages and mixed typography explicitly.
5. Expose a document Contents outline and optional user corrections in app-owned metadata; never rewrite the source PDF. Corrections must preserve citation anchors and be reversible.
6. Navigate between Reading, Original and citation targets using the same source/page coordinates; changing structure must not change existing citation identity.

## Acceptance fixtures and gates

Use redistributable or invented tagged and untagged PDFs: two columns, repeated headers, Roman printed front matter, footnotes/endnotes, numbered headings that resemble Scripture references, multilingual prose and tables. Add EPUB/DOCX regression fixtures. Assert exact retained text/bytes, page-coordinate round trips, deterministic extraction, explicit uncertainty, correction persistence and backup/restore. Compare complete combined-pane layouts at 900, 1100 and 1440 pixels, including keyboard navigation and long Contents. Include native Windows checks and owner-provided examples when available with permission.

No recognition verdict is inherited from these tests. They establish document layout behavior only. A section heading provides navigation evidence; commentary relevance requires its own evidence and must never be inferred from literal Scripture matches.
