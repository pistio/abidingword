#!/bin/bash
# Fetch raw wikitext of ko.wikisource "페이지:셩경 개역.pdf/N" pages (1938 Korean Revised Bible) gently.
out=/root/aw-set-g/sources/wikisource/pages
for r in "446 452" "1426 1437" "1850 1867" "1912 1918" "1934 1941" "1946 1947"; do
  set -- $r
  for ((p=$1;p<=$2;p++)); do
    f=$out/p$p.txt
    [ -s $f ] && continue
    curl -sS -m 60 -A "Mozilla/5.0 (aw-set-g test-set builder)" "https://ko.wikisource.org/w/index.php?title=%ED%8E%98%EC%9D%B4%EC%A7%80:%EC%85%A9%EA%B2%BD_%EA%B0%9C%EC%97%AD.pdf/$p&action=raw" -o $f
    echo "$p $(wc -c < $f)"
    sleep 2
  done
done
echo DONE
