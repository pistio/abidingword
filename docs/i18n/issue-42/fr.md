# fr: AI editorial change log

Base: `38c785be723f4ff4ca7ba8c885aeabff3a89df7a`. This is AI linguistic review, not human native-speaker approval.

Register: Consistent vous; idiomatic French, curly apostrophes.

These rows are the individually edited/reviewed keys. The full application is inventoried in inventory.json and the complete six-language corpus; unchanged keys are not claimed to have all received a new individual linguistic adjudication.

| Key/context | Before | After | Editorial reason |
| --- | --- | --- | --- |
| `errors.validation.entryInvalid` | (new localised message) | Cette entrée n’a pas pu être enregistrée. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.validation.unknownField` | (new localised message) | Cette partie ne contient pas ce champ. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.structure.timeout` | L’analyse a atteint sa limite de temps à une étape sûre. Les métadonnées précédentes sont inchangées. Réessayez ou enregistrez un diagnostic. | L’analyse a dépassé le temps prévu et a été arrêtée. Cette tentative n’a enregistré aucune modification. Réessayez ou enregistrez un rapport de diagnostic. | Replace technical processing-boundary jargon with outcome, unchanged-data assurance and retry. |
| `errors.library.nothingChosen` | (new localised message) | Sélectionnez au moins un élément. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.notInTrash` | (new localised message) | Seuls les éléments de la corbeille peuvent être supprimés définitivement. Déplacez-les d’abord dans la corbeille. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.badYear` | (new localised message) | L’année doit être un nombre compris entre 0 et 3000. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.collectionCycle` | (new localised message) | Une collection ne peut pas être déplacée dans elle-même ni dans l’une de ses sous-collections. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.linkUnsupported` | (new localised message) | Ce type de lien ne peut pas être ouvert. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.tooMany` | (new localised message) | Vous pouvez modifier au maximum {{count}} éléments à la fois. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.badName` | (new localised message) | Utilisez un nom de 1 à {{max}} caractères. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.collectionExists` | (new localised message) | Une collection nommée « {{name}} » existe déjà ici. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.tagExists` | (new localised message) | Une étiquette nommée « {{name}} » existe déjà. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.material.tooManyPassages` | (new localised message) | Ce support contient trop de passages pour en inclure le texte. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.words.documentBible` | (new localised message) | Il n’est pas possible d’ajouter des informations sur les mots à une Bible ajoutée à partir d’un document. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.messageUnavailable` | (new localised message) | Cette action n’a pas pu être effectuée. Veuillez réessayer. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.invalidPassage` | (new localised message) | Le passage n’a pas pu être lu. Saisissez un livre et un chapitre, par exemple Actes 6. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.noCandidates` | (new localised message) | Aucune référence de cette source ne recoupe le passage. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.structuralResult` | (new localised message) | Le premier résultat repose sur la structure du document, à la page PDF {{page}}. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.literalResult` | (new localised message) | Aucune section correspondante n’a été trouvée. Le premier résultat est une mention à la page PDF {{page}}. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.section` | (new localised message) | Titre de section | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.book` | (new localised message) | Section consacrée au livre (plus large) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.chapter` | (new localised message) | Titre de chapitre | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.manual` | (new localised message) | Lien ajouté par vous | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.contents` | (new localised message) | Entrée de la table des matières | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.note` | (new localised message) | Mention dans une note | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.literal` | (new localised message) | Mention du passage | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.inherited` | (new localised message) | Dans une section associée | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.unknown` | (new localised message) | Lien vers un passage | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.manualResult` | (new localised message) | Le premier résultat est un lien que vous avez ajouté, à la page PDF {{page}}. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.timeout` | L’analyse a atteint sa limite de temps à une étape sûre. Les métadonnées précédentes sont inchangées. Réessayez ou enregistrez un diagnostic. | L’analyse a dépassé le temps prévu et a été arrêtée. Cette tentative n’a enregistré aucune modification. Réessayez ou enregistrez un rapport de diagnostic. | Replace technical processing-boundary jargon with outcome, unchanged-data assurance and retry. |
| `library.fileDialog.source` | (new localised message) | Choisir un fichier à ajouter à la bibliothèque | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.sources` | (new localised message) | Choisir des fichiers à ajouter à la bibliothèque | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.folder` | (new localised message) | Choisir un dossier à ajouter à la bibliothèque | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.sourceTypes` | (new localised message) | Livres et notes (PDF, EPUB, Word, Markdown, texte) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.bible` | (new localised message) | Choisir une Bible à ajouter | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.bibleTypes` | (new localised message) | Fichiers bibliques (USFM, .zip, PDF, EPUB, Word, Markdown, texte) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.allFiles` | (new localised message) | Tous les fichiers | Remove an English-only native/UI path; preserve meaning and uncertainty. |
