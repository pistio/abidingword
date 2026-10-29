import { act, screen, waitFor, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { createMemoryApi, type SourceFixture } from "../support/memoryApi";
import { renderApp } from "../support/render";

// Invented sources only. Model paragraph geometry so losing a position can fail a test.
const BOOK: SourceFixture = {
  title: "Orchard Notebook", author: "A. Gardener", format: "markdown", language: "English",
  segments: Array.from({ length: 350 }, (_, i) => ({ text: i === 250 ? "Compare John 15:5 in the orchard." : `Orchard paragraph ${i}.`, sectionPath: i < 250 ? "Opening" : "Harvest" })),
  generatedContents: [{ id: "harvest", title: "Harvest", target: { kind: "segment", position: 250 }, children: [] }],
  mentions: [{ segment: 250, reference: "JHN.15.5", matchedText: "John 15:5" }],
};
const sources = () => screen.getByRole("complementary", { name: "Sources" });
type User = Awaited<ReturnType<typeof renderApp>>["user"];
let top = 0;
const rect = Element.prototype.getBoundingClientRect;
beforeEach(() => {
  top = 0;
  Element.prototype.scrollIntoView = function () {
    const element = this as HTMLElement;
    const at = element.dataset.segPosition ?? (element.nextElementSibling as HTMLElement | null)?.dataset.segPosition;
    if (at !== undefined) {
      top = Number(at);
      const body = element.closest<HTMLElement>('[data-testid="reader-body"]');
      if (body) body.scrollTop = 0;
    }
  };
  Element.prototype.getBoundingClientRect = function () {
    const element = this as HTMLElement;
    const y = element.dataset.segPosition !== undefined ? (Number(element.dataset.segPosition) - top) * 30 - (element.closest<HTMLElement>('[data-testid="reader-body"]')?.scrollTop ?? 0) : 0;
    if (element.dataset.segPosition !== undefined || element.dataset.testid === "reader-body") {
      const height = element.dataset.segPosition !== undefined ? 30 : 300;
      return { top: y, bottom: y + height, left: 0, right: 600, width: 600, height, x: 0, y, toJSON: () => ({}) } as DOMRect;
    }
    return rect.call(this);
  };
});
afterEach(() => {
  vi.restoreAllMocks();
  Element.prototype.getBoundingClientRect = rect;
  delete (Element.prototype as unknown as Record<string, unknown>).scrollIntoView;
});

function setup() {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  api.state.settings.scripturePassage = "PSA.23";
  const book = api.installFixtureSource(BOOK);
  return { api, book };
}
async function library(user: User) {
  await user.click(await within(sources()).findByRole("button", { name: "Open library" }));
  await screen.findByRole("heading", { level: 1, name: "Library" });
}
const sourceList = () => screen.getByRole("list", { name: "Sources" });
async function openBook(user: User) {
  await user.click(await within(sourceList()).findByRole("button", { name: /^Orchard Notebook/ }));
  return screen.findByRole("main", { name: "Orchard Notebook" });
}
async function harvest(user: User, reader: HTMLElement) {
  await user.click(await within(reader).findByRole("button", { name: "Contents" }));
  await user.click(await within(reader).findByRole("button", { name: /^Harvest/ }));
  await waitFor(() => expect(top).toBe(250));
}

describe("Library Stage 3 navigation", () => {
  it("names nested collections by their path and browses the selected membership", async () => {
    const { api, book } = setup();
    const parent = await api.libraryCollectionCreate("Garden shelf", null);
    const child = await api.libraryCollectionCreate("Autumn", parent.id);
    await api.libraryCollect(child.id, [book.id], true);
    const { user } = await renderApp(api);
    await library(user);
    await user.selectOptions(screen.getByRole("combobox", { name: "Collection" }), "Garden shelf / Autumn");
    expect(await within(sourceList()).findByText(BOOK.title)).toBeInTheDocument();
    await user.selectOptions(screen.getByRole("combobox", { name: "Collection" }), parent.id);
    await screen.findByText("No sources match these filters.");
  });

  it("ignores a late Show more response after switching collections", async () => {
    const { api } = setup();
    const empty = await api.libraryCollectionCreate("Empty shelf", null);
    const { user } = await renderApp(api);
    await library(user);
    await user.type(screen.getByLabelText("Search the library", { selector: "input:not([type=search])" }), "orchard{Enter}");
    await screen.findByRole("list", { name: "Library search results" });
    const original = api.librarySearch;
    let release: () => void = () => undefined;
    vi.spyOn(api, "librarySearch").mockImplementationOnce(async (input) => {
      await new Promise<void>((resolve) => { release = resolve; });
      return original(input);
    });
    await user.click(screen.getByRole("button", { name: "Show more results" }));
    await user.selectOptions(screen.getByRole("combobox", { name: "Collection" }), empty.id);
    await screen.findByText("Nothing found in your library.");
    await act(async () => { release(); });
    expect(screen.queryByRole("list", { name: "Library search results" })).toBeNull();
  });
  it("opens a collection by keyboard and applies it to source names and text results", async () => {
    const { api, book } = setup();
    api.installFixtureSource({ title: "Other Orchard", segments: [{ text: "orchard outside the collection" }] });
    const collection = await api.libraryCollectionCreate("Garden shelf", null);
    await api.libraryCollect(collection.id, [book.id], true);
    const { user } = await renderApp(api);
    await library(user);
    const action = await screen.findByRole("button", { name: "Open collection: Garden shelf" });
    action.focus();
    await user.keyboard("{Enter}");
    await waitFor(() => expect(within(sourceList()).queryByText("Other Orchard")).toBeNull());
    expect(screen.getByRole("combobox", { name: "Collection" })).toHaveValue(collection.id);
    await user.type(screen.getByLabelText("Search the library", { selector: "input:not([type=search])" }), "orchard{Enter}");
    const names = await screen.findByRole("list", { name: "Sources matching the title, author or file name" });
    expect(within(names).queryByText("Other Orchard")).toBeNull();
    const results = await screen.findByRole("list", { name: "Library search results" });
    expect(within(results).queryByText("Other Orchard")).toBeNull();
    expect(within(results).getAllByRole("listitem")).toHaveLength(50);
  });

  it("returns to the same collection, filters, page, scroll position and source focus", async () => {
    const { api, book } = setup();
    const collection = await api.libraryCollectionCreate("Garden shelf", null);
    await api.libraryCollect(collection.id, [book.id], true);
    for (let n = 0; n < 50; n++) {
      const volume = api.installFixtureSource({ title: `Volume ${n}`, language: "English", segments: [{ text: "Invented." }] });
      await api.libraryCollect(collection.id, [volume.id], true);
    }
    const { user } = await renderApp(api);
    await library(user);
    await user.selectOptions(screen.getByRole("combobox", { name: "Collection" }), collection.id);
    await user.selectOptions(screen.getByRole("combobox", { name: "Status" }), "active");
    await user.selectOptions(screen.getByRole("combobox", { name: "Language" }), "English");
    await user.click(within(await screen.findByRole("navigation", { name: "Pages of sources" })).getByRole("button", { name: "Next" }));
    const row = await within(sourceList()).findByRole("button", { name: /^Orchard Notebook/ });
    screen.getByRole("main").scrollTop = 321;
    await user.click(row);
    await screen.findByRole("heading", { level: 1, name: BOOK.title });
    await user.click(within(screen.getByTestId("reader-header")).getByRole("button", { name: "Back" }));
    await waitFor(() => expect(within(sourceList()).getByRole("button", { name: /^Orchard Notebook/ })).toHaveFocus());
    expect(screen.getByRole("main").scrollTop).toBe(321);
    expect(screen.getByRole("combobox", { name: "Collection" })).toHaveValue(collection.id);
    expect(screen.getByRole("combobox", { name: "Status" })).toHaveValue("active");
    expect(screen.getByRole("combobox", { name: "Language" })).toHaveValue("English");
    expect(screen.getByRole("navigation", { name: "Pages of sources" })).toHaveTextContent("51–51 of 51");
  });

  it("preserves loaded search pages and an unsubmitted query while reopening an exact result", async () => {
    const { api } = setup();
    const { user } = await renderApp(api);
    await library(user);
    const search = () => screen.getByLabelText("Search the library", { selector: "input:not([type=search])" });
    await user.type(search(), "orchard{Enter}");
    await screen.findByRole("list", { name: "Library search results" });
    await user.click(screen.getByRole("button", { name: "Show more results" }));
    await waitFor(() => expect(within(screen.getByRole("list", { name: "Library search results" })).getAllByRole("listitem")).toHaveLength(100));
    await user.type(search(), " draft");
    const results = screen.getByRole("list", { name: "Library search results" });
    await user.click(within(results).getAllByRole("button")[75]!);
    await screen.findByRole("heading", { level: 1, name: BOOK.title });
    expect(document.querySelector("[data-target]")).toHaveAttribute("data-seg-position", "75");
    await user.click(within(screen.getByTestId("reader-header")).getByRole("button", { name: "Back" }));
    await waitFor(() => expect(within(screen.getByRole("list", { name: "Library search results" })).getAllByRole("listitem")).toHaveLength(100));
    expect(search()).toHaveValue("orchard draft");
    expect(within(screen.getByRole("list", { name: "Library search results" })).getAllByRole("button")[75]).toHaveFocus();
  });

  it("returns from a reader's Scripture reference to its collection without changing the Study", async () => {
    const { api, book } = setup();
    const collection = await api.libraryCollectionCreate("Garden shelf", null);
    await api.libraryCollect(collection.id, [book.id], true);
    const before = JSON.stringify({ studies: api.state.studies, findings: api.state.findings, passages: api.state.passages });
    const { user } = await renderApp(api);
    await library(user);
    await user.selectOptions(screen.getByRole("combobox", { name: "Collection" }), collection.id);
    const reader = await openBook(user);
    await harvest(user, reader);
    await user.click(await within(reader).findByRole("button", { name: "John 15:5 (Scripture reference)" }));
    await user.click(within(reader).getByRole("menuitem", { name: "Open in Bible" }));
    await within(screen.getByRole("complementary", { name: "Scripture" })).findByText(/showing John 15:5/);
    let pane = await within(sources()).findByRole("region", { name: BOOK.title });
    await waitFor(() => expect(top).toBe(250));
    await user.click(within(pane).getByRole("button", { name: "Open full reader" }));
    const expanded = await screen.findByRole("main", { name: BOOK.title });
    await user.click(await within(expanded).findByRole("button", { name: "John 15:5 (Scripture reference)" }));
    await user.click(within(expanded).getByRole("menuitem", { name: "Open in Bible" }));
    pane = await within(sources()).findByRole("region", { name: BOOK.title });
    await user.click(within(pane).getByRole("button", { name: "Back to Library" }));
    await screen.findByRole("heading", { level: 1, name: "Library" });
    expect(screen.getByRole("combobox", { name: "Collection" })).toHaveValue(collection.id);
    expect(JSON.stringify({ studies: api.state.studies, findings: api.state.findings, passages: api.state.passages })).toBe(before);
  });

  it("resumes text at its paragraph and offset after Settings and after closing and reopening", async () => {
    const { api } = setup();
    const { user } = await renderApp(api);
    await library(user);
    let reader = await openBook(user);
    await harvest(user, reader);
    within(reader).getByTestId("reader-body").scrollTop = 7;
    await user.click(screen.getByRole("button", { name: "Settings" }));
    await screen.findByRole("heading", { level: 1, name: "Settings" });
    top = 0; // a new reader DOM starts at the beginning unless the app restores it
    await user.click(screen.getByRole("button", { name: "Settings" }));
    reader = await screen.findByRole("main", { name: BOOK.title });
    await waitFor(() => expect(top).toBe(250));
    expect(within(reader).getByTestId("reader-body").scrollTop).toBe(7);
    expect(within(reader).getByRole("button", { name: "Back to the previous place" })).toBeEnabled();
    await user.click(within(reader).getByRole("button", { name: "Back" }));
    await screen.findByRole("heading", { level: 1, name: "Library" });
    top = 0;
    reader = await openBook(user);
    await waitFor(() => expect(top).toBe(250));
    expect(reader.querySelector('[data-seg-position="250"]')).toHaveFocus();
    expect(within(reader).getByTestId("reader-body").scrollTop).toBe(7);
    await user.click(within(reader).getByRole("button", { name: "Back" }));
    await screen.findByRole("heading", { level: 1, name: "Library" });
    await user.type(screen.getByLabelText("Search the library", { selector: "input:not([type=search])" }), "paragraph 12.{Enter}");
    const results = await screen.findByRole("list", { name: "Library search results" });
    await user.click(within(results).getAllByRole("button")[0]!);
    reader = await screen.findByRole("main", { name: BOOK.title });
    await waitFor(() => expect(reader.querySelector("[data-target]")).toHaveAttribute("data-seg-position", "12"));
    expect(top).toBe(12); // an explicit result wins over the remembered paragraph 250
  });

  it("keeps citation words and provenance through pane/full-reader transfers", async () => {
    const { api, book } = setup();
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Gardens" });
    const study = await api.studyCreate({ projectId: project.id, title: "Garden study" });
    const segment = api.state.library.segments.find((item) => item.sourceId === book.id && item.position === 250)!;
    const finding = await api.findingCreateFromSource({ studyId: study.study.id, findingType: "observation", content: "My own note.",
      sourceId: book.id, start: { segmentId: segment.id, offset: 0 }, end: { segmentId: segment.id, offset: 7 }, quote: "Compare" });
    const before = JSON.stringify(finding.sources);
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Garden study/ }));
    await user.click(await screen.findByTitle("Open this source at the cited place"));
    let reader = await within(sources()).findByRole("region", { name: BOOK.title });
    await waitFor(() => expect(reader.querySelector("[data-quoted]")).toHaveTextContent("Compare"));
    await user.click(within(reader).getByRole("button", { name: "Open full reader" }));
    reader = await screen.findByRole("main", { name: BOOK.title });
    await waitFor(() => expect(reader.querySelector("[data-quoted]")).toHaveTextContent("Compare"));
    await user.click(within(reader).getByText("About this source"));
    expect(within(reader).getAllByText("A. Gardener").length).toBeGreaterThan(0);
    await user.click(within(reader).getByRole("button", { name: "Back" }));
    reader = await within(sources()).findByRole("region", { name: BOOK.title });
    await waitFor(() => expect(reader.querySelector("[data-quoted]")).toHaveTextContent("Compare"));
    expect(JSON.stringify(api.state.findings.find((item) => item.id === finding.id)!.sources)).toBe(before);
  });
});
