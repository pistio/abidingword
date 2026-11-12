#!/usr/bin/env bash
# Scores spablm-docx-derived-v2 with an unchanged harness binary.
#   run-spablm-docx-derived-v2.sh <harness binary> <corpus folder> <v2 folder> <output folder>
# The harness knows the edition as `spablm-docx` with the file
# `spablm_usfm_lo.docx`; v2 is staged under that name in a folder of its
# own. The corpus folder's original (v1) is not touched.
set -euo pipefail
bin=$1; corpus=$2; v2=$3; out=$4
want=7e5c90804f203d780a116af9969c9b2bffbc82d2a9056d5b30f0c0fc80aa4925
got=$(sha256sum "$v2/spablm_usfm_lo.v2.docx" | cut -d' ' -f1)
[ "$got" = "$want" ] || { echo "v2 checksum $got, expected $want" >&2; exit 1; }
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
cp "$v2/spablm_usfm_lo.v2.docx" "$stage/spablm_usfm_lo.docx"
cp "$corpus/spablm_usfm.zip" "$stage/"
mkdir -p "$out"
echo "spablm-docx-derived-v2 $got scored as key spablm-docx" > "$out/IDENTITY.txt"
sha256sum "$bin" >> "$out/IDENTITY.txt"
AW_PIPELINE=structural AW_EDITIONS=spablm-docx AW_ROLES=impl,regression \
  AW_DOCUMENT_EDITIONS="$stage" AW_AUDIT_OUT="$out" "$bin" --ignored --nocapture
