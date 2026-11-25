import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { derive, toMarkdown } from "@/features/materials/derive";
import { MaterialPage } from "@/features/materials/MaterialView";
import type { TeachingBlock, TeachingPreparation } from "@/types/teaching";
import type { MaterialScripture } from "@/types/scripture";

// Invented text only. Missing blocks represent deleted blocks as returned by the API.
const block = (id: string, participantFacing = true): TeachingBlock => ({ id, parentId: null, position: 0, blockType: "scripture", blockKey: null, title: id, body: "", participantFacing, role: null, guidanceKey: null, pointId: null });
const reference = (verse: number) => ({ book: "JHN", startChapter: 15, startVerse: verse, endChapter: 15, endVerse: verse, canonical: `JHN.15.${verse}`, display: `John 15:${verse}` });
const outputPrep = (): TeachingPreparation => ({ id: "prep", projectId: "fixture-project", studyId: "study", title: "Invented teaching", kind: "sermon", audience: null, purpose: null, centralMessage: null, introduction: "", conclusion: "", notes: "", points: [], findings: [], passages: [], materials: [], templateVersion: 2, blocks: [], createdAt: "", updatedAt: "", revision: 1 });
const t = i18n.t.bind(i18n);

describe("Teaching output integrity", () => {
  it.each(["questions", "worksheet", "manuscript"] as const)("excludes private and unknown block references and text from %s", (kind) => {
    const prep = outputPrep();
    prep.blocks = [block("public"), block("private", false)];
    prep.passages = [null, "public", "private", "deleted"].map((blockId, i) => ({ id: `pass${i}`, pointId: null, blockId, reference: reference(i + 1) }));
    const scripture: MaterialScripture = { translationId: "fixture", shortName: "FX", name: "Fixture", copyrightNotice: null, passages: prep.passages.map((p, i) => ({ canonical: p.reference.canonical, missing: [], verses: [{ canonical: p.reference.canonical, chapter: 15, verse: i + 1, text: ["WIDE TEXT", "PUBLIC TEXT", "PRIVATE TEXT", "DELETED TEXT"][i]! }] })) };
    const doc = derive(t, kind, prep, { includePassage: true }, scripture);
    const text = toMarkdown(doc);
    expect(text).toContain("John 15:1"); expect(text).toContain("WIDE TEXT");
    expect(text).toContain("John 15:2"); expect(text).toContain("PUBLIC TEXT");
    expect(text).not.toContain("John 15:3"); expect(text).not.toContain("PRIVATE TEXT");
    expect(text).not.toContain("John 15:4"); expect(text).not.toContain("DELETED TEXT");
    const leader = toMarkdown(derive(t, kind, prep, { includePassage: true, leader: true, includeNotes: true }, scripture));
    if (kind !== "worksheet") { expect(leader).toContain("PRIVATE TEXT"); expect(leader).toContain("John 15:3"); }
    expect(leader).not.toContain("DELETED TEXT");
  });

  it("keeps outline heading before body in preview, print model, Markdown and DOCX model", () => {
    const prep = outputPrep();
    prep.blocks = [{ ...block("point"), blockType: "outline_point", title: "A unique heading", body: "A unique paragraph" }, { ...block("writing"), blockType: "writing", title: "Decorative label", body: "Written words" }];
    const doc = derive(t, "manuscript", prep);
    expect(doc.blocks.slice(1, 3)).toEqual([{ type: "heading", text: "A unique heading" }, { type: "text", text: "A unique paragraph" }]);
    expect(toMarkdown(doc)).toMatch(/## A unique heading[\s\S]*A unique paragraph/);
    expect(toMarkdown(doc)).not.toContain("Decorative label");
    render(<MaterialPage doc={doc} />);
    expect(screen.getByRole("heading", { name: "A unique heading" }).compareDocumentPosition(screen.getByText("A unique paragraph")) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
    // materialSaveDocx receives this same serializable MaterialDocument, without a second derivation.
    expect(JSON.parse(JSON.stringify(doc)).blocks.slice(1, 3)).toEqual(doc.blocks.slice(1, 3));
  });
});

it("duplicates writing without transferring its role or replacing another live slide", async () => {
  const { createMemoryApi } = await import("../support/memoryApi");
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Invented project" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Invented study" });
  const prep = await api.teachingCreate({ studyId: study.id, title: "Sunday", templateVersion: 2, blocks: [{ blockType: "writing", title: "Opening", participantFacing: true, role: "introduction" }] });
  const owner = prep.blocks![0]!;
  await api.teachingBlockUpdate({ id: owner.id, body: "OWNER WORDS" });
  const copied = await api.teachingBlockDuplicate(owner.id);
  const copy = copied.blocks!.find((b) => b.id !== owner.id)!;
  expect(copy.body).toBe("OWNER WORDS"); expect(copy.role).toBeNull();
  await api.teachingBlockUpdate({ id: copy.id, body: "COPY WORDS" });
  const presentation = await api.presentationCreate({ preparationId: prep.id, propose: true, labels: { introduction: "Opening", conclusion: "Closing" } });
  const output = () => api.presentationDocument(presentation.id).then((d) => JSON.stringify(d));
  expect(await output()).toContain("OWNER WORDS"); expect(await output()).toContain("COPY WORDS");
  await api.teachingBlockUpdate({ id: copy.id, body: "EDITED COPY" });
  expect((await api.teachingGet(prep.id)).introduction).toBe("OWNER WORDS");
  expect(await output()).toContain("OWNER WORDS"); expect(await output()).toContain("EDITED COPY");
  await expect(api.teachingBlockUpdate({ id: copy.id, role: "introduction" })).rejects.toThrow();
  await api.teachingBlockDelete(owner.id);
  expect((await api.teachingGet(prep.id)).introduction).toBe(""); expect(await output()).not.toContain("OWNER WORDS");
});
