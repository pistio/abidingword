#!/bin/bash
# Final conversion checks for set G: runs on the files in docs/ (and the renderer inputs kept
# in sources/render/) and writes the results to sources/checks/final/.
set -u
R=/root/aw-set-g
T=$R/tools
O=$R/sources/checks/final
D=$R/docs
mkdir -p $O
py() { python3 -I "$@"; }
seq_check() {  # key ref doctext extra-args...
  local k=$1 ref=$2 txt=$3; shift 3
  printf "%s sequence: " "$k"; py $T/check_sequence.py $ref $txt $O/${k}_seq "$@"
}
for f in $D/G*; do
  case $f in *.usfm.zip) continue;; esac
  k=$(basename $f | cut -c1-3)
  py $T/doc_text.py $f $O/${k}_doc.txt > /dev/null
  py $T/validate_usfm.py $D/$(basename ${f%.*}).usfm.zip $O/${k}_structure.json > $O/${k}_structure.txt
done
pdftotext -enc UTF-8 $D/G04_he_ebible_heblb_a4.pdf $O/G04_doc_default.txt
seq_check G01 $D/G01_*.usfm.zip $O/G01_doc.txt --strip-regex '\[[A-Z]{1,3}\]|\[\d+\]' --numcheck arabic
seq_check G02 $D/G02_*.usfm.zip $O/G02_doc.txt --strip-regex '\[\d+\]' --numcheck arabic --book-order GEN,EXO,LEV,NUM,DEU,JOS,JDG,RUT,1SA,2SA,1KI,2KI,1CH,2CH,EZR,NEH,EST,JOB,PSA,PRO,ECC,SNG,ISA,JER,LAM,EZK,DAN,HOS,JOL,AMO,OBA,JON,MIC,NAM,HAB,ZEP,HAG,ZEC,MAL,MAT,MRK,LUK,JHN,ACT,ROM,1CO,2CO,GAL,EPH,PHP,COL,1TH,2TH,1TI,2TI,TIT,PHM,HEB,1PE,2PE,1JN,2JN,3JN,JAS,JUD,REV
seq_check G03 $D/G03_*.usfm.zip $O/G03_doc.txt --numcheck arabic
seq_check G04 $D/G04_*.usfm.zip $O/G04_doc_default.txt --numcheck arabic
seq_check G05 $D/G05_*.usfm.zip $O/G05_doc.txt --numcheck arabic --book-order GEN,RUT,PSA,ROM,JDT,WIS,TOB,SIR,BAR,1MA,2MA,MAN
seq_check G06 $D/G06_*.usfm.zip $O/G06_doc.txt --numcheck arabic
seq_check G07 $D/G07_*.usfm.zip $O/G07_doc.txt --numcheck arabic
seq_check G08 $D/G08_*.usfm.zip $O/G08_doc.txt --numcheck arabic
seq_check G09 $D/G09_*.usfm.zip $O/G09_doc.txt --numcheck cjk
seq_check G10 $D/G10_*.usfm.zip $O/G10_doc.txt --base --book-order SNG,RUT,LAM,ECC,EST,JON
seq_check G11 $D/G11_*.usfm.zip $O/G11_doc.txt --numcheck arabic
seq_check G12 $D/G12_*.usfm.zip $O/G12_doc.txt --numcheck arabic
seq_check G13 $D/G13_*.usfm.zip $O/G13_doc.txt --numcheck arabic
seq_check G14 $D/G14_*.usfm.zip $O/G14_doc.txt --numcheck arabic
# renderer inputs (exactly what was given to the typesetting program)
for x in "G07 g07/g07.html arabic -" "G09 g09/g09.typ cjk -" "G10 g10/G10_mam.fodt hebrew SNG,RUT,LAM,ECC,EST,JON"; do
  set -- $x
  py $T/render_input_text.py $R/sources/render/$2 $O/$1_input.txt > /dev/null
  extra=""; [ "$4" != "-" ] && extra="--book-order $4"
  printf "%s renderer-input sequence: " $1; py $T/check_sequence.py $D/$1_*.usfm.zip $O/$1_input.txt $O/$1_seqinput --numcheck $3 $extra
done
# verbatim (punctuation included) samples
vb() { local k=$1 txt=$2; shift 2; printf "%s verbatim: " $k; py $T/check_sample_verbatim.py $D/${k}_*.usfm.zip $txt $O/${k}_verbatim.tsv "$@"; }
vb G01 $O/G01_doc.txt --strip '\[[A-Z]{1,3}\]|\[\d+\]'
vb G02 $O/G02_doc.txt --strip '(?<=\n) {8}|\[\d+\] *'
vb G03 $O/G03_doc.txt --join-hyphens
vb G04 $O/G04_doc_default.txt
vb G05 $O/G05_doc.txt --join-hyphens
vb G06 $O/G06_doc.txt
vb G07 $O/G07_input.txt
vb G08 $O/G08_doc.txt
vb G09 $O/G09_input.txt --strip ' \[[^\]]*\] '
vb G10 $O/G10_input.txt
vb G11 $O/G11_doc.txt
vb G12 $O/G12_doc.txt
vb G13 $O/G13_doc.txt --join-hyphens
vb G14 $O/G14_doc.txt
# G10: pointed Hebrew from LibreOffice; pdftotext's default (logical-order) output compared on letters without marks
pdftotext -enc UTF-8 $D/G10_he_mam_megillot_jonah.pdf $O/G10_doc_default.txt
printf "G10 pdf-default base-letter sequence: "; py $T/check_sequence.py $D/G10_*.usfm.zip $O/G10_doc_default.txt $O/G10_seq_default --base --book-order SNG,RUT,LAM,ECC,EST,JON
