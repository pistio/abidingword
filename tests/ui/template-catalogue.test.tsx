import { act, cleanup, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import i18n, { LANGUAGES, setLanguage } from "@/i18n";
import { normaliseLanguage } from "@/i18n/languages";
import { ApiProvider } from "@/lib/ApiContext";
import { ScriptureProvider } from "@/features/scripture/ScriptureContext";
import { BOOKS } from "../support/scriptureFixture";
import { NewStudyForm } from "@/features/studies/NewStudyForm";
import { StudyTemplates } from "@/features/settings/StudyTemplates";
import { AddPart } from "@/features/studies/AddPart";
import { STUDY_TEMPLATE_KEYS, inStudyCatalogue } from "@/features/studies/catalogue";
import { localizedSections, sectionGuidance, sectionTitle, structuredText, templateDescription, templateName } from "@/features/studies/builtinText";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";

for (const { code } of LANGUAGES) {
  describe(`the six-template catalogue in ${code}`, () => {
    it("localises every template, part and structured field, and keeps user wording", async () => {
      await setLanguage(code);
      const api = createMemoryApi();
      const templates = api.state.templates.filter(inStudyCatalogue);
      expect(new Set(templates.map((t) => t.key))).toEqual(new Set(STUDY_TEMPLATE_KEYS));
      for (const template of templates) {
        const path = `studies.builtin.templates.${template.key}`;
        expect(templateName(template)).toBe(i18n.getResource(code, "translation", `${path}.name`));
        expect(templateDescription(template)).toBe(i18n.getResource(code, "translation", `${path}.description`));
        expect(templateName(template).includes("POZETEK")).toBe(code === "de" && template.key === "pozetek-writing-study");
        for (const section of template.sections) {
          const base = `${path}.sections.${section.key}`;
          expect(i18n.getResource(code, "translation", `${base}.title`)).toBeTruthy();
          expect(sectionGuidance(section, template.key)).toBe(i18n.getResource(code, "translation", `${base}.guidance`));
          expect(sectionTitle(section, template.key)).toBe(i18n.getResource(code, "translation", `${base}.title`));
          expect(sectionTitle({ ...section, title: "My own heading" })).toBe("My own heading");
          expect(sectionGuidance({ ...section, guidance: "My own guidance" })).toBe("My own guidance");
          const structured = { key: section.key, fields: { template: template.key } };
          for (const field of section.fields ?? []) {
            const f = `fields.${field.key}`;
            for (const key of ["label", "help", ...(field.kind === "entries" ? ["add"] : [])]) {
              const text = i18n.getResource(code, "translation", `${base}.${f}.${key}`);
              expect(text, `${code} ${base}.${f}.${key}`).toBeTruthy();
              expect(structuredText(structured, `${f}.${key}`, "missing")).toBe(text);
            }
            for (const part of field.parts ?? []) {
              expect(i18n.getResource(code, "translation", `${base}.${f}.parts.${part.key}`)).toBeTruthy();
              for (const option of part.options ?? []) expect(i18n.getResource(code, "translation", `${base}.${f}.options.${option}`)).toBeTruthy();
            }
          }
        }
      }
    });

    it("creates Character and Book studies without parsing their subject as a passage", async () => {
      await setLanguage(code);
      for (const type of ["character", "book"] as const) {
        const api = createMemoryApi();
        const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Studies" });
        const parse = vi.spyOn(api, "referenceParse");
        const created = vi.fn();
        const user = userEvent.setup();
        render(<ApiProvider api={api}><ScriptureProvider settings={api.state.settings} books={BOOKS} translations={[]}><NewStudyForm projectId={project.id} onCreated={created} /></ScriptureProvider></ApiProvider>);
        await user.click(await screen.findByRole("button", { name: i18n.t(`studies.newStudy.approach.${type}`) }));
        await user.type(screen.getByRole("textbox", { name: i18n.t(`studies.newStudy.subjectLabel.${type}`) }), type === "character" ? "Ruth" : "Philippians");
        await user.click(screen.getByRole("button", { name: i18n.t("studies.newStudy.create") }));
        await waitFor(() => expect(created).toHaveBeenCalledOnce());
        expect(parse).not.toHaveBeenCalled();
        const study = api.state.studies[0]!;
        expect(study.studyType).toBe(type);
        const template = api.state.templates.find((t) => t.id === study.templateId)!;
        expect(template.key).toBe(type === "character" ? "character-research-study" : "book-research-study");
        const parts = await api.studySectionList(study.id);
        expect(parts).toHaveLength(7);
        expect(parts.every((s) => s.blocks.every((b) => b.content === ""))).toBe(true);
        cleanup();
      }
    });
  });
}

it("offers both passage methods and uses the selected template", async () => {
  for (const key of ["pozetek-writing-study", "ten-step-writing-study"]) {
    const api = createMemoryApi();
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Passages" });
    const user = userEvent.setup();
    const created = vi.fn();
    render(<ApiProvider api={api}><ScriptureProvider settings={api.state.settings} books={BOOKS} translations={[]}><NewStudyForm projectId={project.id} onCreated={created} /></ScriptureProvider></ApiProvider>);
    const method = await screen.findByRole("combobox", { name: "Template" });
    expect(within(method).getAllByRole("option")).toHaveLength(2);
    await user.selectOptions(method, key);
    await user.type(screen.getByRole("textbox", { name: "Bible passage" }), "John 15:1-11");
    await user.click(screen.getByRole("button", { name: "Create study" }));
    await waitFor(() => expect(created).toHaveBeenCalledOnce());
    expect(api.state.studies[0]!.templateId).toBe(api.state.templates.find((t) => t.key === key)!.id);
    expect(api.state.passages).toHaveLength(1);
    cleanup();
  }
});

it("shows six built-ins in Settings and retains legacy definitions and user templates", async () => {
  const api = createMemoryApi();
  await api.templateCreate({ name: "Saved personal outline", studyType: "biblical_theology", sections: [{ title: "Read" }] });
  render(<ApiProvider api={api}><ScriptureProvider settings={api.state.settings} books={BOOKS} translations={[]}><StudyTemplates onEdit={() => undefined} onEditTeaching={() => undefined} /></ScriptureProvider></ApiProvider>);
  const list = await screen.findByRole("list", { name: i18n.t("studies.templates.listLabel") });
  await waitFor(() => expect(within(list).getAllByRole("listitem")).toHaveLength(7));
  expect(within(list).getByText("Saved personal outline")).toBeInTheDocument();
  expect(api.state.templates.some((t) => t.key === "passage-study")).toBe(true);
  expect(within(list).getAllByRole("listitem")).toHaveLength(7);
});

it("keeps keyless optional parts usable in a saved personal template", async () => {
  const api = createMemoryApi();
  const template = await api.templateCreate({ name: "Personal", studyType: "custom", sections: [{ title: "Read" }, { title: "Reflect later", optional: true }] });
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Personal" });
  const study = await api.studyCreate({ projectId: project.id, title: "Mine", templateId: template.id });
  const add = vi.fn().mockResolvedValue(undefined);
  const user = userEvent.setup();
  render(<AddPart template={template} sections={study.sections} onAdd={add} />);
  await user.click(screen.getByRole("button", { name: "Add a part" }));
  await user.click(screen.getByRole("button", { name: "Reflect later" }));
  expect(add).toHaveBeenCalledWith(expect.objectContaining({ title: "Reflect later" }));
});

it.each(["character", "book"] as const)("edits, adds optional parts, switches language and reopens a %s study", async (type) => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Studies" });
  const study = await api.studyCreate({ projectId: project.id, studyType: type, title: `My ${type}` });
  const first = await renderApp(api);
  await first.user.click(await screen.findByRole("button", { name: new RegExp(`My ${type}`) }));
  const part = await screen.findByRole("region", { name: type === "character" ? "Person and passages" : "Read the whole book" });
  await first.user.type(within(part).getByRole("textbox"), "My evidence");
  await first.user.tab();
  await act(() => setLanguage("de"));
  await first.user.click(screen.getByRole("button", { name: i18n.t("studies.parts.add") }));
  const template = api.state.templates.find((t) => t.id === study.study.templateId)!;
  const optional = template.sections.find((s) => s.optional)!;
  await first.user.click(screen.getByRole("button", { name: sectionTitle(optional) }));
  await waitFor(async () => expect(await api.studySectionList(study.study.id)).toHaveLength(8));
  await act(() => setLanguage("en"));
  expect(await screen.findByRole("region", { name: optional.title })).toBeInTheDocument();
  cleanup();
  await renderApp(api);
  expect(await screen.findByDisplayValue("My evidence")).toBeInTheDocument();
  expect((await api.studySearch("evidence")).some((h) => h.studyId === study.study.id)).toBe(true);
});

