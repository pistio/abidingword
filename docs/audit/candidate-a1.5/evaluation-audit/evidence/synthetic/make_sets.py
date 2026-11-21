#!/usr/bin/env python3
"""Synthetic withheld sets for the independent evaluation audit.

Every text here is invented for the audit. Nothing is taken from Collection H
or from any withheld collection. Writes, for each set, a documents folder
(outside Git) and a seal folder (14-column manifest, checksums, provenance,
allowed lists, ledger, access records); SEAL.txt is written afterwards by the
pinned seal-set.py.

    make_sets.py <scratch repo> <documents root>
"""
import hashlib
import io
import os
import sys
import unicodedata
import zipfile

NFD = lambda s: unicodedata.normalize("NFD", s)
NFC = lambda s: unicodedata.normalize("NFC", s)

# ------------------------------------------------------------ invented text

EN_S = ["the shepherd", "a widow", "the elders", "her mother", "the reapers", "a stranger",
        "the young men", "the household", "the gleaners", "the kinsman", "the women", "a servant",
        "the potter", "the weaver", "the watchman", "the fisherman", "the miller"]
EN_V = ["walked", "rested", "waited", "spoke softly", "listened", "returned", "gathered barley",
        "worked", "travelled", "sat down", "sang", "looked up", "kept watch"]
EN_P = ["by the gate", "in the field", "near the well", "under the olive tree", "beside the road",
        "at the threshing floor", "in the village", "on the hillside", "by the river", "at the market"]
EN_T = ["at dawn", "in the evening", "after the harvest", "before the rain", "during the feast",
        "at midday", "in the cool of the day", "when the lamps were lit"]
EN_E = ["and gave thanks.", "and said nothing more.", "and the day was long.", "and all was quiet.",
        "and they were glad.", "and bread was shared.", "and the wind was still.", "and no one hurried."]

ES_S = ["el pastor", "una viuda", "los ancianos", "su madre", "los segadores", "un forastero",
        "los jóvenes", "la familia", "las espigadoras", "el pariente", "las mujeres", "un siervo",
        "el alfarero", "el tejedor", "el centinela"]
ES_V = ["caminó", "descansó", "esperó", "habló en voz baja", "escuchó", "volvió", "recogió cebada",
        "trabajó", "viajó", "se sentó", "cantó", "levantó la vista"]
ES_P = ["junto a la puerta", "en el campo", "cerca del pozo", "bajo el olivo", "al lado del camino",
        "en la era", "en la aldea", "en la ladera", "junto al río"]
ES_T = ["al amanecer", "por la tarde", "después de la cosecha", "antes de la lluvia",
        "durante la fiesta", "a mediodía", "al caer la noche"]
ES_E = ["y dio gracias.", "y no dijo nada más.", "y el día fue largo.", "y todo estaba en calma.",
        "y se alegraron.", "y compartieron el pan.", "y nadie tuvo prisa."]

GR_S = ["ὁ πρεσβύτερος", "ἡ ἐκκλησία", "οἱ ἀδελφοί", "ὁ ἀγαπητός", "ἡ μήτηρ", "ὁ ξένος",
        "οἱ νεανίσκοι", "ὁ οἶκος", "ὁ δοῦλος", "αἱ γυναῖκες", "ὁ ποιμήν"]
GR_V = ["περιεπάτησεν", "ἀνεπαύσατο", "ἔμεινεν", "ἐλάλησεν", "ἤκουσεν", "ὑπέστρεψεν",
        "εἰργάσατο", "ἐκάθισεν", "ᾖσεν", "ἀνέβλεψεν"]
GR_P = ["παρὰ τὴν πύλην", "ἐν τῷ ἀγρῷ", "ἐγγὺς τοῦ φρέατος", "ὑπὸ τὴν ἐλαίαν", "παρὰ τὴν ὁδόν",
        "ἐν τῇ κώμῃ", "ἐπὶ τοῦ ὄρους", "παρὰ τὸν ποταμόν"]
