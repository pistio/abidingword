import type { ContentsItem, PassageLocation, PdfStructureView, SegmentWindow, Source } from "@/types/library";
import type { MemoryApi } from "./memoryApi";

/** Browser review consumes the invented PDF and serialized real Rust import
 * results emitted by tests/pdf_structure.rs with AW_PDF_REVIEW_DIR set.
 * Mutation coverage belongs to the native/UI regressions, not this adapter. */
export async function pdfStructureReviewFixture(api: MemoryApi, structured = true) {
  const payload = await (await fetch("/structure-review.json")).json() as {
    source: Source; window: SegmentWindow; structure: PdfStructureView;
    contents: ContentsItem[]; locations: PassageLocation[];
  };
  const bytes = new Uint8Array(await (await fetch("/structure-review.pdf")).arrayBuffer());
  const source = api.installFixtureSource({
    title: payload.source.title, format:"pdf", language:"Deutsch", originalPdf:bytes,
    segments:payload.window.segments.map(s=>({text:s.text,pageNumber:s.pageNumber ?? undefined,presentation:s.presentation})),
    contents:payload.contents,
  });
  const id = source.id;
  Object.assign(source,payload.source,{id});
  const segments=(await api.sourceGetSegments({sourceId:id,limit:500})).segments;
  const remap=(old:string)=>segments[payload.window.segments.find(s=>s.id===old)!.position]!.id;
  const locations=payload.locations.map(p=>({...p,segmentId:remap(p.segmentId)}));
  if (!structured) { api.state.settings.scripturePassage="ACT.6"; return id; }
  const previous=api.libraryPdfStructure;
  api.libraryPdfStructure=async sourceId=>sourceId===id?{...payload.structure,sourceId:id}:previous(sourceId);
  const groups=api.libraryPassageSources;
  api.libraryPassageSources=async reference=>reference.startsWith("ACT.6")?[{sourceId:id,sourceTitle:source.title,author:source.author,locations:locations.slice(0,3),moreLocations:Math.max(0,locations.length-3)}]:groups(reference);
  const places=api.libraryPassageSourceLocations;
  api.libraryPassageSourceLocations=async(sourceId,reference)=>sourceId===id && reference.startsWith("ACT.6")?locations:places(sourceId,reference);
  api.state.settings.scripturePassage="ACT.6";
  return id;
}
