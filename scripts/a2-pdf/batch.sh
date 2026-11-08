#!/bin/bash
# sequential run of all documents (must-include first); one engine process at a time
cd /root/a2-pdf
ORDER="G05 G03 G04 G10 G09 F07 kor-pdf kor-r2col kor-lopdf kor1961-lopdf grctr-pdf grcbyz-pdf G13 G07 engtnt-wp3col engwebu-rl2col engbsb-rwpdf2col engkjvcpb-rwpdfmargin hbo-rwpdfrtl-rut heb-wprtl F01 F03 F06 F13 deu1912-rdropcap spabll-rrlab2col"
ALL=$(python3 -I -c "import json;print(' '.join(d['key'] for d in json.load(open('out/docs.json'))))")
for k in $ORDER $ALL; do
  [ -f out/done/$k ] && continue
  python3 -I scripts/run_doc.py $k >> out/batch.log 2>&1 && mkdir -p out/done && touch out/done/$k
done
echo BATCH-DONE >> out/batch.log