GR_T = ["πρωΐ", "ὀψίας", "μετὰ τὸν θερισμόν", "πρὸ τοῦ ὑετοῦ", "ἐν τῇ ἑορτῇ", "μεσημβρίας"]
GR_E = ["καὶ ηὐχαρίστησεν.", "καὶ οὐδὲν εἶπεν.", "καὶ ἡ ἡμέρα μακρὰ ἦν.", "καὶ πάντα ἡσύχαζεν.",
        "καὶ ἐχάρησαν.", "καὶ ἄρτον ἔκλασαν."]

BANKS = {"en": (EN_S, EN_V, EN_P, EN_T, EN_E), "es": (ES_S, ES_V, ES_P, ES_T, ES_E),
         "grc": (GR_S, GR_V, GR_P, GR_T, GR_E)}


def sentence(lang, book, c, v):
    """One invented verse: deterministic, no digits, distinct per (book, c, v)."""
    s, vb, p, t, e = BANKS[lang]
    seed = int(hashlib.sha256(f"{book}{c}:{v}".encode()).hexdigest(), 16)
    pick = lambda xs, k: xs[(seed >> (k * 8)) % len(xs)]
    text = f"{pick(s, 0)} {pick(vb, 1)} {pick(p, 2)} {pick(t, 3)}, {pick(e, 4)}"
    return text[0].upper() + text[1:]


# ------------------------------------------------------------ writers

def usfm_book(code, title, chapters, nfd=False):
    """chapters: [(chapter number, [(label, usfm text or None)])]. None: '\\v n' with no text."""
    out = [f"\\id {code} Audit synthetic edition", f"\\h {title}", f"\\toc1 {title}", f"\\mt1 {title}"]
    for c, verses in chapters:
        out += [f"\\c {c}", "\\p"]
        for label, text in verses:
            out.append(f"\\v {label}" + ("" if text is None else f" {text}"))
    text = "\n".join(out) + "\n"
    return NFD(text) if nfd else text


def markdown(books):
    """books: [(heading, [(chapter heading, [paragraph strings])])]: the
    derivation script's Markdown layout ("# Book", "## n", paragraphs with
    the verse numbers inline)."""
    md = []
    for title, chapters in books:
        md.append(f"# {title}\n")
        for heading, paragraphs in chapters:
            md.append(f"## {heading}\n")
            for p in paragraphs:
                md.append(p + "\n")
    return "\n".join(md)


def paragraphs(pairs, per=5):
    """[(printed number or '', text)] -> paragraphs of `per` verses, numbers inline."""
    out, cur = [], []
    for n, (num, text) in enumerate(pairs):
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

def plain_chapter(lang, book, c, count, overrides=None, start=1):
    """[(label, reference text, document text)] for verses start..count."""
    overrides = overrides or {}
    rows = []
    for v in range(start, count + 1):
        ref, doc = overrides.get(v, (None, None))
        base = sentence(lang, book, c, v)
        rows.append((str(v), ref if ref is not None else base, doc if doc is not None else base))
    return rows


