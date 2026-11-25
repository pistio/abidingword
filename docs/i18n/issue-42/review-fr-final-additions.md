# Independent final-additions consistency review — six languages

**AI consistency review only; no human native-speaker or specialist canon approval.** Reviewer: French-pass AI agent, separate from the native/source auditor who discovered these paths.

Scope: individually inspected all **30 additions per language** (180 label contexts): 19 native unsupported-book display names, two unsupported-format fragments, three source-consent location captions, three exported PowerPoint selection/accessibility labels, and three exported Word style-pane names. Read the native `ADDITIONAL_BOOKS` registry, the bounded code/format mappings in `src/i18n/messages.ts`, and each constructed unsupported-format warning. This is a scoped integration review, not another exhaustive six-language corpus rereading.

## Findings and decisions

- The 19 book IDs and their existing English identities remain exact. **ESG/DAG** identify Greek forms of Esther/Daniel; **1ES (Greek)** remains separate from **EZR**; **2ES (Latin)** remains distinct from Greek Esdras. The four numbered Maccabees labels, Psalm 151 and Prayer of Manasseh remain distinct. Local titles such as **Siracide**, **Jesus Sirach**, **집회서** and **Eclesiástico (Sirácida)** refer to the same source enum identity; they do not introduce new canon policy. No stored Scripture, canonical reference registry, author text or source title is rewritten.
- Source-caption placeholders are exact in every language: pages/paragraphs **first/last**, paragraph **number**. Page abbreviations and Korean suffixes are natural; a range does not silently become a single page/paragraph.
- PowerPoint **band**, **blank** and **slideImage** describe a colour/design rectangle, blank layout and slide thumbnail. They remain distinct from the design name, speaker notes and user-authored text. No shape IDs, export schema or slide content changes are implied. Korean **빈 화면** is an established blank-layout label, while French **Vide**, Spanish/Portuguese **En blanco/Em branco** and German **Leer** retain that layout meaning.
- Word **field label**, **writing line** and **last writing line** use natural style-pane names in all six languages. Korean **항목 제목/답변 줄/마지막 답변 줄**, German **Feldbezeichnung/Schreiblinie/Schreiblinie (letzte)**, French **Libellé de champ/Ligne d’écriture/Ligne d’écriture (dernière)**, Spanish **Etiqueta de campo/Línea de respuesta/Última línea de respuesta** and Brazilian **Rótulo de campo/Linha de resposta/Última linha de resposta** all preserve the field-heading versus ruled-response-space distinction and explicit last-line variant. These are names, not user text or style IDs. No new consistency issue.
- **Contextual format-fragment finding:** isolated labels were insufficient for sentence insertion. French **Page web** yielded **Les fichiers Page web…**, and Spanish **Página web** yielded **Los archivos Página web…**. The idiomatic context fragments are **de pages web** and **de páginas web**. Portuguese already uses a complete natural construction (**Ainda não é possível importar páginas da web…**). German changed **Webseite** to the compound stem **Webseiten**, producing **Webseiten-Dateien**. OSIS/USX remain literal technical format names with local conjunctions. See the integration status below for application of these corrections.

## Integration status

The exact final labels are shown below. The French and Spanish contextual format fragments and German compound stem are now applied by their locale owners. All six locales’ PowerPoint labels are present; every scoped addition has a concrete label below. No uncertain source interpretation was filled in with new teaching guidance.

