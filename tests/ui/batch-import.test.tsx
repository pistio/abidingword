import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createMemoryApi } from "../support/memoryApi";
import { openSettingsSection, renderApp } from "../support/render";

// Adding many sources at once (v0.24): a folder read and checked, reviewed on
// one screen, corrected, added with a shared collection and tags; duplicates,
// problems, retry, and an unfinished import continued.

const FOLDER = "/home/you/Books";
const FILES = ["Romans-Commentary.pdf", "grace.md", "grace-copy.md", "old-scanned.pdf", "picture.png", "sub/hope.epub"].map((f) => `${FOLDER}/${f}`);

async function setup() {
  const api = createMemoryApi();
  api.state.batch.nextFolder = { path: FOLDER, files: FILES };
  const rendered = await renderApp(api);
  const section = await openSettingsSection(rendered.user, "Sources");
  return { ...rendered, api, section };
}

async function readFolder(user: Awaited<ReturnType<typeof renderApp>>["user"], section: HTMLElement, recursive = true) {
  await user.click(within(section).getByRole("button", { name: "Add many…" }));
  const dialog = screen.getByRole("dialog", { name: "Add many sources" });
  if (!recursive) await user.click(within(dialog).getByRole("checkbox", { name: "Include files in subfolders" }));
  await user.click(within(dialog).getByRole("button", { name: "Choose a folder…" }));
  await within(dialog).findByRole("list", { name: "Files found" });
  return dialog;
}

describe("Adding many sources", () => {
  it("reads a folder, shows one review, and adds only what is chosen, with a collection and tags", async () => {
    const { api, user, section } = await setup();
    const dialog = await readFolder(user, section);
    expect(within(dialog).getByText(/5 files found · 1 already in your library or listed twice · 1 cannot be added · 1 file of another kind left out/)).toBeInTheDocument();
    const files = within(dialog).getByRole("list", { name: "Files found" });
    // Category from evidence, and said so.
    const romans = within(files).getByText("Romans-Commentary.pdf").closest("li")!;
    expect(within(romans).getByRole("combobox", { name: "Category" })).toHaveDisplayValue("Commentary");
    expect(within(romans).getByText("Suggested from a word in its file name")).toBeInTheDocument();
    const hope = within(files).getByText("hope.epub").closest("li")!;
    expect(within(hope).getByText("The usual category for this kind of file: check it")).toBeInTheDocument();
    // The duplicate is not chosen; the scanned PDF cannot be.
    expect(within(files).getByRole("checkbox", { name: "Add grace copy" })).not.toBeChecked();
    expect(within(files).getByText(/The same file as “grace” in this list/)).toBeInTheDocument();
    const scanned = within(files).getByText("old-scanned.pdf").closest("li")!;
    expect(within(scanned).queryByRole("checkbox")).not.toBeInTheDocument();
    expect(within(scanned).getByText(/scanned pages/)).toBeInTheDocument();
    // Nothing is added yet.
    expect(api.state.library.sources).toHaveLength(0);

    // Corrections.
    const title = within(romans).getByRole("textbox", { name: "Title" });
    await user.clear(title);
    await user.type(title, "Romans: A Commentary");
    await user.tab();
    await user.click(within(files).getByRole("checkbox", { name: "Add hope" }));
    await user.selectOptions(within(dialog).getByRole("combobox", { name: "Collection" }), "new");
    await user.type(within(dialog).getByRole("textbox", { name: "Name of the new collection" }), "Paul");
    await user.type(within(dialog).getByRole("textbox", { name: "Add tags (separate with commas)" }), "Imported, Grace");
    await user.click(within(dialog).getByRole("button", { name: "Add 2 sources" }));
    expect(await within(dialog).findByText("2 sources were added.")).toBeInTheDocument();
    expect(api.state.library.sources.map((s) => s.title).sort()).toEqual(["Romans: A Commentary", "grace"]);
    const paul = api.state.manage.collections.find((c) => c.name === "Paul")!;
    expect(api.state.manage.members.get(paul.id)?.size).toBe(2);
    expect(api.state.manage.tags.map((x) => x.name)).toEqual(["Imported", "Grace"]);
    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(within(section).getByText("2 in use · 0 archived · 0 in Trash")).toBeInTheDocument());
  });

  it("leaves subfolders out when asked", async () => {
    const { user, section } = await setup();
    const dialog = await readFolder(user, section, false);
    expect(within(dialog).queryByText("hope.epub")).not.toBeInTheDocument();
    expect(within(dialog).getByText(/^4 files found/)).toBeInTheDocument();
  });

  it("a file that changed is not added, and can be read again and added", async () => {
    const { api, user, section } = await setup();
    api.state.batch.nextFolder = { path: FOLDER, files: [`${FOLDER}/grace.md`] };
    const dialog = await readFolder(user, section);
    api.state.batch.changed.add(`${FOLDER}/grace.md`);
    await user.click(within(dialog).getByRole("button", { name: "Add 1 source" }));
    expect(await within(dialog).findByText("0 sources were added.")).toBeInTheDocument();
    expect(within(dialog).getByRole("list", { name: "Not added" })).toHaveTextContent("changed after it was checked");
    await user.click(within(dialog).getByRole("button", { name: "Try again" }));
    await user.click(await within(dialog).findByRole("button", { name: "Add 1 source" }));
    expect(await within(dialog).findByText("1 source was added.")).toBeInTheDocument();
    expect(api.state.library.sources).toHaveLength(1);
  });

  it("offers to continue an unfinished import, or discard it", async () => {
    const api = createMemoryApi();
    const job = await api.libraryBatchScan([`${FOLDER}/grace.md`, `${FOLDER}/hope.epub`], false);
    await api.libraryBatchStep(job.id);
    await api.libraryBatchStep(job.id);
    await api.libraryBatchImportStep(job.id);
    api.state.batch.jobs.get(job.id)!.state = "interrupted";
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Sources");
    expect(within(section).getByText("An import of many sources was not finished (1 of 2 added).")).toBeInTheDocument();
    await user.click(within(section).getByRole("button", { name: "Continue" }));
    const dialog = await screen.findByRole("dialog", { name: "Add many sources" });
    expect(await within(dialog).findByText(/This import stopped before it was finished \(1 added\)/)).toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "Add 1 source" }));
    expect(await within(dialog).findByText("2 sources were added.")).toBeInTheDocument();
    expect(api.state.library.sources).toHaveLength(2);
    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    await waitFor(() => expect(within(section).queryByText(/was not finished/)).not.toBeInTheDocument());
  });
});
