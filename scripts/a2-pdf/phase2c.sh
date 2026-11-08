#!/bin/bash
# pdfminer.six text (optional 4th reference): must-include documents first, only after the main batch
# has finished a document (so results files are not written concurrently); 1800 s timeout each
cd /root/a2-pdf
ORDER="G05 G03 G04 G10 G09 F07 kor-pdf kor-r2col kor-lopdf kor1961-lopdf grctr-pdf grcbyz-pdf G13 G07 engtnt-wp3col engwebu-rl2col engbsb-rwpdf2col engkjvcpb-rwpdfmargin hbo-rwpdfrtl-rut heb-wprtl F01 F03 F06 F13 deu1912-rdropcap spabll-rrlab2col"
ALL=$(python3 -I -c "import json;print(' '.join(d['key'] for d in json.load(open('out/docs.json'))))")
while true; do
  left=0; ran=0
  for k in $ORDER $ALL; do
    [ -f out/done-pdfminer/$k ] && continue
    left=1
    [ -f out/done/$k ] || continue
    nice -n 19 python3 -I scripts/run_doc.py $k pdfminer >> out/pdfminer.log 2>&1; mkdir -p out/done-pdfminer; touch out/done-pdfminer/$k; ran=1; break
  done
  [ $left = 0 ] && break
  [ $ran = 0 ] && sleep 30
done
echo PDFMINER-DONE >> out/pdfminer.log
