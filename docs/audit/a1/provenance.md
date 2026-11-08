# A1 provenance integrity, per edition (structure-aware reading, `205a453`)

Every verse the recogniser read from an EPUB is traced to ordered spans of the original entry. *Exact*: the spans' own text, joined by the fixed quote rule, equals the verse's text. *Spacing differs*: same characters, different white space. *Untraced*: no spans could be named; never verified. *Id agrees / contradicts*: an id at the verse's start encoding chapter and verse. *Gated*: verification withdrawn by the structural check, by cause (stored as the existing reasons `boundary` or `noteSign`). *Links*: roles given to every `<a href>` by where it leads.

| Set | Edition | Verses traced | Exact | Spacing differs | Untraced | Id agrees | Id contradicts | Gated (cause: n) | Links (role: n) |
|---|---|---|---|---|---|---|---|---|---|
| gall | G01 | 25062 | 24917 | 0 | 145 | 24742 | 166 | idContradicts: 166 | navigation: 73771, noteCaller: 21360 |
| gall | G02 | 0 | – | – | – | – | – | – | nothing read |
| gall | G03 | 0 | – | – | – | – | – | – | nothing read |
| gall | G04 | 0 | – | – | – | – | – | – | not an EPUB (frozen reading) |
| gall | G06 | 0 | – | – | – | – | – | – | not an EPUB (frozen reading) |
| gall | G07 | 0 | – | – | – | – | – | – | not an EPUB (frozen reading) |
| gall | G08 | 0 | – | – | – | – | – | – | not an EPUB (frozen reading) |
| gall | G09 | 0 | – | – | – | – | – | – | nothing read |
| gall | G10 | 0 | – | – | – | – | – | – | not an EPUB (frozen reading) |
| gall | G11 | 0 | – | – | – | – | – | – | not an EPUB (frozen reading) |
| gall | G12 | 8400 | 8383 | 17 | 0 | 0 | 0 | – | navigation: 343 |
| gall | G13 | 0 | – | – | – | – | – | – | not an EPUB (frozen reading) |
| gall | G14 | 773 | 764 | 9 | 0 | 0 | 0 | – | navigation: 6 |
| f | F04 | 31140 | 31140 | 0 | 0 | 31140 | 0 | – | navigation: 1251 |
| f | F10 | 4403 | 4403 | 0 | 0 | 0 | 0 | – | – |
| f | F12 | 2526 | 2526 | 0 | 0 | 0 | 0 | – | navigation: 68 |
| e | engwyc2017-epubapoc | 73 | 73 | 0 | 0 | 0 | 0 | – | navigation: 24, noteCaller: 24 |
| e | fra_fob-pandocepub | 10423 | 10421 | 2 | 0 | 0 | 0 | – | navigation: 30 |
| recon | spapddpt-repub2 | 24837 | 24761 | 0 | 76 | 0 | 0 | swallowedVerseNumber: 2, uncertainLink: 13 | ambiguous: 40, navigation: 192, noteCaller: 232 |
| bc | englsv-rpandocepub | 31104 | 30577 | 527 | 0 | 0 | 0 | – | – |
| bc | francl-rebooklibepub | 15918 | 15662 | 0 | 256 | 0 | 0 | – | navigation: 73, noteCaller: 5066 |
| bc | frasbl-rpandocepub | 28331 | 28327 | 3 | 1 | 0 | 0 | – | navigation: 1708, noteCaller: 1708 |
| dev | deu1912-epub | 31092 | 31062 | 30 | 0 | 24965 | 0 | – | navigation: 1326 |
| dev | engwebp-epub | 31098 | 31083 | 14 | 1 | 24969 | 0 | – | navigation: 3370, noteCaller: 1572 |
| dev | fob-epub | 31107 | 31105 | 2 | 0 | 24979 | 0 | – | navigation: 1326 |
| dev | fraLSG-epub | 31170 | 31156 | 0 | 14 | 25029 | 0 | – | navigation: 11077, noteCaller: 9751 |
| dev | frajnd-epub | 30921 | 30919 | 2 | 0 | 24786 | 0 | swallowedVerseNumber: 6 | navigation: 7666, noteCaller: 6340 |
| dev | grctr-epub | 7957 | 7957 | 0 | 0 | 6633 | 0 | – | navigation: 665, noteCaller: 343 |
| dev | hboWLC-epub | 23192 | 23192 | 0 | 0 | 18385 | 0 | – | navigation: 1016 |
| dev | kjv-epub | 31102 | 31100 | 2 | 0 | 24974 | 0 | – | navigation: 8287, noteCaller: 6959 |
| dev | kor-epub | 30990 | 30990 | 0 | 0 | 24896 | 0 | – | navigation: 1324 |
| dev | latVUC-epub | 31228 | 31194 | 33 | 1 | 25101 | 0 | swallowedVerseNumber: 1 | navigation: 15260, noteCaller: 13775 |
| dev | porbr2018-epub | 31102 | 31060 | 24 | 18 | 24957 | 0 | untraced: 16 | navigation: 2817, noteCaller: 1290 |
| dev | rv1909-epub | 31084 | 31078 | 6 | 0 | 24960 | 0 | – | navigation: 1326 |
