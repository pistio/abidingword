import { useState } from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { ApiProvider } from "@/lib/ApiContext";
import { FieldsEditor } from "@/features/studies/FieldsEditor";
import { createMemoryApi } from "../support/memoryApi";
import type { BibleTranslation, ScriptureRef } from "@/types/scripture";
import type { FieldKind, StudySection } from "@/types/domain";

async function setup(kind: FieldKind = "search", translationId = "source") {
  const api = createMemoryApi();
  vi.spyOn(api, "bibleList").mockResolvedValue([{ id: "source", name: "Source Bible", shortName: "SRC", kind: "structured" }] as BibleTranslation[]);
  const reference: ScriptureRef = { canonical: "JHN.15.1-JHN.15.3", display: "John 15:1–3", book: "JHN", startChapter: 15, startVerse: 1, endChapter: 15, endVerse: 3 };
  vi.spyOn(api, "scriptureSearch").mockResolvedValue({ hits: [{ translationId: "source", translationShortName: "SRC", reference: { ...reference, canonical: "JHN.15.1" }, text: "Exact source words" }], truncated: false });
  vi.spyOn(api, "referenceParse").mockResolvedValue(reference);
  vi.spyOn(api, "scriptureGetPassage").mockResolvedValue({ translationId: "source", reference, verses: [1, 3].map(verse => ({ canonical: `JHN.15.${verse}`, chapter: 15, verse, text: `Source verse ${verse}` })) });
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Review fixtures" });
  const template = await api.templateCreate({ name: "Research", studyType: "custom", sections: [{ key: "research", title: "Research", fieldsVersion: 1, fields: [{ key: "tool", kind, config: { query: "John 15:1–3", translationId } }] }] });
  const study = await api.studyCreate({ projectId: project.id, title: "Review", templateId: template.id });
  const errors: string[] = [];
  function Harness() {
    const [section, setSection] = useState<StudySection>(study.sections[0]!);
    return <FieldsEditor section={section} onChange={setSection} onError={error => { if (error) errors.push(error); }} />;
  }
  render(<ApiProvider api={api}><Harness /></ApiProvider>);
  const user = userEvent.setup();
  return { api, user, errors, study };
}

it("keeps quotation and provenance in one create without a fallible follow-up update", async () => {
  const { api, user } = await setup();
  const update = vi.spyOn(api, "studyBlockUpdate").mockRejectedValue(new Error("Update unavailable"));
  await waitFor(() => expect(screen.getByRole("button", { name: "Look up" })).toBeEnabled());
  await user.click(screen.getByRole("button", { name: "Look up" }));
  await user.click(await screen.findByRole("button", { name: "Keep as evidence" }));
  await waitFor(() => expect(api.state.blocks).toHaveLength(1));
  expect(api.state.blocks[0]?.content).toContain("Exact source words");
  expect(JSON.parse(api.state.blocks[0]!.data!)).toMatchObject({ translationId: "source", query: "John 15:1–3", text: "Exact source words" });
  expect(update).not.toHaveBeenCalled();
});

it("retries a failed keep without empty or duplicate quotation blocks", async () => {
  const { api, user, errors } = await setup();
  vi.spyOn(api, "studyBlockCreate").mockRejectedValueOnce(new Error("Disk full"));
  await waitFor(() => expect(screen.getByRole("button", { name: "Look up" })).toBeEnabled());
  await user.click(screen.getByRole("button", { name: "Look up" }));
  await user.click(await screen.findByRole("button", { name: "Keep as evidence" }));
  await waitFor(() => expect(errors).toHaveLength(1));
  expect(api.state.blocks).toHaveLength(0);
  await user.click(screen.getByRole("button", { name: "Keep as evidence" }));
  await waitFor(() => expect(api.state.blocks[0]?.data).toBeTruthy());
  expect(api.state.blocks).toHaveLength(1);
});

for (const kind of ["passage", "cross_references"] as const) {
  it(`cites only returned verses for a partially available ${kind}`, async () => {
    const { api, user } = await setup(kind);
    await waitFor(() => expect(screen.getByRole("button", { name: "Look up" })).toBeEnabled());
    await user.click(screen.getByRole("button", { name: "Look up" }));
    const keep = await screen.findAllByRole("button", { name: "Keep as evidence" });
    expect(keep).toHaveLength(2);
    await user.click(keep[1]!);
    await waitFor(() => expect(api.state.blocks[0]?.data).toBeTruthy());
    expect(JSON.parse(api.state.blocks[0]!.data!)).toMatchObject({ reference: "JHN.15.3", query: "John 15:1–3", text: "3 Source verse 3" });
    expect(api.state.blocks[0]?.content).not.toContain("JHN.15.1-JHN.15.3");
    expect(screen.getByText(/Only the verses available/)).toBeInTheDocument();
  });
}

it("does not silently filter a search response from a different edition", async () => {
  const { api, user, errors } = await setup();
  vi.mocked(api.scriptureSearch).mockResolvedValue({ hits: [{ reference: { canonical: "JHN.15.1" } as ScriptureRef, translationId: "different", translationShortName: "OTHER", text: "Wrong wording" }], truncated: false });
  await waitFor(() => expect(screen.getByRole("button", { name: "Look up" })).toBeEnabled());
  await user.click(screen.getByRole("button", { name: "Look up" }));
  await waitFor(() => expect(errors).toHaveLength(1));
  expect(screen.queryByRole("button", { name: "Keep as evidence" })).toBeNull();
  expect(api.state.blocks).toHaveLength(0);
});

it("does not substitute an installed edition for an unavailable configured source", async () => {
  const { api } = await setup("search", "removed-source");
  await screen.findByRole("option", { name: "Source Bible (SRC)" });
  expect(screen.getByRole("button", { name: "Look up" })).toBeDisabled();
  expect(api.scriptureSearch).not.toHaveBeenCalled();
});
