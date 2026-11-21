#!/usr/bin/env python3
"""SYNTHETIC regression sets for evaluation candidate A1.5-E2.

Every verse here is invented for these tests (sentences built from small
word banks, or single common words for Greek and Hebrew). No text comes from
any withheld collection, and none is a published translation. Each edition
says so in its own \\id line ("SYNTHETIC E2 test edition, invented text").

    make_e2_sets.py <seal root> <documents root> [set,set...]

Writes, for each set, <documents root>/<set>/ (Markdown documents in the
derivation script's layout, and a USFM zip reference each) and
<seal root>/<set>/seal/ (manifest, checksums, provenance, composition,
allowed lists, ledger, access records). SEAL.txt is written afterwards by
seal-set.py. Also writes <seal root>/<set>/expected.json: what each probe is
for and what the corrected rules must find.

The sets (see docs/audit/candidate-a1.5/evaluation-e2/EVALUATION-FREEZE.md):

  e2-fid   F1: wording that differs only in ways the A1.5 rule accepted
           (word joining and splitting, case, punctuation, decomposed marks,
           cantillation), and a canonically equivalent document
  e2-mat   material changes both rules catch (no regression the other way)
  e2-ref   B1/B2: verse zero, chapters above the 66-book counts, a book
           outside the 66, unusual labels, duplicate numbers
  e2-abs   B3: a whole extra chapter verified (Malachi 4), an extra verse,
           a book the reference lacks
  e2-name  C3: a file named like a one-book file (_gen) holding Ruth
  e2-map   C4: a reference archive with a verse map
  e2-utf8  C5: a reference that is not UTF-8
"""
import hashlib
import io
import json
import os
import sys
import unicodedata
import zipfile

NFD = lambda s: unicodedata.normalize("NFD", s)
NFC = lambda s: unicodedata.normalize("NFC", s)
LABEL = "SYNTHETIC E2 test edition, invented text"

# ------------------------------------------------------------ invented text

EN = (["the shepherd", "a widow", "the elders", "her mother", "the reapers", "a stranger",
       "the young men", "the household", "the gleaners", "the kinsman", "the women", "a servant",
       "the potter", "the weaver", "the watchman", "the fisherman", "the miller"],
      ["walked", "rested", "waited", "spoke softly", "listened", "returned", "gathered barley",
       "worked", "travelled", "sat down", "sang", "looked up", "kept watch"],
      ["by the gate", "in the field", "near the well", "under the olive tree", "beside the road",
       "at the threshing floor", "in the village", "on the hillside", "by the river", "at the market"],
      ["at dawn", "in the evening", "after the harvest", "before the rain", "during the feast",
       "at midday", "in the cool of the day", "when the lamps were lit"],
      ["and gave thanks.", "and said nothing more.", "and the day was long.", "and all was quiet.",
       "and they were glad.", "and bread was shared.", "and the wind was still.", "and no one hurried."])
ES = (["el pastor", "una viuda", "los ancianos", "su madre", "los segadores", "un forastero",
       "los jóvenes", "la familia", "las espigadoras", "el pariente", "las mujeres", "un siervo"],
      ["caminó", "descansó", "esperó", "habló en voz baja", "escuchó", "volvió", "recogió cebada",
       "trabajó", "viajó", "se sentó", "cantó"],
      ["junto a la puerta", "en el campo", "cerca del pozo", "bajo el olivo", "al lado del camino",
       "en la era", "en la aldea", "en la ladera", "junto al río"],
      ["al amanecer", "por la tarde", "después de la cosecha", "antes de la lluvia",
       "durante la fiesta", "a mediodía", "al caer la noche"],
      ["y dio gracias.", "y no dijo nada más.", "y el día fue largo.", "y todo estaba en calma.",
       "y se alegraron.", "y compartieron el pan.", "y nadie tuvo prisa."])
