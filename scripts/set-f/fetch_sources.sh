#!/usr/bin/env sh
# Fetches set F's sources into <folder>/src as the builder had them and checks
# each against the checksum recorded when set F was built (2026-11-01). A
# different checksum means the source was updated since: documents rebuilt
# from it are then NOT the evaluated ones.
#   scripts/set-f/fetch_sources.sh <folder>
set -eu
out="${1:?folder}"; here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out/src/eb" "$out/src/scroll" "$out/docs"
cp -r "$here/tools" "$out/tools"
while read -r sha path; do
  case "$path" in
    src/eb/*) url="https://ebible.org/Scriptures/$(basename "$path")" ;;
    src/scroll/*) url="https://raw.githubusercontent.com/scrollmapper/bible_databases/master/formats/json/$(basename "$path")" ;;
  esac
  [ -f "$out/$path" ] || curl -sSfL -o "$out/$path" "$url"
  got="$(sha256sum "$out/$path" | cut -d' ' -f1)"
  [ "$got" = "$sha" ] && echo "$path: ok" || echo "$path: CHANGED since set F ($got)" >&2
done < "$here/sources.sha256"
