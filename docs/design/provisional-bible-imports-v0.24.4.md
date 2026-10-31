# Provisional Bible imports — a safety design for schema 26 (v0.24.4)

**Status:** design for implementation on the schema-26 integration branch
(`integration/windows-library-stage3-teaching-study`), by Codex, in
coordination with the recognition work on `claude/scripture-recognition-v0.24.4`.
Nothing here is implemented on the recognition branch; the recognition branch
is not to be merged first.

## 1. Why

v0.24.3's independent evaluation (withheld sets B and C, `docs/audit/v0.24.3/`)
showed that a Bible read from a document can hold verses marked as checked that
carry a neighbour's words or a note's text, on layouts the recogniser has not
seen. Today nothing downstream can tell such a Bible from one loaded from USFM:
**no reader of `bible_verse_texts` checks verification, `basis`, `kind` or
`import_status`** (survey of the integration branch, §7). The only protection
is upstream, at import, where unverified verses go to `bible_verse_places`
instead of `bible_verse_texts`. That protects against what the recogniser
*knows* it cannot verify, not against what it gets wrong with confidence.

The product therefore needs a standing for Bible text that every consumer
respects, independent of how good the recogniser is.

## 2. Three things, kept distinct

| | What it is | Where it lives today | Shown as |
|---|---|---|---|
| **Document reading** | The file's own text and pages, read and searched as a document | `bible_documents`, `bible_document_segments`, `bible_document_search` | the document view; never a verse |
| **Recognised, unverified** | Verses whose place is known but whose text the recogniser could not verify | `bible_verse_places` (reason), no text in `bible_verse_texts` | "not checked", with a page to open (reader, passages) |
| **Verified Scripture** | Verse text the app may quote as Scripture | `bible_verse_texts` | plain verse text |

Missing: a standing for the *Bible as a whole* that says whether its verified
verses came from a source that is verified by nature (USFM, OSHB, MorphGNT,
bundled) or from recognition — which is **provisional** until the person has
had the means to judge it, and stays provisional even then (§5).

## 3. Data model (one additive migration, 0027)

```sql
-- 0027_provisional_bibles.sql — additive; existing rows keep their meaning.
ALTER TABLE bible_translations
    ADD COLUMN text_standing TEXT NOT NULL DEFAULT 'verified'
    CHECK (text_standing IN ('verified', 'provisional'));

-- Bibles whose verses were read from documents are provisional. (Every such
-- Bible has rows in bible_verse_origins; USFM/OSHB/MorphGNT/bundled Bibles
-- have none.) Bibles with no verse text keep 'verified' as a vacuous default
-- but are never quoted: they have nothing to quote.
UPDATE bible_translations SET text_standing = 'provisional'
 WHERE id IN (SELECT DISTINCT translation_id FROM bible_verse_origins)
    OR id IN (SELECT DISTINCT translation_id FROM bible_verse_places);

-- The person's sample reviews of a provisional Bible (§5).
CREATE TABLE bible_text_reviews (
    id              TEXT PRIMARY KEY NOT NULL,
    translation_id  TEXT NOT NULL REFERENCES bible_translations (id) ON DELETE CASCADE,
    -- The Bible's revision when the sample was drawn: a later re-recognition
    -- (add_verses) makes the review stale, never silently valid.
    revision        INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    -- 'open' | 'no_problem_found' | 'problems_found'
    outcome         TEXT NOT NULL DEFAULT 'open'
                    CHECK (outcome IN ('open', 'no_problem_found', 'problems_found')),
    note            TEXT NOT NULL DEFAULT ''
);
CREATE TABLE bible_text_review_items (
    review_id       TEXT NOT NULL REFERENCES bible_text_reviews (id) ON DELETE CASCADE,
    verse_id        INTEGER NOT NULL REFERENCES bible_verses (id),
    -- 'unseen' | 'ok' | 'wrong_text' | 'wrong_place' | 'apparatus'
    judgement       TEXT NOT NULL DEFAULT 'unseen'
                    CHECK (judgement IN ('unseen', 'ok', 'wrong_text', 'wrong_place', 'apparatus')),
    PRIMARY KEY (review_id, verse_id)
) WITHOUT ROWID;
```

Why a column on `bible_translations` and not per verse: per-verse provenance
already exists (`bible_verse_origins.basis`, `bible_verse_places.reason`) and
is kept; the standing is a property of the *source*, and consumers decide per
Bible, cheaply, in the queries they already run. No existing CHECK constraint
is widened (that would rebuild the table); no row is rewritten except the
single UPDATE above.

Rollback: the migration is one transaction; the existing runner's
backup-before-migrate and the `migration_v26`/`schema26_review` patterns apply
unchanged. Tests: `tests/database.rs` `EXPECTED_TABLES` (+2 tables), the
hard-coded schema version 26 in eight test files (§7 of the survey), and the
raw `INSERT INTO bible_translations` statements (the new column has a default,
so they keep working).

## 4. The gate: one place, every consumer

Add to `db/bibles.rs`:

```rust
pub enum TextUse { Reading, Study, Evidence, AssistantContext, Teaching, Presentation }

/// What a consumer may do with a Bible's verse text.
pub fn standing(conn, translation_id) -> AppResult<TextStanding>   // Verified | Provisional | None

/// Verse text for a use. Provisional text is returned only for `Reading`
/// and `Study` (with the standing beside it); for `Evidence`,
/// `AssistantContext`, `Teaching` and `Presentation` it is returned only
/// when `allow_provisional` is set by an explicit choice of the person, and
/// then every verse carries `standing: Provisional`.
pub fn passage_for(conn, translation_id, range, use_, allow_provisional) -> AppResult<Passage>
```

