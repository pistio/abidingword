// A.4.4 (#27): optional Q&A in Study and Teaching, exports that keep private
// leader content private, and explicit "Add from Bible Study". Invented data.
import { act, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { derive } from "@/features/materials/derive";
import { studyMarkdown } from "@/features/studies/StudyQaExport";
import { qaData } from "@/features/studies/qa";
import type { TeachingPreparation } from "@/types/teaching";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";
import { openPartOptions, openStudyOptions, openTeachingOptions } from "../support/studyActions";

const studyPane = () => screen.getByRole("main", { name: "Study" });
afterEach(async () => { await act(async () => { await i18n.changeLanguage("en"); }); });

const t = i18n.t.bind(i18n);
const teaching = (): TeachingPreparation => ({
  id: "p", projectId: "x", studyId: "s", title: "Sunday", kind: "bible_study", audience: null, purpose: null, centralMessage: null,
  introduction: "", conclusion: "", notes: "", points: [], findings: [], passages: [], materials: [], templateKey: "eva", templateVersion: 2,
  blocks: [
    { id: "shared", parentId: null, position: 0, blockType: "qa", blockKey: null, title: "What does the vine need?", body: "SHARED-ANSWER", participantFacing: true, role: null, guidanceKey: null, pointId: null },
    { id: "private", parentId: null, position: 1, blockType: "qa", blockKey: null, title: "PRIVATE-QUESTION", body: "PRIVATE-ANSWER", participantFacing: false, role: null, guidanceKey: null, pointId: null },
  ],
  createdAt: "", updatedAt: "", revision: 1,
});

describe("A.4.4 Q&A exports keep private content private", () => {
  it("questions only: shared questions, no answers, no private question", () => {
    const sheet = JSON.stringify(derive(t, "questions", teaching()));
    expect(sheet).toContain("What does the vine need?");
    expect(sheet).not.toContain("SHARED-ANSWER");
    expect(sheet).not.toContain("PRIVATE-QUESTION");
    expect(sheet).not.toContain("PRIVATE-ANSWER");
  });

  it("questions and answers: answers of shared questions only", () => {
    const sheet = JSON.stringify(derive(t, "questions", teaching(), { answers: true }));
    expect(sheet).toContain("SHARED-ANSWER");
    expect(sheet).not.toContain("PRIVATE");
  });

  it("the leader's version and the full lesson with notes include the private Q&A; without notes they do not", () => {
    expect(JSON.stringify(derive(t, "questions", teaching(), { leader: true }))).toContain("PRIVATE-ANSWER");
    expect(JSON.stringify(derive(t, "manuscript", teaching()))).not.toContain("PRIVATE");
    const full = JSON.stringify(derive(t, "manuscript", teaching(), { includeNotes: true }));
    expect(full).toContain("PRIVATE-QUESTION");
    expect(full).toContain("SHARED-ANSWER");
    expect(JSON.stringify(derive(t, "worksheet", teaching()))).not.toContain("PRIVATE");
  });

  it("a Study's export: questions only, with answers, or the whole study's writing", () => {
    const sections = [{
      id: "a", studyId: "s", title: "Text", position: 0, guidance: null, key: null, phase: null, status: "not_started", fields: null, templateSectionKey: null,
      blocks: [
        { id: "w", sectionId: "a", blockType: "paragraph", position: 0, content: "My writing.", fieldKey: null, data: null, revision: 1 },
        { id: "q", sectionId: "a", blockType: "question", position: 1, content: "", fieldKey: null, data: qaData({ question: "Who abides?", answer: "The branch." }), revision: 1 },
      ],
    }] as never;
    const questions = studyMarkdown(t, "John 15", sections, "questions");
    expect(questions).toContain("1. Who abides?");
    expect(questions).not.toContain("The branch.");
    expect(questions).not.toContain("My writing.");
    expect(studyMarkdown(t, "John 15", sections, "answers")).toContain("The branch.");
    const full = studyMarkdown(t, "John 15", sections, "full");
    expect(full).toContain("My writing.");
    expect(full).toContain("The branch.");
  });
});

async function openStudy(api: ReturnType<typeof createMemoryApi>, title: string) {
  const { user } = await renderApp(api);
  await user.click(within(await screen.findByRole("list", { name: "Studies" })).getByRole("button", { name: new RegExp(title) }));
  await within(studyPane()).findByRole("textbox", { name: "Study title" });
  return user;
}

describe("A.4.4 Question and answer in a Bible Study", () => {
  it("adds paired, separately editable, numbered Q&A on request, saves them and asks before deleting", async () => {
    const api = createMemoryApi();
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "John" });
    await api.studyCreate({ projectId: project.id, title: "Abiding" });
    const user = await openStudy(api, "Abiding");
    const first = within(studyPane()).getAllByRole("heading", { level: 2 })[0]!.textContent!;
    for (let i = 0; i < 2; i++) {
      await openPartOptions(user, first);
      await user.click(within(studyPane()).getByRole("button", { name: "Question and answer" }));
    }
    const two = await within(studyPane()).findByRole("group", { name: "Question and answer 2" });
    const one = within(studyPane()).getByRole("group", { name: "Question and answer 1" });
    await user.type(within(one).getByRole("textbox", { name: "Question and answer 1 — Question" }), "Who abides?");
    await user.type(within(one).getByRole("textbox", { name: "Question and answer 1 — Answer" }), "The branch that stays.");
    await waitFor(() => {
      const block = api.state.blocks.find((b) => b.data?.includes("Who abides?"));
      expect(block?.blockType).toBe("question");
      expect(block?.content).toBe("Who abides?\n\nThe branch that stays.");
      expect(JSON.parse(block!.data!)).toEqual({ kind: "qa", question: "Who abides?", answer: "The branch that stays." });
    });
    // An empty one goes without asking; a written one asks by name first.
    await user.click(within(two).getByRole("button", { name: "Options for Question and answer 2" }));
    await user.click(within(two).getByRole("button", { name: "Delete" }));
    await waitFor(() => expect(within(studyPane()).queryByRole("group", { name: "Question and answer 2" })).toBeNull());
    await user.click(within(one).getByRole("button", { name: "Options for Question and answer 1" }));
    await user.click(within(one).getByRole("button", { name: "Delete" }));
    const confirm = within(one).getByRole("alertdialog");
    expect(confirm).toHaveTextContent("Question and answer 1");
    await user.click(within(confirm).getByRole("button", { name: "Keep" }));
    expect(api.state.blocks.some((b) => b.data?.includes("Who abides?"))).toBe(true);
  });

  it("exports the Study's questions from the Study options", async () => {
    const api = createMemoryApi();
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "John" });
    const { study } = await api.studyCreate({ projectId: project.id, title: "Abiding" });
    const section = (await api.studySectionList(study.id))[0]!;
    await api.studyBlockCreate({ sectionId: section.id, blockType: "question", data: qaData({ question: "Who abides?", answer: "The branch." }) });
    const user = await openStudy(api, "Abiding");
    await openStudyOptions(user);
    await user.click(within(studyPane()).getByRole("button", { name: "Export questions…" }));
    const dialog = await screen.findByRole("dialog", { name: "Export questions" });
    expect(within(dialog).getByTestId("qa-export-preview")).toHaveTextContent("1. Who abides?");
    expect(within(dialog).getByTestId("qa-export-preview")).not.toHaveTextContent("The branch.");
    await user.click(within(dialog).getByRole("radio", { name: "Questions and answers" }));
    expect(within(dialog).getByTestId("qa-export-preview")).toHaveTextContent("The branch.");
  });
});

