import { screen, waitFor, within } from "@testing-library/react";
import { beforeEach, expect, it } from "vitest";
import { writeLocal } from "@/lib/storage";
import { ApiError } from "@/lib/api";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";

beforeEach(() => window.localStorage.clear());

it("creates successive root documents in the same hidden container", async () => {
  const { api, user } = await renderApp();
  for (const reference of ["John 15", "Romans 8"]) {
    await user.click(await screen.findByRole("button", { name: "New study" }));
    await user.type(await screen.findByRole("textbox", { name: "Bible passage" }), reference);
    await user.click(screen.getByRole("button", { name: "Create study" }));
    await screen.findByRole("textbox", { name: "Study title" });
    await user.click(screen.getByRole("button", { name: "Back to Studies" }));
  }
  expect(api.state.projects).toHaveLength(1);
  expect(api.state.studies).toHaveLength(2);
  expect(api.state.studies.every(s => s.projectId === api.state.projects[0]!.id)).toBe(true);
});

it.each([1, 2, 3])("shows %i named folders without promoting their documents to root or choosing them for root creation", async count => {
  const api = createMemoryApi();
  for (let i = 0; i < count; i++) {
    const p = await api.projectCreate({ workspaceId: api.state.workspace.id, title: `Folder ${i}` });
    await api.studyCreate({ projectId: p.id, title: `Filed ${i}` });
    writeLocal("studyPane.lastProject", p.id);
  }
  const { user } = await renderApp(api);
  const list = await screen.findByRole("list", { name: "Studies" });
  for (let i = 0; i < count; i++) expect(within(list).getByRole("button", { name: `Folder ${i} Project folder` })).toBeVisible();
  expect(within(list).queryByText(/^Filed/)).toBeNull();
  await user.click(screen.getByRole("button", { name: "New study" }));
  await user.type(await screen.findByRole("textbox", { name: "Bible passage" }), "Romans 8");
  await user.click(screen.getByRole("button", { name: "Create study" }));
  await screen.findByRole("textbox", { name: "Study title" });
  expect(api.state.studies.at(-1)!.projectId).toBe((await api.projectRoot(api.state.workspace.id, "My studies")).id);
});

it("requires a folder name and preserves a native creation error without creating a duplicate", async () => {
  const { api, user } = await renderApp();
  await waitFor(() => expect(api.state.projects).toHaveLength(1));
  await user.click(screen.getByRole("button", { name: "New project" }));
  const dialog = await screen.findByRole("dialog", { name: "New project" });
  const save = within(dialog).getByRole("button", { name: "Save" });
  expect(save).toBeDisabled();
  await user.type(within(dialog).getByRole("textbox", { name: "Name" }), "Folder");
  api.projectCreate = async () => { throw new ApiError("database", "Abiding Word could not read or save your data."); };
  await user.click(save);
  expect(await within(dialog).findByRole("alert")).toHaveTextContent("could not read or save your data");
  expect(api.state.projects).toHaveLength(1);
});

it("renames a named folder without changing its identity or moving its documents", async () => {
  const api = createMemoryApi(); const p = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Old folder" });
  const { study } = await api.studyCreate({ projectId: p.id, title: "Retained document" });
  const { user } = await renderApp(api);
  const actions = await screen.findByRole("group", { name: "Actions for “Old folder”" });
  await user.click(within(actions).getByRole("button", { name: "File actions" }));
  await user.click(screen.getByRole("menuitem", { name: "Rename" }));
  const dialog = await screen.findByRole("dialog", { name: "Rename" });
  await user.clear(within(dialog).getByRole("textbox", { name: "Name" }));
  await user.type(within(dialog).getByRole("textbox", { name: "Name" }), "New folder");
  await user.click(within(dialog).getByRole("button", { name: "Save" }));
  expect(await screen.findByRole("button", { name: "New folder Project folder" })).toBeVisible();
  expect((await api.studyGet(study.id)).study.projectId).toBe(p.id);
  await waitFor(() => expect(api.state.projects.find(row => row.id === p.id)?.title).toBe("New folder"));
});
