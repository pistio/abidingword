import type { SourceFixture } from "./memoryApi";

/** A real, deliberately plain PDF of invented text, for PDF.js workspace checks. */
export function pdfReviewFixture(): SourceFixture {
  const pages = [
    ["Invented navigation source", "A reference mention: John 15:5.", "This fixture is not commentary or an actual publisher edition."],
    ["Second physical page", "A different reference mention: Acts 7:1.", "Keep the original page and source identity when navigating."],
    ["Third physical page", "The end of a short invented document."],
  ];
  const objects = ["<< /Type /Catalog /Pages 2 0 R >>", `<< /Type /Pages /Kids [${pages.map((_, index) => `${4 + index * 2} 0 R`).join(" ")}] /Count ${pages.length} >>`, "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"];
  for (const [index, lines] of pages.entries()) {
    const stream = "BT /F1 14 Tf 50 750 Td " + lines.map((line, i) => `${i ? "0 -24 Td " : ""}(${line.replace(/[\\()]/g, "\\$&")}) Tj`).join("\n") + " ET";
    objects.push(`<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents ${5 + index * 2} 0 R >>`);
    objects.push(`<< /Length ${stream.length} >>\nstream\n${stream}\nendstream`);
  }
  const outline = objects.length + 1;
  objects[0] = `<< /Type /Catalog /Pages 2 0 R /Outlines ${outline} 0 R >>`;
  objects.push(`<< /Type /Outlines /First ${outline + 1} 0 R /Last ${outline + pages.length} 0 R /Count ${pages.length} >>`);
  pages.forEach((_, index) => objects.push(`<< /Title (Physical page ${index + 1}) /Parent ${outline} 0 R /Dest [${4 + index * 2} 0 R /Fit] ${index ? `/Prev ${outline + index} 0 R` : ""} ${index < pages.length - 1 ? `/Next ${outline + index + 2} 0 R` : ""} >>`));
  let pdf = "%PDF-1.4\n";
  const offsets = [0];
  objects.forEach((object, index) => { offsets.push(pdf.length); pdf += `${index + 1} 0 obj\n${object}\nendobj\n`; });
  const xref = pdf.length;
  pdf += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`;
  for (const offset of offsets.slice(1)) pdf += `${String(offset).padStart(10, "0")} 00000 n \n`;
  pdf += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${xref}\n%%EOF\n`;
  return {
    title: "Invented PDF navigation review", format: "pdf", language: "English",
    originalPdf: new TextEncoder().encode(pdf),
    segments: pages.flatMap((lines, index) => lines.map(text => ({ text, pageNumber: index + 1 }))),
    mentions: [{ segment: 1, reference: "JHN.15.5", matchedText: "John 15:5" }],
    contents: pages.map((_, index) => ({ id: `page-${index}`, title: `Physical page ${index + 1}`, target: { kind: "page", page: index + 1 }, children: [] })),
  };
}
