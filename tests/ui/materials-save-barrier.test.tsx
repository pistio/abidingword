import { act, fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject } from "../support/render";

async function writingSession() {
  const api = createMemoryApi();
  api.installFixtureBible("FX", "alpha");
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Invented materials" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Invented Study" });
  const prep = await api.teachingCreate({ studyId: study.id, title: "Invented Teaching", kind: "bible_study", templateVersion: 2, blocks: [
    { blockType: "qa", title: "Earlier question", participantFacing: true },
    { blockType: "teaching_notes", title: "Private leader notes", participantFacing: false },
  ] });
  await api.teachingBlockUpdate({ id: prep.blocks![0]!.id, body: "Earlier answer" });
  await api.teachingBlockUpdate({ id: prep.blocks![1]!.id, body: "PRIVATE leader writing" });
  const { user } = await renderAppInProject(api);
  await user.click(within(await screen.findByRole("list", { name: "Studies" })).getByRole("button", { name: /Invented Study/ }));
  const pane = screen.getByRole("main", { name: "Study" });
  await within(pane).findByRole("textbox", { name: "Study title" });
  await user.click(within(pane).getByRole("button", { name: "Bible Teaching" }));
  await within(pane).findByRole("textbox", { name: "Question" });
  return { api, user, prep, pane };
}

function deferred() {
  let resolve!: () => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<void>((done, fail) => { resolve = done; reject = fail; });
  return { promise, resolve, reject };
}

describe("current author writing before materials", () => {
  it("waits for a pending real Teaching Q&A save before allowing preview, Markdown or Word export", async () => {
    const { api, user, pane } = await writingSession();
    const gate = deferred();
    const write = api.teachingBlockUpdate;
    const saving = vi.spyOn(api, "teachingBlockUpdate").mockImplementation(async (input) => {
      await gate.promise;
      return write(input);
    });
    const question = within(pane).getByRole("textbox", { name: "Question" });
    const answer = within(pane).getByRole("textbox", { name: "Answer" });
    fireEvent.change(question, { target: { value: "CURRENT QUESTION written just before export" } });
    fireEvent.blur(question);
    fireEvent.change(answer, { target: { value: "CURRENT ANSWER written just before export" } });
    fireEvent.blur(answer);
    await waitFor(() => expect(saving).toHaveBeenCalledTimes(2));
    await user.click(within(pane).getByRole("button", { name: "Materials" }));
    await user.click(screen.getByRole("menuitem", { name: "Question sheet" }));
    try {
      expect(within(pane).queryByRole("article", { name: "Preview: Question sheet" })).not.toBeInTheDocument();
      expect(within(pane).queryByRole("button", { name: "Save as a text file…" })).not.toBeInTheDocument();
      expect(api.state.presentation.materialSaves).toHaveLength(0);
      expect(api.state.presentation.materialDocx).toHaveLength(0);
    } finally {
      await act(async () => { gate.resolve(); await gate.promise; });
    }
    const preview = await within(pane).findByRole("article", { name: "Preview: Question sheet" });
    expect(preview).toHaveTextContent("CURRENT QUESTION written just before export");
    expect(preview).not.toHaveTextContent("Earlier question");
    await user.click(within(pane).getByRole("checkbox", { name: "With the answers (shared questions only)" }));
    expect(preview).toHaveTextContent("CURRENT ANSWER written just before export");
    await user.click(within(pane).getByRole("button", { name: "Save as a text file…" }));
    await waitFor(() => expect(api.state.presentation.materialSaves).toHaveLength(1));
    expect(api.state.presentation.materialSaves[0]!.text).toContain("CURRENT QUESTION");
    expect(api.state.presentation.materialSaves[0]!.text).toContain("CURRENT ANSWER");
    expect(api.state.presentation.materialSaves[0]!.text).not.toContain("PRIVATE");
    await user.click(within(pane).getByRole("button", { name: "Save as a Word document…" }));
    await waitFor(() => expect(api.state.presentation.materialDocx).toHaveLength(1));
    expect(JSON.stringify(api.state.presentation.materialDocx[0]!.document)).toContain("CURRENT ANSWER");
    expect(JSON.stringify(api.state.presentation.materialDocx[0]!.document)).not.toContain("PRIVATE");
  });

  it("keeps failed writing editable and blocks a stale material until the author successfully retries", async () => {
    const { api, user, pane } = await writingSession();
    const gate = deferred();
    const write = api.teachingBlockUpdate;
    const saving = vi.spyOn(api, "teachingBlockUpdate").mockImplementation(async (input) => {
      await gate.promise;
      return write(input);
    });
    const question = within(pane).getByRole("textbox", { name: "Question" });
    fireEvent.change(question, { target: { value: "UNSAVED QUESTION remains in editor" } });
    fireEvent.blur(question);
    await waitFor(() => expect(saving).toHaveBeenCalledTimes(1));
    await user.click(within(pane).getByRole("button", { name: "Materials" }));
    await user.click(screen.getByRole("menuitem", { name: "Question sheet" }));
    await act(async () => { gate.reject(new Error("Invented material save failure")); });
    await waitFor(() => expect(within(pane).getAllByRole("alert").length).toBeGreaterThan(0));
    expect(within(pane).queryByRole("article", { name: "Preview: Question sheet" })).not.toBeInTheDocument();
    expect(within(pane).getByRole("textbox", { name: "Question" })).toHaveValue("UNSAVED QUESTION remains in editor");
    expect(api.state.presentation.materialSaves).toHaveLength(0);
    saving.mockImplementation(write);
    await user.click(within(pane).getByRole("button", { name: "Materials" }));
    await user.click(screen.getByRole("menuitem", { name: "Question sheet" }));
    const preview = await within(pane).findByRole("article", { name: "Preview: Question sheet" });
    expect(preview).toHaveTextContent("UNSAVED QUESTION remains in editor");
    expect(preview).not.toHaveTextContent("Earlier question");
  });
});
