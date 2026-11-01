#!/usr/bin/env sh
# Rebuilds the documents of withheld set E (v0.24.5, disclosed since its
# single evaluation at fc0f538) from eBible.org USFM, with the generator
# scripts exactly as the independent builder wrote them (tools/, checksums in
# tools.sha256). Nothing here is the sealed set itself: see README.md for what
# is and is not reproduced byte for byte.
#
#   scripts/set-e/rebuild.sh <folder> [key ...]
#
# Downloads each source zip into <folder>/src (checking it against the
# checksum the sealed set used; a different checksum means eBible rebuilt
# the edition, and the rebuilt document is then NOT the evaluated one), then
# writes the documents and the zips into <folder>/docs, where the audit
# harness reads them (AW_DOCUMENT_EDITIONS=<folder>/docs AW_ROLES=withheld-e).
set -eu
out="${1:?folder to build into}"; shift || true
here="$(cd "$(dirname "$0")" && pwd)"
mkdir -p "$out/src" "$out/docs"
only=" $* "
# key | eBible id | script | document | sha256 of the source zip used in set E
while IFS='|' read -r key id script doc sha; do
  [ -z "$key" ] && continue
  [ "$only" != "  " ] && case "$only" in *" $key "*) ;; *) continue ;; esac
  zip="$out/src/${id}_usfm.zip"
  [ -f "$zip" ] || curl -sSfL -o "$zip" "https://ebible.org/Scriptures/${id}_usfm.zip"
  got="$(sha256sum "$zip" | cut -d' ' -f1)"
  if [ "$got" != "$sha" ]; then
    echo "$key: source zip differs from the one set E used ($got): a newer eBible build" >&2
  fi
  rm -rf "$out/src/$id"; mkdir -p "$out/src/$id"
  unzip -q -o "$zip" -d "$out/src/$id"
  cp "$zip" "$out/docs/"
  case "$script" in
    gen_kor_lopdf.py|gen_spav1602p_lodocx.py)
      prof="$(mktemp -d)"
      python3 "$here/tools/$script" "$out/src/$id" "$out/docs/$doc" "$prof"
      rm -rf "$prof" ;;
    *) python3 "$here/tools/$script" "$out/src/$id" "$out/docs/$doc" ;;
  esac
  echo "$key: $(sha256sum "$out/docs/$doc" | cut -d' ' -f1)"
done <<'LIST'
engwebu-rl2col|engwebu|gen_engwebu_rl2col.py|engwebu-rl2col.pdf|6408bba4ee3428d94472092069a7222680abf2382c6f93ee15cec0f0f4cc06fb
enggnv-docxverse|enggnv|gen_enggnv_docx.py|enggnv-docxverse.docx|6ed7356ea8bee3c042e168c8a1650aefed08f1bf24f158794506608b27715ebe
engwyc2017-epubapoc|engwyc2017|gen_engwyc_epub.py|engwyc2017-epubapoc.epub|107912062138c2f17acc3866667602576ebe450ade78f9ec7a7605439b7ef55b
engtnt-wp3col|engtnt|gen_engtnt_wp3col.py|engtnt-wp3col.pdf|a86faa9c69f021686e95bc26a7b598d6d7b33da391f523cac49cf8d3cd93822c
deutkw-fpdfa5|deutkw|gen_deutkw_fpdf.py|deutkw-fpdfa5.pdf|c142f2c8297a8fb2694e23ac380d651b6fe3831dd9bb9f0912c6198ae0f78328
fra_fob-pandocepub|fra_fob|gen_frafob_pandocepub.py|fra_fob-pandocepub.epub|4631037b5b9e137f8496ad02424d1968da5405537f3bc98e2b2a44e08334d3ec
sparvg-pandocdocx|sparvg|gen_sparvg_pandocdocx.py|sparvg-pandocdocx.docx|fa839fbe14a06a9db8e57cea780896338fe8bd7b5b8434de576b3a48ecd9147a
spav1602p-lodocx|spav1602p|gen_spav1602p_lodocx.py|spav1602p-lodocx.docx|fa940cf0240c2ce26a78de7a112bbf96d6fb3a5267bbc8b782313c9a59078c11
porbrbsl-md|porbrbsl|gen_porbrbsl_md.py|porbrbsl-md.md|2ae889138610240bfbff945b49dbb6f1157349564001219fe664ddc11d1dfec4
kor-lopdf|kor|gen_kor_lopdf.py|kor-lopdf.pdf|287f2192323d34d65e2713a3adb24c4ab0f86e9e85430156cf630c5f1a55602e
grcmt-txt|grcmt|gen_grcmt_txt.py|grcmt-txt.txt|33b208ee58a1862696689dcd5db69ed0ae891cd08aea3c7c68a5055501874048
heb-wprtl|heb|gen_heb_wprtl.py|heb-wprtl.pdf|0e62d1d3596a3a2862d257ee59f3cef0354104a574369bc2c39b47d457313451
LIST
