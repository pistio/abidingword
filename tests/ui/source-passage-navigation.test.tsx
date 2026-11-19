import { act, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createMemoryApi, type SourceFixture } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";
import { jumpReaderPage, readerAction, readerActionEnabled, readerPage, readerPageName } from "../support/readerActions";
import { createFakePdfEngine } from "../support/fakePdfEngine";
import { FORMATTED_BOOK } from "../support/epubFixture";
import { FORMATTED_DOCX } from "../support/docxFixture";
import type { SourceCoverage } from "@/types/library";

// Invented content with paragraph geometry: an unexpected jump changes a measurable position.
const BOOK: SourceFixture = {
  title: "Orchard Commentary", author: "An Invented Author", format: "markdown",
  segments: Array.from({ length: 350 }, (_, position) => ({ text: `Orchard paragraph ${position}.` })),
  sections: [
    { segment: 30, reference: "JHN.15.1-JHN.15.11", matchedText: "John 15:1–11", heading: "Vine and branches", through: 49 },
    { segment: 200, reference: "JHN.15.12-JHN.15.17", matchedText: "John 15:12–17", heading: "Love one another", through: 219 },
  ],
  generatedContents: [{ id: "kept", title: "The place I was reading", target: { kind: "segment", position: 250 }, children: [] }],
};
const sources = () => screen.getByRole("complementary", { name: "Sources" });
const scripture = () => screen.getByRole("complementary", { name: "Scripture" });
type User = Awaited<ReturnType<typeof renderApp>>["user"];
let top = 0;
const originalRect = Element.prototype.getBoundingClientRect;
const originalScroll = Element.prototype.scrollIntoView;
beforeEach(() => {
  top = 0;
  Element.prototype.scrollIntoView = function () {
    const element = this as HTMLElement;
    const position = element.dataset.segPosition ?? (element.nextElementSibling as HTMLElement | null)?.dataset.segPosition;
    if (position !== undefined) {
      top = Number(position);
      const body = element.closest<HTMLElement>('[data-testid="reader-body"]');
      if (body) body.scrollTop = 0;
    }
  };
  Element.prototype.getBoundingClientRect = function () {
    const element = this as HTMLElement;
    if (element.dataset.segPosition !== undefined || element.dataset.testid === "reader-body") {
      const y = element.dataset.segPosition === undefined ? 0 : (Number(element.dataset.segPosition) - top) * 30 - (element.closest<HTMLElement>('[data-testid="reader-body"]')?.scrollTop ?? 0);
      const height = element.dataset.segPosition === undefined ? 300 : 30;
      return { top: y, bottom: y + height, left: 0, right: 600, width: 600, height, x: 0, y, toJSON: () => ({}) } as DOMRect;
    }
    return originalRect.call(this);
  };
});
afterEach(() => {
  vi.restoreAllMocks();
  Element.prototype.getBoundingClientRect = originalRect;
  Element.prototype.scrollIntoView = originalScroll;
});
function setup(fixture = BOOK, withBible = true) {
  const api = createMemoryApi();
  if (withBible) api.installFixtureBible("FXA", "alpha");
  api.state.settings.scripturePassage = "PSA.23";
  const book = api.installFixtureSource(fixture);
  return { api, book };
}
async function openAtKeptPlace(user: User, title = BOOK.title) {
  await user.click(await within(sources()).findByRole("button", { name: title }));
  const reader = await within(sources()).findByRole("region", { name: title });
  await readerAction(user, "Contents", reader);
  await user.click(await within(reader).findByRole("button", { name: /^The place I was reading/ }));
  await waitFor(() => expect(top).toBe(250));
  return reader;
}
async function chooseScripture(user: User, passage: string) {
  const input = within(scripture()).getByRole("textbox", { name: "Go to passage" });
  await user.clear(input);
  await user.type(input, `${passage}{Enter}`);
  await within(sources()).findByText(`Scripture: ${passage}`);
}
const sync = (reader: HTMLElement) => within(reader).getByRole("button", { name: "Go to current passage" });
const WHOLE_JOHN: SourceCoverage = { scope: "book", basis: "metadata", reference: { canonical: "JHN.1-JHN.21", display: "John", book: "JHN", startChapter: 1, startVerse: null, endChapter: 21, endVerse: null } };

