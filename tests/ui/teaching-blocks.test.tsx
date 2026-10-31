import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { derive } from "@/features/materials/derive";
import { BUILTIN_BLOCK_TEMPLATES, templateFromTeaching } from "@/features/teaching/teachingTemplates";
import type { TeachingPreparation } from "@/types/teaching";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";

const prep = (): TeachingPreparation => ({
  id: "p",
  projectId: "fixture-project", studyId: "s",
  title: "Sunday",
  kind: "bible_study",
  audience: null,
  purpose: null,
  centralMessage: null,
  introduction: "",
  conclusion: "",
  notes: "",
  points: [],
  findings: [],
  passages: [],
  materials: [],
  templateKey: "eva",
  templateVersion: 2,
  blocks: [
    { id: "q", parentId: null, position: 0, blockType: "discussion_questions", blockKey: null, title: "Questions", body: "What do you see?", participantFacing: true, role: null, guidanceKey: null, pointId: null },
    { id: "n", parentId: null, position: 1, blockType: "teaching_notes", blockKey: null, title: "Leader", body: "Listen for the vine.", participantFacing: false, role: null, guidanceKey: null, pointId: null },
  ],
  createdAt: "",
  updatedAt: "",
  revision: 1,
});

const studyPane = () => screen.getByRole("main", { name: "Study" });

async function openTeaching() {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "John" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Abiding" });
  await api.studyPassageAdd({ studyId: study.id, reference: "JHN.15.1-JHN.15.8" });
  const { user } = await renderApp(api);
  await user.click(within(await screen.findByRole("list", { name: "Studies" })).getByRole("button", { name: /Abiding/ }));
  await within(studyPane()).findByRole("textbox", { name: "Study title" });
  await user.click(within(studyPane()).getByRole("button", { name: "Bible Teaching" }));
  return { api, user };
}

describe("block teachings keep leader notes private", () => {
  const t = i18n.t.bind(i18n);

  it("leaves leader notes off the question sheet and the worksheet unless the leader's version is asked for", () => {
    const group = JSON.stringify(derive(t, "questions", prep()));
    const sheet = JSON.stringify(derive(t, "worksheet", prep()));
    const leader = JSON.stringify(derive(t, "questions", prep(), { leader: true }));
    const manuscript = JSON.stringify(derive(t, "manuscript", prep()));
    const withNotes = JSON.stringify(derive(t, "manuscript", prep(), { includeNotes: true }));
    expect(group).toContain("What do you see?");
    expect(group).not.toContain("Listen for the vine.");
    expect(sheet).toContain("What do you see?");
    expect(sheet).not.toContain("Listen for the vine.");
    expect(leader).toContain("Listen for the vine.");
    expect(manuscript).not.toContain("Listen for the vine.");
    expect(withNotes).toContain("Listen for the vine.");
  });
});

