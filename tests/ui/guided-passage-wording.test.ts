import { expect, it } from "vitest";
import i18n, { LANGUAGES, setLanguage } from "@/i18n";
import { sectionGuidance, sectionTitle, templateName } from "@/features/studies/builtinText";
import structured from "../../resources/templates/pozetek-study.json";
import writing from "../../resources/templates/pozetek-writing-study.json";

const names = {
  en: "Guided Passage Study",
  ko: "성경 본문 공부",
  fr: "Étude guidée d’un passage biblique",
  es: "Estudio guiado de un pasaje bíblico",
  pt: "Estudo guiado de uma passagem bíblica",
};

function strings(value: unknown): string[] {
  if (typeof value === "string") return [value];
  if (Array.isArray(value)) return value.flatMap(strings);
  if (value && typeof value === "object") return Object.values(value).flatMap(strings);
  return [];
}

function resourceText(value: unknown): string[] {
  if (typeof value === "string") return [value];
  if (Array.isArray(value)) return value.flatMap(resourceText);
  if (!value || typeof value !== "object") return [];
  // Stable identifiers and exact historical matches are compatibility data,
  // not text offered as a current built-in label or hint.
  return Object.entries(value)
    .filter(([key]) => !["key", "fieldsSource", "formerTitles", "formerGuidance"].includes(key))
    .flatMap(([, child]) => resourceText(child));
}

it.each(LANGUAGES.filter(({ code }) => code !== "de").map(({ code }) => code))(
  "uses descriptive study wording throughout the %s interface",
  async (code) => {
    await setLanguage(code);
    expect(strings(i18n.getResourceBundle(code, "translation")).filter((text) => /pozetek/i.test(text))).toEqual([]);
    for (const template of [structured, writing]) {
      expect(templateName({ key: template.key, name: "POZETEK", isBuiltin: true })).toBe(names[code as keyof typeof names]);
    }
  },
);

it("retains the German name and application guidance", async () => {
  await setLanguage("de");
  for (const template of [structured, writing]) {
    expect(templateName({ ...template, isBuiltin: true })).toContain("POZETEK");
  }
  expect(i18n.t("studies.builtin.templates.pozetek-study.sections.application.goal")).toContain("POZETEK");
  expect(i18n.t("studies.builtin.templates.pozetek-writing-study.sections.application.guidance")).toContain("POZETEK");
  expect(i18n.t("studies.repair.review")).toBe("POZETEK-Teile prüfen");
  expect(i18n.t("studies.repair.undo")).toBe("POZETEK-Reparatur rückgängig");
});

it("keeps current English resource fallbacks free of the German acronym", () => {
  for (const template of [structured, writing]) {
    expect(template.name).toBe(names.en);
    expect(resourceText(template).filter((text) => /pozetek/i.test(text))).toEqual([]);
  }
});

const oldApplicationGuidance = "What does the truth you found mean for you? Keep it apart from the observation and interpretation above. Respond to the text in a way that stays tied to what it says. This is a closing step of its own, not another letter of POZETEK. Which truth of the text stays valid for every reader? Where does this truth challenge, comfort or correct your own thinking and life? What will you do: believe, repent, trust, obey, pray, love? Make it specific. What differs between the original situation and today? What must not be carried over directly?";

it.each(LANGUAGES.map(({ code }) => code))(
  "localises saved historical application guidance in %s without rewriting it",
  async (code) => {
    await setLanguage(code);
    const application = writing.sections.find((section) => section.key === "application")!;
    expect(application.formerGuidance).toContain(oldApplicationGuidance);
    const saved = { key: "application", guidance: oldApplicationGuidance };
    const displayed = sectionGuidance(saved, writing.key);
    expect(displayed).toBe(i18n.getResource(code, "translation", "studies.builtin.templates.pozetek-writing-study.sections.application.guidance"));
    expect(/pozetek/i.test(displayed ?? "")).toBe(code === "de");
    expect(saved.guidance).toBe(oldApplicationGuidance);
  },
);

it("preserves the person's own template names, headings and guidance", async () => {
  await setLanguage("en");
  expect(templateName({ key: writing.key, name: "My POZETEK notes", isBuiltin: false })).toBe("My POZETEK notes");
  expect(sectionTitle({ key: "application", title: "My POZETEK reflection" }, writing.key)).toBe("My POZETEK reflection");
  expect(sectionGuidance({ key: "application", guidance: "My POZETEK reflection: pray about this passage." }, writing.key)).toBe("My POZETEK reflection: pray about this passage.");
});
