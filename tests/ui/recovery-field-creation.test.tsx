import { useState } from "react";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import { FieldsEditor } from "@/features/studies/FieldsEditor";
import { SaveTrackerProvider, runDatabaseRestore, settleOutstandingWrites } from "@/hooks/SaveTracker";
import { ApiProvider } from "@/lib/ApiContext";
import type { StudySection } from "@/types/domain";
import { createMemoryApi } from "../support/memoryApi";

async function missingHistoricalField() {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Invented recovery project" });
  const template = await api.templateCreate({ name: "Invented writing template", studyType: "custom", sections: [{ key: "part", title: "Invented study part", fields: [{ key: "writing", kind: "writing", label: "Invented Study writing" }], fieldsVersion: 1 }] });
  const study = await api.studyCreate({ projectId: project.id, title: "Invented Study", templateId: template.id });
  for (const block of study.sections[0]!.blocks) await api.studyBlockDelete(block.id);
  const section = (await api.studySectionList(study.study.id))[0]!;
  return { api, section, studyId: study.study.id };
}

function Editor({ initial }: { initial: StudySection }) {
  const [section, setSection] = useState(initial);
  const [error, setError] = useState<string | null>(null);
  return <><FieldsEditor section={section} onChange={setSection} onError={setError} />{error && <p role="alert">{error}</p>}</>;
}

it("retries a failed first field-block creation without losing the current writing", async () => {
  const { api, section, studyId } = await missingHistoricalField();
  const original = api.studyBlockCreate;
  vi.spyOn(api, "studyBlockCreate").mockRejectedValueOnce(new Error("Invented first creation failure")).mockImplementation(original);
  render(<ApiProvider api={api}><SaveTrackerProvider><Editor initial={section} /></SaveTrackerProvider></ApiProvider>);
  fireEvent.change(screen.getByLabelText("Invented Study writing"), { target: { value: "Invented unsaved field writing" } });
  const replace = vi.fn(async () => undefined);
  await act(async () => { await expect(runDatabaseRestore(replace)).rejects.toMatchObject({ kind: "conflict" }); });
  expect(replace).not.toHaveBeenCalled();
  expect(screen.getByLabelText("Invented Study writing")).toHaveValue("Invented unsaved field writing");
  fireEvent.blur(screen.getByLabelText("Invented Study writing"));
  await act(async () => { await expect(settleOutstandingWrites()).resolves.toBeUndefined(); });
  const saved = (await api.studySectionList(studyId))[0]!;
  expect(saved.blocks).toHaveLength(1);
  expect(saved.blocks[0]?.content).toBe("Invented unsaved field writing");
  expect(screen.queryByRole("alert")).not.toBeInTheDocument();
});

it("reuses a field block created by the previous mount before saving newer writing", async () => {
  const { api, section, studyId } = await missingHistoricalField();
  let release!: () => void;
  const wait = new Promise<void>(resolve => { release = resolve; });
  const original = api.studyBlockCreate;
  const create = vi.spyOn(api, "studyBlockCreate").mockImplementation(async input => { await wait; return original(input); });
  const mount = () => render(<ApiProvider api={api}><SaveTrackerProvider><Editor initial={section} /></SaveTrackerProvider></ApiProvider>);
  const first = mount();
  fireEvent.change(screen.getByLabelText("Invented Study writing"), { target: { value: "Invented first revision" } });
  fireEvent.blur(screen.getByLabelText("Invented Study writing"));
  await waitFor(() => expect(create).toHaveBeenCalledOnce());
  first.unmount();
  mount();
  expect(screen.getByLabelText("Invented Study writing")).toHaveValue("Invented first revision");
  fireEvent.change(screen.getByLabelText("Invented Study writing"), { target: { value: "Invented newer revision" } });
  fireEvent.blur(screen.getByLabelText("Invented Study writing"));
  await act(async () => { release(); await settleOutstandingWrites(); });
  const saved = (await api.studySectionList(studyId))[0]!;
  expect(create).toHaveBeenCalledOnce();
  expect(saved.blocks).toHaveLength(1);
  expect(saved.blocks[0]?.content).toBe("Invented newer revision");
});
