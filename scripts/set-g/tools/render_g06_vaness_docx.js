// G06: render the van Ess New Testament (1859 text) as a Word document with the
// "docx" JavaScript library (npm docx 9.x), from the JSON dump of the reference USFM.
// Layout: A4, one section per book, two newspaper columns, running header with the book
// name, centred page number "– n –" in the footer, book title in a large heading, chapter
// headings "Erstes Kapitel"-style ordinal words, captions as italic headings, verse
// numbers bold in parentheses "(12)", van Ess's notes as real Word footnotes.
// Usage: NODE_PATH=... node render_g06_vaness_docx.js IN.json OUT.docx
const fs = require("fs");
const d = require("docx");

const TITLES = {MAT: "Das Evangelium nach Matthäus", MRK: "Das Evangelium nach Markus",
  LUK: "Das Evangelium nach Lukas", JHN: "Das Evangelium nach Johannes",
  ACT: "Die Apostelgeschichte", ROM: "Der Brief Pauli an die Römer",
  "1CO": "Der erste Brief Pauli an die Korinther", "2CO": "Der zweite Brief Pauli an die Korinther",
  GAL: "Der Brief Pauli an die Galater", EPH: "Der Brief Pauli an die Epheser",
  PHP: "Der Brief Pauli an die Philipper", COL: "Der Brief Pauli an die Kolosser",
  "1TH": "Der erste Brief Pauli an die Thessalonicher", "2TH": "Der zweite Brief Pauli an die Thessalonicher",
  "1TI": "Der erste Brief Pauli an Timotheus", "2TI": "Der zweite Brief Pauli an Timotheus",
  TIT: "Der Brief Pauli an Titus", PHM: "Der Brief Pauli an Philemon",
  HEB: "Der Brief an die Hebräer", JAS: "Der Brief des Jakobus",
  "1PE": "Der erste Brief des Petrus", "2PE": "Der zweite Brief des Petrus",
  "1JN": "Der erste Brief des Johannes", "2JN": "Der zweite Brief des Johannes",
  "3JN": "Der dritte Brief des Johannes", JUD: "Der Brief des Judas",
  REV: "Die Offenbarung des Johannes"};
const ORD = ["", "Erstes", "Zweites", "Drittes", "Viertes", "Fünftes", "Sechstes", "Siebentes",
  "Achtes", "Neuntes", "Zehntes", "Elftes", "Zwölftes", "Dreizehntes", "Vierzehntes",
  "Fünfzehntes", "Sechzehntes", "Siebzehntes", "Achtzehntes", "Neunzehntes", "Zwanzigstes",
  "Einundzwanzigstes", "Zweiundzwanzigstes", "Dreiundzwanzigstes", "Vierundzwanzigstes",
  "Fünfundzwanzigstes", "Sechsundzwanzigstes", "Siebenundzwanzigstes", "Achtundzwanzigstes"];

const books = JSON.parse(fs.readFileSync(process.argv[2], "utf8"));
const footnotes = {};
let fnId = 1;
const FONT = "Liberation Serif";

function verseRuns(num, text, notes) {
  const runs = [new d.TextRun({text: "(" + num + ") ", bold: true, size: 18, font: FONT})];
  const parts = text.split(/\x01(\d+)\x01/);
  for (let k = 0; k < parts.length; k++) {
    if (k % 2 === 0) {
      let t = parts[k];
      if (k > 0) t = t.replace(/^\s+/, "");
      if (t) runs.push(new d.TextRun({text: t + (k < parts.length - 1 ? "" : " "), font: FONT, size: 21}));
    } else {
      const [kind, nt] = notes[parseInt(parts[k], 10)];
      footnotes[fnId] = {children: [new d.Paragraph({children: [new d.TextRun({text: nt, italics: kind === "x", font: FONT, size: 16})]})]};
      runs.push(new d.FootnoteReferenceRun(fnId));
      fnId++;
      runs.push(new d.TextRun({text: " ", font: FONT, size: 21}));
    }
  }
  return runs;
}

