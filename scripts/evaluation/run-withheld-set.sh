#!/usr/bin/env bash
# Runs the frozen candidate ONCE on a sealed, withheld set, and refuses to
# run on anything else (Phase A1.5). Prepared for Set H; NOT run.
#
#   run-withheld-set.sh <set folder> <harness binary> <expected binary sha256> <output folder>
#
# <set folder> holds, as the builder sealed it:
#   manifest.tsv   the harness's manifest columns (key, files, reference,
#                  language, script, format, layout, role, source, books,
#                  sha256, generator); a control has reference "-"
#   SEAL.txt       the manifest's SHA-256 on a line of its own
#   checksums.sha256  every document and reference file, `sha256sum` format
#   docs/          the documents and reference files
#
# The run stops before reading anything if: the output folder already
# exists (one run only), the binary is not the frozen one, the manifest is
# not the sealed one, or any file differs from its checksum.
set -euo pipefail
set_dir=$(cd "$1" && pwd); bin=$2; want_bin=$3; out=$4
[ -e "$out" ] && { echo "refused: $out exists; a withheld set is run once" >&2; exit 2; }
got_bin=$(sha256sum "$bin" | cut -d' ' -f1)
[ "$got_bin" = "$want_bin" ] || { echo "refused: binary $got_bin is not the frozen $want_bin" >&2; exit 2; }
seal=$(grep -Eo '^[0-9a-f]{64}$' "$set_dir/SEAL.txt" | head -1)
got_manifest=$(sha256sum "$set_dir/manifest.tsv" | cut -d' ' -f1)
[ "$seal" = "$got_manifest" ] || { echo "refused: manifest $got_manifest is not the sealed $seal" >&2; exit 2; }
(cd "$set_dir" && sha256sum -c --quiet checksums.sha256) || { echo "refused: a sealed file changed" >&2; exit 2; }
role=$(awk -F'\t' 'NR>1 && !/^#/ {print $8; exit}' "$set_dir/manifest.tsv")
mkdir -p "$out"
{
  echo "started $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "binary $got_bin"
  echo "manifest $got_manifest"
  echo "role $role"
  uname -srm
} > "$out/RUN.txt"
# Every document, each in its own process, so that one crash, one timeout
# or one memory overrun is that document's result, not the run's end.
awk -F'\t' 'NR>1 && !/^#/ {print $1}' "$set_dir/manifest.tsv" | while read -r key; do
  start=$(date +%s)
  set +e
  ( ulimit -v 4194304
    AW_PIPELINE=structural AW_CORPUS_MANIFEST="$set_dir/manifest.tsv" AW_EDITIONS="$key" \
    AW_ROLES="$role" AW_DOCUMENT_EDITIONS="$set_dir/docs" AW_AUDIT_OUT="$out" \
    timeout 1800 "$bin" --ignored --nocapture ) > "$out/$key.log" 2>&1
  status=$?
  set -e
  echo "$key exit $status seconds $(( $(date +%s) - start ))" >> "$out/RUN.txt"
done
echo "ended $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$out/RUN.txt"
(cd "$out" && find . -type f ! -name OUTPUTS.sha256 -print0 | sort -z | xargs -0 sha256sum) > "$out/OUTPUTS.sha256"
echo "done; outputs sealed in $out/OUTPUTS.sha256"