def set_a():
    """Expected to PASS the gate: every scored verse whose text differs from
    the reference differs only in whitespace, case, punctuation or a
    decomposed mark."""
    docs = []
    # A1 English, Ruth: materially different readings (whitespace, case, punctuation).
    rut_over = {
        1: ("It's the LORD's doing, said the elders by the gate.", "Its the LORD's doing, said the elders by the gate."),
        2: ("He'll come again at dawn, said the widow in the field.", "Hell come again at dawn, said the widow in the field."),
        3: ("We're his people and the sheep of his pasture, said the reapers.", "Were his people and the sheep of his pasture, said the reapers."),
        4: ("Truly I tell you, today you will rest in the village with me.", "Truly I tell you today, you will rest in the village with me."),
        5: ("You are the kinsman who redeems the field.", "You are the kinsman who redeems the field?"),
        6: ("And she shall be set apart for the harvest of the barley.", "And she shall be set a part for the harvest of the barley."),
        7: ("Her God is now here in the field among the reapers.", "Her God is nowhere in the field among the reapers."),
        8: ("The Spirit of the LORD rested on the reapers at midday.", "The spirit of the LORD rested on the reapers at midday."),
        9: ("The LORD is my keeper in the evening and in the morning.", "The Lord is my keeper in the evening and in the morning."),
        10: ("[[The women wept at the gate when the lamps were lit.]]", "The women wept at the gate when the lamps were lit."),
        11: ("A servant — not the master — spoke first at the market.", "A servant not the master spoke first at the market."),
    }
    rut = [(1, plain_chapter("en", "RUT", 1, 22, rut_over))] + \
          [(c, plain_chapter("en", "RUT", c, n)) for c, n in [(2, 23), (3, 18), (4, 22)]]
    docs.append(dict(key="zzaudit-a-rut", lang="en", script="Latin", books="RUT", class_="scored",
                     ref_books=[("RUT", "Ruth", rut)], md_books=[("Ruth", rut)]))
    # A2 Spanish, Philemon: 'si no'/'sino', the owner's \add case, a decomposed accent.
    phm_over = {
        1: ("no por obras, si no por gracia, dijo el siervo junto a la puerta.",
            "no por obras, sino por gracia, dijo el siervo junto a la puerta."),
        2: (NFD("Y él que cree vivirá en la casa de su padre."), NFD("Y el que cree vivirá en la casa de su padre.")),
        3: ("Y dijo que \\add*él\\add irá a la ciudad al amanecer.", "Y dijo queélirá a la ciudad al amanecer."),
        4: ("Y \\add Dios\\add* habló en la aldea por la tarde.", "Y Dios habló en la aldea por la tarde."),
    }
    phm = [(1, plain_chapter("es", "PHM", 1, 25, phm_over))]
    docs.append(dict(key="zzaudit-a-phm", lang="es", script="Latin", books="PHM", class_="scored",
                     ref_books=[("PHM", "Filemón", phm)], md_books=[("Philemon", phm)]))
    # A3 Greek (decomposed on both sides), 3 John: breathings and accents.
    jn3_over = {
        1: (NFD("ἥ ἀλήθεια μένει ἐν ὑμῖν καὶ μεθ᾽ ὑμῶν ἔσται."), NFD("ἤ ἀλήθεια μένει ἐν ὑμῖν καὶ μεθ᾽ ὑμῶν ἔσται.")),
        2: (NFD("εἰ σὺ ὁ ἀγαπητὸς ὁ ἐν τῷ ἀγρῷ, εἰπὲ ἡμῖν."), NFD("εἶ σὺ ὁ ἀγαπητὸς ὁ ἐν τῷ ἀγρῷ, εἰπὲ ἡμῖν.")),
    }
    jn3 = [(1, [(l, NFD(r), NFD(d)) for l, r, d in plain_chapter("grc", "3JN", 1, 15, jn3_over)])]
    docs.append(dict(key="zzaudit-a-3jn", lang="grc", script="Greek", books="3JN", class_="scored",
                     ref_books=[("3JN", "3 John", jn3)], md_books=[("3 John", jn3)]))
    # A4 control: no Scripture, numbers present.
    docs.append(dict(key="zzaudit-a-ctl", lang="en", script="Latin", books="-", class_="control",
                     md_text="# Field notes\n\nChapter 3 of the manual explains the pump. In 1999 the well was 12 metres deep; "
                             "by 2004 it was 15. Section 4:2 lists 7 valves and 3 seals.\n\n## 2\n\n"
                             " 1 Check the seals. 2 Replace the filter. 3 Record the pressure.\n"))
    # A5 mixed: a short commentary quoting Ruth 1:16, allowed RUT 1.
    docs.append(dict(key="zzaudit-a-mix", lang="en", script="Latin", books="RUT", class_="mixed", allowed="RUT 1\n",
                     md_text="# A note on Ruth\n\nRuth 1:16 is often quoted at weddings: \"Where you go I will go.\" "
                             "The commentary below reads Ruth 1 as a story of loyalty.\n\n"
                             "In Ruth 1:16-17 the vow is made; in Ruth 4:13 the story closes.\n"))
    return "zz-a", docs


