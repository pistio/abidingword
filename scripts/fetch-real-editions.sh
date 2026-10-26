#!/usr/bin/env sh
# Downloads the real-edition corpus the document-recognition audit reads
# (src-tauri/tests/real_document_editions.rs) and checks every file against
# the SHA-256 recorded in docs/corpus/sources.tsv. Nothing is committed to Git.
#
#   scripts/fetch-real-editions.sh <folder>
#   AW_DOCUMENT_EDITIONS=<folder> AW_AUDIT_OUT=<folder>/audit \
#     cargo test --release --test real_document_editions -- --ignored --nocapture
#
# docs/corpus/editions.tsv lists the editions (and which are kept hidden until
# the final evaluation); docs/corpus/sources.tsv says where each file comes
# from. Sources marked "derived" are built afterwards by
# scripts/build-derived-editions.py (LibreOffice writes a fresh date into each,
# so their checksums are for reference only).
#
# eBible.org rebuilds its files from time to time: a checksum mismatch means a
# newer build, not necessarily a problem, but the audit's figures are for the
# recorded builds. A file that is not a PDF, EPUB, ZIP or text as expected is
# removed rather than kept.
set -eu
dir="${1:?folder to download into}"
root="$(cd "$(dirname "$0")/.." && pwd)"
mkdir -p "$dir"
cd "$dir"
status=0
list="$(mktemp)"
trap 'rm -f "$list"' EXIT
tail -n +2 "$root/docs/corpus/sources.tsv" > "$list"
while IFS="$(printf '\t')" read -r file source sha; do
  case "$source" in
    derived) continue ;;
    repo:*) [ -f "$file" ] || cp "$root/${source#repo:}" "$file" ;;
    *'#'*)
      # A file inside a ZIP archive.
      if [ ! -f "$file" ]; then
        archive="$(basename "${source%%#*}")"
        [ -f "$archive" ] || curl -sSfL --max-filesize 400000000 -o "$archive" "${source%%#*}"
        unzip -o -q "$archive" "${source#*#}"
      fi
      ;;
    https://*) [ -f "$file" ] || curl -sSfL --max-filesize 400000000 -o "$file" "$source" ;;
    *) echo "$file: unknown source $source" >&2; continue ;;
  esac
  # The file must be what its name says.
  kind="$(file -bL --mime-type "$file")"
  case "$file:$kind" in
    *.pdf:application/pdf | *.epub:application/epub+zip | *.epub:application/zip | \
      *.zip:application/zip | *.txt:text/*) ;;
    *) echo "$file: unexpected content ($kind); removed" >&2; rm -f "$file"; status=1; continue ;;
  esac
  got="$(sha256sum "$file" | cut -d' ' -f1)"
  if [ "$got" = "$sha" ]; then
    echo "$file: ok"
  else
    echo "$file: checksum differs (a newer build?): $got" >&2
  fi
done < "$list"
cat <<'NEXT'
Derived documents (converted layouts), with LibreOffice installed:
  scripts/build-derived-editions.py <folder> spaRV1909 docx,pdf,md
  scripts/build-derived-editions.py <folder> deu1912 md
  scripts/build-derived-editions.py <folder> porbrbsl docx
  scripts/build-derived-editions.py <folder> kor docx
  scripts/build-derived-editions.py <folder> kor1961 docx,pdf --usfm="제 {n} 장"
Hidden (final evaluation only):
  scripts/build-derived-editions.py <folder> deutkw docx,pdf --usfm="Kapitel {n}"
  scripts/build-derived-editions.py <folder> spablm docx --usfm="Capítulo {n}"
  scripts/build-derived-editions.py <folder> frajnd pdf --usfm="Chapitre {n}"
NEXT
exit $status
