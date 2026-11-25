import { useState } from "react";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import i18n, { setLanguage } from "@/i18n";
import { localizeMessage } from "@/i18n/messages";
import { PassageSourceList } from "@/features/library/PassageSourceList";
import { SourceMetadataFields, type SourceMetadataDraft } from "@/features/library/SourceMetadataEditor";
import type { PassageLocation, PassageSourceGroup, SourceCoverage, SourceCoverageMode } from "@/types/library";
import { createMemoryApi, type SourceFixture } from "../support/memoryApi";
import { openSettingsSection, renderAppInProject as renderApp } from "../support/render";
import { createFakePdfEngine } from "../support/fakePdfEngine";
import { jumpReaderPage, readerPage, readerPageName } from "../support/readerActions";

const ROMANS: SourceCoverage = {
  reference: { canonical: "ROM.1-ROM.16", display: "Romans", book: "ROM", startChapter: 1, startVerse: null, endChapter: 16, endVerse: null },
  scope: "book", basis: "manual",
};
const JOHN: SourceCoverage = { ...ROMANS, reference: { ...ROMANS.reference, canonical: "JHN.1-JHN.21", display: "John", book: "JHN", endChapter: 21 } };
const BOOK: SourceFixture = { title: "Invented Study Volume", author: "Invented Writer", format: "markdown", segments: [{ text: "The whole source has no exact passage target." }] };
const sources = () => screen.getByRole("complementary", { name: "Sources" });
type User = Awaited<ReturnType<typeof renderApp>>["user"];
function setup() {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  api.state.settings.scripturePassage = "PSA.23";
  const source = api.installFixtureSource(BOOK);
  return { api, source };
}
function group(locations: PassageLocation[] = []): PassageSourceGroup {
  return { sourceId: "whole-volume", sourceTitle: BOOK.title, author: BOOK.author!, coverage: [ROMANS], locations, moreLocations: 0 };
}
function mention(index: number): PassageLocation {
  return { segmentId: `mention-${index}`, locator: `p. ${index}`, origin: "explicit", matchKind: "contained", reference: { ...ROMANS.reference, canonical: "ROM.8.1", display: "Romans 8:1", startChapter: 8, endChapter: 8, startVerse: 1, endVerse: 1 }, matchedText: "Romans 8:1", sectionHeading: null };
}
async function editor(user: User) {
  await user.click(await within(sources()).findByRole("button", { name: `Edit details for ${BOOK.title}` }));
  const dialog = await screen.findByRole("dialog", { name: "Edit source details" });
  await user.click(within(dialog).getByText("More details and the file"));
  return dialog;
}
async function change(user: User, root: HTMLElement, label: string, value: string) {
  const field = within(root).getByRole("textbox", { name: label });
  await user.clear(field);
  if (value) await user.type(field, value);
}
afterEach(() => vi.restoreAllMocks());

