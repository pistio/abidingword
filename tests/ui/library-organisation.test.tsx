import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createMemoryApi, type SourceFixture } from "../support/memoryApi";
import { renderApp } from "../support/render";

// Invented library material. Nothing here is a real book or a user's library.
const HIDDEN: SourceFixture = {
  title: "Hidden Title",
  author: "Q. Quill",
  format: "pdf",
  language: "English",
  segments: [{ text: "A paragraph about gardens.", pageNumber: 1 }],
};

const KITCHEN: SourceFixture = {
  title: "Kitchen Notes",
  author: "A. Cook",
  format: "markdown",
  sourceType: "article",
  language: "Deutsch",
  segments: [{ text: "Fresh apples on the table.", sectionPath: "Fruit" }],
};

const SHELVED: SourceFixture = {
  title: "Shelved Essay",
  author: "A. Shelved",
  format: "pdf",
  language: "English",
  segments: [{ text: "The orchards of grace.", pageNumber: 4 }],
};

const sourcesPane = () => screen.getByRole("complementary", { name: "Sources" });

async function openLibrary(user: Awaited<ReturnType<typeof renderApp>>["user"]) {
  await user.click(await within(sourcesPane()).findByRole("button", { name: "Open library" }));
  return screen.findByRole("heading", { level: 1, name: "Library" });
}

const librarySearch = () => screen.getByLabelText("Search the library", { selector: "input:not([type=search])" });

