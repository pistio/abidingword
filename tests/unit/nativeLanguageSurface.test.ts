import { afterEach, expect, it, vi } from "vitest";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import i18n from "@/i18n";
import { localizeError, localizeMessage } from "@/i18n/messages";
import { tauriApi } from "@/lib/api";

const { open } = vi.hoisted(() => ({ open: vi.fn(async () => null) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open, save: vi.fn() }));
afterEach(async () => { open.mockClear(); await i18n.changeLanguage("en"); });

it.each(["en", "de", "ko", "fr", "es", "pt"])("uses %s in native file dialogs and unsupported-error fallbacks", async language => {
  await i18n.changeLanguage(language);
  await tauriApi.chooseSourceFile();
  expect(open).toHaveBeenLastCalledWith(expect.objectContaining({ title: i18n.t("library.fileDialog.source"), filters: [expect.objectContaining({ name: i18n.t("library.fileDialog.sourceTypes") })] }));
  await tauriApi.chooseSourceFiles();
  expect(open).toHaveBeenLastCalledWith(expect.objectContaining({ title: i18n.t("library.fileDialog.sources"), multiple: true }));
  await tauriApi.chooseSourceFolder();
  expect(open).toHaveBeenLastCalledWith(expect.objectContaining({ title: i18n.t("library.fileDialog.folder"), directory: true }));
  await tauriApi.chooseBibleFiles();
  expect(open).toHaveBeenLastCalledWith(expect.objectContaining({ title: i18n.t("library.fileDialog.bible"), filters: expect.arrayContaining([expect.objectContaining({ name: i18n.t("library.fileDialog.allFiles") })]) }));
  for (const count of [1, 2]) {
    const wording = localizeMessage("bibleImport.documentFootnotes", { count: String(count) }, "English fallback");
    expect(wording).toContain(String(count));
    expect(wording).not.toContain("English fallback");
    expect(wording).not.toContain("{{");
  }
  const fallback = localizeError("invalidInput", "Unexpected English technical message", "future.unknownCode");
  expect(fallback).toBe(i18n.t("errors.messageUnavailable"));
  expect(fallback).not.toContain("technical message");
});

it("has translations for every statically named native UserMessage code", () => {
  function files(directory: string): string[] {
    return readdirSync(directory, { withFileTypes: true }).flatMap(entry => entry.isDirectory() ? files(join(directory, entry.name)) : [join(directory, entry.name)]);
  }
  for (const file of files("src-tauri/src").filter(file => file.endsWith(".rs"))) {
    const source = readFileSync(file, "utf8");
    for (const match of source.matchAll(/UserMessage::new\(\s*"([\w.]+)"/g)) {
      for (const language of ["en", "de", "ko", "fr", "es", "pt"]) {
        expect(i18n.exists(`errors.${match[1]}`, { lng: language, count: 2, fallbackLng: false }), `${language}: ${match[1]} in ${file}`).toBe(true);
      }
    }
  }
});
