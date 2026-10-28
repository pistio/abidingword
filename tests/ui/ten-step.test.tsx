import { readFileSync } from "node:fs";
import { join } from "node:path";
import { cleanup, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import tenStep from "../../resources/templates/ten-step-exegesis.json";
import { localizedSections } from "@/features/studies/builtinText";
import { setLanguage } from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";

beforeEach(() => window.localStorage.clear());

async function newStudy() {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Passages" });
  const template = api.state.templates.find((t) => t.key === tenStep.key)!;
  const { study } = await api.studyCreate({ projectId: project.id, title: "Matthew 8:18–27", templateId: template.id });
  return { api, studyId: study.id, template };
}

describe("the Ten-Step exegesis method", () => {
  it("is a separate built-in template with ten ordered, empty, structured steps", async () => {
    const { api, studyId } = await newStudy();
    expect((await api.templateList()).find((t) => t.key === tenStep.key)?.isBuiltin).toBe(true);
    const sections = await api.studySectionList(studyId);
    expect(sections.map((part) => part.key)).toEqual(tenStep.sections.map((part) => part.key));
    expect(sections).toHaveLength(10);
    expect(sections.every((part) => part.status === "not_started" && part.fields?.template === tenStep.key)).toBe(true);
    expect(sections.flatMap((part) => part.blocks).every((block) => !block.content && !block.data)).toBe(true);
    expect(sections[8]!.fields?.fields.map((field) => field.key)).toEqual(["subject", "statement", "theme", "text-check"]);
    expect(sections[9]!.key).toBe("step-10-application");
  });

  it("keeps the Matthew example optional, and never stores it as an answer", async () => {
    const { api, studyId } = await newStudy();
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Matthew 8:18/ }));
    const first = await screen.findByRole("region", { name: /1 — Become familiar/ });
    expect(within(first).getByLabelText("First impression")).toHaveValue("");
    const toggle = within(first).getByRole("button", { name: "Example: Matthew 8:18–27" });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    await user.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(within(first).getByText(/two encounters about following Jesus/)).toBeVisible();
    expect((await api.studySectionList(studyId)).flatMap((part) => part.blocks).every((b) => !b.content.includes("Matthew"))).toBe(true);
  });

  it("persists an answer, an entry and progress through reopening the study", async () => {
    const { api, studyId } = await newStudy();
    const first = await renderApp(api);
    await first.user.click(await screen.findByRole("button", { name: /Matthew 8:18/ }));
    const part = await screen.findByRole("region", { name: /1 — Become familiar/ });
    await first.user.type(within(part).getByLabelText("First impression"), "Following Christ");
    await first.user.click(within(part).getByRole("button", { name: /Add question/ }));
    await first.user.type(await within(part).findByLabelText(/Question \(Questions to investigate, entry 1\)/), "Why this order?");
    await first.user.selectOptions(within(part).getByRole("combobox", { name: /Progress of/ }), "in_progress");
    await waitFor(async () => {
      const saved = (await api.studySectionList(studyId))[0]!;
      expect(saved.status).toBe("in_progress");
      expect(saved.blocks.find((b) => b.fieldKey === "first-impression")?.content).toBe("Following Christ");
      expect(JSON.parse(saved.blocks.find((b) => b.fieldKey === "questions")!.data!)).toEqual({ question: "Why this order?" });
    });
    cleanup();
    await renderApp(api);
    const reopened = await screen.findByRole("region", { name: /1 — Become familiar/ });
    expect(within(reopened).getByLabelText("First impression")).toHaveValue("Following Christ");
    expect(within(reopened).getByRole("combobox", { name: /Progress of/ })).toHaveValue("in_progress");
  });

  it("keeps fields and example localisation in a customised copy", async () => {
    const { api, template } = await newStudy();
    const copy = await api.templateCreate({
      name: "My ten steps",
      studyType: "passage",
      sections: localizedSections(template).map((part) =>
        part.key === "step-09-message" ? { ...part, title: "My summary" } : part),
    });
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Own study" });
    const { study } = await api.studyCreate({ projectId: project.id, title: "Own Matthew study", templateId: copy.id });
    const parts = await api.studySectionList(study.id);
    expect(parts[8]!.title).toBe("My summary");
    expect(parts[8]!.fields?.template).toBe(tenStep.key);
    expect(parts[8]!.fields?.fields).toEqual(tenStep.sections[8]!.fields);
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Own Matthew study/ }));
    const summary = await screen.findByRole("region", { name: "My summary" });
    expect(within(summary).getByLabelText("Text subject — What is it about?")).toBeInTheDocument();
    expect(within(summary).getByRole("button", { name: "Example: Matthew 8:18–27" })).toHaveAttribute("aria-expanded", "false");
  });
});

describe("Ten-Step translations", () => {
  const langs = ["en", "de", "ko", "fr", "es", "pt"];
  const read = (lang: string) => JSON.parse(readFileSync(join(__dirname, `../../src/i18n/locales/${lang}/studies.json`), "utf8")).builtin.templates["ten-step-exegesis"];

  it("provides translated titles, guidance, goals, examples, fields and options in all six languages", () => {
    const en = read("en");
    for (const lang of langs) {
      const translated = read(lang);
      expect(translated.name && translated.description).toBeTruthy();
      for (const section of tenStep.sections) {
        const t = translated.sections[section.key];
        for (const key of ["title", "guidance", "goal", "example"]) {
          expect(t[key], `${lang} ${section.key}.${key}`).toBeTruthy();
          if (lang !== "en") expect(t[key]).not.toBe(en.sections[section.key][key]);
        }
        for (const field of section.fields) {
          const f = t.fields[field.key];
          expect(f.label && f.help, `${lang} ${section.key}.${field.key}`).toBeTruthy();
          if (lang !== "en") expect(f.label).not.toBe(en.sections[section.key].fields[field.key].label);
          for (const part of ("parts" in field && Array.isArray(field.parts) ? field.parts : [])) {
            expect(f.parts[part.key]).toBeTruthy();
            for (const option of ("options" in part && Array.isArray(part.options) ? part.options : [])) expect(f.options[option]).toBeTruthy();
          }
          if (field.kind === "entries") expect(f.add).toBeTruthy();
        }
      }
    }
  });

  it("shows German labels and keeps the example collapsed", async () => {
    const { api } = await newStudy();
    const { user } = await renderApp(api);
    await setLanguage("de");
    try {
      await user.click(await screen.findByRole("button", { name: /Matthew 8:18/ }));
      const step9 = await screen.findByRole("region", { name: /9 — Die zentrale Aussage/ });
      expect(within(step9).getByLabelText("Textgegenstand — Worum geht es?")).toBeInTheDocument();
      expect(within(step9).getByLabelText("Textaussage — Was wird gesagt?")).toBeInTheDocument();
      expect(within(step9).getByLabelText("Textthema — Ein Satz")).toBeInTheDocument();
      expect(within(step9).getByRole("button", { name: "Beispiel: Matthäus 8,18–27" })).toHaveAttribute("aria-expanded", "false");
    } finally {
      await setLanguage("en");
    }
  });
});
