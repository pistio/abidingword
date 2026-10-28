import { readFileSync } from "node:fs";
import { join } from "node:path";
import { cleanup, screen, waitFor, within } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import pozetek from "../../resources/templates/pozetek-study.json";
import { setLanguage } from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";
import { localizedSections } from "@/features/studies/builtinText";

// The POZETEK built-in template (Phase 1). All text is invented.

beforeEach(() => window.localStorage.clear());

async function withPozetek() {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Series" });
  const templateId = api.state.templates.find((t) => t.key === "pozetek-study")!.id;
  const detail = await api.studyCreate({ projectId: project.id, title: "Any passage", templateId });
  return { api, studyId: detail.study.id };
}

describe("a new POZETEK study", () => {
  it("keeps structured fields, status and examples when its template is customised", async () => {
    const api = createMemoryApi();
    const builtin = api.state.templates.find((t) => t.key === "pozetek-study")!;
    const personalised = await api.templateCreate({
      name: "My POZETEK",
      studyType: "passage",
      sections: localizedSections(builtin).map((section, index) =>
        index === 0 ? { ...section, title: "My people" } : section),
    });
    const duplicate = await api.templateDuplicate(personalised.id);
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Study" });
    const { study } = await api.studyCreate({ projectId: project.id, title: "Hosea 11", templateId: duplicate.id });
    const sections = await api.studySectionList(study.id);
    expect(sections).toHaveLength(8);
    expect(sections[0]!.title).toBe("My people");
    expect(sections[0]!.fields).toEqual({
      template: "pozetek-study",
      version: pozetek.sections[0]!.fieldsVersion,
      fields: pozetek.sections[0]!.fields,
    });
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Hosea 11/ }));
    const part = await screen.findByRole("region", { name: "My people" });
    expect(within(part).getByLabelText("Author")).toHaveValue("");
    expect(within(part).getByRole("button", { name: "Example: Hosea 11" })).toHaveAttribute("aria-expanded", "false");
  });

  it("adds an optional structured part later without losing its fields", async () => {
    const api = createMemoryApi();
    const builtin = api.state.templates.find((t) => t.key === "pozetek-study")!;
    const personalised = await api.templateCreate({
      name: "Optional place",
      studyType: "passage",
      sections: localizedSections(builtin).map((part) =>
        part.key === "place" ? { ...part, optional: true } : part),
    });
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Study" });
    const { study } = await api.studyCreate({ projectId: project.id, title: "Deferred place", templateId: personalised.id });
    expect((await api.studySectionList(study.id)).map((s) => s.key)).not.toContain("place");

    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Deferred place/ }));
    await user.click(await screen.findByRole("button", { name: "Add a part" }));
    await user.click(screen.getByRole("button", { name: /^Place$/ }));

    await waitFor(async () => {
      const sections = await api.studySectionList(study.id);
      const place = sections.find((s) => s.key === "place");
      expect(place?.fields?.template).toBe("pozetek-study");
      expect(place?.fields?.fields).toEqual(pozetek.sections[1]!.fields);
    });
    expect(await screen.findByRole("region", { name: /^Place$/ })).toBeInTheDocument();
  });

  it("has the parts in the order P-O-Z-E-T-E-K and then the personal application, all empty", async () => {
    const { api, studyId } = await withPozetek();
    const sections = await api.studySectionList(studyId);
    expect(sections.map((s) => s.key)).toEqual(["persons", "place", "time", "events", "context", "explanations", "core", "application"]);
    expect(sections.every((s) => s.status === "not_started" && s.fields?.template === "pozetek-study")).toBe(true);
    expect(sections.flatMap((s) => s.blocks).every((b) => b.content === "" && b.data === null)).toBe(true);
    expect(sections[6]!.fields!.fields.map((f) => f.key)).toEqual(["subject", "statement", "theme"]);
  });

  it("keeps its field definitions when the template later changes", async () => {
    const { api, studyId } = await withPozetek();
    api.state.templates.find((t) => t.key === "pozetek-study")!.sections = [{ key: "x", title: "Changed" }];
    const sections = await api.studySectionList(studyId);
    expect(sections).toHaveLength(8);
    expect(sections[0]!.fields!.fields.length).toBeGreaterThan(0);
  });

  it("shows labelled fields, an example that is collapsed and never an answer", async () => {
    const { api, studyId } = await withPozetek();
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Any passage/ }));
    const part = await screen.findByRole("region", { name: /^Persons$/ });
    expect(within(part).getByLabelText("Author")).toHaveValue("");
    const toggle = within(part).getByRole("button", { name: "Example: Hosea 11" });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    await user.click(toggle);
    expect(toggle).toHaveAttribute("aria-expanded", "true");
    expect(within(part).getByText(/Hosea proclaims/)).toBeVisible();
    // Nothing from the example was stored.
    expect((await api.studySectionList(studyId)).flatMap((s) => s.blocks).every((b) => !b.content.includes("Hosea"))).toBe(true);
  });

  it("saves text, entries and progress, and shows them again when reopened", async () => {
    const { api, studyId } = await withPozetek();
    const first = await renderApp(api);
    await first.user.click(await screen.findByRole("button", { name: /Any passage/ }));
    const persons = await screen.findByRole("region", { name: /^Persons$/ });
    await first.user.type(within(persons).getByLabelText("Author"), "The prophet");
    await first.user.click(within(persons).getByRole("button", { name: /Add person or group/ }));
    await first.user.type(await within(persons).findByLabelText(/Name or group \(People and groups, entry 1\)/), "Ephraim");
    await first.user.selectOptions(within(persons).getByRole("combobox", { name: /Progress of/ }), "in_progress");
    const explanations = screen.getByRole("region", { name: /^Explanations$/ });
    await first.user.click(within(explanations).getByRole("button", { name: /Add question/ }));
    await first.user.selectOptions(await within(explanations).findByLabelText(/Status \(Question log, entry 1\)/), "answered");
    await waitFor(async () => {
      const saved = await api.studySectionList(studyId);
      expect(saved[0]!.status).toBe("in_progress");
      expect(saved[0]!.blocks.find((b) => b.fieldKey === "author")!.content).toBe("The prophet");
      const entry = saved[0]!.blocks.find((b) => b.fieldKey === "people")!;
      expect(JSON.parse(entry.data!)).toEqual({ name: "Ephraim" });
      expect(entry.content).toBe("Ephraim");
      expect(JSON.parse(saved[5]!.blocks.find((b) => b.fieldKey === "questions")!.data!)).toEqual({ status: "answered" });
    });
    cleanup();

    // The workspace reopens on the study that was open.
    await renderApp(api);
    const reopened = await screen.findByRole("region", { name: /^Persons$/ });
    expect(within(reopened).getByLabelText("Author")).toHaveValue("The prophet");
    expect(within(reopened).getByLabelText(/Name or group \(People and groups, entry 1\)/)).toHaveValue("Ephraim");
    expect(within(reopened).getByRole("combobox", { name: /Progress of/ })).toHaveValue("in_progress");
  });

  it("asks before deleting an entry whose only answer is a changed choice", async () => {
    const { api } = await withPozetek();
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Any passage/ }));
    const part = await screen.findByRole("region", { name: /^Explanations$/ });
    await user.click(within(part).getByRole("button", { name: /Add question/ }));
    await user.selectOptions(await within(part).findByLabelText(/Status \(Question log, entry 1\)/), "answered");
    await user.click(within(part).getByRole("button", { name: "Delete entry 1 of Question log" }));
    const confirm = within(part).getByRole("alertdialog");
    await user.click(within(confirm).getByRole("button", { name: "Keep" }));
    expect(within(part).getByLabelText(/Status \(Question log, entry 1\)/)).toHaveValue("answered");
  });

  it("deletes one entry without changing its neighbours", async () => {
    const { api, studyId } = await withPozetek();
    const section = (await api.studySectionList(studyId))[5]!;
    const make = async (text: string) => {
      const block = await api.studyBlockCreate({ sectionId: section.id, blockType: "paragraph", fieldKey: "questions" });
      await api.studyBlockUpdate({ id: block.id, content: text, data: JSON.stringify({ question: text }) });
      return block.id;
    };
    const [a, b, c] = [await make("one"), await make("two"), await make("three")];
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Any passage/ }));
    const part = await screen.findByRole("region", { name: /^Explanations$/ });
    await user.click(within(part).getByRole("button", { name: "Delete entry 2 of Question log" }));
    await user.click(within(part).getAllByRole("button", { name: "Delete" }).at(-1)!);
    await waitFor(async () => {
      const entries = (await api.studySectionList(studyId))[5]!.blocks.filter((x) => x.fieldKey === "questions");
      expect(entries.map((x) => x.id)).toEqual([a, c]);
      expect(entries.map((x) => x.content)).toEqual(["one", "three"]);
    });
    expect(b).toBeTruthy();
  });
});

