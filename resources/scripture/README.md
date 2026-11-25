# Scripture texts

Bible texts shipped with Abiding Word, one folder each. Only texts whose
licence allows redistribution inside the app may be placed here, and each
folder must contain a `SOURCE.md` recording where the file came from, when,
its checksum and the licence statement in the publisher's own words (eBible's
`copr.htm`, or the publisher's statement in `copyright-statement.txt`).

| Folder | Text | Licence |
|---|---|---|
| `engbsb/` | Berean Standard Bible (66 books), USFM from eBible.org | Public Domain |
| `engwebp/` | World English Bible (66 books), USFM from eBible.org | Public Domain |
| `deu1912/` | Lutherbibel 1912 (66 books), USFM from eBible.org | Public Domain |
| `deu1951/` | Schlachter-Bibel 1951 (66 books), USFM from eBible.org | CC BY 4.0, © 1951 Genfer Bibelgesellschaft (attribution required) |
| `deuelo/` | Unrevidierte Elberfelder 1905 (66 books), USFM from eBible.org | Public Domain |
| `kor1961/` | 성경전서 개역한글판 (Korean Revised Version 1961, 66 books), from the Korean Bible Society's online text | Copyright expired (Korean Bible Society); keep unaltered and attribute |
| `fraLSG/` | Louis Segond 1910 (66 books), USFM from eBible.org | Public Domain |
| `spaRV1909/` | Santa Biblia — Reina Valera 1909 (66 books), USFM from eBible.org | Public Domain |
| `porbr2018/` | Bíblia Livre (66 books), USFM from eBible.org | CC BY 4.0 (attribution required) |
| `grcbyz/` | 1904 Patriarchal Greek New Testament (27 books), eBible.org | Public Domain |
| `grcmt/` | Robinson–Pierpont Byzantine Textform 2018 (27 books), eBible.org | Public Domain |
| `grcsbl/` | SBL Greek New Testament (27 books), eBible.org | CC BY 4.0 (attribution required) |
| `grctr/` | Textus Receptus, R. Adam Boyd 2022 edition (27 books), eBible.org | Public Domain |

There are **nine complete 66-book editions and four additional Greek New Testament editions**. Bundled licences and original hashes are verified by `npm run release:resources`, and the [public distribution notices](../../docs/release/THIRD-PARTY-NOTICES.md) travel in the installer. These checks establish inventory consistency, not legal permission for every jurisdiction; final rights review remains necessary.

Included texts are not installed automatically; the user adds them from
Settings → Bible Texts or the Scripture pane. See `docs/scripture-v0.2.md`.
