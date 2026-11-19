# Independent review: security, resources and robustness (A1.5)

**Reviewer:** a separate agent, given read-only access to the code at
`7c7565a` and no access to earlier reports. It read the code; it built
and ran nothing. Its findings are its own; the triage column is the
engineering response. Each "Fixed" has a test; which tests were run and seen
to fail on the code before the fix is listed in
`docs/audit/candidate-a1.5/regression.md` §2.

"Confirmed" = the reviewer read the path end to end. "[A1]" = the
structure-aware EPUB path, reached only from tests at `7c7565a`.

| # | Sev | Status | Where (at 7c7565a) | Finding | Triage |
|---|---|---|---|---|---|
| 1 | High | Confirmed | `epub.rs:428-433` (`read_entry`, also DOCX) | A zip entry's *declared* size is checked; the inflated size is not. A small entry inflating to gigabytes runs until out of memory. | **Fixed**: read stops one byte past the declared size; test `an_entry_that_inflates_beyond_its_declared_size_is_not_read` |
| 2 | High | Confirmed | `epub.rs:275-370` | The spine is not de-duplicated: a million `<itemref>` to one entry re-reads it each time. | **Fixed**: spine read once per document, 120 MB of spine text per book; test `a_spine_naming_a_document_twice_reads_it_once` |
| 3 | High | Confirmed | pdf-extract 0.12.1 `Do`, `get_inherited` | A form XObject that draws itself, or a `/Parent` cycle, recurses until the stack overflows; `catch_unwind` cannot catch that. | **Fixed**: form cycles, `/Parent` cycles and forms over 32 deep refused before drawing (`pdf::recursion_hazard`); test `a_form_that_draws_itself_is_refused_before_it_is_drawn` |
| 4 | High | Confirmed | `structure.rs:2257` | Each digit run copies its unit's whole prefix; a 5 MB one-line file costs ~10¹² byte copies. | **Fixed**: the unmarked prefix is kept up incrementally, the same left-to-right reading |
| 5 | High | Confirmed | lopdf 0.42 `object.rs:794-816` | Flate/LZW streams inflate without limit while loading. | **Not fixable in-process**: integration requirement (PDF reading in a separate process with a memory cap; handover §6) |
| 6 | High [A1] | Confirmed | `epub_source.rs:707-710`, `provenance.rs:264-274` | N unclosed or nested links each hold a label of their whole span: N × text memory. | **Open**: structure-aware path only; bounded by the 20 MB entry cap, but N × text can still be large. Documented |
| 7 | Medium | Confirmed | `structure.rs:3722-3742`; no adapter strips U+E000–E004 | The private-use marks the layout inserts are in-band: a literal U+E002…U+E003 in a document makes the words between vanish from a verse. | **Fixed**: U+E000-E004 in a document become U+FFFD in every reader; a verse holding a private-use or replacement character is never verified (`noteSign`); test `characters_a_document_writes_never_act_as_the_readers_marks` |
| 8 | Medium | Confirmed | `epub.rs:830,937-946`; `epub_source.rs:540`; `pdf_layout.rs:442` | EPUB text under `hidden`, `aria-hidden`, `<template>`, SVG `<title>`/`<desc>` is read as body text; PDF invisible text likewise. | **Open**: documented as unsupported (handover §7) |
| 9 | Medium [A1] | Confirmed | `provenance.rs:467-468`, `epub_source.rs:725-727` | Any link leading into notes is a note caller whatever its text: `Thou shalt <a href="#n1">not</a> kill` verifies as "Thou shalt kill". | **Fixed**: a link into the notes is a caller only with evidence (declared, raised, bracketed, note-like class, or signs); otherwise a conflict and the verse is withheld; test `a_word_linked_to_a_note_is_not_taken_out_as_a_caller` |
| 10 | Medium [A1] | Confirmed | `epub.rs:856` vs `epub_source.rs:526-536` | The two readers count links differently (`HREF` upper case), so roles can land on the wrong links. | **Fixed**: attribute names compared in any case by both readers; test `a_link_written_in_capitals_is_counted_by_both_readers` |
| 11 | Medium | Confirmed | `epub.rs:948`, `epub_source.rs:485`, `docx.rs:939,1952` | An XML error silently ends the part; the import looks complete. | **Fixed**: EPUB: where an XML error cuts a part, U+FFFD marks the cut and the verse is withheld; DOCX: a body that is not well formed is refused whole; tests `a_verse_cut_by_an_xml_error_is_never_verified_short`, `a_word_body_that_is_not_well_formed_is_refused_whole` |
| 12 | Medium | Confirmed | several | Quadratic loops over input sizes (minutes at the size caps). | **Partly fixed** (finding 4); the rest documented |
| 13 | Medium | Suspected | `pdf_layout.rs` | Layout loops quadratic per page up to 50 000 spans. | Documented |
| 14 | Low | Confirmed | `pdf.rs:91,103,128` | Parts of PDF loading run outside `catch_unwind`. | **Fixed**: the whole PDF read is inside `catch_unwind` |
| 15 | Low | Confirmed | `import.rs:1873`; `provenance.rs:260` | `files[0]` panics on an empty list; an `assert!` on entry size. | **Fixed**: an empty list is an error (`import.noFiles`); test `no_documents_is_an_error_never_a_panic`. The `assert!` in `EntryMarks::new` cannot trigger under the 20 MB entry cap |
| 16 | Low | Confirmed | `markdown.rs:176-182` | Image alt text is read as paragraph text. | **Fixed**: test `an_image_s_description_is_not_text` |
| 17 | Low [A1] | Suspected | `provenance.rs:380-394` | A naive search rescans for each segment not found. | Documented |

**No cancellation, timeout or progress hook exists** anywhere in scope.

**Checked and sound (reviewer):** no zip-slip (entry names never reach
the filesystem); quick-xml 0.42 expands no DTD or external entities;
EPUB readers iterative, DOCX nesting capped; `u32`/`u16` casts safe under
the 200 MB / 400 MB / 100-file caps applied before reading; guarded UTF-8
slicing; encrypted PDFs refused; DOCX deleted, moved-from, hidden text,
comments and headers not read; metadata never verse text.

Pinned: zip 2.4.2, quick-xml 0.42.0, lopdf 0.42.0, pdf-extract 0.12.1,
regex 1.13.1.
