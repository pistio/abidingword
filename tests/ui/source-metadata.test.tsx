import { screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiError } from "@/lib/api";
import { createMemoryApi, type SourceFixture } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";
import { readerAction, readerActionEnabled } from "../support/readerActions";

const BOOK: SourceFixture = {
  title: "Imported Garden Notes", author: "Original Author", format: "markdown", language: "English",
  segments: Array.from({ length: 30 }, (_, index) => ({ text: `Invented garden paragraph ${index}.` })),
  generatedContents: [{ id: "later", title: "Later notes", target: { kind: "segment", position: 20 }, children: [] }],
};
const sources = () => screen.getByRole("complementary", { name: "Sources" });
type User = Awaited<ReturnType<typeof renderApp>>["user"];
function setup() {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  api.state.settings.scripturePassage = "PSA.23";
  const source = api.installFixtureSource(BOOK);
  return { api, source };
}
async function change(user: User, dialog: HTMLElement, field: string, value: string) {
  const input = await within(dialog).findByRole("textbox", { name: field });
  await user.clear(input);
  if (value) await user.type(input, value);
}
afterEach(() => vi.restoreAllMocks());

describe("source details from the Library and reader", () => {
  it("edits bibliographic details from the Sources overview while keeping the source, file, text and marks", async () => {
    const { api, source } = setup();
    const segment = api.state.library.segments.find((segment) => segment.sourceId === source.id)!;
    await api.sourceAnnotationCreate({ sourceId: source.id, kind: "bookmark", place: { segmentId: segment.id } });
    const beforeText = structuredClone(api.state.library.segments);
    const beforeMarks = await api.sourceAnnotationList(source.id);
    const beforeFile = { id: source.id, originalFilename: source.originalFilename, contentHash: source.contentHash, format: source.format };
    const { user } = await renderApp(api);
    await user.click(await within(sources()).findByRole("button", { name: `Edit details for ${BOOK.title}` }));
    const dialog = await screen.findByRole("dialog", { name: "Edit source details" });
    await change(user, dialog, "Title", "Garden Notes, corrected");
    await change(user, dialog, "Author", "Revised Author");
    await user.click(within(dialog).getByText("More details and the file"));
    await change(user, dialog, "Publisher", "Invented Press");
    await change(user, dialog, "Year", "2026");
    await change(user, dialog, "Edition", "Second edition");
    await change(user, dialog, "Subtitle", "A corrected subtitle");
    await change(user, dialog, "ISBN", "9780000000002");
    await change(user, dialog, "DOI", "10.0000/invented");
    await change(user, dialog, "Notes", "My bibliographic note.");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(await within(sources()).findByRole("button", { name: "Garden Notes, corrected" })).toBeInTheDocument();
    expect(await api.libraryGetSource(source.id)).toMatchObject({ ...beforeFile, title: "Garden Notes, corrected", author: "Revised Author", publisher: "Invented Press", publicationYear: 2026, edition: "Second edition", subtitle: "A corrected subtitle", isbn: "9780000000002", doi: "10.0000/invented", notes: "My bibliographic note." });
    expect(api.state.library.segments).toEqual(beforeText);
    expect(await api.sourceAnnotationList(source.id)).toEqual(beforeMarks);
  });

  it("opens the same editor from the Library list, rejects an empty title, and cancels without saving", async () => {
    const { api, source } = setup();
    const update = vi.spyOn(api, "libraryUpdateSource");
    const { user } = await renderApp(api);
    await user.click(await within(sources()).findByRole("button", { name: "Open library" }));
    const list = await screen.findByRole("list", { name: "Sources" });
    await user.click(within(list).getByRole("button", { name: `Edit details for ${BOOK.title}` }));
    const dialog = await screen.findByRole("dialog", { name: "Edit source details" });
    await change(user, dialog, "Title", "   ");
    expect(within(dialog).getByRole("button", { name: "Save" })).toBeDisabled();
    await change(user, dialog, "Author", "An unsaved name");
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(update).not.toHaveBeenCalled();
    expect(await api.libraryGetSource(source.id)).toMatchObject({ title: BOOK.title, author: BOOK.author });
    expect(within(list).getByRole("button", { name: `Edit details for ${BOOK.title}` })).toHaveFocus();
  });

  it("keeps a failed save editable and updates both the list and source-name search after retry", async () => {
    const { api } = setup();
    const { user } = await renderApp(api);
    await user.click(await within(sources()).findByRole("button", { name: "Open library" }));
    await user.type(screen.getByLabelText("Search the library", { selector: "input:not([type=search])" }), "Garden{Enter}");
    const results = await screen.findByRole("list", { name: "Sources matching the title, author or file name" });
    await user.click(within(results).getByRole("button", { name: `Edit details for ${BOOK.title}` }));
    const dialog = await screen.findByRole("dialog", { name: "Edit source details" });
    await change(user, dialog, "Title", "Garden Notebook");
    vi.spyOn(api, "libraryUpdateSource").mockRejectedValueOnce(new ApiError("invalidInput", "Invented save failure."));
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    expect(await within(dialog).findByText("Invented save failure.")).toBeInTheDocument();
    expect(within(dialog).getByRole("textbox", { name: "Title" })).toHaveValue("Garden Notebook");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    await waitFor(() => expect(within(results).getByText("Garden Notebook")).toBeInTheDocument());
    expect(await within(screen.getByRole("list", { name: "Sources" })).findByText("Garden Notebook")).toBeInTheDocument();
  });

  it("edits in the reader without replacing its body or history, and immediately refreshes its title and citation", async () => {
    const { api } = setup();
    const { user } = await renderApp(api);
    await user.click(await within(sources()).findByRole("button", { name: BOOK.title }));
    const reader = await within(sources()).findByRole("region", { name: BOOK.title });
    await readerAction(user, "Contents", reader);
    await user.click(await within(reader).findByRole("button", { name: /^Later notes/ }));
    expect(await readerActionEnabled(user, "Back to the previous place", reader)).toBe(true);
    const body = within(reader).getByTestId("reader-body");
    body.scrollTop = 193;
    await readerAction(user, "Edit details", reader);
    const dialog = await screen.findByRole("dialog", { name: "Edit source details" });
    await change(user, dialog, "Title", "Garden Notebook");
    await change(user, dialog, "Author", "Revised Author");
    await user.click(within(dialog).getByRole("button", { name: "Save" }));
    await waitFor(() => expect(dialog).not.toBeInTheDocument());
    expect(within(sources()).getByRole("region", { name: "Garden Notebook" })).toBe(reader);
    expect(within(reader).getByTestId("reader-body")).toBe(body);
    expect(body.scrollTop).toBe(193);
    expect(await readerActionEnabled(user, "Back to the previous place", reader)).toBe(true);
    await readerAction(user, "About this source", reader);
    const about = await screen.findByRole("dialog", { name: "About this source" });
    const citation = within(about).getByText("Citation").nextElementSibling;
    expect(citation).toHaveTextContent("Revised Author");
    expect(citation).toHaveTextContent("Garden Notebook");
    await user.click(within(about).getByRole("button", { name: "Close" }));
    await user.click(within(reader).getByRole("button", { name: "Back to Sources" }));
    expect(await within(sources()).findByRole("button", { name: "Garden Notebook" })).toBeInTheDocument();
  });
});
