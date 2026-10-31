# v0.24.3 final development and regression run against the baseline

Run at commit 01ac499. *Baseline usable*: verses v0.24.2 accepted. *Detected*: verses in chapters whose structure was accepted. *Text-verified*: verses whose text became the Bible's text. *FPs*: verses the edition does not have. *Verified with extra/changed/cut text*: see `verified-differences.md`. *Location*: verified verses whose recorded page and segment hold their text.

| Edition | Role | Baseline usable | Detected (accepted) | Text-verified | Verified % | Detected-only FPs | Verified FPs | Verified with extra/changed/cut text | Location |
|---|---|---|---|---|---|---|---|---|---|
| deu1912-epub | impl | 30708 | 30708 (98.7 %) | 30624 | 98.5 % | 0 | 0 | 0 | 100.0 % |
| deu1912-md | impl | 30708 | 30708 (98.7 %) | 30624 | 98.5 % | 0 | 0 | 0 | 100.0 % |
| deu1912-pdf | impl | 31102 | 31102 (100.0 %) | 30903 | 99.4 % | 0 | 0 | 0 | 100.0 % |
| deuelo-pdf | impl | 31102 | 31102 (100.0 %) | 30724 | 98.8 % | 0 | 0 | 0 | 100.0 % |
| deutkw-docx | regression | 16946 | 30511 (98.0 %) | 30044 | 96.5 % | 0 | 0 | 0 | 100.0 % |
| deutkw-lopdf | regression | 16895 | 30456 (97.8 %) | 30055 | 96.5 % | 0 | 0 | 3 | 99.8 % |
| deutkw-pdf | regression | 30407 | 30456 (97.8 %) | 30030 | 96.4 % | 0 | 0 | 7 | 100.0 % |
| engwebp-book | impl | 30811 | 30811 (99.1 %) | 30293 | 97.4 % | 0 | 0 | 0 | 100.0 % |
| engwebp-epub | impl | 30994 | 31017 (99.8 %) | 30972 | 99.6 % | 0 | 0 | 0 | 100.0 % |
| engwebp-pdf | impl | 30811 | 30811 (99.1 %) | 30032 | 96.6 % | 0 | 0 | 0 | 100.0 % |
| engwebp-vpl | impl | 29776 | 31098 (100.0 %) | 30928 | 99.5 % | 0 | 0 | 1 | 99.8 % |
| engylt-pdf | regression | 31102 | 31102 (100.0 %) | 31090 | 100.0 % | 0 | 0 | 0 | 100.0 % |
| fob-epub | impl | 31107 | 31107 (100.0 %) | 31074 | 99.9 % | 0 | 0 | 0 | 100.0 % |
| fob-pdf | impl | 31107 | 31107 (100.0 %) | 30992 | 99.6 % | 0 | 0 | 0 | 100.0 % |
| fraLSG-epub | impl | 31122 | 31122 (99.8 %) | 30935 | 99.2 % | 0 | 0 | 0 | 100.0 % |
| fraLSG-luk | impl | 1098 | 1098 (95.4 %) | 1089 | 94.6 % | 0 | 0 | 0 | 93.7 % |
| fraLSG-pdf | impl | 30568 | 30550 (98.0 %) | 29804 | 95.6 % | 0 | 0 | 0 | 95.4 % |
| frajnd-epub | regression | 24323 | 29962 (96.2 %) | 29428 | 94.5 % | 0 | 0 | 0 | 100.0 % |
| frajnd-lopdf | regression | 11024 | 27012 (86.7 %) | 26445 | 84.9 % | 0 | 0 | 0 | 100.0 % |
| frajnd-pdf | regression | 24944 | 30439 (97.7 %) | 29864 | 95.9 % | 0 | 0 | 0 | 97.5 % |
| grcbyz-pdf | regression | 7937 | 7937 (99.7 %) | 7921 | 99.5 % | 0 | 0 | 0 | 100.0 % |
| grctr-epub | impl | 7957 | 7957 (100.0 %) | 7955 | 100.0 % | 0 | 0 | 0 | 100.0 % |
| grctr-pdf | impl | 7570 | 7800 (98.0 %) | 7662 | 96.3 % | 0 | 0 | 0 | 99.9 % |
| hboWLC-epub | impl | 22916 | 22916 (98.8 %) | 22916 | 98.8 % | 0 | 0 | 0 | 100.0 % |
| hboWLC-pdf | impl | 0 | 0 (0.0 %) | 0 | 0.0 % | 0 | 0 | 0 | — |
| kjv-epub | impl | 31102 | 31102 (100.0 %) | 31100 | 100.0 % | 0 | 0 | 0 | 100.0 % |
| kjv-pdf | impl | 31055 | 31055 (99.8 %) | 31046 | 99.8 % | 0 | 0 | 0 | 100.0 % |
| kor-docx | impl | — | 30941 (99.8 %) | 30925 | 99.8 % | 0 | 0 | 0 | 100.0 % |
| kor-epub | impl | 30941 | 30941 (99.8 %) | 30925 | 99.8 % | 0 | 0 | 0 | 100.0 % |
| kor-pdf | impl | 30941 | 30941 (99.8 %) | 30925 | 99.8 % | 0 | 0 | 0 | 100.0 % |
| kor1961-docx | impl | 29827 | 31081 (100.0 %) | 31076 | 100.0 % | 0 | 0 | 0 | 100.0 % |
| kor1961-lopdf | impl | 30770 | 30886 (99.4 %) | 30816 | 99.1 % | 0 | 0 | 0 | 100.0 % |
| latVUC-epub | impl | 29449 | 29781 (95.4 %) | 29721 | 95.2 % | 0 | 0 | 0 | 100.0 % |
| latVUC-pdf | impl | 6860 | 6860 (22.0 %) | 6838 | 21.9 % | 0 | 0 | 0 | 100.0 % |
| porbr2018-epub | impl | 31102 | 31102 (100.0 %) | 31032 | 99.8 % | 0 | 0 | 1 | 100.0 % |
| porbr2018-pdf | impl | 28541 | 30123 (96.9 %) | 30054 | 96.6 % | 0 | 0 | 3 | 100.0 % |
| porbrbsl-docx | impl | 30816 | 30816 (99.1 %) | 30784 | 99.0 % | 0 | 0 | 0 | 100.0 % |
| porbrbsl-pdf | impl | 30815 | 30815 (99.1 %) | 30540 | 98.2 % | 0 | 0 | 0 | 100.0 % |
| rv1909-docx | impl | 31084 | 31084 (100.1 %) | 31026 | 99.9 % | 0 | 0 | 0 | 100.0 % |
| rv1909-epub | impl | 31084 | 31084 (100.1 %) | 31026 | 99.9 % | 0 | 0 | 0 | 100.0 % |
| rv1909-lopdf | impl | 31084 | 31084 (100.1 %) | 30983 | 99.7 % | 0 | 0 | 0 | 99.7 % |
| rv1909-md | impl | 31084 | 31084 (100.1 %) | 31026 | 99.9 % | 0 | 0 | 0 | 100.0 % |
| rv1909-pdf | impl | 30772 | 30772 (99.1 %) | 30609 | 98.5 % | 0 | 0 | 0 | 99.2 % |
| spabes-pdf | impl | 30673 | 30673 (98.6 %) | 30625 | 98.5 % | 0 | 0 | 0 | 100.0 % |
| spablm-docx | regression | 21015 | 31098 (100.0 %) | 31084 | 100.0 % | 0 | 0 | 9 | 100.0 % |
| spablm-pdf | regression | 29476 | 29476 (94.8 %) | 29302 | 94.2 % | 0 | 0 | 0 | 99.9 % |
