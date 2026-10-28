import { render, screen, within, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { useState } from "react";
import { BlockBuilder } from "@/features/settings/BlockBuilder";
import { FieldsEditor } from "@/features/studies/FieldsEditor";
import { ApiProvider } from "@/lib/ApiContext";
import type { TemplateSection } from "@/types/studies";
import type { StudySection } from "@/types/domain";
import { createMemoryApi } from "../support/memoryApi";

it("adds, configures, duplicates, reorders and removes blocks without mutating the original template", async () => {
  const original: TemplateSection = { key: "part", title: "Part", fields: [{ key: "answer", kind: "writing" }] };
  let saved = original;
  function Harness() {
    const [section, setSection] = useState(original);
    return <BlockBuilder section={section} onChange={patch => { saved = { ...section, ...patch }; setSection(saved); }} />;
  }
  render(<Harness />);
  const user = userEvent.setup();
  await user.click(screen.getByText("Template blocks"));
  await user.selectOptions(screen.getByLabelText("Block type"), "concordance");
  await user.click(screen.getByRole("button", { name: "Add block" }));
  const rows = screen.getAllByRole("listitem");
  await user.type(within(rows[1]!).getByLabelText("Block title"), " in John");
  await user.type(within(rows[1]!).getByLabelText("Search text or passage"), "abide");
  await user.click(within(rows[1]!).getByRole("button", { name: "Duplicate block" }));
  expect(saved.fields).toHaveLength(3);
  expect(new Set(saved.fields!.map(field => field.key)).size).toBe(3);
  await user.click(within(screen.getAllByRole("listitem")[2]!).getByRole("button", { name: "Move up" }));
  await user.click(within(screen.getAllByRole("listitem")[0]!).getByRole("button", { name: "Delete" }));
  expect(saved.fields).toHaveLength(2);
  expect(saved.fields?.every(field => field.config?.query === "abide")).toBe(true);
  expect(original.fields).toEqual([{ key: "answer", kind: "writing" }]);
});

it("keeps source provenance from the result even after the query changes and retains evidence on reopen", async () => {
  const api = createMemoryApi();
  vi.spyOn(api, "bibleList").mockResolvedValue([{ id: "edition-a", name: "Source edition", shortName: "SRC", kind: "structured" }] as Awaited<ReturnType<typeof api.bibleList>>);
  const search = vi.spyOn(api, "scriptureSearch").mockResolvedValue({ hits: [{ reference: { canonical: "JHN 15:5", display: "John 15:5" } as never, text: "I am the vine", translationId: "edition-a", translationShortName: "SRC" }], truncated: false });
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Tools" });
  const template = await api.templateCreate({ name: "My tools", studyType: "custom", sections: [{ key: "part", title: "Research", fields: [{ key: "evidence", kind: "search", label: "My research", config: { query: "vine", translationId: "edition-a" } }], fieldsVersion: 1 }] });
  const study = await api.studyCreate({ projectId: project.id, title: "Vine", templateId: template.id });
  let saved: StudySection = study.sections[0]!;
  function Harness() {
    const [section, setSection] = useState(saved);
    return <FieldsEditor section={section} onChange={next => { saved = next; setSection(next); }} onError={error => { if (error) throw Error(error); }} />;
  }
  const rendered = render(<ApiProvider api={api}><Harness /></ApiProvider>);
  const user = userEvent.setup();
  await waitFor(() => expect(screen.getByRole("button", { name: "Look up" })).toBeEnabled());
  await user.click(screen.getByRole("button", { name: "Look up" }));
  await screen.findByText(/JHN 15:5/);
  await user.clear(screen.getByLabelText("Search text or passage"));
  await user.type(screen.getByLabelText("Search text or passage"), "another query");
  await user.click(screen.getByRole("button", { name: "Keep as evidence" }));
  await waitFor(() => expect(saved.blocks).toHaveLength(1));
  await waitFor(() => expect(saved.blocks[0]?.data).toBeTruthy());
  expect(JSON.parse(saved.blocks[0]!.data!)).toMatchObject({ query: "vine", translationId: "edition-a", reference: "JHN 15:5", text: "I am the vine" });
  expect(search).toHaveBeenCalledWith({ query: "vine", translationId: "edition-a" });
  rendered.unmount();
  saved = (await api.studySectionList(study.study.id))[0]!;
  render(<ApiProvider api={api}><Harness /></ApiProvider>);
  expect(screen.getByText(/I am the vine/)).toBeInTheDocument();
});
