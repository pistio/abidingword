# Bible Study template catalogue — implementation brief

Scope: **Studies only**, not Teaching. Follow CLAUDE.md and existing branch conventions. Do not merge, release, deploy, start paid Actions or alter Windows test infrastructure.

## Catalogue (six templates, five study subjects)
1. Passage / single verse: existing POZETEK methodology. Display "POZETEK" **only in German**. EN "Guided Passage Study"; KO/FR/ES/pt-BR use short, idiomatic descriptive names, never invented acronyms.
2. Passage / single verse: Ten-Step Exegesis, based on Helge Stadelmann and Thomas Richter, *Bibelauslegung praktisch. In zehn Schritten den Text verstehen* (SCM R. Brockhaus, 2024). Original explanations and questions; no copied book text.
3. Biblical Character Study (person and character traits together): identify references; timeline and historical context; actions, words, relationships and motivations (distinguish observed from inferred); strengths and failures; God's work and biblical evaluation; synthesis; personal application.
4. Word Study: choose original Hebrew/Greek lemma, distinguish it from translated words; occurrences and range of uses; immediate contexts and grammar; compare representative passages; meaning in the focal passage; theological synthesis and application. Avoid root/etymology fallacy, illegitimate totality transfer, and inferring meaning from frequency alone.
5. Book Study: repeated complete reading; genre, recipients, author/date when evidenced and uncertainties; purpose and historical setting; literary divisions, structure, argument/narrative development; repeated motifs; central message; canonical connections and application. Distinguish textual evidence from historical conjecture.
6. Topical Study: narrowly define question and scope; gather representative passages (including qualifying/counterbalancing texts); interpret each in its own literary and covenantal context; group findings; reconcile apparent tensions without forced harmonisation; theological synthesis; specific application.

## Principles and restraint
- Source of method: user's seminary notes on Bible-study methods and the developed conversation's exegetical principles; do not treat external reference works as direct quotations.
- Scripture interprets Scripture; grammatical-historical reading, original audience, genre, context, and the redemptive-historical place of a passage; avoid proof-texting.
- Application follows interpretation. Biblical aim includes love of God and neighbour (Matt 22:37–40), never displacing the meaning of the text.
- Keep essential fields few and usable. Make deeper linguistic, historical, cross-reference and research prompts optional. No separate devotional, chapter, verse, character-trait or teaching template.
- Keep distinct core fields for subject (Textgegenstand), assertion (Textaussage), and theme (Textthema) where applicable, without mechanically forcing passage-specific categories onto word, person, book or topical studies.
- User-created templates remain supported; existing study snapshots, responses, statuses, exports, backups, restores and search must survive. Do not remove existing persisted template keys or silently change old studies. Legacy built-ins can be hidden from *new study selection* if compatibility is maintained.
- Examples must be collapsible, optional, localised and never prepopulate user answers. Hosea 11 for POZETEK; Matthew 8:18–27 for Ten Steps. No first-person author voice, personal attribution, CNB/YCC/Ealing or biographical details.
- Native-language editorial polish in de, en-GB, ko, fr, es and pt-BR (locale folder pt); no untranslated English, mechanical acronyms or awkward theological terminology.
- Provide short explanatory guidance; do not change application architecture beyond minimal additive compatibility work.

## Implementation and validation
1. Inspect actual branch state, template files, study types, migrations and localisation mechanisms before editing. Reconcile with any work committed since this brief.
2. Draft concise, academically responsible field definitions and examples for all six templates. Preserve existing POZETEK and Ten Steps where appropriate; add missing Character and Book templates.
3. Adjust catalogue display without breaking legacy keys or persisted studies. Avoid new migrations unless unavoidable and justified.
4. Run repository-prescribed interface and Rust checks locally, with disk-safe settings; tests for catalogue, keys, field snapshots, old studies, custom templates, backup/restore and all six locales.
5. Report exact changed files, tests and failures. Do not merge to main or trigger Windows publishing.