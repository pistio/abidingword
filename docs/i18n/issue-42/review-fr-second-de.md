# Second consistency pass — German, reviewed by French-pass AI agent

**AI review only; no human native-speaker approval.** This is an independent consistency review of the German editor’s final changes and selected retained high-risk text, not a second exhaustive rereading of every German leaf or Windows acceptance.

Reviewed the complete German locale diff and its per-key ledger, including the additional native format/book enum labels and source-location captions. Cross-checked the retained backup/restore, irreversible deletion, Source cancellation/timeout, leader-versus-participant notes, Bible Study/Bible Teaching and POZETEK/Kern wording.

## Decisions

- **Accepted:** familiar **du** remains consistent; `Erarbeitet` → `Erarbeite` corrects the direct address without changing teaching guidance. `altersgerecht und verständlich` fixes a non-idiomatic age metaphor while retaining the text’s meaning.
- **Accepted:** **Bibelstudium/Bibelstudien** and **Bibellehre/Bibellehren** remain distinct; **Quellen/Bibliothek/Unterlagen/Präsentation** are not conflated. **Als eigene Vorlage speichern** now matches the actual approved action. `Quelltext` was correctly clarified as `Text der Quelle`, avoiding confusion with program source code.
- **Accepted:** noun-first count labels such as `Bücher: {{count}} · Verse: {{verses}}` handle independently varying totals without introducing incorrect singular/plural forms. All named interpolations remain intact.
- **Accepted:** backup copy privacy, API-key exclusion, replacement on restore, retained originals/citations, deferred cancellation and the non-cancellable final-save warning retain their full meaning. Leader text remains private unless explicitly shown to participants.
- **Accepted:** the selector label `Übergeordnete Sammlung` identifies the actual parent-collection choice. Source diagnostic scope and sorted-candidate wording remain precise; the retry instruction matches **Struktur erstellen**.
- **Resolved finding:** `neutral` → `neutrum` correctly identified the intended grammatical gender, but lowercase `neutrum` was not the best German form. The German editor accepted **sächlich**, parallel to the adjectives **feminin/maskulin**. The locale and ledger now contain **sächlich**. `Personal-` → `personal` correctly removes a dangling compound prefix from a standalone grammatical feature.
- **Accepted:** additional-book display labels preserve USFM identity, source numbering, Greek/Latin qualifiers, and the separation of Esdras from Ezra. These are translations of native warning enum labels, not changes to Scripture, the 66-book registry or canon policy. Confessional numbering alternatives remain an owner specialist-review context.
- **Preserved:** the accepted Ten-Step method, Bible book registry, Kern triplet and source-backed examples were not substantively rewritten. No new theological explanation was introduced.

No unresolved blocking German consistency issue was found within this scope. Qualified human proofreading, specialist theological terminology confirmation and real Windows/OS-owned dialog acceptance are not claimed.

Supplemental exported Word style-name review: **Feldbezeichnung**, **Schreiblinie** and **Schreiblinie (letzte)** are natural, concise German labels for the field-heading style and ruled response-space paragraphs, including the last-line distinction. Only style-pane display names change; stable style IDs, formatting and user text remain untouched. No new finding.