def set_b():
    """Probes of numbering, empty, placeholder, merged and unusual verses."""
    docs = []
    exo16 = plain_chapter("en", "EXO", 16, 36)
    zero = "Before the first verse the whole congregation set out from the wilderness of reeds."
    ref_exo = [(16, [("0", zero, None)] + exo16)]
    # B1a: printed 0 inline before 1.
    docs.append(dict(key="zzaudit-b-exo0a", lang="en", script="Latin", books="EXO", class_="scored",
                     ref_books=[("EXO", "Exodus", ref_exo)],
                     md_books=[("Exodus", [(16, [("0", zero, zero)] + exo16)])]))
    # B1b: verse-0 words printed unnumbered before 1.
    docs.append(dict(key="zzaudit-b-exo0b", lang="en", script="Latin", books="EXO", class_="scored",
                     ref_books=[("EXO", "Exodus", ref_exo)],
                     md_books=[("Exodus", [(16, [("", zero, zero)] + exo16)])]))
    # B1c: verse-0 words not printed.
    docs.append(dict(key="zzaudit-b-exo0c", lang="en", script="Latin", books="EXO", class_="scored",
                     ref_books=[("EXO", "Exodus", ref_exo)],
                     md_books=[("Exodus", [(16, exo16)])]))
    # B2 Malachi: reference in Hebrew numbering (3 chapters, 3:19-24), document in English numbering (4:1-6).
    mal12 = [(1, plain_chapter("en", "MAL", 1, 14)), (2, plain_chapter("en", "MAL", 2, 17))]
    mal3 = plain_chapter("en", "MAL", 3, 24)
    ref_mal = mal12 + [(3, mal3)]
    doc_mal = mal12 + [(3, mal3[:18]), (4, [(str(i + 1), r, d) for i, (_, r, d) in enumerate(mal3[18:])])]
    docs.append(dict(key="zzaudit-b-mal", lang="en", script="Latin", books="MAL", class_="scored",
                     ref_books=[("MAL", "Malachi", ref_mal)], md_books=[("Malachi", doc_mal)]))
    # B3 Joel in Hebrew numbering on both sides (4 chapters: 20, 27, 5, 21).
    jol = [(c, plain_chapter("en", "JOL", c, n)) for c, n in [(1, 20), (2, 27), (3, 5), (4, 21)]]
    docs.append(dict(key="zzaudit-b-jol", lang="en", script="Latin", books="JOL", class_="scored",
                     ref_books=[("JOL", "Joel", jol)], md_books=[("Joel", jol)]))
    # B4 Jude: a bridge, an empty verse, placeholders, an absent number, alternate numbers.
    base = {v: sentence("en", "JUD", 1, v) for v in range(1, 26)}
    jud_ref = []
    jud_doc = []
    for v in range(1, 26):
        if v == 5:
            continue  # bridged into 4
        if v == 20:
            continue  # absent number
        if v == 4:
            jud_ref.append(("4-5", base[4] + " " + base[5], None))
            jud_doc.append(("4-5", base[4] + " " + base[5], base[4] + " " + base[5]))
        elif v == 9:
            jud_ref.append(("9", None, None))  # \v 9 with no text
        elif v == 12:
            jud_ref.append(("12", "—", None))
            jud_doc.append(("12", "—", "—"))
        elif v == 14:
            jud_ref.append(("14", "[Omitted in the earliest copies]", None))
            jud_doc.append(("14", "[Omitted in the earliest copies]", "[Omitted in the earliest copies]"))
        elif v == 21:
            jud_ref.append(("21", "\\va 22\\va* \\vp 21a\\vp* " + base[21], None))
            jud_doc.append(("21", base[21], base[21]))
        else:
            jud_ref.append((str(v), base[v], None))
            jud_doc.append((str(v), base[v], base[v]))
    docs.append(dict(key="zzaudit-b-jud", lang="en", script="Latin", books="JUD", class_="scored",
                     ref_books=[("JUD", "Jude", [(1, jud_ref)])], md_books=[("Jude", [(1, jud_doc)])]))
    # B4b: the same reference; the document prints text for the empty verse 9 and the absent verse 20.
    jud_doc2 = []
    for label, r, d in jud_doc:
        jud_doc2.append((label, r, d))
        if label == "8":
            jud_doc2.append(("9", None, base[9]))
        if label == "19":
            jud_doc2.append(("20", None, base[20]))
    docs.append(dict(key="zzaudit-b-jud2", lang="en", script="Latin", books="JUD", class_="scored",
                     ref_books=[("JUD", "Jude", [(1, jud_ref)])], md_books=[("Jude", [(1, jud_doc2)])]))
    # B5 a deuterocanonical book beside a canonical one.
    oba = [(1, plain_chapter("en", "OBA", 1, 21))]
    tob = [(1, plain_chapter("en", "TOB", 1, 22))]
    docs.append(dict(key="zzaudit-b-tob", lang="en", script="Latin", books="all", class_="scored",
                     ref_books=[("OBA", "Obadiah", oba), ("TOB", "Tobit", tob)],
                     md_books=[("Obadiah", oba), ("Tobit", tob)]))
    # B6 a file name ending in _gen with books 'all'.
    rut1 = [(1, plain_chapter("en", "RUT", 1, 22))]
    docs.append(dict(key="zzaudit-b-gen", lang="en", script="Latin", books="all", class_="scored",
                     file="probe_gen.md", ref_books=[("RUT", "Ruth", rut1)], md_books=[("Ruth", rut1)]))
    # B7 the owner's \add case, with extra words so the reference text is shown.
    phm = plain_chapter("es", "PHM", 1, 25, {
        3: ("Y dijo que \\add*él\\add irá a la ciudad al amanecer.", "Y dijo que él irá a la ciudad del rey al amanecer."),
    })
    docs.append(dict(key="zzaudit-b-add", lang="es", script="Latin", books="PHM", class_="scored",
                     ref_books=[("PHM", "Filemón", [(1, phm)])], md_books=[("Philemon", [(1, phm)])]))
    # B8 contrast: genuinely wrong words and an accent lost between composed texts.
    obaw = [(1, plain_chapter("en", "OBA", 1, 21, {
        3: ("They waited forty days by the river.", "They waited four days by the river."),
    }))]
    docs.append(dict(key="zzaudit-b-wrong", lang="en", script="Latin", books="OBA", class_="scored",
                     ref_books=[("OBA", "Obadiah", obaw)], md_books=[("Obadiah", obaw)]))
    phmw = [(1, plain_chapter("es", "PHM", 1, 25, {
        2: ("Y él que cree vivirá en la casa de su padre.", "Y el que cree vivirá en la casa de su padre."),
    }))]
    docs.append(dict(key="zzaudit-b-accent", lang="es", script="Latin", books="PHM", class_="scored",
                     ref_books=[("PHM", "Filemón", phmw)], md_books=[("Philemon", phmw)]))
    # B9 Joel again, with a verse map inside the reference archive.
    jolmap = '<?xml version="1.0"?><verseMap><book osisID="Joel">' + "".join(
        f'<verse wlc="Joel.3.{v}" kjv="Joel.2.{27 + v}" type="full"/>' for v in range(1, 6)) + "".join(
        f'<verse wlc="Joel.4.{v}" kjv="Joel.3.{v}" type="full"/>' for v in range(1, 22)) + "</book></verseMap>\n"
    docs.append(dict(key="zzaudit-b-map", lang="en", script="Latin", books="JOL", class_="scored",
                     ref_books=[("JOL", "Joel", jol)], md_books=[("Joel", jol)],
                     extra_members=[("VerseMap.xml", jolmap)]))
    # B10 a document the app refuses whole (no text).
    docs.append(dict(key="zzaudit-b-empty", lang="en", script="Latin", books="OBA", class_="scored",
                     file="zzaudit-b-empty.txt", raw=b"\n\n", ref_books=[("OBA", "Obadiah", oba)]))
    return "zz-b", docs


