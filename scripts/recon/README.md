# Reconstructed corpora (regression evidence, never independent)

The generators of withheld sets B, C and D were kept outside Git in the
sessions that built them and are lost. These scripts rebuild documents **in
the layouts their manifests describe**, from the same eBible.org USFM, so the
disclosed failure classes can be reproduced and general fixes tested. A
reconstruction is not the sealed document: other program versions, other
checksums, sometimes another page size, and details the manifests leave open
(listed in each manifest's layout column). Results on reconstructions are
**regression** evidence. The original blind evaluations stay the records:
`docs/audit/v0.24.3/withheld-b-first/`, `docs/audit/v0.24.3/withheld-c/`,
`docs/audit/v0.24.4/withheld-d/`.

| File | What it does |
|---|---|
| `usfm_blocks.py` | A small USFM reader shared by both builders (no knowledge of the recogniser). |
| `build_recon_d.py` | Set D layouts → `docs/corpus/recon-d.tsv` (role `recon-d`). |
| `build_recon_bc.py` | Sets B and C layouts → `docs/corpus/recon-bc.tsv` (role `recon-bc`), plus two eBible original PDFs downloaded as they are. |
| `compare_audits.py` | Before/after tables from two audit folders, with every wrong verified verse. |

## Commands

```sh
mkdir -p ~/recon-src && cd ~/recon-src
for id in eng-Brenton grcbrent kor deu1912 portft spaonbv spapddpt; do
  curl -sSfLO https://ebible.org/Scriptures/${id}_usfm.zip; done
python3 scripts/recon/build_recon_d.py ~/recon-src ~/aw-recon && cp ~/recon-src/*.zip ~/aw-recon/
# (from the repository root)
mkdir -p ~/recon-bc-src
for f in $(cut -f3 docs/corpus/withheld-b.tsv docs/corpus/withheld-c.tsv | grep _usfm.zip | sort -u); do
  curl -sSfL -o ~/recon-bc-src/$f https://ebible.org/Scriptures/$f; done
python3 scripts/recon/build_recon_bc.py ~/recon-bc-src ~/aw-recon-bc   # also fetches the two eBible PDFs
cp ~/recon-bc-src/*.zip ~/aw-recon-bc/
cd src-tauri
AW_DOCUMENT_EDITIONS=~/aw-recon AW_ROLES=recon-d AW_AUDIT_OUT=~/audit/recon-d \
  cargo test --release --test real_document_editions -- --ignored --nocapture
AW_DOCUMENT_EDITIONS=~/aw-recon-bc AW_ROLES=recon-bc AW_AUDIT_OUT=~/audit/recon-bc \
  cargo test --release --test real_document_editions -- --ignored --nocapture
python3 ../scripts/recon/compare_audits.py ~/audit/before ~/audit/after "title" --wrong
```

LibreOffice is single-instance: never run two builders that call `soffice`
at the same time.

## Dependencies used for the v0.24.5 evidence

Python 3.13.18; WeasyPrint 70.0; ReportLab 5.0.1; fpdf2 2.9.0; python-docx
1.2.0; EbookLib (pip, for `francl`); pandoc 3.1.3; LibreOffice 24.2.7.2;
PyMuPDF 2.0 (inspection only); fonts Liberation Serif, Noto Serif, Noto
Serif CJK KR, Noto Serif Hebrew.

## Sources used for the v0.24.5 evidence (2026-10-31)

eBible.org rebuilds its files; a different checksum means a different build
and possibly different documents and results.

### Set D reconstructions

| File | SHA-256 |
|---|---|
| deu1912_usfm.zip | `76fb83f108dd9029e2ec63c582b5a7e2acc46736d7b1659f802dff27f5563af7` |
| eng-Brenton_usfm.zip | `9b1a4e09381885c658c1cc70ea56a64c125c85d8697a4f03cb6ad66a1bcd985c` |
| grcbrent_usfm.zip | `5b777b54e662de634858a409dd6b1b50ea3ef1e18afc395075183a7773184aca` |
| kor_usfm.zip | `287f2192323d34d65e2713a3adb24c4ab0f86e9e85430156cf630c5f1a55602e` |
| portft_usfm.zip | `7ac1efce32577296c38f4965e47d782785ffee70570f9e0a610a96e89b231053` |
| spaonbv_usfm.zip | `9cff82ca8d431c7920db16a0c7784ecb223ffd4be56a091ecbd4ff35ff2b5ef2` |
| spapddpt_usfm.zip | `e0d556bb087b16b039a7b7bded086ec3ab95a7bff5eb18b89751f2e8f8785e1b` |

### Set B/C reconstructions

| File | SHA-256 |
|---|---|
| deu1951_usfm.zip | `c61ecc937a33fb8d9946ba49169bcb6648d4774a4b8a639ba1daac86c7614076` |
| deuelbbk_usfm.zip | `044e2f63f47f45c4648ab5fbd638d1672e862c975c97db7c066e433742a670c3` |
| eng-asv_usfm.zip | `00b7b6c76cb1eb67b22ca51bd5ac13c2df526c13e1c38e78b862a7a2367f7071` |
| eng-rv_usfm.zip | `69252651f89ed5922f650a9a965dff5cc88981708fe5dad87a392f734794208e` |
| engDRA_usfm.zip | `fcb5de45f1f6083d56b738690e5b4b3bb6f0d87522d1d18329c57a4b09e05210` |
| engbsb_usfm.zip | `75ca91d5a5c69ca4cbdeabdb03ea1a458db20cf3966abe15d86df086acf7fbc4` |
| engjps_usfm.zip | `b4034f7ed8ade5c256431d286bb03d651d8a3690040c30c5fb011dd1f8ada807` |
| engkjvcpb_usfm.zip | `70af6d12ca81f1c9b6d18beffa5558914af1517d4f0853533cd61a38e6fa2bd0` |
| englsv_usfm.zip | `b5fd551d49086785430ca4d670e2914b53c63fd65cb84717b7b200466305861f` |
| engoebus_usfm.zip | `da884dcf57e84b79c45fbf3943ec55dab0e834c68723732d9222be1220210bc7` |
| engwebster_usfm.zip | `0f3c1122422adba66cf1e0c98e11db117beebbe40994d2d8afb5e80cc84942cf` |
| francl_usfm.zip | `d1472f1f90ac50e519d6db4c07f4791e9f54fa73118e400a9ac95f52e6a66d31` |
| frasbl_usfm.zip | `1cae473f9d4ded70fc7651f129e03780634997b8265c15bc5c5a93af139bad03` |
| grc-tisch_usfm.zip | `d1556165fa76caf4659ab4b4c3e76d2b8f5783467dc44edad91b699f9f5d78dd` |
| grcsbl_usfm.zip | `04c5c536b8ef977e30de3f37e2ec16300e8452173322dc2e8c25b1a02bb10a73` |
| hbo_usfm.zip | `6d8ff0cfebaf59de3973c66b6403891199d4f049f84d9a62d2d18c8301fd4870` |
| kor_usfm.zip | `287f2192323d34d65e2713a3adb24c4ab0f86e9e85430156cf630c5f1a55602e` |
| porblt_usfm.zip | `88513d3c0e852f219ca64d040591b640858e8f8292fe9ef503bdcc94ca88811b` |
| poronbv_usfm.zip | `7737bf255a0c79d0d802e28d032672e81d275fbddae23da054341ed5e153b79c` |
| spabll_usfm.zip | `02a28e412ac61d1683b18352557fbf73afd6cfe1a1d6da3d9b56c78a5b2c2d33` |
| spavbl_usfm.zip | `7c7b35801c6d18789fd4f4dbc259d0686d1322b6269f9c70db84526627485638` |

### eBible original PDFs (sets B and C, current builds)

| File | SHA-256 |
|---|---|
| eng-rv_book.pdf | `ec7bc275d991d44a324b077f0b0053b6b04c111972a6c9406cd50b9eb76b2a13` |
| engoebus_a4.pdf | `f45bfced0a75816d3dc7a3c7468a061e47516911641be0d5ec7f5963e2fbcd85` |

Nothing generated is committed: the documents are rebuilt from these
sources; the eBible zips and PDFs are fetched from eBible.org.
