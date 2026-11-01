# Development and regression corpus (46 documents) — baseline (2b5d4e2, v0.24.4 recogniser) against v0.24.5 frozen at fc0f538

Both runs on the same current eBible builds (the recorded v0.24.4 figures are for older builds). The 20 remaining verified differences are classified in `verified-differences.md`; they are the same 20 verses at the baseline and at the freeze.

## Results

| Edition | Printed | Detected before | Detected after | Verified before | Verified after | Δ verified | Verified FPs before / after | Verified wrong text before / after |
|---|---|---|---|---|---|---|---|---|
| deu1912-epub | 31102 | 30708 | 30708 (98.7 %) | 30606 | 30606 (98.4 %) | +0 | 0 / 0 | 0 / 0 |
| deu1912-md | 31102 | 30708 | 30708 (98.7 %) | 30606 | 30606 (98.4 %) | +0 | 0 / 0 | 0 / 0 |
| deu1912-pdf | 31102 | 31102 | 31102 (100.0 %) | 30709 | 30709 (98.7 %) | +0 | 0 / 0 | 0 / 0 |
| deuelo-pdf | 31102 | 31102 | 31102 (100.0 %) | 30514 | 30514 (98.1 %) | +0 | 0 / 0 | 0 / 0 |
| deutkw-docx | 31136 | 30511 | 30511 (98.0 %) | 30009 | 30005 (96.4 %) | -4 | 0 / 0 | 0 / 0 |
| deutkw-lopdf | 31136 | 30456 | 30456 (97.8 %) | 30018 | 30005 (96.4 %) | -13 | 0 / 0 | 2 / 2 |
| deutkw-pdf | 31136 | 30456 | 30456 (97.8 %) | 29858 | 29845 (95.9 %) | -13 | 0 / 0 | 5 / 5 |
| engwebp-book | 31098 | 30811 | 30811 (99.1 %) | 30145 | 30118 (96.8 %) | -27 | 0 / 0 | 0 / 0 |
| engwebp-epub | 31098 | 31017 | 31017 (99.7 %) | 30967 | 30967 (99.6 %) | +0 | 0 / 0 | 0 / 0 |
| engwebp-pdf | 31098 | 30811 | 30811 (99.1 %) | 29879 | 29835 (95.9 %) | -44 | 0 / 0 | 0 / 0 |
| engwebp-vpl | 31098 | 31098 | 31098 (100.0 %) | 30924 | 30924 (99.4 %) | +0 | 0 / 0 | 0 / 0 |
| engylt-pdf | 31102 | 31102 | 31102 (100.0 %) | 31061 | 31061 (99.9 %) | +0 | 0 / 0 | 0 / 0 |
| fob-epub | 31107 | 31107 | 31107 (100.0 %) | 31047 | 31047 (99.8 %) | +0 | 0 / 0 | 0 / 0 |
| fob-pdf | 31107 | 31107 | 31107 (100.0 %) | 30803 | 30803 (99.0 %) | +0 | 0 / 0 | 0 / 0 |
| fraLSG-epub | 31170 | 30986 | 30986 (99.4 %) | 30799 | 30797 (98.8 %) | -2 | 0 / 0 | 0 / 0 |
| fraLSG-luk | 1151 | 1098 | 1098 (95.4 %) | 1079 | 1079 (93.7 %) | +0 | 0 / 0 | 0 / 0 |
| fraLSG-pdf | 31170 | 30726 | 30725 (98.6 %) | 29552 | 29547 (94.8 %) | -5 | 0 / 0 | 0 / 0 |
| frajnd-epub | 31146 | 29962 | 29962 (96.2 %) | 29424 | 29424 (94.5 %) | +0 | 0 / 0 | 0 / 0 |
| frajnd-lopdf | 31146 | 27012 | 27012 (86.7 %) | 26413 | 26362 (84.6 %) | -51 | 0 / 0 | 0 / 0 |
| frajnd-pdf | 31146 | 30439 | 30439 (97.7 %) | 29727 | 29628 (95.1 %) | -99 | 0 / 0 | 0 / 0 |
| grcbyz-pdf | 7958 | 7937 | 7937 (99.7 %) | 7921 | 7921 (99.5 %) | +0 | 0 / 0 | 0 / 0 |
| grctr-epub | 7957 | 7957 | 7957 (100.0 %) | 7953 | 7953 (99.9 %) | +0 | 0 / 0 | 0 / 0 |
| grctr-pdf | 7957 | 7800 | 7800 (98.0 %) | 7648 | 7648 (96.1 %) | +0 | 0 / 0 | 0 / 0 |
| hboWLC-epub | 23192 | 22916 | 22916 (98.8 %) | 22916 | 22915 (98.8 %) | -1 | 0 / 0 | 0 / 0 |
| hboWLC-pdf | 23192 | 0 | 0 (0.0 %) | 0 | 0 (0.0 %) | +0 | 0 / 0 | 0 / 0 |
| kjv-epub | 31102 | 31102 | 31102 (100.0 %) | 31097 | 31097 (100.0 %) | +0 | 0 / 0 | 0 / 0 |
| kjv-pdf | 31102 | 31055 | 31055 (99.8 %) | 30993 | 30989 (99.6 %) | -4 | 0 / 0 | 0 / 0 |
| kor-docx | 30991 | 30941 | 30941 (99.8 %) | 30925 | 30923 (99.8 %) | -2 | 0 / 0 | 0 / 0 |
| kor-epub | 30991 | 30941 | 30941 (99.8 %) | 30925 | 30923 (99.8 %) | -2 | 0 / 0 | 0 / 0 |
| kor-pdf | 30991 | 30941 | 30941 (99.8 %) | 30925 | 30923 (99.8 %) | -2 | 0 / 0 | 0 / 0 |
| kor1961-docx | 31081 | 31081 | 31081 (100.0 %) | 31076 | 31053 (99.9 %) | -23 | 0 / 0 | 0 / 0 |
| kor1961-lopdf | 31081 | 30886 | 30886 (99.4 %) | 30816 | 30812 (99.1 %) | -4 | 0 / 0 | 0 / 0 |
| latVUC-epub | 31229 | 29781 | 29781 (95.4 %) | 29700 | 29700 (95.1 %) | +0 | 0 / 0 | 0 / 0 |
| latVUC-pdf | 31229 | 8692 | 8692 (27.8 %) | 8607 | 8528 (27.3 %) | -79 | 0 / 0 | 0 / 0 |
| porbr2018-epub | 31102 | 31102 | 31102 (100.0 %) | 30666 | 30666 (98.6 %) | +0 | 0 / 0 | 1 / 1 |
| porbr2018-pdf | 31102 | 30123 | 30123 (96.9 %) | 29553 | 29553 (95.0 %) | +0 | 0 / 0 | 3 / 3 |
| porbrbsl-docx | 31098 | 30816 | 30816 (99.1 %) | 30781 | 30779 (99.0 %) | -2 | 0 / 0 | 0 / 0 |
| porbrbsl-pdf | 31098 | 30815 | 30814 (99.1 %) | 30265 | 30240 (97.2 %) | -25 | 0 / 0 | 0 / 0 |
| rv1909-docx | 31084 | 31084 | 31084 (100.0 %) | 31009 | 31009 (99.8 %) | +0 | 0 / 0 | 0 / 0 |
| rv1909-epub | 31084 | 31084 | 31084 (100.0 %) | 31009 | 31009 (99.8 %) | +0 | 0 / 0 | 0 / 0 |
| rv1909-lopdf | 31084 | 31084 | 31084 (100.0 %) | 30961 | 30961 (99.6 %) | +0 | 0 / 0 | 0 / 0 |
| rv1909-md | 31084 | 31084 | 31084 (100.0 %) | 31009 | 31009 (99.8 %) | +0 | 0 / 0 | 0 / 0 |
| rv1909-pdf | 31084 | 30772 | 30772 (99.0 %) | 30590 | 30590 (98.4 %) | +0 | 0 / 0 | 0 / 0 |
| spabes-pdf | 31102 | 30673 | 30673 (98.6 %) | 30250 | 30250 (97.3 %) | +0 | 0 / 0 | 0 / 0 |
| spablm-docx | 31098 | 31098 | 31098 (100.0 %) | 31081 | 31081 (99.9 %) | +0 | 0 / 0 | 9 / 9 |
| spablm-pdf | 31098 | 29476 | 29476 (94.8 %) | 29118 | 29110 (93.6 %) | -8 | 0 / 0 | 0 / 0 |
| **Total** | 1315624 | 1253560 | 1253558 (95.3 %) | 1241934 | 1241524 (94.4 %) | -410 | 0 / 0 | 20 / 20 |

