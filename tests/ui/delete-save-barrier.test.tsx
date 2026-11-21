import { useState } from "react";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BlockEditor, type BlockHandle } from "@/features/studies/BlockEditor";
import { QaEditor } from "@/features/studies/QaEditor";
import { FieldsEditor } from "@/features/studies/FieldsEditor";
import { BlockCanvas } from "@/features/teaching/BlockCanvas";
import { ScriptureProvider } from "@/features/scripture/ScriptureContext";
import { SaveTrackerProvider, runDatabaseRestore, settleOutstandingWrites } from "@/hooks/SaveTracker";
import { ApiProvider } from "@/lib/ApiContext";
import { createMemoryApi } from "../support/memoryApi";
import type { StudyBlock, StudySection } from "@/types/domain";
import { qaData } from "@/features/studies/qa";

type Kind = "Study paragraph" | "Study Q&A" | "Study entry" | "Teaching block";
const kinds: Kind[] = ["Study paragraph", "Study Q&A", "Study entry", "Teaching block"];

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>(done => { resolve = done; });
  return { promise, resolve };
}

async function mountEditor(kind: Kind) {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Invented deletion project" });
  const template = await api.templateCreate({ name: "Invented deletion template", studyType: "custom", sections: [{ key: "part", title: "Invented part", fieldsVersion: 1, fields: kind === "Study entry" ? [{ key: "entries", kind: "entries", label: "Invented entries", parts: [{ key: "detail", kind: "text" }] }] : [] }] });
  const detail = await api.studyCreate({ projectId: project.id, title: "Invented Study", templateId: template.id });
  const section = detail.sections[0]!;
  const block = kind === "Teaching block" ? null : await api.studyBlockCreate({ sectionId: section.id, blockType: kind === "Study Q&A" ? "question" : "paragraph", content: kind === "Study Q&A" ? undefined : "Earlier writing", fieldKey: kind === "Study entry" ? "entries" : undefined, data: kind === "Study Q&A" ? qaData({ question: "Earlier question", answer: "Earlier answer" }) : kind === "Study entry" ? JSON.stringify({ detail: "Earlier writing" }) : undefined });
  const prep = kind !== "Teaching block" ? null : await api.teachingCreate({ studyId: detail.study.id, title: "Invented Teaching", templateVersion: 2, blocks: [{ blockType: "teaching_notes", title: "Invented leader block", participantFacing: false }] });
  if (prep) await api.teachingBlockUpdate({ id: prep.blocks![0]!.id, body: "Earlier private writing" });
  let registered: BlockHandle | null = null;
  function Harness() {
    const [current, setCurrent] = useState<StudyBlock | null>(block);
    const [part, setPart] = useState<StudySection>({ ...section, blocks: block ? [block] : [] });
    const [teaching, setTeaching] = useState(prep);
    const [error, setError] = useState<string | null>(null);
    return <>
      {kind === "Study paragraph" && current && <BlockEditor register={(_id, handle) => { registered = handle; }} block={current} sectionTitle="Invented part" placeholder="Invented writing" isFirst isLast onDeleted={() => setCurrent(null)} onRevealScripture={() => {}} onMove={() => {}} />}
      {kind === "Study Q&A" && current && <QaEditor block={current} number={1} isFirst isLast onMove={() => {}} onDeleted={() => setCurrent(null)} onChange={setCurrent} />}
      {kind === "Study entry" && <FieldsEditor section={part} onChange={setPart} onError={setError} />}
      {teaching && <BlockCanvas prep={teaching} findings={[]} assistant={null} onChanged={setTeaching} onRevealScripture={() => {}} />}
      {error && <p role="alert">{error}</p>}
    </>;
  }
  const view = render(<ApiProvider api={api}><ScriptureProvider settings={api.state.settings} books={[]} translations={[]}><SaveTrackerProvider><Harness /></SaveTrackerProvider></ScriptureProvider></ApiProvider>);
  const field = screen.getAllByRole("textbox")[0]!;
  const id = block?.id ?? prep!.blocks![0]!.id;
  return { api, project, field, id, view, handle: () => registered };
}

function confirmDelete(kind: Kind) {
  if (kind !== "Study entry") fireEvent.click(screen.getByRole("button", { name: /options/i }));
  fireEvent.click(screen.getByRole("button", { name: kind === "Teaching block" ? /^remove/i : /^delete/i }));
  const dialog = screen.queryByRole("alertdialog");
  const confirm = dialog ? within(dialog).getByRole("button", { name: "Delete" }) : screen.getByRole("button", { name: "Delete" });
  fireEvent.click(confirm);
  return confirm;
}

