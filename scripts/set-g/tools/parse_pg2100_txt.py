"""Read the verse structure of Project Gutenberg #2100 (Bibeln 1917, plain text) for the
reconciliation of the G02 reference (built from the Zefania SVD encoding).
Lines "CCC:VVV text" start a verse; indented lines continue it; book titles are the
lines listed in the file's own contents (INNEHÅLL) table; "[n] ..." lines are notes.
Output: JSON {"order": [book codes in file order], "verses": {"GEN 1:1": text}, "notes": [...]}.
Usage: python3 -I parse_pg2100_txt.py 2100.txt out.json
"""
import json
import re
import sys

TITLES = ["Första Mosebok (Genesis)", "Andra Mosebok (Exodus)", "Tredje Mosebok (Leviticus)",
          "Fjärde Mosebok (Numeri)", "Femte Mosebok (Deuteronomium)", "Josua", "Domarboken", "Rut",
          "Första Samuelsboken", "Andra Samuelsboken", "Första Konungaboken", "Andra Konungaboken",
          "Första Krönikeboken", "Andra Krönikeboken", "Esra", "Nehemja", "Ester", "Job", "Psaltaren",
          "Ordspråksboken", "Predikaren", "Höga Visan", "Jesaja", "Jeremia", "Klagovisorna", "Hesekiel",
          "Daniel", "Hosea", "Joel", "Amos", "Obadja", "Jona", "Mika", "Nahum", "Habackuk", "Sefanja",
          "Haggai", "Sakarja", "Malaki", "Evangelium enligt Matteus", "Evangelium enligt Markus",
          "Evangelium enligt Lukas", "Evangelium enligt Johannes", "Apostlagärningarna",
          "Paulus' brev till romarna", "Paulus' första brev till korintierna",
          "Paulus' andra brev till korintierna", "Paulus' brev till galaterna",
          "Paulus' brev till efesierna", "Paulus' brev till filipperna", "Paulus' brev till kolosserna",
          "Paulus' första brev till tessalonikerna", "Paulus' andra brev till tessalonikerna",
          "Paulus' första brev till Timoteus", "Paulus' andra brev till Timoteus",
          "Paulus' brev till Titus", "Paulus' brev till Filemon", "Brevet till hebréerna",
          "Petrus' första brev", "Petrus' andra brev", "Johannes' första brev", "Johannes' andra brev",
          "Johannes' tredje brev", "Jakobs' brev", "Judas' brev", "Johannes' uppenbarelse"]
CODES = ["GEN", "EXO", "LEV", "NUM", "DEU", "JOS", "JDG", "RUT", "1SA", "2SA", "1KI", "2KI", "1CH",
         "2CH", "EZR", "NEH", "EST", "JOB", "PSA", "PRO", "ECC", "SNG", "ISA", "JER", "LAM", "EZK",
         "DAN", "HOS", "JOL", "AMO", "OBA", "JON", "MIC", "NAM", "HAB", "ZEP", "HAG", "ZEC", "MAL",
         "MAT", "MRK", "LUK", "JHN", "ACT", "ROM", "1CO", "2CO", "GAL", "EPH", "PHP", "COL", "1TH",
         "2TH", "1TI", "2TI", "TIT", "PHM", "HEB", "1PE", "2PE", "1JN", "2JN", "3JN", "JAS", "JUD",
         "REV"]
T2C = dict(zip(TITLES, CODES))


def main(src, out):
    lines = open(src, encoding="utf-8").read().replace("\r\n", "\n").split("\n")
    start = next(i for i, l in enumerate(lines) if l.startswith("*** START OF"))
    end = next(i for i, l in enumerate(lines) if l.startswith("*** END OF"))
    # skip the contents table: body starts at the second occurrence of the first title
    occ = [i for i in range(start, end) if lines[i].strip() == TITLES[0]]
    i0 = occ[1]
    order, verses, notes, cur, key = [], {}, [], None, None
    for l in lines[i0:end]:
        s = l.strip()
        if s in T2C:
            cur = T2C[s]
            order.append(cur)
            key = None
            continue
        m = re.match(r"^(\d{3}):(\d{3})(?: (.*)|--(.*)|)$", l)
        if m:
            key = f"{cur} {int(m.group(1))}:{int(m.group(2))}"
            txt = m.group(3) if m.group(3) is not None else ("--" + m.group(4) if m.group(4) is not None else "")
            verses[key] = txt.strip()
            continue
        if l.startswith("        ") and key:
            verses[key] = (verses[key] + " " + s).strip()
            continue
        if s.startswith("["):
            notes.append({"after": key, "text": s})
            key = None
            continue
        if s:
            notes.append({"after": key, "unparsed": s})
    json.dump({"order": order, "verses": verses, "notes": notes}, open(out, "w", encoding="utf-8"), ensure_ascii=False, indent=0)
    print(len(order), "books", len(verses), "verses", len(notes), "notes/unparsed")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
