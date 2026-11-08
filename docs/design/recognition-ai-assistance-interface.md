# Optional AI assistance for Bible recognition: interface proposal

**Status:** design only, for the owner's review. Nothing here is
implemented, and no AI service is called anywhere in the recognition work.
It builds on Phase A1's structural reading and exact source spans
(`docs/scripture-recognition-a1-a2.md`). It follows the architecture
contract in Issue #36 and the assessment in
`recognition-architecture-assessment.md` §4.

## 1. Fixed rules

1. **Off by default.** The structural, non-AI reading must work completely
   on its own. AI assistance is enabled per import, by the person, with the
   provider named.
2. **Local first.** Local models are offered first: the app already speaks
   to local servers (Ollama, LM Studio, compatible ones) through its
   assistant provider layer. A cloud provider needs a separate consent for
   that import. The consent shows:
   * the provider;
   * which excerpts would be sent, and how many characters;
   * that the edition's licence may forbid sending its text to a third
     party.

   A licence statement found in the document ("all rights reserved", "may
   not be transmitted") blocks cloud use until the person confirms they hold
   the rights.
3. **Labels and pointers only.** A model answers with labels on spans that
   already exist in the original, never with text. Any text a model returns
   is discarded.
4. **Deterministic validation.** The application checks every proposal
   against the document and the canon before it can have any effect. A
   proposal that fails a check is discarded and recorded.
5. **Never trusted more.** A verse that AI evidence took part in keeps
   provisional standing and the same review as any other. It is marked as
   AI-assisted, and AI evidence can never verify a verse on its own: the
   structural checks must still pass.

## 2. What may be asked

Each question is about evidence the structural reading has already found,
and is identified by **source pointers**. A pointer is an entry path and
character range of `epub_source::Entry` (or, for PDFs later, a page and
glyph range from the shared extraction interface).

| Question | Asked when | Answer allowed |
|---|---|---|
| `classifyBlock` | A block whose role the structure leaves open: a short line between verses, a block of a class with no stylesheet evidence. | One of `verseText`, `heading`, `runningHead`, `note`, `crossReference`, `caption`, `pageFurniture`, `unknown`. |
| `classifyLink` | A link whose role is `Ambiguous` (it leads nowhere, or is an undirected pair). | `noteCaller`, `verseNumber`, `navigation`, `unknown`. |
| `identifyBookTitle` | A heading the canon tables do not name (G02's Swedish titles, G14's Latin Gospel titles). | A USFM book code from the canon list, or `none`. |
| `readingOrder` | A page or block sequence the layout could not order (PDF, later). | A permutation of the given block ids. |
| `mapSource` | The person asks where a verse is in the original. | A pointer to existing spans, for the person to confirm. |

None of these questions asks for verse text, a correction, a translation,
a normalisation or a missing word. Those are not part of the interface.

## 3. Request and response shapes

```jsonc
// Request (built by the app; the excerpt is copied from the original by range)
{
  "question": "classifyLink",
  "document": { "fileSha256": "…", "entry": "OEBPS/text/ch08.xhtml", "entrySha256": "…" },
  "subject": { "charStart": 1204, "charEnd": 1207 },            // the link's visible text "[2]"
  "context": [ { "charStart": 1100, "charEnd": 1320 } ],          // excerpts, by range, minimal
  "structure": { "href": "#Footnote_78", "evidence": "noTarget", "declared": false },
  "allowed": ["noteCaller", "verseNumber", "navigation", "unknown"]
}

// Response (from the model, untrusted)
{ "label": "noteCaller", "confidence": 0.82, "pointers": [ { "charStart": 1204, "charEnd": 1207 } ] }
```

**The app's validation of a response:**

* `label` must be in `allowed`; anything else gives `unknown`.
* Every pointer must lie inside the request's subject or context ranges of
  the same entry, and must be an exact range of the original.
* Any other field, and any text, is ignored.
* The proposal must then pass the domain checks for its question:
  * **`classifyLink` as `noteCaller`:** the chapter is read again with the
    link excluded, and the result must pass every structural check: tracing,
    ids, verse sequence and chapter length.
  * **`classifyLink` as `verseNumber`:** the number must fit the verse
    sequence exactly.
  * **`identifyBookTitle`:** the book's chapters and verse counts must fit
    the proposed book's versification, and no other heading may claim that
    book.
  * **`readingOrder`:** must be a permutation of the given ids.

  Then the shared checks apply: every verse in the affected region must
  trace to exact spans, and the region's verse text must not change except
  by inclusion or exclusion of the subject's own range.

A proposal that passes still does not verify anything by itself. It only
removes the specific uncertainty it was asked about, and every structural
check then runs again.

## 4. Provenance to record

Each proposal, accepted or not, would be recorded with:

* question, subject and context ranges;
* model identifier and version as the provider reports them;
* provider kind (local or cloud);
* a digest of the request;
* the label, confidence and pointers;
* validation outcome (`accepted`, `rejected:<check>`);
* time.

Verses whose reading depended on an accepted proposal list its record.

Storage would need a table for these records and an evidence-kind column
for verse places. This is **not** reserved here. Migration 0027 belongs to
the Sources library's PDF structure, and 0028 is reserved for Codex's
provisional Bible-import standing and gates. An AI-evidence migration would
be proposed to Codex as a later number once 0028 lands.

## 5. Measurement before any adoption

* Run on the disclosed sets with the same checksummed inputs. Report AI-off
  and AI-on side by side, with accuracy (verified wrong text, nonexistent
  verses, wrong book or chapter) and coverage recorded separately.
* **Gate:** accuracy failures must stay at zero with AI on. Only coverage
  gains count as value, and only on a blind set built after the AI
  component is frozen.
* Also report:
  * the share of proposals rejected by validation (a high rate means the
    questions are badly posed);
  * latency and memory with a local model;
  * how much text each import would send to a cloud provider.
