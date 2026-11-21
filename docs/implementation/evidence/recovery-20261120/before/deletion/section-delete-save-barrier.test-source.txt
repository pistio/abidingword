import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import { settleOutstandingWrites } from "@/hooks/SaveTracker";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject } from "../support/render";

it("prevents fresh section writing after settlement until native deletion finishes", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Invented folder" });
  await api.studyCreate({ projectId: project.id, title: "Invented section Study", studyType: "custom" });
  let finish!: () => void;
  const pending = new Promise<void>(done => { finish = done; });
  const remove = api.studySectionDelete;
  const deletion = vi.spyOn(api, "studySectionDelete").mockImplementation(async input => { await pending; return remove(input); });
  const { user } = await renderAppInProject(api);
  await user.click(await screen.findByRole("button", { name: /Invented section Study/ }));
  const field = await screen.findByRole("textbox", { name: "Notes: paragraph" });
  fireEvent.change(field, { target: { value: "Latest invented section writing" } });
  await user.click(screen.getByRole("button", { name: "Options for part Notes" }));
  await user.click(screen.getByRole("button", { name: "Delete part Notes" }));
  const dialog = screen.getByRole("alertdialog", { name: "Delete part Notes?" });
  await user.click(within(dialog).getByRole("button", { name: "Delete part" }));
  await waitFor(() => expect(deletion).toHaveBeenCalled());
  try {
    expect(api.state.blocks.some(block => block.content === "Latest invented section writing")).toBe(true);
    await user.click(field);
    await user.keyboard(" late section draft{Escape}");
    expect(field).toHaveValue("Latest invented section writing");
    expect(dialog).toBeInTheDocument();
  } finally {
    await act(async () => { finish(); await pending; });
  }
  await waitFor(() => expect(field).not.toBeInTheDocument());
  expect(document.querySelector("[data-writing-operation]")).not.toBeInTheDocument();
  await expect(settleOutstandingWrites()).resolves.toBeUndefined();
});
