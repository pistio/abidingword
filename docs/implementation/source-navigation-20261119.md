# Source details, reading position and structural passage navigation

Owner follow-up, 19 November 2026. This change builds on source
`59c8f5e04d6c76a72c7a8613ba1332a4d758703f` (draft PR #50).
The accepted Source → Original scrolling correction and the German-only
POZETEK terminology policy remain in that base.

## Intended reading behaviour

Changing Scripture changes the suggested sources and the passage shown beside
**Go to current passage**. It does not navigate the source already being read.
The reader keeps its current source, Reading/Original view and position.

Clicking **Go to current passage** searches only that source. A valid structural
destination opens through the reader's existing navigation/history. **Back**
returns to the previous reading place. Book/chapter fallback is identified as
broader than the requested verse, so the interface does not claim an exact
match when the source only has a book introduction.

Literal references are offered as a list when no relevant usable structural
destination exists. Even a single literal mention requires the reader to choose
it. If nothing matches, the reading position stays unchanged. A delayed result from a
previous source or Scripture selection cannot navigate the current reader.
The reader also shares one request generation across explicit passage lookup,
Contents, history, pages and view changes: a newer action supersedes an earlier
pending arrival. EPUB/Word Previous and Next part buttons follow the same rule.

| Requested passage and source structure | Destination |
| --- | --- |
| Romans 8:20; book, chapter 8 and a section containing verses 18–25 exist | The verses 18–25 section |
| Romans 8:1; chapter 8 exists but no containing verse section | Chapter 8 |
| Romans 8; the source has only a Romans introduction | Romans opening |
| Romans 8; a verse 20 subsection exists but no chapter heading | The Romans opening; one child verse does not represent the whole chapter |
| No relevant book/chapter/passage structure, but literal mentions exist | A choice of mentions; no automatic jump |

Explicit user-confirmed links and source-structure corrections retain priority.
Unrelated headings do not suppress literal fallback. A fully qualified standalone
heading can itself provide a destination without a separately printed book heading.

## Editing an existing source

**Edit details** is available from Library and Sources lists and from the reader.
The shared editor exposes title, author, category and language; More details
contains subtitle, publisher, edition, publication year, ISBN, DOI, licence and
notes. Settings uses the same fields and metadata-only update.

Save updates the existing source in place. Cancel discards the draft. The source
identity, managed file, extracted text, annotations and current reader position
are preserved. This also makes incorrect publisher-supplied PDF metadata editable
without importing a duplicate copy.

## Why the supplied older PDF failed

The owner supplied a 400-page handbook with a selectable text layer. It has no
embedded PDF outline or structure tags, but its printed contents on physical
pages 5–6 supplies real destination pages.

The old typography pass retained only the three largest heading sizes in the
whole document. Large cover/front-matter text displaced the smaller book-opening
headings. The printed-contents pass then depended on headings that had already
survived that filter. Fresh import and Rebuild consequently produced only three
front-matter headings and no mapped Bible-book destinations for this file.

The correction keeps headings that are prominent relative to body text, capping
their displayed nesting depth instead of dropping smaller heading sizes outright.
Printed contents is recovered independently from corroborated page mappings and
real destination anchors. The contents label remains separate from the exact
drawn text, accommodating a different displayed title or a publisher's spelling
error without fuzzy Bible-book matching or a file-specific exception.

Body references, footnotes, running furniture and lists of biblical verse ranges
are not promoted into physical source sections. In this handbook, **Acts 6 and
Acts 6:1 belong at the Apostelgeschichte opening on physical page 232**, while
**Romans 8 and Romans 8:1 belong at the Römer opening on physical page 239**.
An incidental mention of a verse elsewhere is not a deeper structural match.

The final private-file run checks every canonical book against a separately
recorded opening-page map, through both fresh import and Rebuild:

| Actual-file check | Result in both paths |
| --- | --- |
| Printed Contents destinations recovered | 75 of 75 |
| Entries with a validated text anchor | 69 |
| Appendix/index entries with an honest page-only target | 6 |
| Real Bible-book opening nodes | 63, covering all 66 books |
| Invented chapter/verse destinations | 0 |
| Stored segments retained during Rebuild | All 1,849, including IDs and text |

Samuel, Kings and Chronicles each have a shared opening for their two books.
Both books resolve to that real opening; the resolver does not manufacture a
separate second-book page or attribute relative chapter headings arbitrarily.
All three shared openings and their inherited context are checked. Import and
Rebuild agree, and the original PDF bytes remain unchanged.

Only aggregate results and book/page metadata belong in public verification.
The owner's PDF, extracted text and private page renders are not repository
fixtures or build inputs. Committed regressions use invented documents.

## Other formats and supplied destinations

PDF outline labels need not equal their drawn headings. A valid page-only or
named destination can align to a unique, prominent opening heading on its supplied
physical page; the original text remains available for exact segment alignment.
Printed page labels are never substituted for physical page numbers. Ambiguous
precise coordinates and ambiguous separate column headings remain unresolved
rather than inventing a text target.

EPUB navigation and NCX hierarchies supply their actual internal destinations.
Targetless grouping nodes carry context but cannot become opening locations.
EPUB, Word and Markdown heading paths also resolve bare book titles followed by
relative chapter and verse headings. A real child heading may refine a supplied
book-level destination even when the child's evidence is typography rather than
an embedded bookmark, provided it belongs inside that actual parent section.

A missing or ambiguous EPUB fragment has no destination. An empty spine item
cannot borrow the following document's first paragraph, and an anchor after the
last paragraph cannot silently select the preceding paragraph. A targetless
group may retain valid children. Text preceding a heading no longer inherits
that later heading as its own opening.

## Existing libraries

PDF analysis is version 4. An existing saved version 3 analysis recommends
**Rebuild structure** in **Chapters and headings** after installing this change.
The reader does not silently
reparse the book on every open. One rebuild upgrades the saved analysis; repeated
reimports are unnecessary. Source corrections remain separate and survive rebuild.

Source passage-index version 5 refreshes derived section links for existing
version 4 indexes. Literal-reference identities/timestamps and manual links are
preserved; this is not a database schema migration or a change to Scripture
recognition. Existing EPUB navigation is refreshed from the managed file only
after its identity and extracted text have been checked. Explicit passage
navigation waits for that refresh before matching.

## Verification and delivery

The accompanying [verification record](source-navigation-20261119-verification.json)
lists the final command results and actual
file outcomes. Synthetic checks cover import, hierarchy containment, fallback,
named PDF destinations, cross-format heading paths, data preservation, metadata
editing, explicit navigation, stale responses and history.

The final complete Rust run passes **1,122 tests**, with no failures and nine
ignored diagnostics (the new private-file diagnostic passes separately). The
final complete interface run passes **1,338 of 1,340 tests**. Its two failures
are the exact presenter/finding cases already recorded on the base commit in
[the earlier verification](owner-followup-20261119-verification.json); their
test files are unchanged. Both pass unchanged in one focused run; neither
fixture opens a source reader. All new source-navigation tests pass. The verification
record keeps the full result and focused follow-up separate, rather than
claiming a wholly green final interface run.

Typecheck, ESLint, translations, the generated translation corpus, production
frontend build, Rust formatting and all-target Clippy with warnings denied pass.

The candidate needs a separately approved Windows installer build through the
existing one-off pathway, followed by normal Windows use. Local native-library
and interface-component tests are not Windows desktop acceptance. A browser
preview could not be completed in this environment, so this change does not
claim visual or Windows acceptance. The last delivered
installer is still source `539b54d65c2cecf010cd1573b8bd51c91ac640bd`; it cannot
contain these later source changes.

### Prepared installer step

The proposed next step is one Windows installer build of the exact source commit
recorded in this change's PR, using the existing
`build/abidingword-i35-i42-20261109` pathway and
`windows-combined-owner-20261109.yml` workflow. It uses the established single
compile and NSIS packaging path, pinned toolchain and verified packaging tools.
The source branch does not modify or activate that workflow. There is no retry,
merge, release, updater publication or deployment in this proposed build.

After explicit build approval, verify the resulting installer and payload against
that exact source before delivery. The owner then uses the normal Windows
installation, edits source details in place, rebuilds this PDF's saved structure
once, and checks both sticky reading and deliberate passage navigation. This
replaces another disposable diagnostic package with the actual application flow.
