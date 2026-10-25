# Original-language Bibles: licence audit and what can be installed (v0.24)

Checked 2026-10-25: every statement below was re-read from the downloaded
files or the publisher's repository on that date (it builds on
`docs/original-language-v0.22.md` §1). This record says, for each edition,
whether Abiding Word **ships** it, offers it as a **verified download**, or
leaves it out, and why. User guide: [Getting started → Add a
Bible](guide/01-getting-started.md#add-a-bible).

## Installation readiness

| Edition | How it is offered | Contents (verified) | Licence | Ready? |
|---|---|---|---|---|
| SBL Greek New Testament (`grcsbl`) | **Included** (Add a Bible → Included) | 27 NT books; Strong's on every word; no grammar | CC BY 4.0 (attribution kept in the Bible's details) | Ready |
| Robinson–Pierpont 2018 (`grcmt`) | **Included** | 27 NT books; Strong's + Robinson grammar on every word; **unaccented, lower-case text** | Public Domain | Ready (display limitation stated) |
| Textus Receptus, Boyd 2022 (`grctr`) | **Included** | 27 NT books; Strong's on every word; no grammar | Public Domain | Ready |
| 1904 Patriarchal Greek NT (`grcbyz`) | **Included** | 27 NT books; Strong's on every word | Public Domain | Ready |
| Open Scriptures Hebrew Bible (morphhb `3d15126`) | **Verified download**, 40 files, 28.6 MB | 39 OT books; lexical number + grammar on every word; VerseMap.xml maps Hebrew to English verse numbers | PD text; CC BY 4.0 word data | Ready (needs internet once) |
| MorphGNT SBLGNT (`aaed91e`) | **Verified download**, 27 files, 8.9 MB | 27 NT books; dictionary form + grammar on every word; no Strong's; no John 7:53–8:11 | CC BY 4.0 text; CC BY-SA 3.0 word data | Ready (needs internet once) |
| eBible `hboWLC` | Not offered (superseded by OSHB) | old 2017 snapshot: grammar on 2,352 words only | PD / CC BY-SA 3.0 | — |
| Brenton Greek LXX (`grcbrent`) | **Not offered yet** | LXX books and order, LXX Psalm numbering | Public Domain | **Not ready**: its verse numbers differ from the English ones Abiding Word stores (Psalms by one, Jeremiah's order); adding it would put text under wrong references. Needs LXX versification (Part G foundations) |
| Clementine Vulgate (`latVUC`) | **Not offered yet** | Vulgate books, Vulgate Psalm numbering, 13,775 notes | Public Domain | **Not ready**: same versification reason |
| TR parsed (byztxt beta code) | Excluded | Strong's + grammar, unreadable format | PD | Excluded (format) |
| Swete LXX (TEI), CCAT LXX, `grclxx` | Excluded | — | see below | Excluded |
| **NA28** | **Excluded** | — | © Deutsche Bibelgesellschaft | Excluded |

The included zips are byte-identical to the downloads recorded here
(`resources/scripture/<id>/SOURCE.md` carries each SHA-256; a test checks
it). eBible rebuilds its zips, so their hashes identify **these copies**;
that is why eBible editions are shipped, not downloaded.

Verified downloads (`scripture::catalogue`, `resources/scripture/catalogue.json`):

- start only from **Add a Bible → Available to download**, after a
  confirmation that shows the number of files and the size;
- every file comes from `raw.githubusercontent.com` at a **fixed commit**
  and must have exactly the listed size and SHA-256; one difference, a
  missing file or an unlisted host and **nothing is kept** (the download
  folder is removed);
- the files then go through the same importer and review screen as any
  Bible; the copyright notice and licence are filled in and kept in the
  Bible's details; once added the Bible works offline;
- no redirects are followed; the request goes through the app's single HTTP
  client.

Never claimed: Strong's for MorphGNT, grammar for SBLGNT/TR, dictionary
forms for OSHB (its `lemma` field holds lexical numbers), or any
deuterocanonical text.

## 1. Westminster Leningrad Codex / Open Scriptures Hebrew Bible

**Upstream (recommended).** github.com/openscriptures/morphhb, commit
`3d15126fb1ef74867fc1434be1942e837932691f` (2024-08-27, "Merge pull request
#92 from Clear-Bible/earlier-requests"); no tags. Files `wlc/*.xml` (39 books)
plus `wlc/VerseMap.xml`, 28,634,502 bytes in total. The per-file hashes are in
`resources/scripture/catalogue.json`. OSIS header: `<revisionDesc>` 2016.03.03 "Updated to WLC
version 4.20", 2018.12.14 "Release of full morphology".

- Text: `<title>Westminster Leningrad Codex</title> … <rights>Public Domain</rights>`.
- Annotations: `<rights type="x-BY">Creative Commons Attribution 4.0</rights>`
  on the OSHB and "Open Scriptures Hebrew Morphology" works; README: "Lemma
  and morphology data are licensed under a Creative Commons Attribution 4.0
  International license. For attribution purposes, credit the Open
  Scriptures Hebrew Bible Project." LICENSE.md is CC BY 4.0.
- **Correction to v0.22:** `VerseMap.xml` has *no* rights statement (only
  an XML comment describing WLC→KJV mapping). It is covered only by the
  repository LICENSE.md (CC BY 4.0).
- Data: every one of 232,636 `<w>` elements has `lemma=` (augmented Strong's,
  e.g. `b/7225`, not lexical forms) and `morph=` (e.g. `HVqp3ms`).
- No share-alike; commercial use allowed; attribution and a licence link
  required. Hebrew (MT) versification; use VerseMap for English alignment.

**eBible `hboWLC`.** `hboWLC_usfm.zip`, 2,219,076 bytes, SHA-256
`2629165c…166587`, eBible update 2017-12-27. `copr.htm`: "Lemma and
morphology data are licensed under a Creative Commons Attribution-
ShareAlike license (http://creativecommons.org/licenses/by-sa/3.0/). For
attribution purposes, credit the Open Scriptures Hebrew Bible Project. The
text of the WLC remains in the public domain." It is a 2017 snapshot from
before OSHB's full morphology (2018): `x-morph` on 2,352 of 299,878 `\w`
words; Strong's zero-padded (`H0430`). Prefer upstream: newer data and a more
permissive licence.

## 2. Robinson–Pierpont Byzantine Textform 2018

eBible `grcmt_usfm.zip`, 866,796 bytes, SHA-256 `fa1a59e1…96d6`. copr.htm:
"The Greek New Testament according to the Byzantine Text form.
Robinson-Pierpont 2018 edition — Public Domain … Translation by: Maurice A.
Robinson and William G. Pierpont … This public domain Bible translation is
brought to you courtesy of eBible.org." Upstream byztxt/byzantine-majority-text
v3.3.2 = `27a45ff1b7be6c17ccbfeac414f3f55732ae8e28`; README "All the code and
text contained in this folder is in the Public Domain"; LICENSE.txt is the
Unlicense.

Verified: 140,121 `\w` tokens, all with `strong=` and `x-morph=` (Robinson
codes; `00-FRT` reproduces Robinson's 2004 code documentation). **Caveat:**
the wording is Robinson's *parsed* source: 0 accented words, all lower case,
word-final σ (27,431 tokens such as `λογοσ`, no `ς`). Upstream's Unicode CSV
with Strong's is unaccented too. Byzantine versification (Rom 14:24–26
doxology; Rom 16 ends at v. 24).

## 3. Textus Receptus with Strong's and morphology

eBible `grctr_usfm.zip`, 787,932 bytes, SHA-256 `32324123…6615`. INT: collated
from Stephanus 1550, Elzevir 1624 and Scrivener 1881 (majority of the three),
"Robert Adam Boyd, Ed., September 2022". copr.htm: "Manuscript comparison
footnotes dedicated to the Public Domain by Adam Boyd. The Greek Textus
Receptus New Testament is firmly in the Public Domain due to its age."
Verified: 138,126 `\w` tokens, all `strong=`, **no `x-morph`, no `lemma`**.
Who added the Strong's numbers is not stated (licence is "Public Domain").

Morphology on a TR exists only in byztxt/greektext-textus-receptus
(`7fd4d02c…`, 2017; README "Public Domain. Copy freely."), but in Robinson's
transliterated beta code (`en 1722 {PREP} arch 746 {N-DSF}`), which the app
cannot read. Exclude until a converter exists. The *TR with Strong's and
morphology* the brief asked for is not available in an importable format;
`grctr` is offered as Strong's-only.

## 4. SBL Greek New Testament

eBible `grcsbl_usfm.zip`, 917,905 bytes, SHA-256 `6807ec1c…a0`. copr.htm:
"copyright © 2010 Society of Biblical Literature and Logos Bible Software …
Updated to source dated 2023-07-10 from https://github.com/LogosBible/SBLGNT
… made available to you under the terms of the Creative Commons Attribution
license 4.0 … provided that: You include the above copyright and source
information. If you make any changes to the text, you must indicate that you
did so…". LogosBible/SBLGNT (HEAD `d7f35adc…`, 2026-10-24) LICENSE is CC BY
4.0; README: "v1.1 2022-12-19 Update public version and license on github",
"v1.2 2023-07-10 Add Jn 7:53-8:11". Verified: 136,377 `\w`, all Strong's, no
morph. No `signature.txt.asc` in this zip.

## 5. MorphGNT

github.com/morphgnt/sblgnt, master `aaed91e57c8e4a8dc9a2383e129ca5e75fe6393d`
(2024-01-21). It is ahead of tag `6.12` (`a2afca0e`) by corrections to 10 files.
27 files, 8,936,874 bytes; per-file hashes in
`resources/scripture/catalogue.json`. README: "The SBLGNT text itself is subject to the SBLGNT
EULA and the morphological parsing and lemmatization is made available under
a CC-BY-SA License (…/by-sa/3.0/)". The EULA reference is out of date: the
SBLGNT has been CC BY 4.0 since 2022. Cite: "Tauber, J. K., ed. (2017)
MorphGNT: SBLGNT Edition. Version 6.12 … DOI: 10.5281/zenodo.376200". README
notes the codes "were inherited from the CCAT tagging". MorphGNT relicensed
them as CC BY-SA and attaches no CCAT terms, but this is a provenance point
to record. Verified: 137,554 rows, all with 7 columns: lemma and parsing on
every word, no Strong's. SBLGNT v1.0 base, so no Jn 7:53–8:11.

## 6. Septuagint (Swete wanted)

- **No Swete in USFM or OSIS exists on eBible.** `grcsweteLXX` does not exist
  (404); `grcbrent` is *Brenton's* Greek text.
- **First1KGreek** (OpenGreekAndLatin, `03776b39…`, `data/tlg0527`, 119 files):
  Swete, TEI XML, `<licence target="https://creativecommons.org/licenses/by-sa/4.0/">`.
  The licence would allow download, but the app cannot import TEI → exclude for now.
  Derivatives nathans/lxx-swete and sleeptillseven/LXX-Swete are CC BY-SA 4.0
  in custom text formats. eliranwong/LXX-Swete-1930 is GPL-3 CSV, text
  "supplied by Pasquale Amicarelli … as a bibleworks module". Provenance
  unclear, and GPL on data complicates bundling → exclude.
- **eBible `grcbrent`** (`e8a6051a…72ea`, 1,729,129 bytes, source 2026-04-08):
  copr.htm "Public Domain". No `\w` markup. Books: 36 OT (`EZR` = Esdras B
  incl. Nehemiah, 23 ch; no separate NEH/EST/DAN), plus TOB JDT ESG WIS SIR
  BAR LJE SUS BEL 1MA 2MA 1ES MAN 3MA 4MA DAG (Azariah inside DAG 3).
  LXX versification (PSA has 151 chapters). **bundle-ok** as the importable
  LXX with deuterocanon.
- **eBible `grclxx`** (`81836c05…697b`): "Public Domain ℗ Orthodox Media
  Network … the only condition to publish the present text". The printed
  edition behind it is not identified, and eBible does not mark it
  certified. **Data error:** file `\id 2MA` is headed "ΜΑΚΚΑΒΑΙΩΝ Δ" and has
  18 chapters of 4 Maccabees; 2 Maccabees is missing. → exclude.
- **CCAT/CATSS LXX morphology** (Rahlfs-based): ccat.sas.upenn.edu could not
  be reached. Terms as quoted by a mirror: "for the use of students, teachers
  and scholars in study and education contexts. They are not to be used,
  either directly or indirectly, for commercial purposes without prior
  written consent…", and copies must be registered via the CCAT "User
  Declaration". Exclude it and every derivative. CenterBLC/LXX shows that a
  repository licence (MIT there) does not clear upstream data terms: its
  README says the data come from CATSS.

## 7. Clementine Vulgate

eBible `latVUC_usfm.zip`, 3,554,559 bytes, SHA-256 `439d1789…7826`. copr.htm:
"Clementine Vulgate of 1598 with Glossa Ordinaria Migne edition 1880 in
Latin — Public Domain … Translation by: Jerome". The digitisation source is
not stated. It contains 13,775 `\f` footnotes (Glossa Ordinaria) that the app
will show as notes. Books: 66 protocanonical, plus TOB JDT WIS SIR BAR (6 ch;
ch 6 = Letter of Jeremiah) 1MA 2MA. Greek additions sit inside EST (16 ch)
and DAN (14 ch: Song of the Three, Susanna, Bel). The Clementine appendix
(Prayer of Manasses, 3–4 Esdras) is absent. Vulgate versification. The
Clementine Vulgate Project text (VulSearch on SourceForge, GPL-2 package;
clementinevulgate.org and vulgate.org unreachable) is unverified and not in a
readable format.

## NA28

© 2012 Deutsche Bibelgesellschaft; no open licence or open digital release
found. die-bibel.de and academic-bible.com could not be reached from here, so
the publisher's terms were not re-read. Exclude: do not bundle, download or
link it as a source, and do not import NA28-derived apparatus.

## Bundling vs downloading; share-alike

- PD and CC BY 4.0 items (OSHB upstream, grcmt, grctr, grcsbl, grcbyz,
  grcbrent, latVUC) can be bundled or mirrored, including commercially. CC BY
  needs the attribution line in the app (an About/Sources screen and the Bible's
  info page) and a note that the data were converted (CC BY §3(a)(1)(B)).
- CC BY-SA 3.0 (MorphGNT annotations, eBible hboWLC annotations): bundling is
  lawful too. Share-alike attaches to the data and to adaptations of it,
  such as the app's converted word index if it is distributed. It does not
  attach to the app's code, which only aggregates it ("Collection"). Any
  redistributed copy or adaptation must stay CC BY-SA with attribution. When
  the user downloads at their own request from the original source, the
  project redistributes nothing; it should still show the attribution and
  licence. Hence these are *download-ok*, not *bundle-ok*.
- GPL-3 data (eliranwong Swete) and non-commercial or declaration-bound data
  (CCAT) are excluded.