GR = (["ὁ πρεσβύτερος", "ἡ ἐκκλησία", "οἱ ἀδελφοί", "ὁ ἀγαπητός", "ἡ μήτηρ", "ὁ ξένος",
       "οἱ νεανίσκοι", "ὁ οἶκος", "ὁ δοῦλος", "αἱ γυναῖκες", "ὁ ποιμήν"],
      ["περιεπάτησεν", "ἀνεπαύσατο", "ἔμεινεν", "ἐλάλησεν", "ἤκουσεν", "ὑπέστρεψεν",
       "εἰργάσατο", "ἐκάθισεν", "ᾖσεν", "ἀνέβλεψεν"],
      ["παρὰ τὴν πύλην", "ἐν τῷ ἀγρῷ", "ἐγγὺς τοῦ φρέατος", "ὑπὸ τὴν ἐλαίαν", "παρὰ τὴν ὁδόν",
       "ἐν τῇ κώμῃ", "ἐπὶ τοῦ ὄρους", "παρὰ τὸν ποταμόν"],
      ["πρωΐ", "ὀψίας", "μετὰ τὸν θερισμόν", "πρὸ τοῦ ὑετοῦ", "ἐν τῇ ἑορτῇ", "μεσημβρίας"],
      ["καὶ ηὐχαρίστησεν.", "καὶ οὐδὲν εἶπεν.", "καὶ ἡ ἡμέρα μακρὰ ἦν.", "καὶ πάντα ἡσύχαζεν.",
       "καὶ ἐχάρησαν.", "καὶ ἄρτον ἔκλασαν."])
# Common pointed Hebrew words, assembled into invented sentences.
HE = (["הָאִישׁ", "הָאִשָּׁה", "הַזְּקֵנִים", "הָרֹעֶה", "הַקּוֹצְרִים", "הַגֵּר", "הַנְּעָרִים"],
      ["הָלַךְ", "יָשַׁב", "שָׁמַע", "שָׁב", "עָבַד", "נָח", "שָׁר"],
      ["בַּשָּׂדֶה", "בָּעִיר", "עַל־הַדֶּרֶךְ", "בַּשַּׁעַר", "אֵצֶל הַבְּאֵר", "בַּכֶּרֶם"],
      ["בַּבֹּקֶר", "בָּעֶרֶב", "בַּיּוֹם הַהוּא", "אַחֲרֵי הַקָּצִיר"],
      ["וַיִּשְׂמַח׃", "וְלֹא אָמַר דָּבָר׃", "וַיֹּאכַל לֶחֶם׃", "וַיִּשְׁקֹט׃"])
BANKS = {"en": EN, "es": ES, "grc": GR, "he": HE}


def sentence(lang, book, c, v):
    """One invented verse: deterministic, no digits, distinct per (book, c, v)."""
    s, vb, p, t, e = BANKS[lang]
    seed = int(hashlib.sha256(f"E2{book}{c}:{v}".encode()).hexdigest(), 16)
    pick = lambda xs, k: xs[(seed >> (k * 8)) % len(xs)]
    if lang == "he":
        return f"{pick(vb, 1)} {pick(s, 0)} {pick(p, 2)} {pick(t, 3)} {pick(e, 4)}"
    text = f"{pick(s, 0)} {pick(vb, 1)} {pick(p, 2)} {pick(t, 3)}, {pick(e, 4)}"
    return text[0].upper() + text[1:]


def chapter(lang, book, c, count, overrides=None, start=1):
    """[(label, reference text, document text)] for verses start..count.
    overrides: {verse: (reference text or None, document text or None)}."""
    overrides = overrides or {}
    rows = []
    for v in range(start, count + 1):
        ref, doc = overrides.get(v, (None, None))
        text = sentence(lang, book, c, v)
        rows.append((str(v), text if ref is None else ref, text if doc is None else doc))
    return rows


# ------------------------------------------------------------ writers