# ------------------------------------------------------------ assembly

COLUMNS = ["key", "files", "reference", "language", "script", "format", "layout", "role",
           "source and licence", "books", "sha256", "generator", "kind", "class"]
PROVENANCE = ["key", "publisher", "edition", "revision", "retrieved_from", "retrieved_utc",
              "licence", "document_sha256", "reference_source", "reference_method",
              "identity_checked_by"]


def build(repo, docs_root, name, docs):
    seal = os.path.join(repo, "docs", "evaluation", name, "seal")
    folder = os.path.join(docs_root, name)
    os.makedirs(os.path.join(seal, "allowed"), exist_ok=True)
    os.makedirs(folder, exist_ok=True)
    rows, prov, sums = [], [], []
    for d in docs:
        key = d["key"]
        fname = d.get("file") or f"{key}.md"
        path = os.path.join(folder, fname)
        if "raw" in d:
            open(path, "wb").write(d["raw"])
        elif "md_text" in d:
            open(path, "w", encoding="utf-8").write(d["md_text"])
        else:
            books = []
            for title, chapters in d["md_books"]:
                chs = []
                for c, verses in chapters:
                    chs.append((str(c), paragraphs([(lab if lab != "" else "", doc) for lab, _, doc in verses
                                                    if doc is not None])))
                books.append((title, chs))
            open(path, "w", encoding="utf-8").write(markdown(books))
        sums.append((sha(path), fname))
        reference = "-"
        if d.get("ref_books"):
            reference = f"{key}_usfm.zip"
            members = []
            for i, (code, title, chapters) in enumerate(d["ref_books"]):
                members.append((f"{i + 41:02d}-{code}.usfm",
                                usfm_book(code, title, [(c, [(lab, r) for lab, r, _ in vs]) for c, vs in chapters])))
            members += d.get("extra_members", [])
            write_zip(os.path.join(folder, reference), members)
            sums.append((sha(os.path.join(folder, reference)), reference))
        rows.append([key, fname, reference, d["lang"], d["script"], os.path.splitext(fname)[1][1:],
                     "synthetic", "withheld-audit", "invented text, audit only", d["books"],
                     sha(path), "audit make_sets.py", "generated", d["class_"]])
        prov.append([key, "audit", "synthetic", "1", "generated locally", "2026-11-21T00:00:00Z",
                     "none (invented)", sha(path), "generated with the document" if reference != "-" else "-",
                     "same generator" if reference != "-" else "-", "auditor"])
        if d["class_"] == "mixed":
            open(os.path.join(seal, "allowed", f"{key}.txt"), "w", encoding="utf-8").write(d["allowed"])
    with open(os.path.join(seal, "manifest.tsv"), "w", encoding="utf-8") as f:
        f.write("\t".join(COLUMNS) + "\n" + "".join("\t".join(r) + "\n" for r in rows))
    with open(os.path.join(seal, "provenance.tsv"), "w", encoding="utf-8") as f:
        f.write("\t".join(PROVENANCE) + "\n" + "".join("\t".join(r) + "\n" for r in prov))
    with open(os.path.join(seal, "checksums.sha256"), "w", encoding="utf-8") as f:
        f.write("".join(f"{h}  {n}\n" for h, n in sorted(sums, key=lambda x: x[1])))
    open(os.path.join(seal, "review-ledger.md"), "w").write("Synthetic audit set: invented text, no reviewer.\n")
    open(os.path.join(seal, "access-records.md"), "w").write("Auditor only. No withheld document was read.\n")
    if not os.listdir(os.path.join(seal, "allowed")):
        os.rmdir(os.path.join(seal, "allowed"))
    return seal, folder