### Every verified verse with wrong text or not in the edition, after

* deutkw-lopdf 2KI 4:25 (extra, page 368): insert «» → « Als sie aber der Mann Gottes in einiger Entfernung erblickt»
* deutkw-lopdf LUK 9:43 (extra, page 936): insert «» → « Da aber alles sich verwunderte über allen seinen Thaten, sp»
* deutkw-pdf 1KI 8:12 (extra, page 459): insert «» → « Die Sonne stellte er ans Himmelszelt, Jahwe selbst aber hat»
* deutkw-pdf 1KI 8:13 (extra, page 459): insert «» → « eine Stätte zu deinem Wohnsitz für ewige Zeiten. Das steht »
* deutkw-pdf PSA 60:13 (extra, page 737): insert «» → « denn eitel ist Menschenhilfe!»
* deutkw-pdf ACT 2:14 (extra, page 1302): insert «» → « Ihr judäische Männer und ihr sämtliche Bewohner von Jerusal»
* deutkw-pdf 2CO 6:16 (extra, page 1387): insert «» → « Ich will unter ihnen wohnen und unter ihnen wandeln, und ic»
* porbr2018-epub HEB 13:25 (missing, page None): delete «Escrita da Itália para os Hebreus, e enviada por Timóteo» → «»
* porbr2018-pdf ROM 16:27 (missing, page 989): delete « Amém! Escrita de Corinto aos Romanos, e enviada por Febe, s» → «»
* porbr2018-pdf GAL 6:18 (missing, page 1019): delete « Amém!Escrita de Roma para os Gálatas» → «»
* porbr2018-pdf 1TI 6:21 (missing, page 1041): delete « A primeira carta a Timóteo foi escrita de Laodiceia, que é » → «»
* spablm-docx EZR 4:11 (missing, page None): delete « Al rey Artajerjes, de parte de tus siervos, los hombres de » → «»
* spablm-docx EZR 4:17 (missing, page None): delete « Salud.» → «»
* spablm-docx EZR 5:7 (missing, page None): delete « Al rey Darío, toda paz.» → «»
* spablm-docx EZR 7:12 (missing, page None): delete « al sacerdote Esdras, escriba instruido en la ley del Dios d» → «»
* spablm-docx DAN 4:1 (missing, page None): delete « a todos los pueblos, naciones y lenguas que habitan en toda» → «»
* spablm-docx DAN 4:19 (missing, page None): delete « Beltsasar respondió y dijo: “Señor mío, que el sueño sea pa» → «»
* spablm-docx DAN 6:25 (missing, page None): delete « “Que la paz se os multiplique.» → «»
* spablm-docx DAN 9:4 (missing, page None): delete « “Oh Señor, Dios grande y temible, que guardas el pacto y la» → «»
* spablm-docx DAN 9:11 (missing, page None): delete « “Por ello se ha derramado sobre nosotros la maldición y el » → «»
