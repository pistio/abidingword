# Study flow and reusable template blocks

The study-flow redesign starts at `d2d2c283`. Historical schema migrations 1–24 and the original ten built-in template resources remain unchanged. New template identities separate new defaults from legacy definitions. Studies keep independent JSON snapshots; changing or duplicating a template never transforms answers already written.

## Stage 1: writing

New POZETEK studies contain seven steps: Personen, Ort, Zeit, Ereignisse, Thema, Erkenntnisse, Konsequenzen. Other interface languages translate the method's purpose rather than its German acronym. Ten-Step retains ten steps. Each step has one spacious writing answer. Guidance is optional and collapsed. Continuous writing overrides folding without modifying saved section contents or the device's folded-section preferences. Legacy structured editors remain usable and labelled; no answers are concatenated or removed.

## Stage 2: reusable blocks

Personal templates can add, label, configure, duplicate, remove and reorder writing, guidance, headings, dividers, passage, concordance, cross-reference, word-lookup and concept/person-search blocks. Built-ins use exactly the same renderer and JSON definitions. Original field kinds remain supported. Default queries, editions and guidance are copied into study snapshots. Original built-in resources remain available for previously saved studies; six current starting templates are offered in the new-study catalogue.

Research uses existing local Bible services. Passage and user-supplied cross-reference lists require a selected installed structured edition. Concordance uses the edition's word/phrase search. Word lookup explicitly chooses a lemma/lexical entry returned by the edition. Concept/person search is textual search, with no inferred theological relationships. Kept evidence is a separate quotation block with the query, edition identity/name, canonical reference and source text in its metadata; lemma selections also retain their identifiers. Switching controls cannot relabel already returned evidence. Writing remains separate. Missing annotations and incomplete results are reported without silently substituting another edition.

No database migration is required: field definitions, configuration and source evidence use existing snapshot and block JSON columns. New enum values are additive; older binaries do not understand new templates and should not be used to edit databases containing them.

Validation results and review commits will be recorded here after the complete suites finish.