def usfm_book(code, title, chapters, nfd=False, id_line=True):
    """chapters: [(chapter label, [(verse label, usfm text or None)])]."""
    out = [f"\\id {code} {LABEL}"] if id_line else []
    out += [f"\\h {title}", f"\\toc1 {title}", f"\\mt1 {title}"]
    for c, verses in chapters:
        out += [f"\\c {c}", "\\p"]
        for label, text in verses:
            if label is None:
                continue
            out.append(f"\\v {label}" + ("" if text is None else f" {text}"))
    text = "\n".join(out) + "\n"
    return NFD(text) if nfd else text


def markdown(books):
    md = []
    for title, chapters in books:
        md.append(f"# {title}\n")
        for heading, paragraphs in chapters:
            md.append(f"## {heading}\n")
            for p in paragraphs:
                md.append(p + "\n")
    return "\n".join(md)


def paragraphs(pairs, per=5):
    out, cur = [], []
    for num, text in pairs:
        cur.append(f"{num} {text}".strip() if num != "" else text)
        if len(cur) == per:
            out.append(" " + " ".join(cur))
            cur = []
    if cur:
        out.append(" " + " ".join(cur))
    return out


def write_zip(path, members):
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as z:
        for name, data in members:
            info = zipfile.ZipInfo(name, date_time=(2026, 11, 21, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            z.writestr(info, data.encode("utf-8") if isinstance(data, str) else data)
    open(path, "wb").write(buf.getvalue())


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


# ------------------------------------------------------------ the sets

def set_fid():
    """F1. Every scored verse that differs from its reference differs only in
    a way the A1.5 rule counted as correct; none is faithful under E2 except
    the canonically equivalent document."""
    docs = []
    rut_over = {
        1: ("It's the elders' doing, said the widow by the gate.", "Its the elders' doing, said the widow by the gate."),
        2: ("He'll come again at dawn, said the stranger in the field.", "Hell come again at dawn, said the stranger in the field."),
        3: ("We're the reapers of his field, said the young men.", "Were the reapers of his field, said the young men."),
        4: ("Truly I tell you, today you will rest in the village with me.", "Truly I tell you today, you will rest in the village with me."),
        5: ("You are the kinsman who keeps the field.", "You are the kinsman who keeps the field?"),
        6: ("And she shall be set apart for the harvest of the barley.", "And she shall be set a part for the harvest of the barley."),
        7: ("Her keeper is now here in the field among the reapers.", "Her keeper is nowhere in the field among the reapers."),
        8: ("The Spirit of the LORD rested on the reapers at midday.", "The spirit of the LORD rested on the reapers at midday."),
        9: ("The LORD is my keeper in the evening and in the morning.", "The Lord is my keeper in the evening and in the morning."),
        10: ("[[The women wept at the gate when the lamps were lit.]]", "The women wept at the gate when the lamps were lit."),
        11: ("A servant — not the master — spoke first at the market.", "A servant not the master spoke first at the market."),
        12: ("The watchman said, “Open the gate at dawn.”", "The watchman said, \"Open the gate at dawn.\""),
    }
    rut = [(1, chapter("en", "RUT", 1, 22, rut_over))] + \
          [(c, chapter("en", "RUT", c, n)) for c, n in [(2, 23), (3, 18), (4, 22)]]
    docs.append(dict(key="e2-fid-rut", lang="en", script="Latin", books="RUT", class_="scored",
                     ref_books=[("RUT", "Ruth", rut)], md_books=[("Ruth", rut)],
                     expect={"mismatch": {f"RUT 1:{v}": k for v, k in [
                         (1, "punctuation"), (2, "punctuation"), (3, "punctuation"), (4, "punctuation"),
                         (5, "punctuation"), (6, "word_boundary"), (7, "word_boundary"), (8, "case"),
                         (9, "case"), (10, "punctuation"), (11, "punctuation"), (12, "punctuation")]}}))
    phm_over = {
        1: ("no por obras, si no por gracia, dijo el siervo junto a la puerta.",
            "no por obras, sino por gracia, dijo el siervo junto a la puerta."),
        2: (NFD("Y él que espera vivirá en la casa de su padre."), NFD("Y el que espera vivirá en la casa de su padre.")),
        3: ("Y dijo que \\add*él\\add irá a la aldea al amanecer.", "Y dijo queélirá a la aldea al amanecer."),
        4: ("Y \\add el pastor\\add* habló en la aldea por la tarde.", "Y el pastor habló en la aldea por la tarde."),
    }
    phm = [(1, chapter("es", "PHM", 1, 25, phm_over))]
    docs.append(dict(key="e2-fid-phm", lang="es", script="Latin", books="PHM", class_="scored",
                     ref_books=[("PHM", "Filemón", phm)], md_books=[("Philemon", phm)],
                     expect={"mismatch": {"PHM 1:1": "word_boundary", "PHM 1:2": "marks",
                                          "PHM 1:3": "word_boundary"}}))
    jn3_over = {
        1: (NFD("ἥ ἀλήθεια μένει ἐν ὑμῖν παρὰ τὴν πύλην."), NFD("ἤ ἀλήθεια μένει ἐν ὑμῖν παρὰ τὴν πύλην.")),
        2: (NFD("εἰ σὺ ὁ ἀγαπητὸς ὁ ἐν τῷ ἀγρῷ, εἰπὲ ἡμῖν."), NFD("εἶ σὺ ὁ ἀγαπητὸς ὁ ἐν τῷ ἀγρῷ, εἰπὲ ἡμῖν.")),
    }
    jn3 = [(1, [(l, NFD(r), NFD(d)) for l, r, d in chapter("grc", "3JN", 1, 15, jn3_over)])]
    docs.append(dict(key="e2-fid-3jn", lang="grc", script="Greek", books="3JN", class_="scored",
                     ref_books=[("3JN", "3 John", jn3)], md_books=[("3 John", jn3)],
                     expect={"mismatch": {"3JN 1:1": "marks", "3JN 1:2": "marks"}}))
    # Hebrew: a cantillation mark lost (the A1.5 letters() rule ignored it).
    oba_over = {3: ("הָאִישׁ הָלַ\u0596ךְ בַּשָּׂדֶה בַּבֹּקֶר וַיִּשְׂמַח׃", "הָאִישׁ הָלַךְ בַּשָּׂדֶה בַּבֹּקֶר וַיִּשְׂמַח׃")}
    oba = [(1, chapter("he", "OBA", 1, 21, oba_over))]
    docs.append(dict(key="e2-fid-oba-he", lang="he", script="Hebrew", books="OBA", class_="scored",
                     ref_books=[("OBA", "Obadiah", oba)], md_books=[("Obadiah", oba)],
                     expect={"mismatch": {"OBA 1:3": "marks"}, "note": "may be refused or unverified: right-to-left text"}))
    # Canonically equivalent: reference composed, document decomposed. Faithful.
    jud = [(1, [(l, NFC(r), NFD(d)) for l, r, d in chapter("grc", "JUD", 1, 25)])]
    docs.append(dict(key="e2-fid-jud-nfd", lang="grc", script="Greek", books="JUD", class_="scored",
                     ref_books=[("JUD", "Jude", jud)], md_books=[("Jude", jud)],
                     expect={"mismatch": {}, "faithful": "every verified verse (canonical)"}))
    return docs


def set_mat():
    """Material changes: the A1.5 rule and E2 both count them wrong."""
    oba = [(1, chapter("en", "OBA", 1, 21, {
        3: ("They waited forty days by the river.", "They waited four days by the river."),
        5: ("The elders were glad in the evening.", "The elders were not glad in the evening."),
    }))]
    phm = [(1, chapter("es", "PHM", 1, 25, {
        2: ("Y él que espera vivirá en la casa de su padre.", "Y el que espera vivirá en la casa de su padre."),
    }))]
    return [dict(key="e2-mat-oba", lang="en", script="Latin", books="OBA", class_="scored",
                 ref_books=[("OBA", "Obadiah", oba)], md_books=[("Obadiah", oba)],
                 expect={"mismatch": {"OBA 1:3": "changed", "OBA 1:5": "changed"}}),
            dict(key="e2-mat-phm", lang="es", script="Latin", books="PHM", class_="scored",
                 ref_books=[("PHM", "Filemón", phm)], md_books=[("Philemon", phm)],
                 expect={"mismatch": {"PHM 1:2": "marks"}})]


def set_ref():
    """B1/B2: reference content the 66-book importer cannot hold."""
    docs = []
    exo = chapter("en", "EXO", 16, 36)
    zero = "Before the first verse the whole household set out from the village of reeds."
    ref_exo = [(16, [("0", zero, zero)] + exo)]
    for suffix, doc_rows, how in [
            ("a", [("0", zero, zero)] + exo, "verse 0 printed with its number"),
            ("b", [("", zero, zero)] + exo, "verse 0 printed without a number"),
            ("c", exo, "verse 0 not printed")]:
        docs.append(dict(key=f"e2-ref-exo0{suffix}", lang="en", script="Latin", books="EXO", class_="scored",
                         ref_books=[("EXO", "Exodus", ref_exo)], md_books=[("Exodus", [(16, doc_rows)])],
                         expect={"unsupported": {"verse_zero": ["EXO 16:0"]}, "how": how}))
    jol = [(c, chapter("en", "JOL", c, n)) for c, n in [(1, 20), (2, 27), (3, 5), (4, 21)]]
    docs.append(dict(key="e2-ref-jol", lang="en", script="Latin", books="JOL", class_="scored",
                     ref_books=[("JOL", "Joel", jol)], md_books=[("Joel", jol)],
                     expect={"unsupported": {"chapter_beyond_canon": "JOL 4 (21 verses)"}}))
    oba = [(1, chapter("en", "OBA", 1, 21))]
    tob = [(1, chapter("en", "TOB", 1, 22))]
    docs.append(dict(key="e2-ref-tob", lang="en", script="Latin", books="all", class_="scored",
                     ref_books=[("OBA", "Obadiah", oba), ("TOB", "Tobit", tob)],
                     md_books=[("Obadiah", oba), ("Tobit", tob)],
                     expect={"unsupported": {"book_outside_canon": "TOB 1 (22 verses)"}}))
    # 3 John with a second chapter; unusual labels; a duplicate number.
    jn = chapter("en", "3JN", 1, 15)
    odd = [r for r in jn if r[0] not in ("12",)]
    odd.insert(11, ("12a", sentence("en", "3JN", 1, 12), sentence("en", "3JN", 1, 12)))
    odd.append(("201", "A verse numbered beyond any chapter's length, invented for this test.", None))
    odd.append(("7", "A second verse numbered seven, invented for this test.", None))
    ch2 = chapter("en", "3JN", 2, 4)
    docs.append(dict(key="e2-ref-3jn", lang="en", script="Latin", books="3JN", class_="scored",
                     ref_books=[("3JN", "3 John", [(1, odd), (2, ch2)])],
                     md_books=[("3 John", [(1, [r for r in odd if r[2] is not None]), (2, ch2)])],
                     expect={"unsupported": {"verse_label_unusual": ["3JN 1:12a", "3JN 1:201"],
                                             "verse_duplicate": ["3JN 1:7", "3JN 1:7"],
                                             "chapter_beyond_canon": "3JN 2 (4 verses)"}}))
    return docs


def set_abs():
    """B3: verified references the reference does not have."""
    docs = []
    mal12 = [(1, chapter("en", "MAL", 1, 14)), (2, chapter("en", "MAL", 2, 17))]
    mal3 = chapter("en", "MAL", 3, 24)
    ref_mal = mal12 + [(3, mal3)]
    doc_mal = mal12 + [(3, mal3[:18]), (4, [(str(i + 1), r, d) for i, (_, r, d) in enumerate(mal3[18:])])]
    docs.append(dict(key="e2-abs-mal", lang="en", script="Latin", books="MAL", class_="scored",
                     ref_books=[("MAL", "Malachi", ref_mal)], md_books=[("Malachi", doc_mal)],
                     expect={"verified_absent": [f"MAL 4:{v}" for v in range(1, 7)],
                             "reason": "chapter_not_in_book"}))
    jud = chapter("en", "JUD", 1, 25)
    extra = ("26", None, "A further verse printed after the last one, invented for this test.")
    docs.append(dict(key="e2-abs-jud", lang="en", script="Latin", books="JUD", class_="scored",
                     ref_books=[("JUD", "Jude", [(1, jud)])], md_books=[("Jude", [(1, jud + [extra])])],
                     expect={"verified_absent": ["JUD 1:26"], "reason": "verse_not_in_chapter"}))
    oba = [(1, chapter("en", "OBA", 1, 21))]
    rut1 = [(1, chapter("en", "RUT", 1, 22))]
    docs.append(dict(key="e2-abs-rut", lang="en", script="Latin", books="all", class_="scored",
                     ref_books=[("OBA", "Obadiah", oba)], md_books=[("Ruth", rut1), ("Obadiah", oba)],
                     expect={"verified_absent": "RUT 1:1-22", "reason": "book_not_in_reference"}))
    return docs


def set_name():
    """C3: a file named like a one-book file (_gen) that holds Ruth."""
    rut1 = [(1, chapter("en", "RUT", 1, 22))]
    return [dict(key="e2-name-gen", lang="en", script="Latin", books="all", class_="scored",
                 file="probe_gen.md", ref_books=[("RUT", "Ruth", rut1)], md_books=[("Ruth", rut1)],
                 expect={"verified_faithful": 22, "verified_absent": 0})]


def set_map():
    """C4: the reference archive carries a verse map."""
    jol = [(c, chapter("en", "JOL", c, n)) for c, n in [(1, 20), (2, 27), (3, 5), (4, 21)]]
    vmap = '<?xml version="1.0"?><verseMap><book osisID="Joel">' + "".join(
        f'<verse wlc="Joel.3.{v}" kjv="Joel.2.{27 + v}" type="full"/>' for v in range(1, 6)) + "".join(
        f'<verse wlc="Joel.4.{v}" kjv="Joel.3.{v}" type="full"/>' for v in range(1, 22)) + "</book></verseMap>\n"
    return [dict(key="e2-map-jol", lang="en", script="Latin", books="JOL", class_="scored",
                 ref_books=[("JOL", "Joel", jol)], md_books=[("Joel", jol)],
                 extra_members=[("VerseMap.xml", vmap)],
                 expect={"seal": "refused: the reference holds a verse map"})]


def set_utf8():
    """C5: a reference USFM that is not valid UTF-8 (one byte is Latin-1)."""
    oba = [(1, chapter("en", "OBA", 1, 21, {4: ("The miller sang beside the caf\udce9 at dawn.",
                                                    "The miller sang beside the café at dawn.")}))]
    return [dict(key="e2-utf8-oba", lang="en", script="Latin", books="OBA", class_="scored",
                 ref_books=[("OBA", "Obadiah", oba)],
                 md_books=[("Obadiah", oba)],
                 expect={"seal": "refused: the reference is not UTF-8"})]


SETS = {"e2-fid": set_fid, "e2-mat": set_mat, "e2-ref": set_ref, "e2-abs": set_abs,
        "e2-name": set_name, "e2-map": set_map, "e2-utf8": set_utf8}

COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout", "role",
           "source and licence", "books", "sha256", "generator", "kind", "class"]
PROVENANCE = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc",
              "licence", "document_sha256", "reference_source", "reference_method",
              "identity_checked_by"]


