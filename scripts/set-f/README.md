# Withheld set F — generator tooling (disclosed after its single evaluation)

Set F was built by an independent agent after the v0.24.6 recogniser freeze
(`324cfd1`), sealed at 2026-11-01T20:30:29Z and evaluated once
(`docs/audit/set-f/`, original results, unchanged). It is now a **disclosed
regression set**: no later run on it is independent.

| File | What it is |
|---|---|
| `tools/*.py` | The builder's scripts, byte for byte (`tools.sha256`). They find their folder from their own location (`tools/../src`, `tools/../docs`). |
| `sources.sha256` | The 13 source files (5 eBible USFM zips, 8 CrossWire texts as JSON from scrollmapper/bible_databases) with the checksums they had when set F was built. |
| `fetch_sources.sh` | Copies `tools/` into a folder, downloads the sources into its `src/`, checks the checksums. |
| `SEAL.txt` | The seal as written by the builder. |
| `../../docs/corpus/withheld-f.tsv`, `../../docs/audit/set-f/GROUND_TRUTH.md` | The sealed manifest and the ground-truth record: per document the exact command that made its reference USFM ("Reference made") and its generator script. |

Not committed: the documents, the reference zips and the sources (one text
is CC BY-SA 4.0; all are reproducible from their sources). Checked before
committing: the scripts hold no Scripture text (only titles, attribution
notices and book-name tables), no secrets, no binary files.

## Rebuilding

```sh
scripts/set-f/fetch_sources.sh ~/set-f
cd ~/set-f
# reference USFM, e.g. (all commands are in GROUND_TRUTH.md, "Reference made"):
python3 tools/scroll2usfm.py src/scroll/GreVamvas.json ell all docs/F11-ell-vamvas.usfm.zip docs/f11.log --cp-greek
python3 tools/reduce_ebible.py src/eb/grctcgnt_usfm.zip docs/F09-grc-tcgnt.usfm.zip nt
# documents: one script per document
python3 tools/make_f11_vamvas_txt.py
python3 tools/make_f09_tcgnt_md.py
```

Checked on 2026-11-02 from this copy of the scripts: F09 and F11 (documents
and reference zips) rebuild **byte-identical** to the sealed files. The
other eleven were not rebuilt in this check; PDF, EPUB and DOCX files embed
dates and identifiers, and depend on the program and font versions recorded
in the manifest's generator column (LibreOffice and Typst need their
programs installed).
