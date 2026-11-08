#!/bin/bash
# structure probes for every document (runs alongside the batch; timings are re-measured later in a clean pass)
cd /root/a2-pdf
KEYS=$(python3 -I -c "import json;print(' '.join(d['key'] for d in json.load(open('out/docs.json'))))")
for k in $KEYS; do [ -f out/struct/$k.json ] || nice -n 19 python3 -I scripts/struct_doc.py $k >> out/struct.log 2>&1; done
echo STRUCT-DONE >> out/struct.log