def encode(text):
    """UTF-8, except a lone surrogate \\udcXX, which stands for the raw byte XX."""
    return text.encode("utf-8", "surrogateescape")


def build(seal_root, docs_root, name, docs):
    seal = os.path.join(seal_root, name, "seal")
    folder = os.path.join(docs_root, name)
    os.makedirs(os.path.join(seal, "allowed"), exist_ok=True)
    os.makedirs(folder, exist_ok=True)
    rows, prov, sums, expected = [], [], [], {}
    for d in docs:
        key = d["key"]
        fname = d.get("file") or f"{key}.md"
        path = os.path.join(folder, fname)
        books = []
        for title, chapters in d["md_books"]:
            chs = [(str(c), paragraphs([(lab, doc) for lab, _, doc in verses if doc is not None]))
                   for c, verses in chapters]
            books.append((title, chs))
        open(path, "w", encoding="utf-8").write(markdown(books))
        sums.append((sha(path), fname))
        reference = f"{key}_usfm.zip"
        members = []
        for i, (code, title, chapters) in enumerate(d["ref_books"]):
            text = usfm_book(code, title, [(c, [(lab, r) for lab, r, _ in vs]) for c, vs in chapters])
            members.append((f"{i + 41:02d}-{code}.usfm", encode(text)))
        members += d.get("extra_members", [])
        write_zip(os.path.join(folder, reference), members)
        sums.append((sha(os.path.join(folder, reference)), reference))
        rows.append([key, fname, reference, d["lang"], d["script"], "md", "synthetic", "e2-regression",
                     "SYNTHETIC invented text, tests only", d["books"], sha(path), "make_e2_sets.py",
                     "generated", d["class_"]])
        prov.append([key, "SYNTHETIC", "E2 test edition", "1", "generated locally", "2026-11-21T00:00:00Z",
                     "none (invented text)", sha(path), "generated with the document", "same generator",
                     "generator (synthetic)"])
        expected[key] = d.get("expect", {})
    with open(os.path.join(seal, "manifest.tsv"), "w", encoding="utf-8") as f:
        f.write("\t".join(COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
    with open(os.path.join(seal, "provenance.tsv"), "w", encoding="utf-8") as f:
        f.write("\t".join(PROVENANCE) + "\n" + "".join("\t".join(r) + "\n" for r in prov))
    with open(os.path.join(seal, "checksums.sha256"), "w", encoding="utf-8") as f:
        f.write("".join(f"{h}  {n}\n" for h, n in sorted(sums, key=lambda x: x[1])))
    counts = {c: sum(r[13] == c for r in rows) for c in ("scored", "control", "mixed", "exploratory")}
    counts.update({k: sum(r[12] == k for r in rows) for k in ("authentic", "generated")})
    with open(os.path.join(seal, "composition.tsv"), "w", encoding="utf-8") as f:
        f.write("measure\tdocuments\n" + "".join(f"{k}\t{v}\n" for k, v in counts.items()))
    open(os.path.join(seal, "review-ledger.md"), "w").write(
        "SYNTHETIC regression set: invented text, no reviewer. Not a withheld set.\n")
    open(os.path.join(seal, "access-records.md"), "w").write(
        "Generated by make_e2_sets.py. No withheld document was read.\n")
    if not os.listdir(os.path.join(seal, "allowed")):
        os.rmdir(os.path.join(seal, "allowed"))
    json.dump(expected, open(os.path.join(seal_root, name, "expected.json"), "w", encoding="utf-8"),
              ensure_ascii=False, indent=1)
    return seal, folder


def main():
    seal_root, docs_root = sys.argv[1], sys.argv[2]
    chosen = sys.argv[3].split(",") if len(sys.argv) > 3 else list(SETS)
    for name in chosen:
        seal, folder = build(seal_root, docs_root, name, SETS[name]())
        print(name, seal, folder)


if __name__ == "__main__":
    main()
