# Withheld set E — generator tooling (disclosed after its single evaluation)

Set E was built by an independent agent after the v0.24.5 recogniser freeze
(`fc0f538`), sealed at 2026-10-31T23:17:25Z and evaluated once
(`docs/audit/v0.24.5/withheld-e/`, original results, unchanged). Since that
run it is a **disclosed regression set**: no result on it is independent any
more. This folder keeps what is needed to rebuild it, so it can still serve
as regression evidence after the container that built it is gone.

## What is here

| File | What it is |
|---|---|
| `tools/*.py` | The builder's scripts, **byte for byte as sealed** (`tools.sha256`). They still name `/root/aw-set-e/` in a few places (`sys.path` entries, `make_manifest.py`'s output folder); Python falls back to the script's own folder for `import usfm`, so the generators run from here unchanged. |
| `SEAL.txt` | The seal as written by the builder (manifest SHA-256 and time). |
| `rebuild.sh` | Downloads each edition's USFM from eBible.org, checks it against the checksum set E used, and runs the matching generator. |
| `../../docs/corpus/withheld-e.tsv` | The sealed manifest (SHA-256 `18d145453f7236595a9b22619aaf3dc02b061e447ac22ba14157d97c9a7d5846`), with each document's checksum, layout and generator. |

**Not committed:** the generated documents and the eBible source zips. Three
of the twelve editions carry redistribution conditions (Reina Valera Gómez
and Valera 1602 Purificada: unaltered, not-for-profit reproduction only;
Wycliffe Modern Spelling: CC BY-NC-ND 4.0), and all of them are reproducible
from their authorised source. Before committing, the scripts were checked:
no Scripture text (only titles and the publishers' own attribution notices,
as printed on the documents' title pages), no secrets, no binary files.

## Rebuilding

```sh
scripts/set-e/rebuild.sh ~/aw-set-e-rebuilt          # all twelve
scripts/set-e/rebuild.sh ~/aw-set-e-rebuilt grcmt-txt  # one
cd src-tauri
AW_DOCUMENT_EDITIONS=~/aw-set-e-rebuilt/docs AW_ROLES=withheld-e \
  AW_AUDIT_OUT=~/aw-set-e-rebuilt/audit \
  cargo test --release --test real_document_editions -- --ignored --nocapture
```

### Sources (as used by the sealed set, 2026-10-31)

| Edition (eBible id) | Source zip SHA-256 |
|---|---|
| engwebu | `6408bba4ee3428d94472092069a7222680abf2382c6f93ee15cec0f0f4cc06fb` |
| enggnv | `6ed7356ea8bee3c042e168c8a1650aefed08f1bf24f158794506608b27715ebe` |
| engwyc2017 | `107912062138c2f17acc3866667602576ebe450ade78f9ec7a7605439b7ef55b` |
| engtnt | `a86faa9c69f021686e95bc26a7b598d6d7b33da391f523cac49cf8d3cd93822c` |
| deutkw | `c142f2c8297a8fb2694e23ac380d651b6fe3831dd9bb9f0912c6198ae0f78328` |
| fra_fob | `4631037b5b9e137f8496ad02424d1968da5405537f3bc98e2b2a44e08334d3ec` |
| sparvg | `fa839fbe14a06a9db8e57cea780896338fe8bd7b5b8434de576b3a48ecd9147a` |
| spav1602p | `fa940cf0240c2ce26a78de7a112bbf96d6fb3a5267bbc8b782313c9a59078c11` |
| porbrbsl | `2ae889138610240bfbff945b49dbb6f1157349564001219fe664ddc11d1dfec4` |
| kor | `287f2192323d34d65e2713a3adb24c4ab0f86e9e85430156cf630c5f1a55602e` |
| grcmt | `33b208ee58a1862696689dcd5db69ed0ae891cd08aea3c7c68a5055501874048` |
| heb | `0e62d1d3596a3a2862d257ee59f3cef0354104a574369bc2c39b47d457313451` |

### Dependencies (versions the sealed set was built with)

Python 3.13.18; WeasyPrint 70.0 (pydyf 0.13.0); ReportLab 5.0.1; fpdf2
2.9.0; python-docx 1.2.0; pandoc 3.1.3; LibreOffice 24.2.7.2 (headless);
fonts: Noto Serif / Noto Serif CJK KR / Noto Serif Hebrew, Liberation Serif
(Debian/Ubuntu `fonts-noto-core`, `fonts-noto-cjk`, `fonts-liberation`).

## What reproduces exactly, and what does not

Checked on 2026-11-01 by rebuilding all twelve from fresh eBible downloads
(every source zip matched the checksums above) with the committed scripts
and the versions listed:

| Document | Rebuilt file |
|---|---|
| porbrbsl-md.md, grcmt-txt.txt, engwyc2017-epubapoc.epub, engtnt-wp3col.pdf, heb-wprtl.pdf | **byte-identical** to the sealed file (manifest checksum) |
| engwebu-rl2col.pdf, deutkw-fpdfa5.pdf, kor-lopdf.pdf, enggnv-docxverse.docx, sparvg-pandocdocx.docx, spav1602p-lodocx.docx, fra_fob-pandocepub.epub | not byte-identical: the files embed creation dates and generated identifiers. Their **extracted text is identical** (PDF: same page count and same text, page by page; DOCX: same paragraphs; EPUB: same content documents without markup) |

So a rebuild reproduces what the recogniser reads, except where a future
program or font version changes the layout; the sealed checksums remain the
reference for the evaluated files.

## Limitations

* eBible.org rebuilds its files from time to time. A source zip whose
  checksum differs from the table above gives a different document; the
  script says so. Keeping the zips themselves (outside Git, where their
  licences allow) is the only way to guarantee an exact rebuild later.
* Different versions of the layout programs or fonts change line breaks and
  pages, and so may change recognition results. The table above records the
  versions used.
* Rebuilt documents are evidence for regression only. The evaluation of
  record is the single original run in `docs/audit/v0.24.5/withheld-e/`.
