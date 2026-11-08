// A.4.1: honest book/chapter evidence labels, the rebuild notice for structures
// built before A.4.1, and the metadata-only diagnostic with the Reader's own
// state after a Source-result click. Invented data; not the owner's PDF.
import { screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { PdfAnchor, PdfDiagnostic, PdfResolvedHeading, PdfStructureView } from "@/types/library";
import { createMemoryApi } from "../support/memoryApi";
import { createFakePdfEngine } from "../support/fakePdfEngine";
import { renderApp } from "../support/render";
import { readerAction } from "../support/readerActions";

const anchor: PdfAnchor = { page: 8, pageWidth: 595, pageHeight: 842, x0: 72, x1: 280, baseline: 220, size: 24, firstChar: 0, column: 0, byteStart: 0, byteEnd: 17, originalText: "Apostelgeschichte" };
const heading: PdfResolvedHeading = { id: "p8-c0", title: "Apostelgeschichte", level: 2, parents: [], basis: "layout", anchor, segmentPosition: 1, start: 0, end: 17, disabled: false, mapping: null, mappingScope: null, canonical: "ACT.1-ACT.28", scope: "book", endPosition: 3, path: "Apostelgeschichte" };

async function open({ rebuild = false } = {}) {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  const source = api.installFixtureSource({
    title: "Invented handbook", format: "pdf", language: "Deutsch", segments: [
      { text: "Leitvers dieses Buches: Apg 6,4.", pageNumber: 2 },
      { text: "Apostelgeschichte\nErfundener Absatz.", pageNumber: 8, presentation: { role: "paragraph", basis: "layout", uncertain: false, printedLabel: "14", anchors: [anchor], headings: [heading], sectionPath: heading.path, runs: [] } },
      { text: "Ein beiläufiger Verweis: Apg 7,58.", pageNumber: 9 },
    ],
  });
  const view: PdfStructureView = { sourceId: source.id, fileHash: "hash", status: "ready", complete: true, tagged: false, unalignedTags: 0, uncertainPages: 0, headings: [heading], corrections: [], canUndo: false, analysis: rebuild ? 0 : 2, rebuildRecommended: rebuild };
  api.libraryPdfStructure = async () => view;
  api.librarySourceContents = async () => [{ id: heading.id, title: heading.title, target: { kind: "segment", position: 1 }, children: [] }];
  api.state.settings.scripturePassage = "ACT.6";
  const segments = await api.sourceGetSegments({ sourceId: source.id });
  const target = segments.segments[1]!.id;
  api.libraryPassageSources = async () => [{
    sourceId: source.id, sourceTitle: source.title, author: null, moreLocations: 0,
    locations: [{ segmentId: target, locator: "p. 8", origin: "section", matchKind: "contained", matchedText: heading.title, sectionHeading: heading.title, structureNodeId: heading.id, structureBasis: "layout", structureScope: "book", reference: { canonical: "ACT.1-ACT.28", display: "Acts 1–28", book: "ACT", startChapter: 1, startVerse: null, endChapter: 28, endVerse: null } }],
  }];
  const diagnostic = vi.fn(async (sourceId: string, passage: string | null, observed: PdfDiagnostic["readerObserved"]): Promise<PdfDiagnostic> => ({
    schema: "abidingword.pdf-structure-diagnostic/1", appVersion: "test", generatedAt: "2026-11-01T00:00:00Z", privacy: "Metadata only.",
    source: { id: sourceId, fileSha256: "a".repeat(64), fileSizeBytes: 1000, language: "Deutsch", indexLanguage: "De", segmentCount: 3, pagesWithText: 3 },
    structure: { status: "ready", analysis: 2, currentAnalysis: 2, rebuildRecommended: false, pageCount: 15, headingCount: 1, listingPages: [3], corrections: 0, byScope: { book: 1 }, byBasis: { layout: 1 } },
    headings: [],
    query: { input: passage ?? "", canonical: "ACT.6", verdict: "First result is grounded document structure: book section (broader) on PDF page 8.", readerTarget: { segmentId: target, segmentPosition: 1, structureNodeId: heading.id, physicalPage: 8, printedPage: "14" },
      candidates: [{ rank: 1, class: 3, origin: "section", evidence: "Book section (broader)", scope: "book", basis: "layout", role: null, structureNodeId: heading.id, segmentPosition: 1, physicalPage: 8, printedPage: "14", passage: "ACT.1-ACT.28", matched: "Apostelgeschichte" }] },
    readerObserved: observed,
  }));
  api.libraryPdfStructureDiagnostic = diagnostic;
  const engine = createFakePdfEngine({ pages: 15 });
  const { user } = await renderApp(api, { pdfEngine: engine, width: 900 });
  await user.click(screen.getByRole("button", { name: "Show Sources pane" }));
  return { api, user, diagnostic, heading, target };
}

describe("A.4.1 structure evidence and diagnostic", () => {
  it("labels a book-level heading as a broader book section, not as a section about the passage", async () => {
    await open();
    const result = await screen.findByRole("button", { name: /^Invented handbook/ });
    expect(result).toHaveTextContent("Book section Acts 1–28 (broader)");
    expect(result).not.toHaveTextContent("Heading names");
  });

  it("records the reader's mode, page and opened heading in the metadata-only report", async () => {
    const { user, diagnostic, heading, target } = await open();
    await user.click(await screen.findByRole("button", { name: /^Invented handbook/ }));
    await screen.findByRole("heading", { name: "Apostelgeschichte" });
    await readerAction(user, "Chapters and headings");
    await user.click(await screen.findByText("Diagnostic report (metadata only)"));
    expect(screen.getByRole("button", { name: "Copy report" })).toBeDisabled();
    await user.type(screen.getByRole("textbox", { name: "Passage to check (optional)" }), "Apg 6");
    await user.click(screen.getByRole("button", { name: "Create report" }));
    await screen.findByText(/based on document structure/);
    expect(screen.getByText(/Book section \(broader\) · Apostelgeschichte · PDF page 8 \(14\)/)).toBeVisible();
    const [, passage, observed] = diagnostic.mock.calls[0]!;
    expect(passage).toBe("Apg 6");
    expect(observed).toMatchObject({ mode: "reading", openedSegmentId: target, openedStructureNodeId: heading.id });
    expect(Object.keys(observed ?? {}).sort()).toEqual(["focusedHeadingId", "mode", "openedSegmentId", "openedStructureNodeId", "physicalPage", "printedPage", "variant"]);
    expect(screen.getByRole("button", { name: "Copy report" })).toBeEnabled();
  });

  it("asks for a rebuild when the stored structure predates the A.4.1 analysis", async () => {
    const { user } = await open({ rebuild: true });
    await user.click(await screen.findByRole("button", { name: /^Invented handbook/ }));
    await screen.findByRole("heading", { name: "Apostelgeschichte" });
    await readerAction(user, "Chapters and headings");
    await waitFor(() => expect(screen.getByText(/built with earlier analysis rules/)).toBeVisible());
  });
});
