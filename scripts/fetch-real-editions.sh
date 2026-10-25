#!/usr/bin/env sh
# Downloads the real PDF Bibles the document-recognition audit reads
# (src-tauri/tests/real_document_editions.rs) and checks each against the
# SHA-256 recorded when the audit was run. Nothing is committed to Git.
#
#   scripts/fetch-real-editions.sh <folder>
#   AW_DOCUMENT_EDITIONS=<folder> AW_AUDIT_OUT=<folder>/audit \
#     cargo test --release --test real_document_editions -- --ignored --nocapture
#
# eBible.org rebuilds its PDFs from time to time: a checksum mismatch means a
# newer build, not necessarily a problem. The audit compares each PDF with the
# USFM of the same edition shipped in resources/scripture/.
set -eu
dir="${1:?folder to download into}"
mkdir -p "$dir"
cd "$dir"
while read -r file sha; do
  id="${file%%_*}"
  [ -f "$file" ] || curl -sSfL -o "$file" "https://ebible.org/pdf/$id/$file"
  got="$(sha256sum "$file" | cut -d' ' -f1)"
  if [ "$got" = "$sha" ]; then
    echo "$file: ok"
  else
    echo "$file: checksum differs (a newer eBible build?): $got" >&2
  fi
done <<'LIST'
engwebp_all.pdf 4c36e4008a65d568c6f8ad0ab0483b268db59e2afff98f3f194bccef7aae8d2e
deu1912_all.pdf 7a7cdc3f4db5a94ef2d5043149a05a8deaf658fcc794e2ae978b76d9a37f0fd9
engwebp_book.pdf cb7e17798a1e042a6e729c7f43a61837d6f6a7c450288c411e3400c7c55cf943
fraLSG_all.pdf c6463fdc8679e13575b9b8d0891f2fe4db324e193bd68b57b6f62caffa2388f2
LIST
