# Native-source language audit

This AI source-context audit supplements all six individual locale reviews. It does not claim human native-speaker approval or editorial approval of arbitrary imported Scripture, owner material or AI output.

The companion `review-de-native-source-decisions.json` classifies all 1,978 sentence-shaped lexical tokens from the initial branch Rust-source lexer (baseline `64dadc0d1bb263362aa384951612e9395d705725`). The older inventory's 4,387 baseline regex candidates include cross-string spans and are a search aid, not 4,387 genuine visible sentences. SQL, protocol/XML, tests and internal diagnostics are classified as such, not represented as native-language editorial rereads. Test classification uses balanced cfg(test) item bodies rather than treating everything after the first cfg(test) as test code; later production code remains in scope.

Runtime findings and dispositions:

- Native coded errors and import issues pass stable code/kind and parameters to `localizeMessage`/`localizeError`; the English native text is a fallback. Existing unknown-code safety remains intact.
- Unsupported-format diagnostics exposed English enum values “Web page” and “OSIS and USX”. A guarded map now localises only these generated enum values in the applicable warning. Imported titles and filenames are preserved.
- Additional-book warnings exposed English canonical names. A separate 19-code display map now localises only those known warning paths, preserving the recogniser, numbering, Greek/Latin distinctions, canon identities and main book registry. Unknown code/name pairs remain verbatim.
- Inquiry evidence/consent labels were generated in English. The English reviewer added transient structured label metadata and frontend localisation; evidence, questions and user writing remain verbatim.
- Wikimedia attribution's “via” wrapper was English. The English reviewer provided structured display localisation, preserving authorship/licence provenance.
- PowerPoint selection-pane/accessibility shape names, blank layout, notes placeholders and notes-theme name were English inside raw XML. They now use the active ExportLocale, with XML escaping; IDs, schema names and source text are unchanged. Notes-master proofing language now follows the export locale. Three new labels (colour band, blank, slide image) were individually reviewed in all six languages; existing text/image/design/notes labels are reused.
- Word custom style display names in `materials/docx.rs` were found in a raw XML block and are now localised through active-language styleNames. New labels for field label/writing line/last writing line were individually reviewed; existing reviewed labels supply the other names. Built-in Word style identities, custom styleIds and user text remain intact. This finding was not silently classified as machine-only XML.
- Generic provider identity fallback “Other compatible service” is now localised by the English reviewer’s guarded Inquiry display helper. Brand and model identity strings remain provenance.
- PDF diagnostic JSON intentionally contains English technical evidence/privacy/verdict text: source explicitly cites owner request #22. `PdfStructurePanel` reconstructs visible verdict/evidence labels in the interface language. This is retained as an owner-defined technical report, rather than silently translated.
- Stored default workspace/project identity matching is retained for compatibility, not rewritten as translated user content. Bible edition titles/licences and source metadata are retained exactly. Parser aliases, canonical identities, SQL and package schema values remain intact.

Verification performed by this agent: 17 targeted import-diagnostic/presentation-label tests passed and TypeScript typecheck passed including the final German compound-stem correction. Parent runs the final complete frontend/native checks and records their exact results; this document does not substitute lexical coverage for those results.

All identified application-generated runtime display gaps assigned to this agent are resolved in the shared final implementation. The focused PPTX Rust regression was added and formatted; parent records its execution with the complete native suite.

The seven final native display/export source diffs were independently inspected during integration; new metadata, attribution, custom-style and PowerPoint names have focused regressions, followed by the complete native suite and local release compilation. These final results are in verification.json; added test XML does not retrospectively inflate the initial candidate count.