const sections = [];
// title page
sections.push({
  properties: {page: {size: {width: 11906, height: 16838}, margin: {top: 1700, bottom: 1400, left: 1300, right: 1300}}},
  children: [
    new d.Paragraph({spacing: {before: 3000}, alignment: d.AlignmentType.CENTER,
      children: [new d.TextRun({text: "Die heiligen Schriften des Neuen Testamentes", bold: true, size: 44, font: FONT})]}),
    new d.Paragraph({spacing: {before: 400}, alignment: d.AlignmentType.CENTER,
      children: [new d.TextRun({text: "übersetzt von Dr. Leander van Ess", size: 28, font: FONT})]}),
    new d.Paragraph({spacing: {before: 1200}, alignment: d.AlignmentType.CENTER,
      children: [new d.TextRun({text: "Nach der Ausgabe Sulzbach, Seidel’sche Buchhandlung, 1859; digitalisiert von Hans J. Herbst (2012), Zefania XML. Gemeinfrei.", size: 18, font: FONT, italics: true})]}),
  ],
});
for (const b of books) {
  const ch = [];
  ch.push(new d.Paragraph({heading: d.HeadingLevel.HEADING_1, alignment: d.AlignmentType.CENTER, spacing: {after: 240},
    children: [new d.TextRun({text: TITLES[b.code], font: FONT, size: 34, bold: true})]}));
  let cur = null;
  const flush = () => { if (cur && cur.length) ch.push(new d.Paragraph({alignment: d.AlignmentType.JUSTIFIED, spacing: {after: 60}, indent: {firstLine: 240}, children: cur})); cur = null; };
  for (const it of b.items) {
    if (it[0] === "c") {
      flush();
      ch.push(new d.Paragraph({heading: d.HeadingLevel.HEADING_2, alignment: d.AlignmentType.CENTER, spacing: {before: 200, after: 80},
        keepNext: true, children: [new d.TextRun({text: (ORD[it[1]] || (it[1] + ".")) + " Kapitel", font: FONT, size: 22, smallCaps: true, bold: true})]}));
    } else if (it[0] === "s") {
      flush();
      ch.push(new d.Paragraph({heading: d.HeadingLevel.HEADING_3, keepNext: true, spacing: {before: 80, after: 40},
        children: [new d.TextRun({text: it[1], italics: true, font: FONT, size: 19})]}));
    } else if (it[0] === "p") {
      flush();
    } else if (it[0] === "v") {
      if (!cur) cur = [];
      cur.push(...verseRuns(it[1], it[2], it[3]));
    }
  }
  flush();
  sections.push({
    properties: {type: d.SectionType.NEXT_PAGE, column: {space: 500, count: 2, separate: true},
      page: {size: {width: 11906, height: 16838}, margin: {top: 1500, bottom: 1300, left: 1200, right: 1200, header: 700, footer: 600}}},
    headers: {default: new d.Header({children: [new d.Paragraph({alignment: d.AlignmentType.CENTER,
      border: {bottom: {style: d.BorderStyle.SINGLE, size: 4, color: "777777", space: 2}},
      children: [new d.TextRun({text: TITLES[b.code], italics: true, size: 17, font: FONT})]})]})},
    footers: {default: new d.Footer({children: [new d.Paragraph({alignment: d.AlignmentType.CENTER,
      children: [new d.TextRun({children: ["– ", d.PageNumber.CURRENT, " –"], size: 17, font: FONT})]})]})},
    children: ch,
  });
}
const doc = new d.Document({creator: "aw-set-g", title: "Die heiligen Schriften des Neuen Testamentes (van Ess)",
  description: "Rendered test document", footnotes, sections,
  styles: {default: {document: {run: {font: FONT}}}}});
d.Packer.toBuffer(doc).then((buf) => { fs.writeFileSync(process.argv[3], buf); console.log("written", process.argv[3], fnId - 1, "footnotes"); });
