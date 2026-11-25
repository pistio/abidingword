import { storedZip } from './original-epub-fixture.mjs';

const counts = [22, 23, 18, 22];
export const verseText = (chapter, verse) =>
  `Invented words of Ruth c${chapter}v${verse} for testing.`;
const chapters = counts.map((count, i) => ({
  chapter: i + 1,
  name: `c${i + 1}.xhtml`,
  verses: Array.from({ length: count }, (_, j) => verseText(i + 1, j + 1)),
}));

function epub() {
  return storedZip([
    ['mimetype', 'application/epub+zip'],
    ['META-INF/container.xml', '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>'],
    ['OEBPS/content.opf', `<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="id">urn:invented:scripture-acceptance</dc:identifier><dc:title>Invented Ruth EPUB</dc:title><dc:language>en</dc:language></metadata><manifest>${chapters.map(({ chapter, name }) => `<item id="c${chapter}" href="${name}" media-type="application/xhtml+xml"/>`).join('')}</manifest><spine>${chapters.map(({ chapter }) => `<itemref idref="c${chapter}"/>`).join('')}</spine></package>`],
    ...chapters.map(({ chapter, name, verses }) => {
      const body = verses.map((text, i) => {
        if (chapter === 1 && i === 0) text = 'Invented words<a epub:type="noteref" href="#note-a">[a]</a> of Ruth c1v1 for testing.';
        const id = chapter === 1 && i === 1 ? ' id="Ruth1-9"' : '';
        return `<p${id}><sup class="v">${i + 1}</sup>${text}</p>`;
      }).join('');
      const note = chapter === 1 ? '<aside epub:type="footnote" id="note-a"><p>INVENTED APPARATUS.</p></aside>' : '';
      return [`OEBPS/${name}`, `<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Invented Ruth</title></head><body>${chapter === 1 ? '<h1>Ruth</h1>' : ''}<h2>Chapter ${chapter}</h2>${body}${note}</body></html>`];
    }),
  ]);
}

function docx() {
  const w = 'http://schemas.openxmlformats.org/wordprocessingml/2006/main';
  const rel = 'http://schemas.openxmlformats.org/officeDocument/2006/relationships';
  const p = (text, heading) => `<w:p>${heading ? `<w:pPr><w:pStyle w:val="Heading${heading}"/></w:pPr>` : ''}<w:r><w:t>${text}</w:t></w:r></w:p>`;
  const body = p('The Story of the Moabite Woman', 1) + chapters.map(({ chapter, verses }) =>
    p(`Chapter ${chapter}`, 2) + verses.map((text, i) => p(`${i + 1} ${text}`)).join('')).join('');
  return storedZip([
    ['[Content_Types].xml', '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/><Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/></Types>'],
    ['_rels/.rels', `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="r1" Type="${rel}/officeDocument" Target="word/document.xml"/></Relationships>`],
    ['word/_rels/document.xml.rels', `<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="r1" Type="${rel}/styles" Target="styles.xml"/></Relationships>`],
    ['word/document.xml', `<w:document xmlns:w="${w}"><w:body>${body}</w:body></w:document>`],
    ['word/styles.xml', `<w:styles xmlns:w="${w}">${[1, 2].map(n => `<w:style w:type="paragraph" w:styleId="Heading${n}"><w:name w:val="heading ${n}"/></w:style>`).join('')}</w:styles>`],
  ]);
}

function pdf(pages, imageOnly = false) {
  const objects = ['<< /Type /Catalog /Pages 2 0 R >>', '', '<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>'];
  const image = imageOnly ? objects.push('<< /Type /XObject /Subtype /Image /Width 1 /Height 1 /ColorSpace /DeviceGray /BitsPerComponent 8 /Filter /ASCIIHexDecode /Length 3 >>\nstream\n80>\nendstream') : null;
  const kids = [];
  for (const lines of pages) {
    const stream = imageOnly ? 'q 300 0 0 300 72 400 cm /Scan Do Q\n' : lines.map((line, i) => `BT /F1 11 Tf 72 ${750 - 60 * i} Td (${line}) Tj ET\n`).join('');
    const streamId = objects.push(`<< /Length ${stream.length} >>\nstream\n${stream}endstream`);
    kids.push(objects.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Contents ${streamId} 0 R >>`));
  }
  objects[1] = `<< /Type /Pages /Kids [${kids.map(id => `${id} 0 R`).join(' ')}] /Count ${pages.length} /Resources << /Font << /F1 3 0 R >> ${image ? `/XObject << /Scan ${image} 0 R >>` : ''} >> >>`;
  let contents = '%PDF-1.5\n';
  const offsets = objects.map((object, i) => {
    const offset = contents.length;
    contents += `${i + 1} 0 obj\n${object}\nendobj\n`;
    return offset;
  });
  return Buffer.from(contents + `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n${offsets.map(n => String(n).padStart(10, '0') + ' 00000 n \n').join('')}trailer\n<< /Root 1 0 R /Size ${objects.length + 1} >>\nstartxref\n${contents.length}\n%%EOF\n`);
}

export function scriptureFixtures() {
  const lines = ['Ruth', ...chapters.flatMap(({ chapter, verses }) =>
    [`Chapter ${chapter}`, ...verses.map((text, i) => `${i + 1} ${text}`)])];
  const pages = Array.from({ length: Math.ceil(lines.length / 9) }, (_, i) =>
    ['THE BOOK OF RUTH', ...lines.slice(i * 9, i * 9 + 9), String(i + 1)]);
  return { 'джерело-성경.epub': epub(), '룻기-документ.docx': docx(), 'invented.pdf': pdf(pages), 'image-only.pdf': pdf([[], []], true) };
}
