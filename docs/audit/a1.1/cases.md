# A1.1 cases: the 8 incorrectly verified verses, the 16 refused ones, and what the new rules refuse

All on disclosed documents (Sets B–G, recon and development corpora). None of
this is independent evidence. The structure-aware reading is still a research
prototype, off unless `structural::stage_documents` is called.

## 1. The 8 verses verified with wrong text at A1 (`205a453`)

Each was verified with words missing, because the recogniser had left out
text that the edition's own source places inside the verse. In the original
document, that text stands between the verse and the next one, and nothing
in the document sets it apart: it is in the verses' own paragraph style and
is not in a heading element, a note or a floated block.

| Edition | Verse | What follows the verse in the original | A1.1 |
|---|---|---|---|
| `francl-rebooklibepub` | PRO 22:16 | "II. — RECUEILS SUPPLÉMENTAIRES. PAROLES DES SAGES …" | refused, `UnaccountedText` |
| `francl-rebooklibepub` | PRO 24:22 | "AUTRES PAROLES DES SAGES." | refused, `UnaccountedText` |
| `francl-rebooklibepub` | PRO 29:27 | "PAROLES D'AGUR." (before the untraced 30:1) | refused, `UnaccountedText` |
| `francl-rebooklibepub` | PRO 30:33 | "PAROLES DU ROI LAMUEL." | refused, `UnaccountedText` |
| `francl-rebooklibepub` | SNG 1:7 | "LE CHŒUR." | refused, `UnaccountedText` |
| `francl-rebooklibepub` | SNG 3:5 | "LE CHŒUR." | refused, `UnaccountedText` |
| `francl-rebooklibepub` | SNG 5:8 | "LE CHŒUR." | refused, `UnaccountedText` |
| `porbr2018-epub` | HEB 13:25 | "Escrita da Itália para os Hebreus, e enviada por Timóteo" (the subscription) | refused, `UnaccountedText` |

The rule (`structural::unaccounted_text`): every letter of the original
between a traced verse's last character and the next verse's first must be
accounted for. A letter is accounted for when it is:

* excluded from a verse: a note caller, a note sign, or a removal that is
  grounded as described in §2;
* inside an element the document's structure sets apart: `h1`–`h6`, a note
  container, `nav` or `aside`, hidden content, a floated block, or a block a
  note caller leads to;
* inside an element whose class is set apart by measurement in this
  document, or (A1.1, second round) a classless inline element measured by
  its name.

Measurement means the class or name occurs at least three times in the book
and never around a **verified** verse's words.

Nothing is removed, rewritten or joined: a verse with unaccounted letters
after it simply stays unverified, stored as the existing `boundary` reason.

## 2. The 16 `porbr2018` verses refused for inline references

The recogniser takes inline quotation references ("Oseías 6:6") out of verse
text. At A1 it did not say where it had taken them from, so the tracer could
not account for them, and 16 correct verses were refused.

A1.1:

* **The recogniser reports the position.** `Apparatus::within_verse` gives
  the number of the verse's non-space characters standing before each piece
  it removes. Pieces with a position are no longer merged. In the frozen
  reading the only effect is the apparatus piece count of 8 editions.
* **A removal is grounded only when exact.** It needs one whole inline
  element in the verse's place, whose text is exactly the piece, and that
  the document sets apart (§1). The tracer then excludes that element as
  `ExcludedKind::Apparatus`, and the trace must find each removal after
  exactly the reported number of characters (`removals_in_place`).
* **A reference after the verse's last word** must begin where the verse
  ends; only note callers or note signs may stand between. It is reported as
  an excluded range too (`provenance::removed_range`).

Result: all 16 are verified, each traced exactly, and the id check agrees in
15 of them (the 16th has no id evidence). In the whole `porbr2018` book 165
removed ranges are reported, every one an exact range of the stored file
(`removed-ranges.md`). No other disclosed EPUB has grounded removals.

## 3. What A1.1 refuses that A1 verified (coverage cost)

The verified verses the new rule withdraws were correct at A1. Each was
checked by hand by edition, from the original's markup:

| Edition | Verified A1 → A1.1 | What follows the refused verses |
|---|---|---|
| `frasbl-rpandocepub` | 27 783 → 27 666 | Psalm titles in `<p><em>…</em></p>`. `em` also carries verse words in this book (*Sélah*), so it is not set apart, and the title is in no heading element. |
| `F12` (Set F) | 1 188 → 1 140 | Psalm titles in a classless `<p>`, the verses' own paragraph: the same shape as Crampon's errors. |
| `francl-rebooklibepub` | 11 783 → 11 764 | The 7 wrong verses of §1, and 12 correct verses before the same shapes: speaker labels ("L'ÉPOUSE."), psalm and section titles in the verses' own paragraph style, and note texts at a chapter's end that no caller in the document leads to. |
| `fraLSG-epub` | 30 797 → 30 795 | A verse before an untraced verse whose opening could not be found. |
| `latVUC-epub`, `F10` | −1 each | DAN 12:13 before Susanna (DAN 13), which no recognised verse holds. |

The remaining `UnaccountedText` gates fall on verses the recogniser itself
had already refused (G12: 149, `spapddpt-repub2`: 2, …), so they change no
count.

## 4. Faults found and fixed during A1.1

1. **Trailing removals were refused.** A reference removed after a verse's
   last word stood outside the trace (porbr2018: 146 untraced verses after
   the first A1.1 commit). Fixed as in §2.
2. **The verse before an untraced verse was judged too far.** The untraced
   verse's start was its segment's start, which can lie before the verse
   being judged; the gap then ran over the untraced verse's own words. It is
   now where that verse's own opening characters are found in the original
   (`provenance::find_opening`), else the segment start as before.
3. **Byte ranges were 3 bytes short in 12 EPUBs.** Their entries begin with
   a UTF-8 byte-order mark, which quick-xml skips without counting.
   Character ranges, tracing and verification were unaffected; the byte
   ranges of A1 and of `e96d241` for those books were wrong. Fixed in
   `epub_source::read_entry_structure`, with a test.

Each has a test that fails on the previous code and passes now
(`tests/recognition_a1_1_accuracy.rs`, 12 tests).
