# Golden vectors: the A1.5 rule against E2

Every vector is SYNTHETIC (invented sentences, or single common Greek and Hebrew words).
A1.5: the frozen harness lines 45–105 and 712–720 (byte for byte, `extract-*.rs`) and the frozen
scorer's rule (wrong = verified, class 2, kind not typography or unicode), run by `src/main.rs`
(`results-a15.jsonl`). E2: `scripts/evaluation/fidelity.py`; `src-tauri/tests/evaluation_rules.rs`
checks that the Rust implementation gives the same on every vector.

| Id | What differs | A1.5 class / kind | A1.5 scorer | E2 class / kind | E2 strict criterion |
|---|---|---|---|---|---|
| F01 | identical | exact | correct | exact | faithful |
| F02 | outer white space only | exact | correct | exact | faithful |
| F03 | composed vs decomposed acute (el/él), canonically equal | different / unicode | correct | canonical | faithful |
| F04 | CJK compatibility ideograph vs unified (canonical singleton) | different / unicode | correct | canonical | faithful |
| F05 | Greek eta with smooth breathing and acute, composed vs decomposed | different / unicode | correct | canonical | faithful |
| F06 | Hebrew dagesh and sheva in the other canonical order | different / unicode | correct | canonical | faithful |
| F07 | a double space | normalised | correct | layout | faithful |
| F08 | no-break spaces inside guillemets | normalised | correct | layout | faithful |
| F09 | narrow no-break space, tab and line feed | normalised | correct | layout | faithful |
| F10 | decomposed accent and a double space | different / unicode | correct | layout | faithful |
| W01 | word split: apart / a part | normalised | correct | mismatch / word_boundary | **wrong text** |
| W02 | words joined: now here / nowhere | normalised | correct | mismatch / word_boundary | **wrong text** |
| W03 | Spanish words joined: si no / sino | normalised | correct | mismatch / word_boundary | **wrong text** |
| W04 | the owner's case: que élirá / queélirá | normalised | correct | mismatch / word_boundary | **wrong text** |
| W05 | que él irá / queélirá | normalised | correct | mismatch / word_boundary | **wrong text** |
| W06 | space after a comma lost | normalised | correct | mismatch / word_boundary | **wrong text** |
| W07 | every space removed | normalised | correct | mismatch / word_boundary | **wrong text** |
| W08 | a zero-width space where a space was | different / typography | correct | mismatch / word_boundary | **wrong text** |
| W09 | Hebrew maqaf replaced by a space | different / typography | correct | mismatch / word_boundary | **wrong text** |
| S01 | space before a semicolon lost | normalised | correct | mismatch / punctuation_spacing | **wrong text** |
| C01 | case: Spirit / spirit | normalised | correct | mismatch / case | **wrong text** |
| C02 | case: LORD / Lord | normalised | correct | mismatch / case | **wrong text** |
| C03 | all capitals | normalised | correct | mismatch / case | **wrong text** |
| C04 | Greek case | normalised | correct | mismatch / case | **wrong text** |
| P01 | apostrophe: It's / Its | different / typography | correct | mismatch / punctuation | **wrong text** |
| P02 | apostrophe: He'll / Hell | different / typography | correct | mismatch / punctuation | **wrong text** |
| P03 | apostrophe: We're / Were | different / typography | correct | mismatch / punctuation | **wrong text** |
| P04 | comma moved, meaning changed | different / typography | correct | mismatch / punctuation | **wrong text** |
| P05 | statement / question | different / typography | correct | mismatch / punctuation | **wrong text** |
| P06 | editorial double brackets dropped | different / typography | correct | mismatch / punctuation | **wrong text** |
| P07 | dashes dropped | different / typography | correct | mismatch / punctuation | **wrong text** |
| P08 | curly vs straight quotation marks | different / typography | correct | mismatch / punctuation | **wrong text** |
| P09 | right single quotation mark vs apostrophe | different / typography | correct | mismatch / punctuation | **wrong text** |
| P10 | a note sign added | different / sign | **wrong** | mismatch / sign | **wrong text** |
| P11 | a note sign absent | different / typography | correct | mismatch / punctuation | **wrong text** |
| P12 | a punctuation placeholder vs empty text | different / typography | correct | mismatch / punctuation | **wrong text** |
| P13 | one placeholder for another | different / typography | correct | mismatch / punctuation | **wrong text** |
| P14 | a bracketed placeholder word unbracketed | different / typography | correct | mismatch / punctuation | **wrong text** |
| P15 | hyphen vs en dash | different / typography | correct | mismatch / punctuation | **wrong text** |
| P16 | case and apostrophe together | different / typography | correct | mismatch / punctuation | **wrong text** |
| M01 | composed accent dropped (él / el) | different / changed | **wrong** | mismatch / marks | **wrong text** |
| M02 | decomposed accent in the edition dropped | different / typography | correct | mismatch / marks | **wrong text** |
| M03 | decomposed accent added in the document | different / typography | correct | mismatch / marks | **wrong text** |
| M04 | Greek: composed smooth breathing vs decomposed rough breathing | different / changed | **wrong** | mismatch / marks | **wrong text** |
| M05 | Greek: both decomposed, breathing differs | different / typography | correct | mismatch / marks | **wrong text** |
| M06 | Hebrew vowel points differ | different / changed | **wrong** | mismatch / marks | **wrong text** |
| M07 | Hebrew dagesh dropped | different / changed | **wrong** | mismatch / marks | **wrong text** |
| M08 | Hebrew cantillation mark dropped | different / typography | correct | mismatch / marks | **wrong text** |
| M09 | a spacing acute on a plain letter | different / typography | correct | mismatch / marks | **wrong text** |
| M10 | Greek accent dropped in one word of a phrase | different / changed | **wrong** | mismatch / marks | **wrong text** |
| K01 | a ligature | different / changed | **wrong** | mismatch / compatibility | **wrong text** |
| K02 | a superscript digit for a digit | different / changed | **wrong** | mismatch / compatibility | **wrong text** |
| Z01 | a soft hyphen | different / typography | correct | mismatch / format_characters | **wrong text** |
| Z02 | a zero-width joiner | different / typography | correct | mismatch / format_characters | **wrong text** |
| X01 | a number word changed: forty / four | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X02 | a digit dropped | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X03 | number word vs digits | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X04 | words added | different / extra | **wrong** | mismatch / extra | **wrong text** |
| X05 | words missing | different / missing | **wrong** | mismatch / missing | **wrong text** |
| X06 | a heading read into the verse | different / extra | **wrong** | mismatch / extra | **wrong text** |
| X07 | another verse's words (the harness found them elsewhere in the reference) | different / wrong_reference | **wrong** | mismatch / wrong_reference | **wrong text** |
| X08 | the verse's text empty | different / changed | **wrong** | mismatch / missing | **wrong text** |
| X09 | a placeholder edition, real words found | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X10 | a negation inserted, meaning reversed | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X11 | a Hebrew letter changed | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X12 | a Greek ending changed | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X13 | words transposed | different / changed | **wrong** | mismatch / changed | **wrong text** |
| X14 | another verse's words, not found elsewhere | different / changed | **wrong** | mismatch / changed | **wrong text** |

| A1.5 scorer | E2 | Vectors |
|---|---|---:|
| correct | faithful | 10 |
| correct | mismatch | 36 |
| wrong | mismatch | 22 |

36 of the 58 vectors that are wrong text under E2 were counted correct by A1.5; none that A1.5 counted wrong is faithful under E2.