`verses_between`, `passage`, `chapter`, `search`, `material_passages`,
`any_has_verse`, `concordance::search`, `bible_words::search` and
`annotations::verse_texts` route through it or carry the standing in their
result. Each consumer then:

| Consumer (survey §3) | Behaviour |
|---|---|
| Reader, passage view, search | Shows the text with a persistent **Provisional** badge on the Bible and each result; search results group provisional Bibles after verified ones. |
| Research "keep as evidence" (`study_block_create` from `ResearchBlock`) | The backend stamps `data.textStanding` from the database (not from the frontend); a provisional quotation renders with the badge and is excluded from "verified Scripture evidence" counts. Default edition picker excludes provisional Bibles; choosing one is explicit. |
| Citations / passage references | References never carry text; unchanged. Where a citation is rendered with text, the standing is rendered with it. |
| AI assistant context (`assistant/context.rs::passages`, `inquiry/study.rs::scripture`) | **Excluded by default.** If the chosen translation is provisional, the context block is replaced by a one-line note that the Bible's text is provisional and was not included; the person can switch the setting to a verified Bible. No silent fallback to another edition (never substitute). |
| Teaching materials (`material_scripture`, DOCX/Markdown export) | Allowed only by explicit choice; every Scripture block carries the badge and exports carry "(provisional text — read from a document, not verified)" after the reference. |
| Presentations (`choose_bible`, slides, PPTX/PDF) | `choose_bible` never falls back to a provisional Bible; an explicitly chosen one shows the badge in the presenter view and a footer on each Scripture slide. |
| Concordance, lemma search, annotations | Results carry the standing; the Scripture pane shows the badge. |
| Backup / packages | Unchanged: the column travels with the table; `.awproject` carries no Bible text, and retained-evidence blocks carry their stamped standing. |

The frontend filters that exist today (`ScriptureContext` `kind !== "document"`,
the presentation and research pickers) are kept but are no longer the
protection: the backend is.

## 5. Guided sample review (optional, never a promotion)

Purpose: help the person find an import problem early, not certify the Bible.

* **Drawing a sample.** From a provisional Bible, draw 12 verses: 4 at random
  from verses whose origin page also holds apparatus (`bible_document_apparatus`
  on the same page), 4 at random from verses at chapter ends or starts, 4 at
  random overall; always include verses with `basis = 'corrected'` if any. The
  sample is recorded (`bible_text_reviews` + items) with the Bible's revision.
* **Reviewing.** Each item shows the verse text beside its origin page
  (`bible_document_locate`) and the person marks ok / wrong text / wrong place /
  apparatus. Judgements are saved as given.
* **Outcome.** `problems_found` when any item is not ok: the Bible shows
  "problems found in a sample" in Settings → Bible texts and the reader, with a
  button to re-read the documents (`bible_document_recognise`) and to report
  the verses. `no_problem_found` records that fact with the date and revision.
* **What it never does.** Approving a sample does **not** change
  `text_standing`. A provisional Bible stays provisional: the standing is about
  the source, and twelve verses do not verify thirty-one thousand. The only
  verified standing is the one a verified source gives.
* A re-recognition (`add_verses`) bumps the revision; earlier reviews are shown
  as "for an earlier reading".

## 6. Interface

* A **Provisional** badge (six languages) wherever a Bible's name is shown:
  Settings → Bible texts, the Scripture pane's edition picker, search results,
  the reader's chapter title, retained evidence, material and presentation
  Scripture blocks.
* Settings → Bible texts: the standing, the last sample review and its
  outcome, "Review a sample" and "Read the documents again".
* The assistant panel: the note that Scripture was not included because the
  Bible is provisional, with a link to the setting.
* No new place where a Bible can be *promoted*; the word "verified" is
  reserved for sources verified by nature.

## 7. What the recognition branch provides and what stays separate

* The recogniser already separates found from verified (`DocumentStage.unverified`,
  `bible_verse_places`) and keeps apparatus with places. This design does not
  duplicate that: provisional is a standing *above* those per-verse facts.
* The recognition branch's migration 0023 is already on the integration branch
  (PR #12). Migration 0027 is independent of recognition changes and can be
  implemented and tested on the integration branch now, with disposable data.
* Coordination: Codex implements §3–§6 on the integration branch; the
  recognition branch continues on its own; neither merges into `main` or the
  Windows development branch without the owner's authorisation.

## 8. Tests the implementation must add

1. Migration 0027 forward on a schema-26 database with one USFM Bible, one
   document-read Bible with places, one document-only Bible: standings
   verified / provisional / verified-vacuous; all verse texts, origins, places
   unchanged; a failing later migration rolls back and leaves 0027 intact.
2. `passage_for(.., AssistantContext, false)` on a provisional Bible returns
   nothing and the context renders the note; `assemble()` never contains
   provisional verse text unless `allow_provisional`.
3. `choose_bible` never picks a provisional Bible by fallback.
4. Research keep: the stored block's `data.textStanding` equals the database's
   standing, whatever the frontend sent.
5. Sample review: approving all items leaves `text_standing` unchanged;
   `problems_found` is shown; `add_verses` makes the review stale.
6. Exports (DOCX, PPTX, PDF) of a provisional Bible's passage carry the marker.
7. Frontend: badge rendering in the six places above; pickers' defaults.
