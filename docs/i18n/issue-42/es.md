# es: AI editorial change log

Base: `38c785be723f4ff4ca7ba8c885aeabff3a89df7a`. This is AI linguistic review, not human native-speaker approval.

Register: Consistent tú; broadly understandable Spanish. Biblical quotations retain their original register.

These rows are the individually edited/reviewed keys. The full application is inventoried in inventory.json and the complete six-language corpus; unchanged keys are not claimed to have all received a new individual linguistic adjudication.

| Key/context | Before | After | Editorial reason |
| --- | --- | --- | --- |
| `errors.validation.entryInvalid` | (new localised message) | No se pudo guardar esta entrada. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.validation.unknownField` | (new localised message) | Esta parte no contiene ese campo. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.structure.timeout` | El análisis alcanzó su límite de tiempo en un punto seguro. Los metadatos anteriores no cambiaron. Reintenta o guarda un diagnóstico. | El análisis superó el límite de tiempo y se detuvo. Este intento no guardó ningún cambio. Vuelve a intentarlo o guarda un informe de diagnóstico. | Replace technical processing-boundary jargon with outcome, unchanged-data assurance and retry. |
| `errors.library.nothingChosen` | (new localised message) | Selecciona al menos un elemento. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.notInTrash` | (new localised message) | Solo los elementos de la papelera se pueden eliminar permanentemente. Muévelos primero a la papelera. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.badYear` | (new localised message) | El año debe ser un número entre 0 y 3000. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.collectionCycle` | (new localised message) | Una colección no se puede mover a sí misma ni a una de sus subcolecciones. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.linkUnsupported` | (new localised message) | No se puede abrir este tipo de enlace. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.tooMany` | (new localised message) | Puedes modificar un máximo de {{count}} elementos a la vez. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.badName` | (new localised message) | Usa un nombre de entre 1 y {{max}} caracteres. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.collectionExists` | (new localised message) | Ya existe aquí una colección llamada «{{name}}». | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.tagExists` | (new localised message) | Ya existe una etiqueta llamada «{{name}}». | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.material.tooManyPassages` | (new localised message) | Este material contiene demasiados pasajes para incluir su texto. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.words.documentBible` | (new localised message) | No se puede añadir información de palabras a una Biblia añadida desde un documento. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.messageUnavailable` | (new localised message) | No se pudo completar esta acción. Vuelve a intentarlo. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.invalidPassage` | (new localised message) | No se pudo interpretar el pasaje. Escribe un libro y un capítulo, por ejemplo Hechos 6. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.noCandidates` | (new localised message) | Ninguna referencia de esta fuente coincide con los versículos del pasaje. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.structuralResult` | (new localised message) | El primer resultado se basa en la estructura del documento, en la página {{page}} del PDF. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.literalResult` | (new localised message) | No se encontró una sección coincidente. El primer resultado es una mención en la página {{page}} del PDF. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.section` | (new localised message) | Título de sección | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.book` | (new localised message) | Sección del libro (más amplia) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.chapter` | (new localised message) | Título de capítulo | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.manual` | (new localised message) | Vínculo añadido por ti | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.contents` | (new localised message) | Entrada del índice | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.note` | (new localised message) | Mención en una nota | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.literal` | (new localised message) | Mención del pasaje | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.inherited` | (new localised message) | Dentro de una sección vinculada | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.unknown` | (new localised message) | Referencia a un pasaje | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.manualResult` | (new localised message) | El primer resultado es un vínculo que añadiste, en la página {{page}} del PDF. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.timeout` | El análisis alcanzó su límite de tiempo en un punto seguro. Los metadatos anteriores no cambiaron. Reintenta o guarda un diagnóstico. | El análisis superó el límite de tiempo y se detuvo. Este intento no guardó ningún cambio. Vuelve a intentarlo o guarda un informe de diagnóstico. | Replace technical processing-boundary jargon with outcome, unchanged-data assurance and retry. |
| `library.fileDialog.source` | (new localised message) | Elegir un archivo para añadir a tu biblioteca | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.sources` | (new localised message) | Elegir archivos para añadir a tu biblioteca | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.folder` | (new localised message) | Elegir una carpeta para añadir a tu biblioteca | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.sourceTypes` | (new localised message) | Libros y notas (PDF, EPUB, Word, Markdown, texto) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.bible` | (new localised message) | Elegir una Biblia para añadir | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.bibleTypes` | (new localised message) | Archivos de Biblia (USFM, .zip, PDF, EPUB, Word, Markdown, texto) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.allFiles` | (new localised message) | Todos los archivos | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `studies.builtin.templates.pozetek-study.sections.application.guidance` | ¿Qué significa para usted la verdad hallada? Manténgala separada de la observación y la interpretación anteriores. | ¿Qué significa para ti la verdad que has encontrado? Mantenla separada de la observación y la interpretación anteriores. | Correct register or localise diagnostic evidence without changing the method or source data. |
| `studies.builtin.templates.pozetek-writing-study.sections.application.guidance` | ¿Qué significa para usted la verdad hallada? Manténgala separada de la observación y la interpretación anteriores. Responder al texto manteniéndose ligado a lo que dice. Es un cierre aparte, no otra letra de POZETEK. ¿Qué verdad del texto sigue siendo válida para todo lector? ¿Dónde esta verdad desafía, consuela o corrige el propio pensar y vivir? ¿Qué hará: creer, arrepentirse, confiar, obedecer, orar, amar? Sea concreto. ¿Qué distingue la situación de entonces de la de hoy? ¿Qué no debe trasladarse directamente? | ¿Qué significa para ti la verdad que has encontrado? Mantenla separada de la observación y la interpretación anteriores. Responde al texto manteniendo la aplicación ligada a lo que dice. Es un cierre aparte, no otra letra de POZETEK. ¿Qué verdad del texto sigue siendo válida para todo lector? ¿Dónde desafía, consuela o corrige esta verdad tu forma de pensar y vivir? ¿Cómo responderás: creer, arrepentirte, confiar, obedecer, orar, amar? Sé concreto. ¿Qué distingue la situación de entonces de la actual? ¿Qué no debe trasladarse directamente? | Correct register or localise diagnostic evidence without changing the method or source data. |
