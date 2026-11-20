# Abiding Word — bundled Scripture and third-party notices

**Public distribution review draft · 20 November 2026.** This is the attribution inventory for the `pistio/abidingword` Windows app, not a software licence for Abiding Word itself. **Do not publish the installer without the owner's approval and a final licence/rights review.**

This list describes **files shipped as optional bundled resources**, even when a user has not installed a particular Bible into their library. It does not grant distribution rights to private sources, user-imported books, downloaded media, original MacArthur material or any other copyrighted user content.

## Bible editions bundled in `resources/scripture/`

| Edition | Source and licence position | Required notice/qualification |
|---|---|---|
| Berean Standard Bible (`engbsb`) | [eBible.org](https://ebible.org/find/details.php?id=engbsb): public domain | BSB Publishing, LLC, source identifier. |
| World English Bible, Protestant 66-book edition (`engwebp`) | [eBible.org](https://ebible.org/find/details.php?id=engwebp): public domain | **World English Bible is an eBible.org trademark.** Do not mislabel an altered edition; Abiding Word preserves verse wording. |
| Lutherbibel 1912 (`deu1912`) | [eBible.org](https://ebible.org/find/details.php?id=deu1912): public domain | Martin Luther; identify the 1912 edition, not a modern revised Luther Bible. |
| Schlachter-Bibel 1951 (`deu1951`) | [eBible.org](https://ebible.org/find/details.php?id=deu1951): **CC BY 4.0** | **© 1951 Genfer Bibelgesellschaft**, translation by Franz Eugen Schlachter, revision 1951. [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). Mention the source, provide the licence link, and explain that display/import omit some non-verse markup but do not rewrite verse wording. Do not confuse with Schlachter 2000. |
| Unrevidierte Elberfelder 1905 (`deuelo`) | [eBible.org](https://ebible.org/find/details.php?id=deuelo): public domain | Identify the unrevised 1905 edition and the named translators, not a recent revised Elberfelder. |
| 1904 Patriarchal Greek New Testament (`grcbyz`) | [eBible.org](https://ebible.org/find/details.php?id=grcbyz): public domain | State the 1904 Patriarchal edition; the publisher reports differing counts of later corrections in its files, so do not invent a definitive count. |
| Robinson–Pierpont Byzantine Textform 2018 (`grcmt`) | [eBible.org](https://ebible.org/find/details.php?id=grcmt): public domain | Name Maurice A. Robinson and William G. Pierpont. Display imported unaccented/un-cased text as supplied, not as a corrected print edition. |
| SBL Greek New Testament (`grcsbl`) | [eBible.org](https://ebible.org/find/details.php?id=grcsbl): **CC BY 4.0** | **© 2010 Society of Biblical Literature and Logos Bible Software**; cite [SBLGNT source](https://github.com/LogosBible/SBLGNT) and [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). The bundled archive's source record notes which markup is omitted at import. |
| Textus Receptus, Adam Boyd edition 2022 (`grctr`) | [eBible.org](https://ebible.org/find/details.php?id=grctr): public domain as claimed by its copyright statement | Give the named editor R. Adam Boyd and identify this *edition*, not all future or critical Greek texts. The comparison annotations are stated to be dedicated to the public domain. |
| 성경전서 개역한글판 / Korean Revised Version 1961 (`kor1961`) | [대한성서공회 FAQ](https://www.bskorea.or.kr/bbs/board.php?bo_table=copyright_faq&wr_id=5): economic copyright expired | **대한성서공회, 1961**. Preserve edition title and actual verse text, and meet the Society's stated rights of attribution (**성명표시권**) and integrity (**동일성유지권**). The app converts the Society's web text into USFM while retaining original wording; the detailed conversion and provenance record remains at `resources/scripture/kor1961/SOURCE.md`. Verify the exact distributed archive again before stable release. |
| Louis Segond 1910 (`fraLSG`) | [eBible.org](https://ebible.org/find/details.php?id=fraLSG): public domain | Identify the 1910 edition. |
| Reina-Valera 1909 (`spaRV1909`) | [eBible.org](https://ebible.org/find/details.php?id=spaRV1909): public domain | Identify the 1909 edition. |
| Bíblia Livre (`porbr2018`) | [eBible.org](https://ebible.org/find/details.php?id=porbr2018): **CC BY 4.0 Brasil** | **© 2018 Diego Santos, Mario Sérgio and Marco Teles**, *Bíblia Livre (BLIVRE)*. [CC BY 4.0 Brasil](https://creativecommons.org/licenses/by/4.0/br/); source credited by the publisher: [Bíblia Livre](http://sites.google.com/site/biblialivre/). Credit authors, link licence, and state that presentation/import omit non-verse markup but preserve verse wording. |

Nine bundled complete-Bible editions and **four additional Greek New Testament editions** are currently present under `resources/scripture/`. This inventory covers all 13 source folders; publishing must still verify the actual packaged file list against it.

For each source, the exact downloaded file, SHA-256 and longer original copyright statement are in `resources/scripture/<edition>/SOURCE.md` or the bundled `copr.htm` / `copyright-statement.txt`.

**Important:** The CC BY licences allow redistribution (including commercial distribution) subject to their terms; they do not automatically license *Abiding Word's own source code, brand or unrelated materials*. Keep all notices and resource-level provenance. The Korean Society's FAQ establishes the economic copyright position for *개역한글판*, not unrestricted permission to alter its text or use separately protected editions.

## Fonts and other code

- **Inter and Noto Serif** in the current application are SIL Open Font License families. The redistributable OFL licence notices for the exact font packages must accompany the installer. The package's licence files and attribution should be verified in the final generated installer. **Do not place proprietary Pistio fonts in a release merely because the owner created them**; separate permission and licence review would be required.
- **JavaScript/npm and Rust/Cargo dependencies** have their own licences. Verify the actual pinned lockfile versions, record attribution/copyleft obligations, and check incompatible or missing licences before publishing. This document does not claim that the dependency audit is complete.
- **Presentation templates, image assets, sample projects and Bible-study methods** require their own content/source review. User-imported third-party works are personal data, not redistributable release assets.
- **App brand, logomark and software source:** the repository README currently states no software licence has been selected. Do not add an open-source licence, claim a proprietary EULA, or permit commercial redistribution on the owner's behalf without an explicit owner decision.

## Privacy and distribution wording

Abiding Word is local-first and works offline for its core functions. **Optional remote AI, source/image search and test updates may contact external providers when enabled or invoked.** Never market the product as making *no* network connections in every configuration. Describe precisely what leaves the device, when, and where.

The normal `.awbackup` may contain private notes and copyrighted imported books. It is a **private personal backup**, not a file for a user to publish. Daily SQLite-only recovery snapshots contain personal writing and source text in the database; they are not complete portable backups and must never be uploaded automatically.

## Publication sign-off still required

Owner must select the software distribution/licence model, validate the final binary's actual bundled resources and font licence files, approve attribution/Privacy Policy/Terms, verify code signing and SmartScreen behaviour, and explicitly authorise a public release. **No release or licence approval is implied by this inventory.**
