import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createMemoryApi, type SourceFixture } from "../support/memoryApi";
import { openSettingsSection, renderApp } from "../support/render";

// Settings → Sources (v0.24): finding, details, collections, tags, archive,
// Trash and deleting for good, one at a time and many at once; and the shared
// confirmation's keyboard safety.

const SOURCES: SourceFixture[] = [
  { title: "Romans Commentary", author: "A. Writer", sourceType: "book", language: "English", format: "pdf", segments: [{ text: "Justified by faith.", pageNumber: 1 }] },
  { title: "Galatians Sermon", author: "B. Preacher", sourceType: "sermon", language: "English", format: "markdown", segments: [{ text: "Freedom in Christ.", sectionPath: "Intro" }] },
  { title: "Psalmen-Notizen", author: "C. Leser", sourceType: "notes", language: "Deutsch", format: "text", segments: [{ text: "Der Herr ist mein Hirte." }] },
];

async function setup(prepare?: (api: ReturnType<typeof createMemoryApi>) => Promise<unknown>) {
  const api = createMemoryApi();
  for (const s of SOURCES) api.installFixtureSource(s);
  await prepare?.(api);
  const rendered = await renderApp(api);
  const section = await openSettingsSection(rendered.user, "Sources");
  await within(section).findByRole("list", { name: "Sources" });
  return { ...rendered, api, section };
}

const titles = (section: HTMLElement) =>
  [...within(section).getByRole("list", { name: "Sources" }).querySelectorAll(":scope > li .type-item")].map((x) => x.textContent);