for (const kind of kinds) describe(`${kind} deletion settles current writing`, () => {
  it("waits for pending writing, then permits later backup/export/recovery settlement", async () => {
    const { api, field, id, project } = await mountEditor(kind);
    const gate = deferred();
    const update = kind === "Teaching block" ? api.teachingBlockUpdate : api.studyBlockUpdate;
    const save = kind === "Teaching block"
      ? vi.spyOn(api, "teachingBlockUpdate").mockImplementation(async input => { await gate.promise; return (update as typeof api.teachingBlockUpdate)(input); })
      : vi.spyOn(api, "studyBlockUpdate").mockImplementation(async input => { await gate.promise; return (update as typeof api.studyBlockUpdate)(input); });
    const remove = kind === "Teaching block" ? vi.spyOn(api, "teachingBlockDelete") : vi.spyOn(api, "studyBlockDelete");
    fireEvent.change(field, { target: { value: "Invented pending authoritative writing" } });
    fireEvent.blur(field);
    await waitFor(() => expect(save).toHaveBeenCalledOnce());
    confirmDelete(kind);
    try {
      expect(remove).not.toHaveBeenCalled();
      expect(field.closest("[inert]")).not.toBeNull();
      fireEvent.change(field, { target: { value: "Late input must not enter the deleting record" } });
      expect(field).toHaveValue("Invented pending authoritative writing");
    } finally {
      await act(async () => { gate.resolve(); await gate.promise; });
    }
    await waitFor(() => expect(remove).toHaveBeenCalledWith(id));
    await waitFor(() => expect(field).not.toBeInTheDocument());
    await expect(settleOutstandingWrites()).resolves.toBeUndefined();
    await expect(api.backupCreate("invented-after-delete.awbackup")).resolves.toBeDefined();
    await expect(api.projectExport({ projectId: project.id, path: "invented-after-delete.awproject" })).resolves.toBeDefined();
    const replace = vi.fn(async () => undefined);
    await runDatabaseRestore(replace);
    expect(replace).toHaveBeenCalledOnce();
  });

  it("preserves the record and visible draft when saving fails, then allows an explicit retry", async () => {
    const { api, field, id } = await mountEditor(kind);
    let failing = true;
    const update = kind === "Teaching block" ? api.teachingBlockUpdate : api.studyBlockUpdate;
    const save = kind === "Teaching block"
      ? vi.spyOn(api, "teachingBlockUpdate").mockImplementation(async input => { if (failing) throw new Error("Invented write failure"); return (update as typeof api.teachingBlockUpdate)(input); })
      : vi.spyOn(api, "studyBlockUpdate").mockImplementation(async input => { if (failing) throw new Error("Invented write failure"); return (update as typeof api.studyBlockUpdate)(input); });
    const remove = kind === "Teaching block" ? vi.spyOn(api, "teachingBlockDelete") : vi.spyOn(api, "studyBlockDelete");
    fireEvent.change(field, { target: { value: "Invented unsaved writing remains visible" } });
    fireEvent.blur(field);
    await waitFor(() => expect(save).toHaveBeenCalledOnce());
    confirmDelete(kind);
    await act(async () => { await Promise.resolve(); });
    expect(remove).not.toHaveBeenCalled();
    expect(field).toBeInTheDocument();
    expect(field).toHaveValue("Invented unsaved writing remains visible");
    expect(field.closest("[inert]")).toBeNull();
    if (kind === "Teaching block") expect(api.state.teaching.preparations.some(p => p.blocks?.some(b => b.id === id))).toBe(true);
    else expect(api.state.blocks.some(b => b.id === id)).toBe(true);
    failing = false;
    await act(async () => { await settleOutstandingWrites(); });
    // The still-open confirmation is an explicit new attempt after the error.
    const dialog = screen.queryByRole("alertdialog");
    if (dialog) fireEvent.click(within(dialog).getByRole("button", { name: "Delete" }));
    else {
      fireEvent.click(screen.getByRole("button", { name: kind === "Teaching block" ? /^remove/i : /^delete/i }));
      fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    }
    await waitFor(() => expect(remove).toHaveBeenCalledWith(id));
    await expect(settleOutstandingWrites()).resolves.toBeUndefined();
  });
});

it("keeps the editor frozen while native deletion is pending and refuses an external Help replacement", async () => {
  const { api, field, handle } = await mountEditor("Study paragraph");
  const gate = deferred();
  const original = api.studyBlockDelete;
  const remove = vi.spyOn(api, "studyBlockDelete").mockImplementation(async id => { await gate.promise; return original(id); });
  const retainedHandle = handle();
  const confirm = confirmDelete("Study paragraph");
  await waitFor(() => expect(remove).toHaveBeenCalledOnce());
  expect(field.closest("[inert]")).not.toBeNull();
  expect(retainedHandle?.replace("Earlier writing", "Late external suggestion")).toBe(false);
  expect(field).toHaveValue("Earlier writing");
  fireEvent.click(confirm);
  expect(remove).toHaveBeenCalledOnce();
  await act(async () => { gate.resolve(); await gate.promise; });
  await waitFor(() => expect(field).not.toBeInTheDocument());
  expect(retainedHandle?.replace("Earlier writing", "Suggestion after deletion")).toBe(false);
  await expect(settleOutstandingWrites()).resolves.toBeUndefined();
});
