import { afterEach, expect, it } from "vitest";
import i18n from "@/i18n";
import { inquiryAssistantLabel, inquiryBlockText, inquiryBlockLabel, inquiryDisplayLabel, inquiryLocation } from "@/features/ai/inquiryDisplay";
import type { DisplayLabel, SharedBlock } from "@/types/inquiry";

const source: DisplayLabel = {
  type: "source", title: 'Your writing — “part” › 은혜', author: 'paragraphs 12 — A. “Writer”',
  location: { section: 'Section — “Your note” › 한글', firstPage: "xii", lastPage: "xiii", firstParagraph: 12, lastParagraph: 16 },
};

afterEach(async () => { await i18n.changeLanguage("en"); });

it.each([
  ["en", "paragraphs 12–16"], ["de", "Absätze 12–16"], ["ko", "12–16번째 단락"],
  ["fr", "paragraphes 12–16"], ["es", "párrafos 12–16"], ["pt", "parágrafos 12–16"],
])("localises %s provenance without parsing or changing source/user titles", async (language, paragraphRange) => {
  await i18n.changeLanguage(language);
  const label = inquiryDisplayLabel(i18n.t, source);
  expect(label).toContain(source.title);
  expect(label).toContain(source.author);
  expect(label).toContain(source.location.section);
  expect(label).toContain("xii");
  expect(label).toContain("xiii");
  expect(label).toContain(paragraphRange);
  expect(label).not.toContain("{{");
  if (language !== "en") expect(label).not.toContain(" · paragraphs 12–16");
  const note = inquiryDisplayLabel(i18n.t, { type: "note", findingType: "observation", part: { key: null, title: "Never translate my own title — 한글" }, study: "My Study “Name”" });
  expect(note).toContain(i18n.t("studies.findingType.observation"));
  expect(note).toContain("Never translate my own title — 한글");
  expect(note).toContain("My Study “Name”");
  expect(inquiryDisplayLabel(i18n.t, { type: "request" })).toBe(i18n.t("studies.inquiry.block.request"));
  expect(inquiryDisplayLabel(i18n.t, { type: "writing", selected: true, part: null })).toBe(i18n.t("scripture.selection.selectedWords"));
  expect(inquiryDisplayLabel(i18n.t, { type: "quote", citation: 'Exact author, title, p. xii', note: "N2" })).toContain('Exact author, title, p. xii');
  const other = inquiryAssistantLabel(i18n.t, "Other compatible service · My Model — 한글");
  expect(other).toBe(i18n.t("settings.assistant.providers.openai_compatible") + " · My Model — 한글");
  expect(inquiryAssistantLabel(i18n.t, "OpenAI · Other compatible service")).toBe("OpenAI · Other compatible service");
});

it("uses the same display metadata for previews and checked evidence while leaving payload labels unchanged", async () => {
  await i18n.changeLanguage("de");
  const block: SharedBlock = { id: "S1", kind: "source", label: "A generated English prompt label", text: "Exact source text.", reference: null, translation: null, sourceId: "source", location: "English internal locator", display: source };
  const before = JSON.stringify(block);
  expect(inquiryBlockLabel(i18n.t, block)).toBe(inquiryDisplayLabel(i18n.t, source));
  expect(inquiryBlockLabel(i18n.t, block)).not.toContain("A generated English prompt label");
  expect(JSON.stringify(block)).toBe(before);
  expect(inquiryLocation(i18n.t, { section: null, firstPage: "52", lastPage: "52", firstParagraph: 1, lastParagraph: 1 })).toBe("S. 52 · Absatz 1");
});

it("localises only unchanged built-in headings and preserves renamed writing headings", async () => {
  await i18n.changeLanguage("de");
  expect(inquiryDisplayLabel(i18n.t, { type: "writing", selected: false, part: { key: "persons", title: "Persons" } })).toContain("Personen");
  expect(inquiryDisplayLabel(i18n.t, { type: "writing", selected: false, part: { key: "persons", title: "Persons — my own title" } })).toContain("Persons — my own title");
});

it.each(["en", "de", "ko", "fr", "es", "pt"])("localises only the app-generated missing Bible marker in %s", async language => {
  await i18n.changeLanguage(language);
  const block: SharedBlock = { id: "S1", kind: "scripture", label: "Reference", text: "(The Bible text is not installed.)", reference: null, translation: null, sourceId: null, location: null, display: { type: "scripture", reference: { canonical: "JHN 1:1", display: "John 1:1", book: "JHN", startChapter: 1, startVerse: 1, endChapter: 1, endVerse: 1 }, translation: null, textAvailable: false } };
  expect(inquiryBlockText(i18n.t, block)).toBe(i18n.t("presentation.export.labels.scriptureUnavailable"));
  if (block.display?.type === "scripture") block.display.textAvailable = true;
  expect(inquiryBlockText(i18n.t, block)).toBe(block.text);
});