describe("the translations of POZETEK", () => {
  const LANGS = ["en", "de", "ko", "fr", "es", "pt"];
  const read = (lang: string) => JSON.parse(readFileSync(join(__dirname, `../../src/i18n/locales/${lang}/studies.json`), "utf8")).builtin.templates["pozetek-study"];

  it("has every title, question, goal, example, field label and help in all six languages, none left in English", () => {
    const english = read("en");
    for (const lang of LANGS) {
      const t = read(lang);
      expect(t.name && t.description).toBeTruthy();
      for (const section of pozetek.sections) {
        const s = t.sections[section.key];
        for (const part of ["title", "guidance", "goal", "example"]) {
          expect(s[part], `${lang} ${section.key}.${part}`).toBeTruthy();
          // Long texts must really be translated, not left as the English.
          if (lang !== "en" && s[part].length > 40) expect(s[part], `${lang} ${section.key}.${part}`).not.toBe(english.sections[section.key][part]);
        }
        for (const field of section.fields) {
          const f = s.fields[field.key];
          expect(f.label && f.help, `${lang} ${section.key}.${field.key}`).toBeTruthy();
          for (const part of (field as { parts?: { key: string; options?: string[] }[] }).parts ?? []) {
            expect(f.parts[part.key]).toBeTruthy();
            for (const option of part.options ?? []) expect(f.options[option]).toBeTruthy();
          }
          if (field.kind === "entries") expect(f.add).toBeTruthy();
        }
      }
    }
  });

  it("shows the German interface", async () => {
    const { api } = await withPozetek();
    const { user } = await renderApp(api);
    await setLanguage("de");
    await user.click(await screen.findByRole("button", { name: /Any passage/ }));
    const kern = await screen.findByRole("region", { name: "K — Kern" });
    expect(within(kern).getByLabelText("Textgegenstand — Worum geht es?")).toBeInTheDocument();
    expect(within(kern).getByLabelText("Textaussage — Was sagt der Text darüber?")).toBeInTheDocument();
    expect(within(kern).getByLabelText("Textthema — Ein prägnanter Satz")).toBeInTheDocument();
    expect(screen.getByRole("region", { name: "Persönliche Anwendung" })).toBeInTheDocument();
    expect(within(kern).getByRole("button", { name: "Beispiel: Hosea 11" })).toHaveAttribute("aria-expanded", "false");
    await setLanguage("en");
  });
});
