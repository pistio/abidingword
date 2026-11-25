# Word custom-style display names — source and boundary evidence

Generated Word files previously exposed nine fixed English custom paragraph-style names in the styles pane. The interface now supplies reviewed localised names with the transient export `format.styleNames` map. Only those nine `w:name` values change. `w:styleId`, references from `w:pStyle`/`w:basedOn`, built-in Normal/Title/heading 1 names, all paragraph text, private-material filtering, numbers and Scripture remain unchanged. Built-in names stay invariant for Word itself to localise.

The optional map retains compatibility with old callers: absent, whitespace-only or unusable display names retain their existing English fallback. No format or style-label preference is stored in a teaching, manuscript, source or database. The proofing language still comes from `formatLocale()` (British English for `en`), while the existing system-paper-region policy is retained.

`MaterialView.docxFormat(t)` supplies six existing reviewed role labels and three newly reviewed Word-specific labels. The latter were individually reviewed by all six language agents; French independently checked their consistency. Korean's three were also checked in [the separate second-pass report](review-pt-second-ko.md).

| Role / stable style ID | Locale-key source | en-GB | de | ko | fr | es | pt-BR |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `reference` / `AwReference` | `presentation.editor.passage` | Passage | Bibelstelle | 본문 | Passage | Pasaje | Passagem |
| `notice` / `AwNotice` | `scripture.bibles.field.notice` | Notice | Hinweis | 고지 사항 | Mention | Aviso | Aviso |
| `label` / `AwLabel` | `studies.materials.wordStyles.label` | Field label | Feldbezeichnung | 항목 제목 | Libellé de champ | Etiqueta de campo | Rótulo de campo |
| `message` / `AwMessage` | `studies.teaching.shape.points.message` | Central message | Kernbotschaft | 핵심 메시지 | Message central | Mensaje central | Mensagem central |
| `question` / `AwQuestion` | `studies.qa.question` | Question | Frage | 질문 | Question | Pregunta | Pergunta |
| `scripture` / `AwScripture` | `studies.inquiry.block.scripture` | Bible text | Bibeltext | 성경 본문 | Texte biblique | Texto bíblico | Texto bíblico |
| `note` / `AwNote` | `scripture.marks.result.note` | Note | Notiz | 메모 | Note | Nota | Nota |
| `line` / `AwLine` | `studies.materials.wordStyles.writingLine` | Writing line | Schreiblinie | 답변 줄 | Ligne d’écriture | Línea de respuesta | Linha de resposta |
| `lineLast` / `AwLineLast` | `studies.materials.wordStyles.writingLineLast` | Writing line (last) | Schreiblinie (letzte) | 마지막 답변 줄 | Ligne d’écriture (dernière) | Última línea de respuesta | Última linha de resposta |

The reused **Note** label is the generic verse/source-note noun, appropriate for the Word style that formats Scripture notices and attribution. It does not relabel Study findings or add private teacher notes to any material. **Bible text** deliberately uses the reviewed content-text label rather than the slide-kind label, because the German slide-kind label **Bibelstelle** would duplicate the reference style. All nine display names are distinct in every language. **Notice** is distinct from **Note** in all six languages; the response line and its last-line style also remain distinct.

This table records the current application-supplied map from `MaterialView.docxFormat(t)`, including the actual existing `studies.inquiry.block.scripture` value (**Bible text** in English). A native fixture or older caller using `Format::default()` instead retains the compatibility fallback **Scripture** from the native style template. Those are separate export inputs; the fallback does not replace the application's localised map. The boundary test resolves this existing reviewed locale key with `t(...)` while still asserting the fixed British proofing-language policy and all other style names.

## Verification scope

- Native focused tests in `materials/docx.rs` cover all nine custom names, escaping of XML metacharacters/quotes/control characters, preservation of literal `{LANG}` inside names, insertion that cannot reinterpret one name as another, old-client defaults, fixed built-in names/style IDs, and exact document-body equality in a rendered ZIP package.
- The existing Word-save UI test spies on the actual API call before clicking Save. It checks all nine English display names plus `format.language = en-GB` while `navigator.language = en-US`; the original exact derived-material and no-LEADER/no-PRIVATE assertions remain intact.
- All six mapping columns above were read from current locale files, without changing locale IDs, Bible identities, source quotations or stored writing. Coordinating native/full-suite logs provide execution results; this source table does not itself claim Word desktop, Windows or human-native acceptance.
