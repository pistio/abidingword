#!/bin/bash
# after the main batch: structure probes, then pdfminer text (optional 4th reference), then rescore
cd /root/a2-pdf
until grep -q BATCH-DONE out/batch.log; do sleep 15; done
KEYS=$(python3 -I -c "import json;print(' '.join(d['key'] for d in json.load(open('out/docs.json'))))")
for k in $KEYS; do [ -f out/struct/$k.json ] || python3 -I scripts/struct_doc.py $k --pdfminer >> out/struct.log 2>&1; done
echo STRUCT-DONE >> out/struct.log
for k in $KEYS; do grep -q '"pdfminer"' out/results/$k.json 2>/dev/null || python3 -I scripts/run_doc.py $k pdfminer >> out/pdfminer.log 2>&1; done
echo PDFMINER-DONE >> out/pdfminer.log