describe("whole-source relevance without invented destinations", () => {
  it("shows a coverage-only source and opens without a segment, page or chapter target", async () => {
    const open = vi.fn();
    const user = userEvent.setup();
    render(<PassageSourceList groups={[group()]} label="Matching sources" passageLabel={ref => ref.display} onOpen={open} />);
    const result = screen.getByRole("button", { name: /^Invented Study Volume/ });
    expect(result).toHaveTextContent("Covers Romans");
    expect(result).toHaveTextContent("Open source");
    expect(result).not.toHaveAttribute("data-source-segment");
    expect(result).not.toHaveTextContent(/p\. \d|Romans 1–16|Mentions/);
    await user.click(result);
    expect(open).toHaveBeenCalledExactlyOnceWith("whole-volume");
  });

  it("keeps every literal mention available while the primary action opens the whole source", async () => {
    const places = [1, 2, 3, 4, 5].map(mention);
    const result = { ...group(places.slice(0, 2)), moreLocations: 3 };
    const open = vi.fn();
    function Results() {
      const [expanded, setExpanded] = useState(false);
      return <PassageSourceList groups={[result]} label="Matching sources" passageLabel={ref => ref.display} onOpen={open} expansion={{ "whole-volume": { open: expanded, places, status: "ready" } }} onToggleMore={() => setExpanded(!expanded)} />;
    }
    const user = userEvent.setup();
    render(<Results />);
    await user.click(screen.getByRole("button", { name: /^Invented Study Volume/ }));
    expect(open).toHaveBeenLastCalledWith("whole-volume");
    await user.click(screen.getByRole("button", { name: "5 mentions in this source" }));
    const choices = screen.getAllByRole("button", { name: /Mentions Romans 8:1/ });
    expect(choices).toHaveLength(5);
    await user.click(choices[0]!);
    expect(open).toHaveBeenLastCalledWith("whole-volume", "mention-1", undefined);
    await user.click(choices[4]!);
    expect(open).toHaveBeenLastCalledWith("whole-volume", "mention-5", undefined);
  });

  it.each(["section", "manual"] as const)("keeps a real %s destination primary alongside whole-source coverage", async (origin) => {
    const target: PassageLocation = { ...mention(2), origin, structureNodeId: "real-heading" };
    const open = vi.fn();
    const user = userEvent.setup();
    render(<PassageSourceList groups={[group([target])]} label="Matching sources" passageLabel={ref => ref.display} onOpen={open} />);
    const result = screen.getByRole("button", { name: /^Invented Study Volume/ });
    expect(result).toHaveTextContent("Covers Romans");
    expect(result).toHaveTextContent("p. 2");
    expect(screen.getAllByRole("listitem")).toHaveLength(1);
    await user.click(result);
    expect(open).toHaveBeenCalledExactlyOnceWith("whole-volume", "mention-2", "real-heading");
  });

  it("offers coverage-only results without the mentions-only warning and resumes the Original PDF page", async () => {
    const { api } = setup();
    const source = api.installFixtureSource({ title: "Whole John Volume", format: "pdf", segments: [1, 2, 3].map(pageNumber => ({ text: `Invented page ${pageNumber}.`, pageNumber })) });
    vi.spyOn(api, "libraryPassageSources").mockImplementation(async reference => reference.startsWith("JHN.") ? [{ sourceId: source.id, sourceTitle: source.title, author: null, coverage: [JOHN], locations: [], moreLocations: 0 }] : []);
    const { user } = await renderApp(api, { pdfEngine: createFakePdfEngine({ pages: 3 }) });
    await user.click(await within(sources()).findByRole("button", { name: source.title }));
    let reader = await within(sources()).findByRole("region", { name: source.title });
    await user.click(await within(reader).findByRole("button", { name: "Original" }));
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(1)));
    await jumpReaderPage(user, 2, reader);
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(2)));
    await user.click(within(reader).getByRole("button", { name: "Back to Sources" }));
    await user.type(within(screen.getByRole("complementary", { name: "Scripture" })).getByRole("textbox", { name: "Go to passage" }), "John 15:5{Enter}");
    const list = await within(sources()).findByRole("list", { name: "Sources for John 15:5" });
    expect(within(sources()).queryByText("Only references or mentions are indexed here; no direct discussion has been verified.")).not.toBeInTheDocument();
    await user.click(within(list).getByRole("button", { name: /^Whole John Volume/ }));
    reader = await within(sources()).findByRole("region", { name: source.title });
    await waitFor(() => expect(readerPage(reader)).toHaveAccessibleName(readerPageName(2)));
    expect(within(reader).getByRole("button", { name: "Original" })).toHaveAttribute("aria-pressed", "true");
  });
});