describe("the three built-in block templates", () => {
  it("are exactly EVA, Sermon/Talk and Bible Lesson, with guidance keys that exist", () => {
    expect(BUILTIN_BLOCK_TEMPLATES.map((b) => b.key)).toEqual(["eva", "sermon-talk", "bible-lesson"]);
    const tr = i18n.t.bind(i18n) as (key: string) => string;
    for (const template of BUILTIN_BLOCK_TEMPLATES) {
      for (const block of template.blocks) {
        if (!block.guidanceKey) continue;
        const text = tr(block.guidanceKey);
        expect(text, block.guidanceKey).not.toBe(block.guidanceKey);
        expect(text.length).toBeGreaterThan(10);
      }
    }
    const evaGuidance = BUILTIN_BLOCK_TEMPLATES.find((b) => b.key === "eva")!.blocks.map((b) => b.guidanceKey).filter(Boolean) as string[];
    expect(evaGuidance.every((key) => tr(key).includes("Campus Connect 3×3"))).toBe(true);
    expect(BUILTIN_BLOCK_TEMPLATES.find((b) => b.key === "bible-lesson")!.blocks.some((b) => b.key === "prepare")).toBe(true);
  });

  it("starts EVA with three writing areas and Campus Connect guidance, not extra fields", async () => {
    const { api, user } = await openTeaching();
    const form = within(studyPane()).getByRole("form", { name: "Prepare a teaching from this study" });
    await user.click(within(form).getByRole("radio", { name: "Bible Study — Observation, Understanding, Application" }));
    await user.click(within(form).getByRole("button", { name: "Start" }));
    const article = await within(studyPane()).findByRole("article", { name: "Group Bible study · Abiding" });
    expect(within(article).getByRole("heading", { name: "Observation" })).toBeInTheDocument();
    expect(within(article).getByRole("heading", { name: "Comprehension" })).toBeInTheDocument();
    expect(within(article).getByRole("heading", { name: "Application" })).toBeInTheDocument();
    const guidance = within(article).getAllByText("Guidance");
    expect(guidance).toHaveLength(3);
    expect(within(article).getAllByText(/Campus Connect 3×3/)).toHaveLength(3);
    expect(api.state.teaching.preparations[0]!.templateKey).toBe("eva");
    expect(api.state.teaching.preparations[0]!.blocks?.filter((b) => b.blockType === "writing")).toHaveLength(3);
  });

  it("keeps continuous and outline views on the same sermon text", async () => {
    const { api, user } = await openTeaching();
    const form = within(studyPane()).getByRole("form", { name: "Prepare a teaching from this study" });
    await user.click(within(form).getByRole("radio", { name: "Sermon / Talk" }));
    await user.click(within(form).getByRole("button", { name: "Start" }));
    const article = await within(studyPane()).findByRole("article", { name: "Sermon · Abiding" });
    const view = within(article).getByRole("radiogroup", { name: "How you write" });
    expect(within(view).getByRole("radio", { name: "Continuous" })).toBeChecked();
    const introduction = within(article).getByRole("textbox", { name: "Introduction" });
    await user.type(introduction, "Last week a vine.");
    await waitFor(() => {
      const writing = api.state.teaching.preparations[0]!.blocks?.find((b) => b.role === "introduction");
      expect(writing?.body).toContain("Last week a vine.");
    });
    await user.click(within(view).getByRole("radio", { name: "Outline" }));
    expect(within(article).getByRole("textbox", { name: "Introduction" })).toHaveValue("Last week a vine.");
    await user.click(within(article).getByRole("button", { name: "Add a block in Main part" }));
    await user.click(within(article).getByRole("menuitem", { name: "Outline point" }));
    await waitFor(() => expect(api.state.teaching.preparations[0]!.blocks?.some((b) => b.blockType === "outline_point")).toBe(true));
    const outline = api.state.teaching.preparations[0]!.blocks!.find((b) => b.blockType === "outline_point")!;
    await user.type(within(article).getByRole("textbox", { name: "Heading" }), "The true vine");
    await waitFor(() => expect(api.state.teaching.preparations[0]!.blocks?.find((b) => b.id === outline.id)?.title).toContain("The true vine"));
    await user.click(within(view).getByRole("radio", { name: "Continuous" }));
    expect(within(article).getByRole("textbox", { name: "Introduction" })).toHaveValue("Last week a vine.");
    expect(api.state.teaching.preparations[0]!.blocks?.find((b) => b.id === outline.id)?.title).toContain("The true vine");
  });

  it("starts a children's lesson with optional preparation guidance", async () => {
    const { api, user } = await openTeaching();
    const form = within(studyPane()).getByRole("form", { name: "Prepare a teaching from this study" });
    await user.click(within(form).getByRole("radio", { name: "Bible Lesson — Children / Youth" }));
    await user.click(within(form).getByRole("button", { name: "Start" }));
    const article = await within(studyPane()).findByRole("article", { name: "Lesson · Abiding" });
    expect(within(article).getByRole("heading", { name: "Preparation" })).toBeInTheDocument();
    expect(within(article).getByRole("heading", { name: "Story" })).toBeInTheDocument();
    await user.click(within(article).getAllByText("Guidance")[0]!);
    expect(within(article).getByText(/Age changes how you tell the story/)).toBeInTheDocument();
    expect(api.state.teaching.preparations[0]!.templateKey).toBe("bible-lesson");
  });

  it("saves a block teaching as a personal template without the writing", async () => {
    const { api, user } = await openTeaching();
    const form = within(studyPane()).getByRole("form", { name: "Prepare a teaching from this study" });
    await user.click(within(form).getByRole("button", { name: "Start" }));
    const article = await within(studyPane()).findByRole("article", { name: "Sermon · Abiding" });
    await user.type(within(article).getByRole("textbox", { name: "Introduction" }), "Do not copy this sentence.");
    await waitFor(() => expect(api.state.teaching.preparations[0]!.introduction).toContain("Do not copy this sentence."));
    await user.click(within(article).getByRole("button", { name: "Bible Teaching information" }));
    const about = await screen.findByRole("dialog", { name: "Bible Teaching information" });
    await user.click(within(about).getByRole("button", { name: "Save as my template" }));
    const save = within(about).getByRole("form", { name: "Save this teaching’s structure as a template" });
    expect(save).toHaveTextContent("sections and blocks");
    await user.click(within(save).getByRole("button", { name: "Save template" }));
    expect(await within(about).findByRole("status")).toHaveTextContent("Saved as your template");
    const mine = api.state.teaching.templates[0]!;
    expect(mine.version).toBe(2);
    expect(mine.blocks?.some((b) => b.blockType === "section" && b.title === "Introduction")).toBe(true);
    expect(JSON.stringify(mine)).not.toMatch(/Do not copy this sentence/);
  });

  it("reloads what was autosaved in a writing block", async () => {
    const { api, user } = await openTeaching();
    const form = within(studyPane()).getByRole("form", { name: "Prepare a teaching from this study" });
    await user.click(within(form).getByRole("radio", { name: "Bible Study — Observation, Understanding, Application" }));
    await user.click(within(form).getByRole("button", { name: "Start" }));
    const article = await within(studyPane()).findByRole("article", { name: "Group Bible study · Abiding" });
    await user.type(within(article).getByRole("textbox", { name: "Observation" }), "The vine remains.");
    await waitFor(() => {
      const writing = api.state.teaching.preparations[0]!.blocks?.find((b) => b.blockKey === "erkennen-writing");
      expect(writing?.body).toContain("The vine remains.");
    });
    const reloaded = await api.teachingGet(api.state.teaching.preparations[0]!.id);
    expect(reloaded.blocks?.find((b) => b.blockKey === "erkennen-writing")?.body).toContain("The vine remains.");
  });
});

describe("saving a block teaching as a template", () => {
  it("keeps headings and kinds, never the bodies", () => {
    const saved = templateFromTeaching(prep());
    expect(saved.version).toBe(2);
    expect(saved.blocks?.map((b) => [b.blockType, b.title])).toEqual([
      ["discussion_questions", "Questions"],
      ["teaching_notes", "Leader"],
    ]);
    expect(JSON.stringify(saved)).not.toMatch(/What do you see|Listen for the vine/);
  });
});
