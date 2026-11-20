# Phase A1.5: regression on the 113 disclosed editions

The A1.5 candidate against the A1.3 candidate (`7c7565a`, PR #51) and the
frozen v0.24.7 recogniser (`c96756d`), on identical checksummed inputs
(`CORPUS_OK`, `SETG_OK`), structure-aware EPUB reading. Sets B-G are
disclosed: regression evidence, not independent evaluation.

## 1. Totals

| | v0.24.7 | A1.3 | **A1.5** |
|---|---:|---:|---:|
| Verified, correct text | 1 956 673 | 1 977 517 | **1 980 257** |
| **Verified, wrong text** | 4 172 | 9 | **9** |
| **Verified, not in the edition** | 7 | 0 | **0** |
| Refused (correct text, not verified) | 44 832 | 45 397 | **45 376** |
| Caught (wrong text, not verified) | 17 723 | 17 706 | **17 518** |

* The 9 wrong are `spablm-docx` v1's, a defect of that test document
  (`docs/audit/candidate-a1.3/ledger.md`); on the corrected v2, 0.
* The gain is one edition (`engbsb-rwpdf2col`, +2 740 verified, 0 wrong):
  a note letter and the next verse number drawn as one raised run were read
  as a plain number (commit `179f72f`). A1.3 had withheld 230 of its verses
  for that reason and refused 20 chapters.

## 2. Editions that changed from A1.3

| Set | Edition | Correct | Wrong | Not in edition | Refused | Refused chapters | Why |
|---|---|---|---|---|---|---|---|
| e | `engwyc2017-epubapoc` | 0 → 0 | 0 → 0 | 0 → 0 | 0 → 0 | 1 → 0 | "Epistle Of Jeremiah" left out as the book outside the 66 (`2536495`) |
| bc | `engbsb-rwpdf2col` | 22133 → 24873 | 0 → 0 | 0 → 0 | 2557 → 2536 | 67 → 47 | raised caller and verse number split (`179f72f`) |

Every other edition is identical to A1.3 in every measure above.

Field by field, ignoring time, memory and the new `numbering` field, the
edition reports are identical to those before the hardening commit
(`2536495`) except `engwyc2017-epubapoc`. Provenance reports are identical
except that edition and `francl-rebooklibepub`, where 2 533 links outside
any verse are now conflicts instead of callers (no verse changes).

## 3. Fixes shown necessary

Each fix of the hardening commit has a test. Each test was run with only
its fix reverted and failed, then passed with it:

| Test | Without the fix |
|---|---|
| `a_word_linked_to_a_note_is_not_taken_out_as_a_caller` | the verse verified without the linked word |
| `characters_a_document_writes_never_act_as_the_readers_marks` | words vanished from a verified verse |
| `a_verse_cut_by_an_xml_error_is_never_verified_short` | the cut verse verified short |
| `a_word_body_that_is_not_well_formed_is_refused_whole` | the DOCX was read in part |
| `a_link_written_in_capitals_is_counted_by_both_readers` | a caller's role landed on the wrong link |
| `an_entry_that_inflates_beyond_its_declared_size_is_not_read` | 100 000 bytes read from an entry declaring 10 |
| `a_spine_naming_a_document_twice_reads_it_once` | every verse read three times; chapters refused |
| `a_form_that_draws_itself_is_refused_before_it_is_drawn` | **the process died of a stack overflow (SIGABRT)** |
| `no_documents_is_an_error_never_a_panic` | panic |
| `an_image_s_description_is_not_text` | image alt text read as verse text |
| `every_name_of_the_letter_of_jeremiah_is_the_book_outside_the_66` | read as Jeremiah |
| `a_raised_note_letter_run_together_with_a_raised_verse_number_is_marked_as_both` | read as a plain number |

The incremental prefix (review finding 4) has no test of its own: its
evidence is the regression above, identical on all 113 editions.

## 4. Numbering (new)

`scripture::numbering::assess` marks books whose printed numbering
departs from the reference. Editions with at least one such book:
56 of 113. Per edition: `data/editions.json`
(`numbering_departs`). It flags genuine traditions (Hebrew-style
chapter divisions, Vulgate Psalms, WEB's Romans 14:24-26) and also
misread numbers inside refused chapters: a reason to distrust alignment,
never to trust it.