describe("A.4.4 Add from Bible Study", () => {
  it("copies chosen findings, writing, Q&A and passages as leader-only editable copies that name their origin", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "John" });
    const { study } = await api.studyCreate({ projectId: project.id, title: "Abiding" });
    const section = (await api.studySectionList(study.id))[0]!;
    await api.studyBlockUpdate({ id: section.blocks[0]!.id, content: "My study writing." });
    await api.studyBlockCreate({ sectionId: section.id, blockType: "question", data: qaData({ question: "Who abides?", answer: "The branch." }) });
    await api.findingCreate({ studyId: study.id, findingType: "observation", content: "Branches do not strain." });
    await api.studyPassageAdd({ studyId: study.id, reference: "JHN.15.1-JHN.15.8" });
    const user = await openStudy(api, "Abiding");
    await user.click(within(studyPane()).getByRole("button", { name: "Bible Teaching" }));
    const form = within(studyPane()).getByRole("form", { name: "Prepare a teaching from this study" });
    await user.click(within(form).getByRole("radio", { name: "Group Bible Study — Observation, Understanding, Application" }));
    await user.click(within(form).getByRole("button", { name: "Start" }));
    const article = await within(studyPane()).findByRole("article", { name: "Group Bible study · Abiding" });
    const before = api.state.teaching.preparations[0]!.blocks!.length;
    await openTeachingOptions(user);
    await user.click(within(article).getByRole("button", { name: "Add from Bible Study…" }));
    const dialog = await screen.findByRole("dialog", { name: "Add from Bible Study" });
    expect(dialog).toHaveTextContent("starts for the leader only");
    const add = within(dialog).getByRole("button", { name: "Add selected" });
    expect(add).toBeDisabled();
    await user.click(await within(dialog).findByRole("checkbox", { name: /Observation/ }));
    await user.click(within(dialog).getByRole("checkbox", { name: /Who abides\?/ }));
    await user.click(within(dialog).getByRole("checkbox", { name: /John 15:1–8/ }));
    await user.click(add);
    await waitFor(() => expect(screen.queryByRole("dialog", { name: "Add from Bible Study" })).toBeNull());
    const blocks = api.state.teaching.preparations[0]!.blocks!;
    expect(blocks).toHaveLength(before + 3);
    const copies = blocks.filter((b) => b.origin);
    expect(copies.map((b) => b.origin!.kind).sort()).toEqual(["finding", "passage", "qa"]);
    expect(copies.every((b) => !b.participantFacing)).toBe(true);
    expect(copies.find((b) => b.origin!.kind === "qa")).toMatchObject({ blockType: "qa", title: "Who abides?", body: "The branch." });
    expect(within(article).getAllByText(/From the Bible Study:/).length).toBeGreaterThanOrEqual(2);
    expect(await within(article).findByText(/for the leader only/)).toBeInTheDocument();
    // The study is unchanged; nothing is kept in step.
    expect(api.state.findings.find((f) => f.content === "Branches do not strain.")).toBeTruthy();
  });
});
