import { afterEach, describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { localizeMessage } from "@/i18n/messages";

const expected = {
  en: { wisdom: "Wisdom of Solomon", web: "Web page", osis: "OSIS and USX" },
  de: { wisdom: "Weisheit Salomos", web: "Webseiten", osis: "OSIS und USX" },
  ko: { wisdom: "솔로몬의 지혜서", web: "웹 페이지", osis: "OSIS 및 USX" },
  fr: { wisdom: "Sagesse de Salomon", web: "de pages web", osis: "OSIS et USX" },
  es: { wisdom: "Sabiduría de Salomón", web: "de páginas web", osis: "OSIS y USX" },
  pt: { wisdom: "Sabedoria de Salomão", web: "páginas da web", osis: "OSIS e USX" },
};

afterEach(async () => { await i18n.changeLanguage("en"); });

describe("native import diagnostics with generated display-name parameters", () => {
  for (const [language, text] of Object.entries(expected)) {
    it(`names the additional book and unsupported formats in ${language}`, async () => {
      await i18n.changeLanguage(language);
      const params = { fileName: "Wisdom of Solomon — ORIGINAL.usfm", book: "Wisdom of Solomon", code: "WIS" };
      const warning = localizeMessage("bibleImport.additionalBook", params, "raw English fallback");
      expect(warning).toContain(text.wisdom);
      expect(warning).toContain(params.fileName);
      const heading = "UNMODIFIED source heading / Sagesse";
      expect(localizeMessage("bibleImport.documentAdditionalBook", { ...params, heading }, "raw English fallback"))
        .toContain(text.wisdom);
      expect(localizeMessage("bibleImport.documentAdditionalBook", { ...params, heading }, "raw English fallback"))
        .toContain(heading);
      expect(localizeMessage("bibleImport.formatNotYet", { format: "Web page" }, "raw English fallback"))
        .toContain(text.web);
      expect(localizeMessage("bibleImport.formatNotYet", { format: "OSIS and USX" }, "raw English fallback"))
        .toContain(text.osis);
      expect(params.book).toBe("Wisdom of Solomon");
      expect(params.code).toBe("WIS");
    });
  }

  it("preserves unknown names/formats and does not rewrite unrelated parameters or source content", async () => {
    await i18n.changeLanguage("de");
    expect(localizeMessage("bibleImport.additionalBook", { fileName: "a.usfm", book: "Owner's exact title", code: "FUTURE" }, ""))
      .toContain("Owner's exact title");
    for (const format of ["Owner's exact format", "constructor", "__proto__"]) {
      expect(localizeMessage("bibleImport.formatNotYet", { format }, "")).toContain(format);
    }
    expect(localizeMessage("bibleImport.unknownBook", { fileName: "Book title", code: "WIS", book: "Wisdom of Solomon" }, ""))
      .toContain("WIS");
    expect(localizeMessage("reference.unknownBook", { text: "Wisdom of Solomon", code: "WIS", book: "Wisdom of Solomon" }, ""))
      .toContain("Wisdom of Solomon");
  });
});