describe("Settings → Sources", () => {
  it("lists every source and finds them by words, category and language", async () => {
    const { user, section } = await setup();
    expect(within(section).getByText("3 in use · 0 archived · 0 in Trash")).toBeInTheDocument();
    await user.type(within(section).getByRole("searchbox", { name: "Find sources by title, author or file name" }), "preacher");
    await waitFor(() => expect(titles(section)).toEqual(["Galatians Sermon"]));
    await user.clear(within(section).getByRole("searchbox"));
    await user.click(within(section).getByText("Filters"));
    await user.selectOptions(within(section).getByRole("combobox", { name: "Language" }), "Deutsch");
    await waitFor(() => expect(titles(section)).toEqual(["Psalmen-Notizen"]));
    await user.selectOptions(within(section).getByRole("combobox", { name: "Language" }), "");
    await user.selectOptions(within(section).getByRole("combobox", { name: "Category" }), "sermon");
    await waitFor(() => expect(titles(section)).toEqual(["Galatians Sermon"]));
  });

  it("archives many at once, keeps them out of the lists, and restores them", async () => {
    const { api, user, section } = await setup();
    await user.click(within(section).getByRole("checkbox", { name: "Select all on this page" }));
    expect(within(section).getByRole("region", { name: "Selected sources" })).toHaveTextContent("3 selected");
    await user.click(within(section).getByRole("button", { name: "Archive" }));
    await within(section).findByText("3 sources were archived.");
    expect(await api.libraryList({})).toEqual([]);
    await user.click(within(section).getByRole("radio", { name: /^Archived/ }));
    await waitFor(() => expect(titles(section)).toHaveLength(3));
    await user.click(within(section).getByRole("button", { name: "Undo" }));
    await waitFor(() => expect(api.state.library.sources.every((s) => api.state.manage.states.get(s.id)?.state === "active")).toBe(true));
  });

  it("moves to Trash with a confirmation, and deleting many for good asks for their number to be typed", async () => {
    const { api, user, section } = await setup();
    await user.click(within(section).getByRole("checkbox", { name: "Select Romans Commentary" }));
    await user.click(within(section).getByRole("checkbox", { name: "Select Galatians Sermon" }));
    await user.click(within(section).getByRole("button", { name: "Move to Trash…" }));
    const trash = screen.getByRole("alertdialog", { name: "Move 2 sources to Trash?" });
    await user.click(within(trash).getByRole("button", { name: "Move to Trash" }));
    await within(section).findByText("2 sources were moved to Trash.");

    await user.click(within(section).getByRole("radio", { name: /^Trash/ }));
    await waitFor(() => expect(titles(section)).toHaveLength(2));
    await user.click(within(section).getByRole("checkbox", { name: "Select all on this page" }));
    await user.click(within(section).getByRole("button", { name: "Delete permanently" }));
    const really = screen.getByRole("alertdialog", { name: "Delete 2 sources permanently?" });
    const button = within(really).getByRole("button", { name: "Delete permanently" });
    expect(button).toBeDisabled();
    const field = within(really).getByRole("textbox", { name: /To confirm, type 2/ });
    await user.type(field, "2{Enter}");
    // Enter in the field never confirms.
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    expect(api.state.library.sources.filter((s) => s.deleted)).toHaveLength(0);
    await user.click(button);
    await within(section).findByText("2 sources were deleted permanently.");
    expect(api.state.library.sources.filter((s) => s.deleted).map((s) => s.title).sort()).toEqual(["Galatians Sermon", "Romans Commentary"]);
  });

  it("never deletes from the backdrop, Escape or Enter on Cancel", async () => {
    const { api, user, section } = await setup();
    await user.click(within(section).getByRole("button", { name: "More for Romans Commentary" }));
    await user.click(screen.getByRole("menuitem", { name: /^Move to Trash/ }));
    let dialog = screen.getByRole("alertdialog");
    expect(dialog).toHaveAccessibleDescription(/You can restore it from Trash/);
    expect(within(dialog).getByRole("button", { name: "Cancel" })).toHaveFocus();
    await user.keyboard("{Enter}");
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    await user.click(within(section).getByRole("button", { name: "More for Romans Commentary" }));
    await user.click(screen.getByRole("menuitem", { name: /^Move to Trash/ }));
    dialog = screen.getByRole("alertdialog");
    fireEvent.mouseDown(dialog.parentElement!);
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(api.state.manage.states.size).toBe(0);
  });

  it("edits a source's details, collections and tags without touching its text", async () => {
    const { api, user, section } = await setup();
    const list = within(section).getByRole("list", { name: "Sources" });
    await user.click(within(list).getByRole("button", { name: /^Romans Commentary/ }));
    const title = await within(list).findByRole("textbox", { name: "Title" });
    await user.clear(title);
    await user.type(title, "Romans: A Commentary");
    await user.selectOptions(within(list).getByRole("combobox", { name: "Category" }), "commentary");
    await user.type(within(list).getByRole("textbox", { name: "Add tags (separate with commas)" }), "Grace, Romans");
    await user.click(within(list).getByRole("button", { name: "Save" }));
    await within(list).findByText("Saved.");
    const source = api.state.library.sources.find((s) => s.title === "Romans: A Commentary")!;
    expect(source.sourceType).toBe("commentary");
    expect((await api.libraryTags()).map((t) => t.name)).toEqual(["Grace", "Romans"]);
    expect(api.state.library.segments.filter((s) => s.sourceId === source.id)).toHaveLength(1);
  });

  it("puts many sources in a new collection, nested, and finds them by it", async () => {
    let parentId = "";
    const { api, user, section } = await setup(async (a) => (parentId = (await a.libraryCollectionCreate("Paul", null)).id));
    await user.click(within(section).getByRole("checkbox", { name: "Select Romans Commentary" }));
    await user.click(within(section).getByRole("checkbox", { name: "Select Galatians Sermon" }));
    await user.click(within(section).getByRole("button", { name: "Organise" }));
    await user.click(screen.getByRole("menuitem", { name: "Add to a collection…" }));
    const dialog = screen.getByRole("dialog", { name: "Add 2 sources to a collection" });
    await user.selectOptions(within(dialog).getByRole("combobox", { name: "Collection" }), "new");
    await user.type(within(dialog).getByRole("textbox", { name: "Name of the new collection" }), "Letters");
    await user.selectOptions(within(dialog).getByRole("combobox", { name: "Inside" }), parentId);
    await user.click(within(dialog).getByRole("button", { name: "Add to collection" }));
    await within(section).findByText("2 sources are now in “Letters”.");
    const letters = (await api.libraryCollections()).find((c) => c.name === "Letters")!;
    expect(letters).toMatchObject({ parentId, count: 2 });
    await user.click(within(section).getByText("Filters"));
    await user.selectOptions(within(section).getByRole("combobox", { name: "Collection" }), letters.id);
    await waitFor(() => expect(titles(section).sort()).toEqual(["Galatians Sermon", "Romans Commentary"]));
  });
});
