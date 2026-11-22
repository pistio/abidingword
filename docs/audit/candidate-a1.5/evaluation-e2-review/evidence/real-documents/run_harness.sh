#!/usr/bin/env bash
# run_harness.sh <binary> <docs folder> <manifest> <out folder>: each manifest key in its own process,
# exactly as run-withheld-set.sh invokes the harness (structural pipeline, protocol ceilings).
set -u
bin=$1; docs=$2; manifest=$3; out=$4
mkdir -p "$out"
awk -F'\t' 'NR>1 && !/^#/ && NF>1 {print $1}' "$manifest" | while read -r key; do
  start=$(date +%s)
  ( ulimit -v 4194304
    AW_PIPELINE=structural AW_CORPUS_MANIFEST="$manifest" AW_EDITIONS="$key" \
    AW_DOCUMENT_EDITIONS="$docs" AW_AUDIT_OUT="$out" \
    timeout 1800 "$bin" --ignored --nocapture ) > "$out/$key.log" 2>&1
  echo "$key exit $? seconds $(( $(date +%s) - start ))" | tee -a "$out/RUN.txt"
done