describe("coverage in the shared source-details editor", () => {
  it.each([true, false])("omits untouched coverage during a normal metadata save (existing manual coverage: %s)", async (existing) => {
    const { api, source } = setup();
    if (existing) Object.assign(api.state.library.sources.find(item => item.id === source.id)!, { coverage: [ROMANS], coverageMode: "manual" });
    const update = vi.spyOn(api, "libraryUpdateSource");
    const { user } = await renderApp(api);
    const dialog = await editor(user);
    if (existing) expect(within(dialog).getByRole("textbox", { name: "Books or passages" })).toHaveValue("Romans");
    await change(user, dialog, "Author", "Corrected Writer");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(update).toHaveBeenCalledTimes(1);
    expect(update.mock.calls[0]![0]).not.toHaveProperty("coverage");
    expect(await api.libraryGetSource(source.id)).toMatchObject({ author: "Corrected Writer", ...(existing ? { coverage: [ROMANS], coverageMode: "manual" } : {}) });
  });

  it("cancels a manual coverage draft without saving either coverage or title", async () => {
    const { api, source } = setup();
    const update = vi.spyOn(api, "libraryUpdateSource");
    const { user } = await renderApp(api);
    const dialog = await editor(user);
    await user.selectOptions(within(dialog).getByRole("combobox", { name: "How coverage is set" }), "manual");
    await change(user, dialog, "Books or passages", "Romans\nJohn 15:1–11");
    await change(user, dialog, "Title", "Unsaved title");
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(update).not.toHaveBeenCalled();
    expect(await api.libraryGetSource(source.id)).toMatchObject({ title: BOOK.title });
    expect((await api.libraryGetSource(source.id)).coverage).toBeUndefined();
  });

  it("keeps a rejected native save intact, then sends natural and canonical references together with the corrected details", async () => {
    const { api, source } = setup();
    const segment = api.state.library.segments.find(segment => segment.sourceId === source.id)!;
    await api.sourceAnnotationCreate({ sourceId: source.id, kind: "bookmark", place: { segmentId: segment.id } });
    const marks = await api.sourceAnnotationList(source.id);
    const text = structuredClone(api.state.library.segments);
    const original = api.libraryUpdateSource;
    const update = vi.spyOn(api, "libraryUpdateSource").mockRejectedValueOnce({ kind: "invalidInput", message: "Invalid reference", code: "library.coverage.invalidPassage", params: { passage: "Unclear book" } }).mockImplementation(async input => {
      const { coverage, ...details } = input;
      expect(coverage).toEqual({ mode: "manual", passages: ["Romans", "JHN.15.1-JHN.15.11"] });
      await original(details);
      Object.assign(api.state.library.sources.find(item => item.id === source.id)!, { coverage: [ROMANS], coverageMode: "manual" });
      return api.libraryGetSource(source.id);
    });
    const { user } = await renderApp(api);
    const dialog = await editor(user);
    await user.selectOptions(within(dialog).getByRole("combobox", { name: "How coverage is set" }), "manual");
    await change(user, dialog, "Books or passages", "Unclear book");
    await change(user, dialog, "Title", "Corrected volume");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(await within(dialog).findByText(/“Unclear book” could not be identified/)).toBeInTheDocument();
    expect(within(dialog).getByRole("textbox", { name: "Title" })).toHaveValue("Corrected volume");
    expect(within(dialog).getByRole("textbox", { name: "Books or passages" })).toHaveValue("Unclear book");
    expect(await api.libraryGetSource(source.id)).toMatchObject({ title: BOOK.title });
    await change(user, dialog, "Books or passages", " Romans \n\n JHN.15.1-JHN.15.11 ");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(update).toHaveBeenCalledTimes(2);
    expect(await api.libraryGetSource(source.id)).toMatchObject({ id: source.id, title: "Corrected volume", coverageMode: "manual", originalFilename: source.originalFilename, contentHash: source.contentHash });
    expect(api.state.library.segments).toEqual(text);
    expect(await api.sourceAnnotationList(source.id)).toEqual(marks);
  });

  it.each(["none", "automatic"] as SourceCoverageMode[])("saves the explicit %s choice through Settings using the shared controls", async (mode) => {
    const { api, source } = setup();
    Object.assign(api.state.library.sources.find(item => item.id === source.id)!, { coverage: [ROMANS], coverageMode: "manual" });
    const original = api.libraryUpdateSource;
    const update = vi.spyOn(api, "libraryUpdateSource").mockImplementation(async input => {
      const { coverage, ...details } = input;
      await original(details);
      if (coverage) Object.assign(api.state.library.sources.find(item => item.id === source.id)!, { coverage: [], coverageMode: coverage.mode });
      return api.libraryGetSource(source.id);
    });
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Sources");
    const list = await within(section).findByRole("list", { name: "Sources" });
    await user.click(within(list).getByRole("button", { name: /^Invented Study Volume/ }));
    await user.click(await within(list).findByText("More details and the file"));
    await user.selectOptions(within(list).getByRole("combobox", { name: "How coverage is set" }), mode);
    await user.click(within(list).getByRole("button", { name: "Save" }));
    await within(list).findByText("Saved.");
    expect(update.mock.calls[0]![0].coverage).toEqual({ mode });
    expect((await api.libraryGetSource(source.id)).coverageMode).toBe(mode);
    await change(user, list, "Author", "Later author correction");
    await user.click(within(list).getByRole("button", { name: "Save" }));
    await within(list).findByText("Saved.");
    expect(update).toHaveBeenCalledTimes(2);
    expect(update.mock.calls[1]![0]).not.toHaveProperty("coverage");
  });

  it.each([
    ["en", "Covers Romans", "Set coverage"], ["de", "Behandelt Römer", "Themenbereich angeben"],
    ["ko", "다루는 본문: 로마서", "직접 지정"], ["fr", "Traite de Romains", "Définir l’étendue"],
    ["es", "Trata Romanos", "Definir el contenido"], ["pt", "Trata de Romanos", "Definir abrangência"],
  ] as const)("localizes coverage display, editable controls and native validation in %s", async (language, covers, manual) => {
    const { source } = setup();
    await act(() => setLanguage(language));
    function Fields() {
      const [draft, setDraft] = useState<SourceMetadataDraft>({ ...source, coverage: [ROMANS], coverageMode: "manual" });
      return <SourceMetadataFields draft={draft} onChange={setDraft} />;
    }
    render(<><PassageSourceList groups={[group()]} label="Matching sources" passageLabel={ref => ref.display} onOpen={vi.fn()} /><Fields /></>);
    expect(screen.getByRole("button", { name: /^Invented Study Volume/ })).toHaveTextContent(covers);
    const user = userEvent.setup();
    await user.click(screen.getByText(i18n.t("library.manage.moreDetails")));
    expect(screen.getByRole("option", { name: manual })).toBeInTheDocument();
    const error = localizeMessage("library.coverage.invalidPassage", { passage: "Synthetic bad input" }, "Untranslated error");
    expect(error).toContain("Synthetic bad input");
    expect(error).not.toContain("Untranslated error");
    for (const code of ["tooMany", "empty", "metadataUnreadable"]) expect(localizeMessage(`library.coverage.${code}`, { max: "8" }, "Untranslated error")).not.toBe("Untranslated error");
  });
});