def set_c():
    """D4 with Scripture actually present: a control that holds a Bible
    text, and a mixed document verifying outside its allowed list."""
    docs = []
    oba = [(1, plain_chapter("en", "OBA", 1, 21))]
    docs.append(dict(key="zzaudit-c-oba", lang="en", script="Latin", books="OBA", class_="scored",
                     ref_books=[("OBA", "Obadiah", oba)], md_books=[("Obadiah", oba)]))
    docs.append(dict(key="zzaudit-c-ctl", lang="en", script="Latin", books="-", class_="control",
                     file="zzaudit-c-ctl.md", md_books=[("Obadiah", oba)]))
    rut12 = [(1, plain_chapter("en", "RUT", 1, 22)), (2, plain_chapter("en", "RUT", 2, 23))]
    docs.append(dict(key="zzaudit-c-mix", lang="en", script="Latin", books="RUT", class_="mixed",
                     allowed="RUT 1:1-21\nRUT 1:22  # a single verse\n", md_books=[("Ruth", rut12)]))
    return "zz-c", docs


def main():
    repo, docs_root = sys.argv[1], sys.argv[2]
    chosen = sys.argv[3].split(",") if len(sys.argv) > 3 else ["a", "b"]
    for maker in [{"a": set_a, "b": set_b, "c": set_c}[c] for c in chosen]:
        name, docs = maker()
        seal, folder = build(repo, docs_root, name, docs)
        print(name, seal, folder)


if __name__ == "__main__":
    main()