| Key | EN | DE | KO | FR | ES | pt-BR |
| --- | --- | --- | --- | --- | --- | --- |
| `errors.importFormats.webPage` | Web page | Webseiten | 웹 페이지 | de pages web | de páginas web | páginas da web |
| `errors.importFormats.osisUsx` | OSIS and USX | OSIS und USX | OSIS 및 USX | OSIS et USX | OSIS y USX | OSIS e USX |
| `scripture.importBookNames.TOB` | Tobit | Tobit | 토빗기 | Tobie | Tobías | Tobias |
| `scripture.importBookNames.JDT` | Judith | Judit | 유딧기 | Judith | Judit | Judite |
| `scripture.importBookNames.ESG` | Esther (Greek) | Ester (griechisch) | 에스더(그리스어) | Esther (version grecque) | Ester (griego) | Ester (texto grego) |
| `scripture.importBookNames.WIS` | Wisdom of Solomon | Weisheit Salomos | 솔로몬의 지혜서 | Sagesse de Salomon | Sabiduría de Salomón | Sabedoria de Salomão |
| `scripture.importBookNames.SIR` | Sirach | Jesus Sirach | 집회서 | Siracide | Sirácida | Eclesiástico (Sirácida) |
| `scripture.importBookNames.BAR` | Baruch | Baruch | 바룩서 | Baruch | Baruc | Baruc |
| `scripture.importBookNames.LJE` | Letter of Jeremiah | Brief des Jeremia | 예레미야의 편지 | Lettre de Jérémie | Carta de Jeremías | Carta de Jeremias |
| `scripture.importBookNames.S3Y` | Song of the Three Young Men | Gesang der drei jungen Männer | 세 청년의 노래 | Cantique des trois jeunes gens | Cántico de los tres jóvenes | Cântico dos Três Jovens |
| `scripture.importBookNames.SUS` | Susanna | Susanna | 수산나 | Suzanne | Susana | Susana |
| `scripture.importBookNames.BEL` | Bel and the Dragon | Bel und der Drache | 벨과 용 | Bel et le dragon | Bel y el dragón | Bel e o Dragão |
| `scripture.importBookNames.1MA` | 1 Maccabees | 1. Makkabäer | 마카베오 1서 | 1 Maccabées | 1 Macabeos | 1 Macabeus |
| `scripture.importBookNames.2MA` | 2 Maccabees | 2. Makkabäer | 마카베오 2서 | 2 Maccabées | 2 Macabeos | 2 Macabeus |
| `scripture.importBookNames.3MA` | 3 Maccabees | 3. Makkabäer | 마카베오 3서 | 3 Maccabées | 3 Macabeos | 3 Macabeus |
| `scripture.importBookNames.4MA` | 4 Maccabees | 4. Makkabäer | 마카베오 4서 | 4 Maccabées | 4 Macabeos | 4 Macabeus |
| `scripture.importBookNames.1ES` | 1 Esdras (Greek) | 1. Esdras (griechisch) | 에스드라 1서(그리스어) | 1 Esdras (version grecque) | 1 Esdras (griego) | 1 Esdras (texto grego) |
| `scripture.importBookNames.MAN` | Prayer of Manasseh | Gebet Manasses | 므낫세의 기도 | Prière de Manassé | Oración de Manasés | Oração de Manassés |
| `scripture.importBookNames.PS2` | Psalm 151 | Psalm 151 | 시편 151편 | Psaume 151 | Salmo 151 | Salmo 151 |
| `scripture.importBookNames.DAG` | Daniel (Greek) | Daniel (griechisch) | 다니엘(그리스어) | Daniel (version grecque) | Daniel (griego) | Daniel (texto grego) |
| `scripture.importBookNames.2ES` | 2 Esdras (Latin) | 2. Esdras (lateinisch) | 에스드라 2서(라틴어) | 2 Esdras (version latine) | 2 Esdras (latino) | 2 Esdras (texto latino) |
| `studies.inquiry.location.pages` | pp. {{first}}–{{last}} | S. {{first}}–{{last}} | {{first}}–{{last}}쪽 | p. {{first}}–{{last}} | pp. {{first}}–{{last}} | pp. {{first}}–{{last}} |
| `studies.inquiry.location.paragraph` | paragraph {{number}} | Absatz {{number}} | {{number}}번째 단락 | paragraphe {{number}} | párrafo {{number}} | parágrafo {{number}} |
| `studies.inquiry.location.paragraphs` | paragraphs {{first}}–{{last}} | Absätze {{first}}–{{last}} | {{first}}–{{last}}번째 단락 | paragraphes {{first}}–{{last}} | párrafos {{first}}–{{last}} | parágrafos {{first}}–{{last}} |
| `presentation.export.labels.band` | Colour band | Farbfläche | 색 띠 | Bandeau de couleur | Banda de color | Faixa colorida |
| `presentation.export.labels.blank` | Blank | Leer | 빈 화면 | Vide | En blanco | Em branco |
| `presentation.export.labels.slideImage` | Slide image | Folienbild | 슬라이드 이미지 | Image de la diapositive | Imagen de la diapositiva | Imagem do slide |
| `studies.materials.wordStyles.label` | Field label | Feldbezeichnung | 항목 제목 | Libellé de champ | Etiqueta de campo | Rótulo de campo |
| `studies.materials.wordStyles.writingLine` | Writing line | Schreiblinie | 답변 줄 | Ligne d’écriture | Línea de respuesta | Linha de resposta |
| `studies.materials.wordStyles.writingLineLast` | Writing line (last) | Schreiblinie (letzte) | 마지막 답변 줄 | Ligne d’écriture (dernière) | Última línea de respuesta | Última linha de resposta |
