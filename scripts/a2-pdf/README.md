# A2 PDF engine measurement scripts

The scripts that produced `docs/audit/a2/pdf-engines.md`, kept as they ran.
They expect the measurement work directory `/root/a2-pdf` and the disclosed
corpora at the paths recorded in `docs/audit/v0.24.7/corpus-checksums.sha256`,
and they verify every input's checksum first (`manifest.py`). The engines were
run from Python: pypdfium2 (PDFium), PyMuPDF (MuPDF), `pdftotext` (Poppler) and
pdfminer.six. The current extractor ran as the app's own test binary
(`dump-ea5f944`, the frozen code's unit dump). These scripts are measurement
tooling only: nothing in the application uses them.