describe("explicit passage navigation in the current source", () => {
  it("keeps reading in place when Scripture changes, then opens the selected section on request with Back and marks preserved", async () => {
    const { api, book } = setup();
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Garden study folder" });
    const study = await api.studyCreate({ projectId: project.id, title: "A study with its own passage" });
    await api.studyPassageAdd({ studyId: study.study.id, reference: "PSA.23", role: "primary" });
    const kept = api.state.library.segments.find((segment) => segment.sourceId === book.id && segment.position === 250)!;
    await api.sourceAnnotationCreate({ sourceId: book.id, kind: "bookmark", place: { segmentId: kept.id } });
    const marks = await api.sourceAnnotationList(book.id);
    const primary = structuredClone(api.state.passages);
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    const body = within(reader).getByTestId("reader-body");
    body.scrollTop = 7;
    const open = vi.spyOn(api, "libraryOpenSegment");
    await chooseScripture(user, "John 15:5");
    expect(top).toBe(250);
    expect(body.scrollTop).toBe(7);
    expect(api.libraryLocationsCalls).toHaveLength(0);
    expect(open).not.toHaveBeenCalled();
    await user.click(sync(reader));
    await waitFor(() => expect(top).toBe(30));
    expect(await within(reader).findByText("Last passage opened here: John 15:1–11")).toBeInTheDocument();
    expect(api.libraryLocationsCalls).toEqual([{ sourceId: book.id, reference: "JHN.15.5" }]);
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(top).toBe(250));
    expect(body.scrollTop).toBe(7);
    expect(await api.sourceAnnotationList(book.id)).toEqual(marks);
    expect(api.state.passages).toEqual(primary);
    expect(within(scripture()).getByRole("textbox", { name: "Go to passage" })).toHaveValue("");
  });

  it("lists literal mentions and waits for the chosen mention, including when there is only one", async () => {
    const fixture: SourceFixture = { ...BOOK, title: "Orchard Lecture", sections: [], mentions: [
      { segment: 30, reference: "JHN.15.5", matchedText: "John 15:5" },
      { segment: 200, reference: "JHN.15.5", matchedText: "John 15:5" },
    ] };
    const { api } = setup(fixture);
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user, fixture.title);
    await chooseScripture(user, "John 15:5");
    await user.click(sync(reader));
    const dialog = await screen.findByRole("dialog", { name: "Mentions of John 15:5 in this source" });
    expect(top).toBe(250);
    const choices = within(dialog).getAllByRole("button", { name: /Mentions John 15:5/ });
    expect(choices).toHaveLength(2);
    await user.click(choices[1]!);
    await waitFor(() => expect(top).toBe(200));
    expect(dialog).not.toBeInTheDocument();
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(top).toBe(250));
    api.state.library.mentions.splice(1, 1);
    await user.click(sync(reader));
    const one = await screen.findByRole("dialog", { name: "Mentions of John 15:5 in this source" });
    expect(within(one).getAllByRole("button", { name: /Mentions John 15:5/ })).toHaveLength(1);
    expect(top).toBe(250);
    await user.click(within(one).getByRole("button", { name: "Cancel" }));
    expect(top).toBe(250);
  });

  it("identifies a book-level fallback as broader than the selected verse", async () => {
    const { api, book } = setup();
    const [location] = await api.libraryPassageSourceLocations(book.id, "JHN.15.5");
    vi.spyOn(api, "libraryPassageSourceLocations").mockResolvedValueOnce([{ ...location!, structureScope: "book", reference: { canonical: "JHN.1-JHN.21", display: "John", book: "JHN", startChapter: 1, startVerse: null, endChapter: 21, endVerse: null } }]);
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    await user.click(sync(reader));
    await waitFor(() => expect(top).toBe(30));
    expect(await within(reader).findByText("Last passage opened here: Book section John (broader)")).toBeInTheDocument();
    expect(within(reader).getByText("Scripture: John 15:5")).toBeInTheDocument();
  });

  it("keeps an Original PDF at its page until requested, then preserves the view and Back page", async () => {
    const fixture: SourceFixture = {
      title: "Orchard PDF", format: "pdf",
      segments: [{ text: "Opening page.", pageNumber: 1 }, { text: "The page being read.", pageNumber: 2 }, { text: "A section on the vine.", pageNumber: 3 }],
      sections: [{ segment: 2, reference: "JHN.15.1-JHN.15.11", matchedText: "John 15:1–11", heading: "The vine" }],
    };
    const { api } = setup(fixture);
    const { user } = await renderApp(api, { pdfEngine: createFakePdfEngine({ pages: 3 }) });
    await user.click(await within(sources()).findByRole("button", { name: fixture.title }));
    const reader = await within(sources()).findByRole("region", { name: fixture.title });
    await user.click(await within(reader).findByRole("button", { name: "Original" }));
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(1)));
    await jumpReaderPage(user, 2, reader);
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(2)));
    await chooseScripture(user, "John 15:5");
    expect(readerPage(reader)).toHaveAccessibleName(readerPageName(2));
    await user.click(sync(reader));
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(3)));
    expect(within(reader).getByRole("button", { name: "Original" })).toHaveAttribute("aria-pressed", "true");
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(2)));
  });

  it("waits for existing EPUB contents to be ready before matching and ignores a passage changed during that wait", async () => {
    const { api } = setup({ ...BOOK, format: "epub" });
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    let release: () => void = () => undefined;
    const contents = api.librarySourceContents;
    vi.spyOn(api, "librarySourceContents").mockImplementationOnce(async (id) => {
      await new Promise<void>((resolve) => { release = resolve; });
      return contents(id);
    });
    const matches = vi.spyOn(api, "libraryPassageSourceLocations");
    await user.click(sync(reader));
    expect(matches).not.toHaveBeenCalled();
    await chooseScripture(user, "John 15:14");
    await act(async () => release());
    expect(matches).not.toHaveBeenCalled();
    expect(top).toBe(250);
    await user.click(sync(reader));
    await waitFor(() => expect(top).toBe(200));
  });

  it.each(["matching", "loading the chosen target"] as const)("ignores a stale response while %s after Scripture changes", async (phase) => {
    const { api } = setup();
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    let release: () => void = () => undefined;
    const delayed = new Promise<void>((resolve) => { release = resolve; });
    if (phase === "matching") {
      const original = api.libraryPassageSourceLocations;
      vi.spyOn(api, "libraryPassageSourceLocations").mockImplementationOnce(async (...args) => { await delayed; return original(...args); });
    } else {
      const original = api.libraryOpenSegment;
      vi.spyOn(api, "libraryOpenSegment").mockImplementationOnce(async (...args) => { await delayed; return original(...args); });
    }
    await user.click(sync(reader));
    await waitFor(() => expect(sync(reader)).toBeDisabled());
    await chooseScripture(user, "John 15:14");
    await act(async () => release());
    expect(top).toBe(250);
    expect(within(reader).queryByText(/Last passage opened here:/)).toBeNull();
    expect(sync(reader)).toBeEnabled();
    await user.click(sync(reader));
    await waitFor(() => expect(top).toBe(200));
    expect(await within(reader).findByText("Last passage opened here: John 15:12–17")).toBeInTheDocument();
  });

  it("cancels a pending request when this source closes and leaves a newly opened source alone", async () => {
    const { api } = setup();
    api.installFixtureSource({ ...BOOK, title: "Other Orchard", sections: [] });
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    let release: () => void = () => undefined;
    const original = api.libraryPassageSourceLocations;
    vi.spyOn(api, "libraryPassageSourceLocations").mockImplementationOnce(async (...args) => {
      await new Promise<void>((resolve) => { release = resolve; });
      return original(...args);
    });
    await user.click(sync(reader));
    await user.click(within(reader).getByRole("button", { name: "Back to Sources" }));
    // A recently read source makes other library entries an explicit disclosure.
    const others = within(sources()).queryByText("Other sources");
    if (others) await user.click(others);
    await user.click(await within(sources()).findByRole("button", { name: "Other Orchard" }));
    const other = await within(sources()).findByRole("region", { name: "Other Orchard" });
    const body = within(other).getByTestId("reader-body");
    body.scrollTop = 57;
    await act(async () => release());
    expect(body.scrollTop).toBe(57);
    expect(within(other).queryByText(/Last passage opened here:/)).toBeNull();
    expect(await readerActionEnabled(user, "Back to the previous place", other)).toBe(false);
  });

  it.each(["Contents", "Back"] as const)("keeps a newer %s choice when the pending passage request finishes", async (action) => {
    const { api } = setup({ ...BOOK, generatedContents: [
      ...BOOK.generatedContents!,
      { id: "newer", title: "My newer choice", target: { kind: "segment", position: 300 }, children: [] },
    ] });
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    let release: () => void = () => undefined;
    const pending = new Promise<void>((resolve) => { release = resolve; });
    // Cover both the lookup and the second async boundary that loads its target.
    const method = action === "Contents" ? "libraryPassageSourceLocations" : "libraryOpenSegment";
    if (method === "libraryPassageSourceLocations") {
      const original = api.libraryPassageSourceLocations;
      vi.spyOn(api, method).mockImplementationOnce(async (...args) => { await pending; return original(...args); });
    } else {
      const original = api.libraryOpenSegment;
      vi.spyOn(api, method).mockImplementationOnce(async (...args) => { await pending; return original(...args); });
    }
    await user.click(sync(reader));
    await waitFor(() => expect(sync(reader)).toBeDisabled());
    if (action === "Contents") {
      await readerAction(user, "Contents", reader);
      await user.click(await within(reader).findByRole("button", { name: /^My newer choice/ }));
    } else await readerAction(user, "Back to the previous place", reader);
    const wanted = action === "Contents" ? 300 : 0;
    await waitFor(() => expect(top).toBe(wanted));
    expect(sync(reader)).toBeEnabled();
    await act(async () => release());
    expect(top).toBe(wanted);
    expect(within(reader).getByText(`Orchard paragraph ${wanted}.`)).toBeInTheDocument();
    expect(within(reader).queryByText(/Last passage opened here:/)).toBeNull();
    if (action === "Contents") {
      await readerAction(user, "Back to the previous place", reader);
      await waitFor(() => expect(top).toBe(250));
    } else {
      await readerAction(user, "Forward to the next place", reader);
      await waitFor(() => expect(top).toBe(250));
    }
  });

  it("keeps a newer Original PDF page choice when a passage lookup finishes", async () => {
    const { api } = setup({ title: "Orchard PDF", format: "pdf", segments: [
      { text: "First page.", pageNumber: 1 }, { text: "My chosen page.", pageNumber: 2 }, { text: "The matching section.", pageNumber: 3 },
    ], sections: [{ segment: 2, reference: "JHN.15.5", matchedText: "John 15:5", heading: "Vine" }] });
    const { user } = await renderApp(api, { pdfEngine: createFakePdfEngine({ pages: 3 }) });
    await user.click(await within(sources()).findByRole("button", { name: "Orchard PDF" }));
    const reader = await within(sources()).findByRole("region", { name: "Orchard PDF" });
    await user.click(within(reader).getByRole("button", { name: "Original" }));
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(1)));
    await chooseScripture(user, "John 15:5");
    let release: () => void = () => undefined;
    const original = api.libraryPassageSourceLocations;
    vi.spyOn(api, "libraryPassageSourceLocations").mockImplementationOnce(async (...args) => {
      await new Promise<void>((resolve) => { release = resolve; });
      return original(...args);
    });
    await user.click(sync(reader));
    await jumpReaderPage(user, 2, reader);
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(2)));
    expect(sync(reader)).toBeEnabled();
    await act(async () => release());
    expect(readerPage(reader)).toHaveAccessibleName(readerPageName(2));
    expect(within(reader).getByRole("button", { name: "Original" })).toHaveAttribute("aria-pressed", "true");
    expect(within(reader).queryByText(/Last passage opened here:/)).toBeNull();
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(1)));
  });

  it.each(["Reading section", "Reading PDF page", "Original PDF section"] as const)("ignores an older async %s load after an explicit passage jump", async (action) => {
    const isPdf = action !== "Reading section";
    const fixture: SourceFixture = { ...BOOK, format: isPdf ? "pdf" : "markdown",
      segments: BOOK.segments.map((segment, position) => ({ ...segment, ...(isPdf ? { pageNumber: Math.floor(position / 70) + 1 } : {}) })),
      generatedContents: [{ id: "older", title: "The earlier requested place", target: { kind: "segment", position: 300 }, children: [] }],
    };
    const { api } = setup(fixture);
    const engine = createFakePdfEngine({ pages: 5, outline: fixture.generatedContents });
    const { user } = await renderApp(api, { pdfEngine: engine });
    await user.click(await within(sources()).findByRole("button", { name: BOOK.title }));
    const reader = await within(sources()).findByRole("region", { name: BOOK.title });
    if (action === "Original PDF section") {
      await user.click(within(reader).getByRole("button", { name: "Original" }));
      await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(1)));
    }
    await chooseScripture(user, "John 15:14");
    let release: () => void = () => undefined;
    let waiting = false;
    const original = api.sourceGetSegments;
    vi.spyOn(api, "sourceGetSegments").mockImplementation(async (query) => {
      const older = action === "Reading PDF page" ? query.aroundPage === 5 : action === "Original PDF section" ? query.offset === 300 && query.limit === 1 : query.offset === 280;
      if (older && !waiting) {
        waiting = true;
        await new Promise<void>((resolve) => { release = resolve; });
      }
      return original(query);
    });
    if (action === "Reading PDF page") await jumpReaderPage(user, 5, reader);
    else {
      await readerAction(user, "Contents", reader);
      await user.click(await within(reader).findByRole("button", { name: /^The earlier requested place/ }));
    }
    await waitFor(() => expect(waiting).toBe(true));
    await user.click(sync(reader));
    await within(reader).findByText("Last passage opened here: John 15:12–17");
    if (action === "Original PDF section") await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(3)));
    else await waitFor(() => expect(top).toBe(200));
    await act(async () => release());
    if (action === "Original PDF section") {
      expect(readerPage(reader)).toHaveAccessibleName(readerPageName(3));
      expect(within(reader).getByRole("button", { name: "Original" })).toHaveAttribute("aria-pressed", "true");
    } else {
      expect(top).toBe(200);
      expect(within(reader).getByText("Orchard paragraph 200.")).toBeInTheDocument();
      if (isPdf) expect(readerPage(reader)).toHaveAccessibleName(readerPageName(3));
    }
    await readerAction(user, "Back to the previous place", reader);
    if (action === "Original PDF section") await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(1)));
    else await waitFor(() => expect(top).toBe(0));
  });

  it.each([FORMATTED_BOOK, FORMATTED_DOCX])("keeps newer Previous/Next part choices in $format when a passage lookup finishes", async (fixture) => {
    const { api } = setup({ ...fixture, sections: [{ segment: fixture.formatted!.structure[2]!.startSegment, reference: "JHN.15.5", matchedText: "John 15:5", heading: "Notes on the vine" }] });
    const { user } = await renderApp(api);
    await user.click(await within(sources()).findByRole("button", { name: fixture.title }));
    const reader = await within(sources()).findByRole("region", { name: fixture.title });
    await user.click(await within(reader).findByRole("button", { name: "Original" }));
    const controls = await within(reader).findByRole("toolbar", { name: "Original document controls" });
    await waitFor(() => expect(within(controls).getByRole("status")).toHaveTextContent("Part 1 of 3"));
    await chooseScripture(user, "John 15:5");
    const original = api.libraryPassageSourceLocations;
    const matching = vi.spyOn(api, "libraryPassageSourceLocations");
    for (const [action, wanted] of [["Next part", 2], ["Previous part", 1]] as const) {
      let release: () => void = () => undefined;
      matching.mockImplementationOnce(async (...args) => {
        await new Promise<void>((resolve) => { release = resolve; });
        return original(...args);
      });
      const calls = matching.mock.calls.length;
      await user.click(sync(reader));
      await waitFor(() => expect(matching.mock.calls.length).toBe(calls + 1));
      await user.click(within(controls).getByRole("button", { name: action }));
      await waitFor(() => expect(within(controls).getByRole("status")).toHaveTextContent(`Part ${wanted} of 3`));
      expect(sync(reader)).toBeEnabled();
      await act(async () => release());
      expect(within(controls).getByRole("status")).toHaveTextContent(`Part ${wanted} of 3`);
      expect(within(reader).queryByText(/Last passage opened here:/)).toBeNull();
      expect(await readerActionEnabled(user, "Back to the previous place", reader)).toBe(false);
    }
    expect(within(reader).getByRole("button", { name: "Original" })).toHaveAttribute("aria-pressed", "true");
  });

  it.each(["completed", "pending"] as const)("refreshes rebuilt PDF metadata without overriding a newer %s passage request", async (phase) => {
    const { api, book } = setup({ ...BOOK, format: "pdf", segments: BOOK.segments.map((segment, position) => ({ ...segment, pageNumber: Math.floor(position / 70) + 1 })) });
    const view = { ...await api.libraryPdfStructure(book.id), status: "ready" as const, complete: true };
    const structure = vi.spyOn(api, "libraryPdfStructure").mockResolvedValue(view);
    vi.spyOn(api, "libraryPdfStructureBuild").mockResolvedValue(view);
    const { user } = await renderApp(api, { pdfEngine: createFakePdfEngine({ pages: 5 }) });
    await user.click(await within(sources()).findByRole("button", { name: BOOK.title }));
    const reader = await within(sources()).findByRole("region", { name: BOOK.title });
    await chooseScripture(user, "John 15:14");
    await readerAction(user, "Chapters and headings", reader);
    const dialog = await screen.findByRole("dialog", { name: "Chapters and headings" });
    let releaseRefresh: () => void = () => undefined;
    let refreshing = false;
    const originalSegments = api.sourceGetSegments;
    vi.spyOn(api, "sourceGetSegments").mockImplementationOnce(async (query) => {
      refreshing = true;
      await new Promise<void>((resolve) => { releaseRefresh = resolve; });
      return originalSegments(query);
    });
    await user.click(await within(dialog).findByRole("button", { name: "Rebuild structure" }));
    await waitFor(() => expect(refreshing).toBe(true));
    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    let releasePassage: () => void = () => undefined;
    if (phase === "pending") {
      const originalLocations = api.libraryPassageSourceLocations;
      vi.spyOn(api, "libraryPassageSourceLocations").mockImplementationOnce(async (...args) => {
        await new Promise<void>((resolve) => { releasePassage = resolve; });
        return originalLocations(...args);
      });
    }
    await user.click(sync(reader));
    if (phase === "completed") await waitFor(() => expect(top).toBe(200));
    else expect(sync(reader)).toBeDisabled();
    const beforeRefresh = structure.mock.calls.length;
    await act(async () => releaseRefresh());
    // The metadata revision still refreshes Contents even when its old text window is discarded.
    await waitFor(() => expect(structure.mock.calls.length).toBeGreaterThan(beforeRefresh));
    if (phase === "pending") {
      expect(sync(reader)).toBeDisabled();
      await act(async () => releasePassage());
      await waitFor(() => expect(top).toBe(200));
    }
    expect(top).toBe(200);
    expect(within(reader).getByText("Orchard paragraph 200.")).toBeInTheDocument();
    expect(within(reader).getByText("Last passage opened here: John 15:12–17")).toBeInTheDocument();
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(top).toBe(0));
  });

  it("refreshes the current text window when a PDF rebuild finishes after the reader has moved", async () => {
    const { api, book } = setup({ ...BOOK, format: "pdf", segments: BOOK.segments.map((segment, position) => ({ ...segment, pageNumber: Math.floor(position / 70) + 1 })) });
    const view = { ...await api.libraryPdfStructure(book.id), status: "ready" as const, complete: true };
    const structure = vi.spyOn(api, "libraryPdfStructure").mockResolvedValue(view);
    let releaseBuild: () => void = () => undefined;
    vi.spyOn(api, "libraryPdfStructureBuild").mockImplementationOnce(async () => {
      await new Promise<void>((resolve) => { releaseBuild = resolve; });
      return view;
    });
    const { user } = await renderApp(api, { pdfEngine: createFakePdfEngine({ pages: 5 }) });
    await user.click(await within(sources()).findByRole("button", { name: BOOK.title }));
    const reader = await within(sources()).findByRole("region", { name: BOOK.title });
    await chooseScripture(user, "John 15:14");
    await readerAction(user, "Chapters and headings", reader);
    const dialog = await screen.findByRole("dialog", { name: "Chapters and headings" });
    await user.click(await within(dialog).findByRole("button", { name: "Rebuild structure" }));
    await waitFor(() => expect(api.libraryPdfStructureBuild).toHaveBeenCalledOnce());
    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    await user.click(sync(reader));
    await waitFor(() => expect(top).toBe(200));
    const refresh = vi.spyOn(api, "sourceGetSegments");
    const beforeRefresh = structure.mock.calls.length;
    await act(async () => releaseBuild());
    await waitFor(() => expect(refresh).toHaveBeenCalledWith({ sourceId: book.id, offset: 150, limit: 150 }));
    await waitFor(() => expect(structure.mock.calls.length).toBeGreaterThan(beforeRefresh));
    expect(top).toBe(200);
    expect(within(reader).getByText("Orchard paragraph 200.")).toBeInTheDocument();
    expect(readerPage(reader)).toHaveAccessibleName(readerPageName(3));
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(top).toBe(0));
  });

  it("keeps the current place when there is no match", async () => {
    const { api } = setup();
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await user.click(sync(reader));
    expect(await within(reader).findByText("No matching place was found in this source for Psalm 23.")).toBeInTheDocument();
    expect(top).toBe(250);
  });

  it("reads refreshed whole-source coverage without moving position, replacing metadata or adding history", async () => {
    const { api, book } = setup({ ...BOOK, sections: [] });
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    const body = within(reader).getByTestId("reader-body");
    body.scrollTop = 9;
    const source = api.state.library.sources.find(source => source.id === book.id)!;
    // A rebuild can infer coverage after the reader's initial metadata loaded.
    source.coverage = [WHOLE_JOHN];
    const open = vi.spyOn(api, "libraryOpenSegment");
    await chooseScripture(user, "John 15:5");
    expect(top).toBe(250);
    await user.click(sync(reader));
    expect(await within(reader).findByText("This source covers John, but has no precise location for this passage. Your reading place is unchanged.")).toBeInTheDocument();
    expect(open).not.toHaveBeenCalled();
    expect(within(reader).getByTestId("reader-body")).toBe(body);
    expect(top).toBe(250);
    expect(body.scrollTop).toBe(9);
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(top).toBe(0));
    expect(await readerActionEnabled(user, "Back to the previous place", reader)).toBe(false);
  });

  it("offers the sole literal mention as a choice even when the whole source covers the passage", async () => {
    const { api, book } = setup({ ...BOOK, sections: [], mentions: [{ segment: 30, reference: "JHN.15.5", matchedText: "John 15:5" }] });
    api.state.library.sources.find(source => source.id === book.id)!.coverage = [WHOLE_JOHN];
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    const open = vi.spyOn(api, "libraryOpenSegment");
    await user.click(sync(reader));
    const dialog = await screen.findByRole("dialog", { name: "Mentions of John 15:5 in this source" });
    expect(within(dialog).getByText("This source covers John. Choose a mention below to open that place, or cancel to keep reading where you are.")).toBeInTheDocument();
    expect(open).not.toHaveBeenCalled();
    expect(top).toBe(250);
    await user.click(within(dialog).getByRole("button", { name: /Mentions John 15:5/ }));
    await waitFor(() => expect(top).toBe(30));
    await readerAction(user, "Back to the previous place", reader);
    await waitFor(() => expect(top).toBe(250));
  });

  it("does not claim whole-source relevance when the source covers a different chapter", async () => {
    const { api, book } = setup({ ...BOOK, sections: [] });
    api.state.library.sources.find(source => source.id === book.id)!.coverage = [{ ...WHOLE_JOHN, scope: "chapter", reference: { ...WHOLE_JOHN.reference, canonical: "JHN.14", display: "John 14", startChapter: 14, endChapter: 14 } }];
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    await user.click(sync(reader));
    expect(await within(reader).findByText("No matching place was found in this source for John 15:5.")).toBeInTheDocument();
    expect(top).toBe(250);
  });

  it.each(["Scripture", "Contents", "another source"] as const)("ignores a delayed coverage read after the reader chooses %s", async (next) => {
    const { api, book } = setup({ ...BOOK, sections: [], generatedContents: [...BOOK.generatedContents!, { id: "newer-coverage", title: "Newer reading choice", target: { kind: "segment", position: 300 }, children: [] }] });
    api.installFixtureSource({ ...BOOK, title: "Another volume", sections: [] });
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    let release: () => void = () => undefined;
    const previous = await api.libraryGetSource(book.id);
    const read = vi.spyOn(api, "libraryGetSource").mockImplementationOnce(async () => {
      await new Promise<void>(resolve => { release = resolve; });
      return { ...previous, coverage: [WHOLE_JOHN] };
    });
    await user.click(sync(reader));
    await waitFor(() => expect(read).toHaveBeenCalled());
    if (next === "Scripture") await chooseScripture(user, "John 15:14");
    else if (next === "Contents") {
      await readerAction(user, "Contents", reader);
      await user.click(await within(reader).findByRole("button", { name: /^Newer reading choice/ }));
      await waitFor(() => expect(top).toBe(300));
    } else {
      await user.click(within(reader).getByRole("button", { name: "Back to Sources" }));
      await user.click(await within(sources()).findByRole("button", { name: "Another volume" }));
      await within(sources()).findByRole("region", { name: "Another volume" });
    }
    await act(async () => release());
    expect(within(sources()).queryByText(/This source covers John, but/)).not.toBeInTheDocument();
    if (next !== "another source") {
      expect(top).toBe(next === "Contents" ? 300 : 250);
      expect(sync(reader)).toBeEnabled();
    } else expect(within(sources()).getByRole("region", { name: "Another volume" })).toBeInTheDocument();
  });

  it("does not let a pending coverage read overwrite newer saved details or revive an outdated message", async () => {
    const { api, book } = setup({ ...BOOK, sections: [] });
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    await chooseScripture(user, "John 15:5");
    let release: () => void = () => undefined;
    const previous = await api.libraryGetSource(book.id);
    const read = vi.spyOn(api, "libraryGetSource").mockImplementationOnce(async () => {
      await new Promise<void>(resolve => { release = resolve; });
      return { ...previous, coverage: [WHOLE_JOHN] };
    });
    await user.click(sync(reader));
    await waitFor(() => expect(read).toHaveBeenCalled());
    await readerAction(user, "Edit details", reader);
    const dialog = await screen.findByRole("dialog", { name: "Edit source details" });
    const title = await within(dialog).findByRole("textbox", { name: "Title" });
    await user.clear(title);
    await user.type(title, "Corrected current title");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    await act(async () => release());
    expect(within(sources()).getByRole("region", { name: "Corrected current title" })).toBe(reader);
    expect(within(reader).queryByText(/This source covers John, but/)).not.toBeInTheDocument();
    expect(sync(reader)).toBeEnabled();
    expect(top).toBe(250);
    expect(await api.libraryGetSource(book.id)).toMatchObject({ title: "Corrected current title" });
  });

  it("explains the unavailable action when no Scripture Bible or passage has been chosen", async () => {
    const { api } = setup(BOOK, false);
    const { user } = await renderApp(api);
    const reader = await openAtKeptPlace(user);
    expect(sync(reader)).toBeDisabled();
    expect(within(reader).getByText("Choose a Scripture passage first.")).toBeInTheDocument();
    expect(api.libraryLocationsCalls).toHaveLength(0);
  });
});