it("falls back to English for missing translations, and to saved text for unknown definitions", async () => {
  expect(normaliseLanguage("pt-BR")).toBe("pt");
  await setLanguage("fr");
  const path = "studies.builtin.templates.pozetek-study.sections.persons.fields.author.label";
  const bundle = i18n.getResourceBundle("fr", "translation");
  const old = bundle.studies.builtin.templates["pozetek-study"].sections.persons.fields.author.label;
  delete bundle.studies.builtin.templates["pozetek-study"].sections.persons.fields.author.label;
  try {
    expect(structuredText({ key: "persons", fields: { template: "pozetek-study" } }, "fields.author.label", "author")).toBe("Author");
    expect(structuredText({ key: "future", fields: { template: "unknown" } }, "fields.label", "Saved label")).toBe("Saved label");
    expect(sectionTitle({ key: "future", title: "Saved heading" })).toBe("Saved heading");
    expect(localizedSections({ key: "user", isBuiltin: false, sections: [{ title: "My heading" }] })).toEqual([{ title: "My heading" }]);
  } finally {
    i18n.addResource("fr", "translation", path, old);
  }
});

it("keeps structured localisation after the personal source template is deleted", async () => {
  const api = createMemoryApi();
  const builtin = api.state.templates.find((t) => t.key === "pozetek-study")!;
  const own = await api.templateDuplicate(builtin.id);
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Studies" });
  await api.studyCreate({ projectId: project.id, title: "Kept study", templateId: own.id });
  await api.templateDelete(own.id);
  await setLanguage("fr");
  const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: /Kept study/ }));
  const name = i18n.t("studies.builtin.templates.pozetek-study.sections.persons.title");
  const part = await screen.findByRole("region", { name });
  expect(within(part).getByLabelText(i18n.t("studies.builtin.templates.pozetek-study.sections.persons.fields.author.label"))).toBeInTheDocument();
});
