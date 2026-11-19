// Invented EPUB for the existing native diagnostic launcher. No publisher text.
// Stored ZIP entries avoid adding a runtime/package dependency to the protocol.
export const originalTitle = 'INVENTED Original navigation';
export function originalParagraph(index) {
  return `INVENTED formatted paragraph ${index + 1}. ${'Read this invented content to test scrolling and navigation. '.repeat(8)}`;
}
function crc32(bytes) {
  let crc = 0xffffffff;
  for (const byte of bytes) {
    crc ^= byte;
    for (let bit = 0; bit < 8; bit++) crc = (crc >>> 1) ^ (crc & 1 ? 0xedb88320 : 0);
  }
  return (crc ^ 0xffffffff) >>> 0;
}
function storedZip(entries) {
  const local = [], directory = [];let offset = 0;
  for (const [path, text] of entries) {
    const name = Buffer.from(path), data = Buffer.from(text), crc = crc32(data);
    const header = Buffer.alloc(30);header.writeUInt32LE(0x04034b50);header.writeUInt16LE(20, 4);
    header.writeUInt32LE(crc, 14);header.writeUInt32LE(data.length, 18);header.writeUInt32LE(data.length, 22);header.writeUInt16LE(name.length, 26);
    const entry = Buffer.alloc(46);entry.writeUInt32LE(0x02014b50);entry.writeUInt16LE(20, 4);entry.writeUInt16LE(20, 6);
    entry.writeUInt32LE(crc, 16);entry.writeUInt32LE(data.length, 20);entry.writeUInt32LE(data.length, 24);entry.writeUInt16LE(name.length, 28);entry.writeUInt32LE(offset, 42);
    local.push(header, name, data);directory.push(entry, name);offset += header.length + name.length + data.length;
  }
  const central = Buffer.concat(directory), end = Buffer.alloc(22);end.writeUInt32LE(0x06054b50);
  end.writeUInt16LE(entries.length, 8);end.writeUInt16LE(entries.length, 10);end.writeUInt32LE(central.length, 12);end.writeUInt32LE(offset, 16);
  return Buffer.concat([...local, central, end]);
}
export function originalEpub() {
  const paragraphs = Array.from({length:70}, (_, i) => `<p id="p${i}">${originalParagraph(i)}${i === 40 ? ' John 15:1. <a href="#p65" epub:type="noteref">Footnote 1</a>' : i === 65 ? ' <a href="#p40" epub:type="backlink">Return to passage</a>' : ''}</p>`).join('\n');
  return storedZip([
    ['mimetype', 'application/epub+zip'],
    ['META-INF/container.xml', '<?xml version="1.0"?><container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles></container>'],
    ['OEBPS/content.opf', `<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="id">urn:invented:issue49-original</dc:identifier><dc:title>${originalTitle}</dc:title><dc:language>en</dc:language><meta property="dcterms:modified">2026-11-13T00:00:00Z</meta></metadata><manifest><item id="chapter" href="chapter.xhtml" media-type="application/xhtml+xml"/><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/></manifest><spine><itemref idref="chapter"/></spine></package>`],
    ['OEBPS/chapter.xhtml', `<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>${originalTitle}</title><style>p { margin: 1em 0; }</style></head><body>${paragraphs}</body></html>`],
    ['OEBPS/nav.xhtml', '<?xml version="1.0"?><html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops"><head><title>Contents</title></head><body><nav epub:type="toc"><ol><li><a href="chapter.xhtml#p0">Opening</a></li><li><a href="chapter.xhtml#p40">Deep passage</a></li><li><a href="chapter.xhtml#p65">Footnotes</a></li></ol></nav></body></html>'],
  ]);
}