describe("Library organisation", () => {
  it("shows collections, tags and an archived badge, and pages the list", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    const hidden = api.installFixtureSource(HIDDEN);
    api.installFixtureSource(KITCHEN);
    const shelved = api.installFixtureSource(SHELVED);
    const collection = await api.libraryCollectionCreate("Commentaries", null);
    await api.libraryCollect(collection.id, [hidden.id], true);
    await api.libraryRetag([hidden.id], ["grace"], []);
    await api.librarySetState([shelved.id], "archived");
    const { user } = await renderApp(api);

    expect(await within(sourcesPane()).findByRole("button", { name: /^Hidden Title/ })).toBeInTheDocument();
    expect(within(sourcesPane()).queryByRole("button", { name: /^Shelved Essay/ })).not.toBeInTheDocument();

    await openLibrary(user);
    const list = () => screen.getByRole("list", { name: "Sources" });
    const row = (title: string) => {
      const item = within(list()).getByText(title).closest("li");
      expect(item).not.toBeNull();
      return item as HTMLElement;
    };
    await waitFor(() => expect(row("Hidden Title")).toHaveTextContent("Collections · Commentaries"));
    expect(row("Hidden Title")).toHaveTextContent("Tags · grace");
    expect(within(row("Shelved Essay")).getByText("Archived")).toBeInTheDocument();

    const filters = screen.getByRole("group", { name: "Filter sources" });
    await user.selectOptions(within(filters).getByLabelText("Status"), "archived");
    await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
    expect(within(list()).getByText("Shelved Essay")).toBeInTheDocument();

    await user.selectOptions(within(filters).getByLabelText("Status"), "active");
    await waitFor(() => expect(within(list()).queryByText("Shelved Essay")).not.toBeInTheDocument());
    await user.selectOptions(within(filters).getByLabelText("Language"), "Deutsch");
    await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
    expect(within(list()).getByText("Kitchen Notes")).toBeInTheDocument();

    await user.selectOptions(within(filters).getByLabelText("Language"), "");
    await user.selectOptions(within(filters).getByLabelText("Type"), "book");
    await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
    expect(within(list()).getByText("Hidden Title")).toBeInTheDocument();
  });

  it("pages a long library fifty sources at a time", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    for (let n = 1; n <= 51; n += 1) {
      api.installFixtureSource({ title: `Volume ${n}`, segments: [{ text: "A short invented paragraph." }] });
    }
    const { user } = await renderApp(api);
    await openLibrary(user);
    const list = () => screen.getByRole("list", { name: "Sources" });
    await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(50));
    const pages = () => screen.getByRole("navigation", { name: "Pages of sources" });
    expect(pages()).toHaveTextContent("1–50 of 51");
    await user.click(within(pages()).getByRole("button", { name: "Next" }));
    await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(1));
    expect(pages()).toHaveTextContent("51–51 of 51");
    await user.click(within(pages()).getByRole("button", { name: "Previous" }));
    await waitFor(() => expect(within(list()).getAllByRole("listitem")).toHaveLength(50));
    expect(pages()).toHaveTextContent("1–50 of 51");
  });

  it("searches by title, author and the text, and labels an archived hit", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    api.installFixtureSource(HIDDEN);
    api.installFixtureSource(KITCHEN);
    const shelved = api.installFixtureSource(SHELVED);
    await api.librarySetState([shelved.id], "archived");
    const { user } = await renderApp(api);
    await openLibrary(user);

    await user.type(librarySearch(), "Hidden Title{Enter}");
    const byName = await screen.findByRole("list", { name: "Sources matching the title, author or file name" });
    expect(within(byName).getByRole("button", { name: /Hidden Title/ })).toBeInTheDocument();
    expect(screen.queryByRole("list", { name: "Library search results" })).not.toBeInTheDocument();

    await user.clear(librarySearch());
    await user.type(librarySearch(), "Cook{Enter}");
    const byAuthor = await screen.findByRole("list", { name: "Sources matching the title, author or file name" });
    await waitFor(() => expect(within(byAuthor).getByText("Kitchen Notes")).toBeInTheDocument());
    expect(within(byAuthor).queryByText("Hidden Title")).not.toBeInTheDocument();

    await user.clear(librarySearch());
    await user.type(librarySearch(), "apples{Enter}");
    const passages = await screen.findByRole("list", { name: "Library search results" });
    expect(within(passages).getByText("Kitchen Notes")).toBeInTheDocument();
    expect(within(passages).getByText("apples", { selector: "mark" })).toBeInTheDocument();
    expect(screen.queryByRole("list", { name: "Sources matching the title, author or file name" })).not.toBeInTheDocument();

    await user.clear(librarySearch());
    await user.type(librarySearch(), "orchards{Enter}");
    const archivedHit = within(await screen.findByRole("list", { name: "Library search results" })).getByRole("button", { name: /Shelved Essay/ });
    expect(archivedHit).toHaveTextContent("Archived");

    const filters = screen.getByRole("group", { name: "Filter sources" });
    await user.selectOptions(within(filters).getByLabelText("Status"), "active");
    await waitFor(() => expect(screen.getByText("Nothing found in your library.")).toBeInTheDocument());
  });

  it("opens Settings → Sources from the library", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    api.installFixtureSource(HIDDEN);
    const { user } = await renderApp(api);
    await openLibrary(user);
    await user.click(screen.getByRole("button", { name: "Manage in Settings" }));
    const sources = await screen.findByRole("region", { name: "Sources" });
    await waitFor(() => expect(within(sources).getByRole("button", { name: "Sources" })).toHaveAttribute("aria-expanded", "true"));
  });

  it("labels an archived passage in the citation picker", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    api.state.settings.scripturePassage = "PSA.23";
    const shelved = api.installFixtureSource(SHELVED);
    await api.librarySetState([shelved.id], "archived");
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "John Series" });
    const study = await api.studyCreate({ projectId: project.id, title: "The True Vine" });
    await api.findingCreate({ studyId: study.study.id, findingType: "observation", content: "A note." });
    const { user } = await renderApp(api);
    await user.click(within(await screen.findByRole("list", { name: "Studies" })).getByRole("button", { name: /The True Vine/ }));
    const findings = await screen.findByRole("list", { name: "Other notes" });
    const details = within(findings).queryByRole("button", { name: /^Details for/, expanded: false });
    if (details) await user.click(details);
    await user.click(within(findings).getByRole("button", { name: "+ Add source" }));
    await user.type(screen.getByLabelText("Search your library for the passage to cite"), "orchards{Enter}");
    const choices = await screen.findByRole("list", { name: "Passages to cite" });
    const hit = within(choices).getByRole("button", { name: /Cite A\. Shelved, Shelved Essay, p\. 4/ });
    expect(hit).toHaveTextContent("Archived");
  });

  it("keeps source details behind About this source, including a missing file", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    const garden = api.installFixtureSource({
      title: "Garden Notes",
      author: "Q. Reader",
      language: "English",
      format: "pdf",
      segments: [{ text: "A garden paragraph.", pageNumber: 1 }],
    });
    Object.assign(garden, {
      publisher: "Example Press",
      publicationYear: 1998,
      edition: "Second",
      isbn: "9780000000001",
      doi: "10.1000/example",
      licence: "Test licence",
      subtitle: "On tending",
    });
    const collection = await api.libraryCollectionCreate("Gardens", null);
    await api.libraryCollect(collection.id, [garden.id], true);
    await api.libraryRetag([garden.id], ["tending"], []);
    const lost = api.installFixtureSource({
      title: "Lost Pages",
      author: "M. Missing",
      segments: [{ text: "The pages are gone.", pageNumber: 2 }],
      filePresent: false,
    });
    await api.librarySetState([lost.id], "archived");
    const { user } = await renderApp(api);
    await openLibrary(user);

    await user.click(screen.getByRole("button", { name: /^Garden Notes/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Garden Notes" })).toBeInTheDocument();
    expect(screen.getByText(/Book · PDF · 1 page · English · 1998/)).toBeInTheDocument();
    const about = screen.getByText("About this source").closest("details");
    expect(about).not.toBeNull();
    expect(about).not.toHaveAttribute("open");
    expect(about).toHaveTextContent("Example Press");
    expect(about).toHaveTextContent("Second");
    expect(about).toHaveTextContent("On tending");
    expect(about).toHaveTextContent("9780000000001");
    expect(about).toHaveTextContent("10.1000/example");
    expect(about).toHaveTextContent("Gardens");
    expect(about).toHaveTextContent("tending");
    expect(about).toHaveTextContent("Test licence");
    expect(about).toHaveTextContent("Abiding Word’s copy is on this computer.");
    await waitFor(() => expect(about).toHaveTextContent("Q. Reader, Garden Notes, p. 1"));
    await user.click(screen.getByText("About this source"));
    expect(about).toHaveAttribute("open");

    await user.click(screen.getByRole("button", { name: "Back" }));
    await user.click(await screen.findByRole("button", { name: /^Lost Pages/ }));
    expect(await screen.findByRole("heading", { level: 1, name: "Lost Pages" })).toBeInTheDocument();
    const bar = screen.getByText("About this source").closest("div");
    expect(bar?.querySelector("span.rounded-full")).toHaveTextContent("Archived");
    expect(screen.getByText("Archived. It stays out of the Sources pane until you restore it in Settings.")).toBeInTheDocument();
    expect(screen.getAllByText("Abiding Word’s copy of this file can’t be found. The record of this source is kept.").length).toBeGreaterThan(0);
  });
});
