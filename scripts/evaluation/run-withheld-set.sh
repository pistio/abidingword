#!/usr/bin/env bash
# Runs the frozen candidate ONCE on a sealed, withheld set, and refuses to
# run on anything else (Phase A1.5). Prepared for Set H; NOT run on it.
#
#   run-withheld-set.sh <seal folder> <seal commit> <documents folder> \
#                       <harness binary> <expected binary sha256> <output folder>
#
# <seal folder>      in this repository, committed alone in <seal commit>:
#                    manifest.tsv, checksums.sha256, provenance.tsv, allowed/,
#                    review-ledger.md, access-records.md, SEAL.txt
#                    (see scripts/evaluation/check-set.py)
# <documents folder> outside Git: the documents and references, nothing else
# <output folder>    outside Git (reports quote the documents' text); must
#                    not exist
#
# Every name in the seal must be a plain relative path, and no file in the
# seal or documents folder may be a symbolic link (check-set.py).
#
# Before reading any document, it refuses if: the output folder exists; the
# binary is not the frozen one; any check of check-set.py fails (the seal
# folder is not exactly what the seal commit added, or has changed since;
# a file differs from its checksum or is unlisted; a document is
# byte-identical to a disclosed one; a key is a disclosed edition's; a run
# record exists in the repository; the set was started before).
#
# Then it marks the documents folder as started (.run-started), so this set
# is never started again from that folder, and runs every document in its
# own process under the ceilings below. When the last document ends it
# checksums the outputs and COMMITS the run record (RUN.txt, OUTPUTS.sha256,
# CHECK.json) to <seal folder>/../run/ in a commit of its own; that commit
# is the anchor score-set.py requires (--run-record). The score is added to
# the same folder later, in another commit.
#
# AW_RUN_TIMEOUT_SECONDS, AW_RUN_MEMORY_KB and AW_DISCLOSED_INVENTORY exist
# for rehearsals on disclosed documents only. RUN.txt records the values
# used, and score-set.py declares any run that departs from the protocol's
# values not a protocol run.
set -euo pipefail
[ $# -eq 6 ] || { sed -n '2,30p' "$0" >&2; exit 2; }
here=$(cd "$(dirname "$0")" && pwd)
seal=$(cd "$1" && pwd); seal_commit=$2; docs=$(cd "$3" && pwd); bin=$4; want_bin=$5; out=$6
timeout_s=${AW_RUN_TIMEOUT_SECONDS:-1800}
memory_kb=${AW_RUN_MEMORY_KB:-4194304}
inventory=${AW_DISCLOSED_INVENTORY:-$here/../../docs/evaluation/disclosed-documents.sha256}
repo=$(git -C "$seal" rev-parse --show-toplevel) || { echo "refused: the seal folder is not in a Git repository" >&2; exit 2; }
seal_rel=$(python3 -c 'import os,sys; print(os.path.relpath(os.path.realpath(sys.argv[1]), os.path.realpath(sys.argv[2])))' "$seal" "$repo")
record_rel="$(dirname "$seal_rel")/run"

[ -e "$out" ] && { echo "refused: $out exists; a withheld set is run once" >&2; exit 2; }
got_bin=$(sha256sum "$bin" | cut -d' ' -f1)
[ "$got_bin" = "$want_bin" ] || { echo "refused: binary $got_bin is not the frozen $want_bin" >&2; exit 2; }
git -C "$repo" var GIT_COMMITTER_IDENT > /dev/null 2>&1 \
  || { echo "refused: Git cannot commit here (no committer identity); the run record could not be committed" >&2; exit 2; }
check_report=$(mktemp)
trap 'rm -f "$check_report"' EXIT
python3 "$here/check-set.py" "$seal" "$docs" --seal-commit="$seal_commit" \
  --inventory="$inventory" --report="$check_report" > /dev/null \
  || { echo "refused: the set failed its checks (above); nothing was read" >&2; exit 2; }

# From here on the set counts as run, whatever happens next.
( set -o noclobber; echo "started $(date -u +%Y-%m-%dT%H:%M:%SZ) seal $seal_commit" > "$docs/.run-started" ) \
  || { echo "refused: $docs/.run-started exists; this set has been run" >&2; exit 2; }
mkdir -p "$out"
cp "$check_report" "$out/CHECK.json"
manifest="$seal/manifest.tsv"
{
  echo "started $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "seal_commit $(git -C "$seal" rev-parse "$seal_commit^{commit}")"
  echo "manifest $(sha256sum "$manifest" | cut -d' ' -f1)"
  echo "checksums $(sha256sum "$seal/checksums.sha256" | cut -d' ' -f1)"
  echo "binary $got_bin"
  echo "runner $(sha256sum "$0" | cut -d' ' -f1)"
  echo "check_set $(sha256sum "$here/check-set.py" | cut -d' ' -f1)"
  echo "inventory $(sha256sum "$inventory" | cut -d' ' -f1)"
  echo "limits timeout_seconds $timeout_s memory_kb $memory_kb"
  echo "host $(uname -srm)"
} > "$out/RUN.txt"
# Every document, each in its own process, so that one crash, one timeout
# or one memory overrun is that document's result, not the run's end.
awk -F'\t' 'NR>1 && !/^#/ && NF>1 {print $1}' "$manifest" | while read -r key; do
  start=$(date +%s)
  set +e
  ( ulimit -v "$memory_kb"
    AW_PIPELINE=structural AW_CORPUS_MANIFEST="$manifest" AW_EDITIONS="$key" \
    AW_DOCUMENT_EDITIONS="$docs" AW_AUDIT_OUT="$out" \
    timeout "$timeout_s" "$bin" --ignored --nocapture ) > "$out/$key.log" 2>&1
  status=$?
  set -e
  echo "$key exit $status seconds $(( $(date +%s) - start ))" >> "$out/RUN.txt"
done
echo "ended $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> "$out/RUN.txt"
(cd "$out" && find . -type f ! -name OUTPUTS.sha256 -print0 | sort -z | xargs -0 sha256sum) > "$out/OUTPUTS.sha256"
# The run record, committed at once and alone: the anchor of the assessment.
mkdir -p "$repo/$record_rel"
for f in RUN.txt OUTPUTS.sha256 CHECK.json; do cp "$out/$f" "$repo/$record_rel/$f"; done
git -C "$repo" add -- "$record_rel/RUN.txt" "$record_rel/OUTPUTS.sha256" "$record_rel/CHECK.json"
git -C "$repo" commit -q -m "run record: $seal_rel (seal $seal_commit)" \
  -- "$record_rel/RUN.txt" "$record_rel/OUTPUTS.sha256" "$record_rel/CHECK.json" \
  || { echo "the run finished but its record could not be committed; commit $record_rel/ alone before any assessment" >&2; exit 3; }
echo "done; run record committed: $(git -C "$repo" rev-parse HEAD) ($record_rel/). Score with --run-record=<that commit>."
