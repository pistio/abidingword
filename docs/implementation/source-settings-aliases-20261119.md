# Settings source editing and multilingual source hierarchy

**19 November 2026 — follow-up to draft PR #54.** This is additional application code after the Windows installer pinned to `bcf4858591f6d306838773ebc884550ca27d19b6`. That installer does not contain this follow-up. Exact source identity and final verification are recorded in [the companion evidence](source-settings-aliases-20261119-verification.json).

Published application commit: [`288076c3a8117bca429d2e62d8e0b2185525e731`](https://github.com/pistio/abidingword/commit/288076c3a8117bca429d2e62d8e0b2185525e731). Its tree `7b939be156fb836e7e13f328dfcbaa1363d940f9` exactly matches tested local commit `749179ab9f29857242913692e0985c59bc6ca50b`. The subsequent documentation commit contains only this record, verification and continuity updates.

## Owner request and observed acceptance

The owner requested an explicit **Edit details** action in **Settings → Sources**, in addition to the existing editing paths in Library/Sources/the reader. They also requested that the book → chapter → verse hierarchy recognize names across languages, including German synonyms and keyboard spellings.

The owner reports that the previously supplied handbook now works. This records acceptance of that reported correction, not acceptance of every other PDF, all native Windows workflows, or the separate Scripture-recognition programme. The [complete continuity record](continuity-20261119.md) retains the overall product objective and remaining work.

## Settings editing

- Active and archived sources now offer **⋯ → Edit details**.
- The action opens the existing inline editor, which uses the shared metadata and whole-source coverage fields. Existing tags and collection controls remain available.
- Keyboard activation focuses Title after the details load; typing and saving do not repeatedly move focus.
- Saving refreshes the source list and language-filter options while preserving source identity and imported text.
- Clicking the source title still expands the same editor. Trash retains its existing restore/delete actions.

The existing Settings interaction test now opens the action through the menu, checks focus, edits the title and language, saves, and verifies the original source ID, imported segment count and refreshed language option.

## One source-heading lookup across the installed catalogue

The hierarchy previously used a narrower foreign-language fallback. It now has a cached lookup over all **101 installed reference-language dictionaries**. The source's declared language resolves its own unambiguous labels; a foreign fallback requires agreement among the available dictionaries and retains protections for short abbreviations, ordinary words and incomplete numbered-book names.

| Examples | Common identity |
|---|---|
| Sprüche, Sprichwörter, Sprueche, Sprichwoerter | Proverbs (`PRO`) |
| Prediger, Kohelet, Prediger (Kohelet) | Ecclesiastes (`ECC`) |
| 1. Mose, 1.Mose, 1Mose, Genesis, 1. Mose (Genesis) | Genesis (`GEN`) |
| 1. Buch Mose, Das erste Buch Mose, Erstes Buch Mose | Genesis (`GEN`) |
| Fünftes Buch Mose, Fuenftes Buch Mose | Deuteronomy (`DEU`) |
| Römer / Roemer; Matthäus / Matthaeus | Romans (`ROM`); Matthew (`MAT`) |

German keyboard forms are derived from the existing German dictionary. Parenthetical or slash-separated names act as synonyms only when both independently identify the same book. Canonical book IDs and actual document anchors stay unchanged.

The same source-only lookup is used for PDF headings and printed Contents, imported section paths and supplied EPUB Contents, and manual whole-source coverage. Explicit references such as **Sprueche 3,5–6** therefore use the same identity as the book heading. A bare book in whole-source coverage still means the whole book; it does not invent a page destination.

The implementation preserves Unicode composition, meaningful accents, Indic vowel signs and word boundaries. Portuguese **Jó** and **Jo**, and **I Sa** and **Isa**, remain distinct. Existing catalogue chapter/verse units and native decimal digits are reused. Explicit one-chapter references keep the shared verse convention (**Jude 1** means **Jude 1:1**); a separate structural child heading **1** beneath a Jude book heading can still identify chapter 1. Korean/Han ranges retain both endpoints. Malformed adjacent verse tokens are not concatenated into a new verse number.

The catalogue regression exercises **6,390 available language/book pairs**, requiring at least one documented resolvable heading for each, except the existing explicit Igbo Jude limitation: its supplied entry contains only **Jud**, which remains withheld by the pre-existing canon-ambiguity rule. This is not evidence for every spelling or every language in the world. Missing or conflicting dictionary evidence remains unresolved. These changes do not broaden incidental prose recognition or change Claude's Bible-text recognizer.

## Existing imports and rebuilds

**Source index version 6** performs a structural refresh for previous versions 4 and 5. It preserves literal/manual reference IDs and timestamps, source segments and text, annotations, citations and user coverage. Supplied EPUB Contents links refresh when Contents is opened, using the existing file-hash and exact-text checks before binding to saved anchors.

**PDF analysis version 5** also covers extraction: a printed Contents spelling can differ from the actual book opening. Earlier analysis may have omitted an entire destination, so simply resolving already stored headings is insufficient. Existing PDFs offer **Chapters and headings → Rebuild structure** to recover such targets. This remains an explicit operation; changing the analysis version does not automatically reread the file. The stored JSON format remains version 1 and the database remains schema 27.

An invented-PDF regression uses three independently corroborating Contents/body name variants with no native page-label shortcut. It checks actual physical destinations, the older-analysis rebuild recommendation, and preservation during rebuild. No production rule depends on the owner's filename, fingerprint, font, page count or fixed page numbers.

## Delivery and limits

The complete frontend suite passes **1,364/1,364** on this follow-up. The two assertions that failed in the earlier candidate's full run pass unchanged here; this does not establish or repair their previous timing cause. The complete locked Rust suite passes **1,146 tests, zero failures, nine ignored**; all **34 focused source/PDF checks** pass. Typecheck, lint, all six interface-language checks, the frontend production build, Rust formatting and locked all-target Clippy with warnings denied pass. The final native checks used an unchanged recorded input manifest. Commands, counts and changed-file hashes are in the companion evidence.

The older Windows ZIP remains the verified `bcf4858` build. This follow-up requires a subsequent Windows build and native acceptance. No additional build, merge, tag, release or updater publication is included in this change.

The owner's report confirms the supplied handbook correction. The shared fix and invented regressions support broader use, but do not measure an arbitrary-PDF failure rate. Image-only scans, missing/broken text, unknown aliases and ambiguous structure remain limitations.

Claude is continuing the separate Scripture-recognition track. Its frozen candidate, independent-evaluation gate, resource isolation, provenance, numbering and integration obligations remain in the continuity record. This source-navigation follow-up does not complete those obligations.
