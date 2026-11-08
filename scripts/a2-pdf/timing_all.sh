#!/bin/bash
cd /root/a2-pdf
while kill -0 $(cat out/timing.pid) 2>/dev/null; do sleep 10; done
python3 -I scripts/timing.py >> out/timing.log 2>&1
python3 -I scripts/timing_pdfminer.py G13 G09 F06 kor-lopdf engtnt-wp3col F03 G05 >> out/timing.log 2>&1
echo TIMING-DONE >> out/timing.log
