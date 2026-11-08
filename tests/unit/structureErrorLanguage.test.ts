import { afterEach, expect, it } from "vitest";
import i18n from "@/i18n";
import { localizeError } from "@/i18n/messages";

afterEach(async () => { await i18n.changeLanguage("en"); });
it.each(["en", "de", "ko", "fr", "es", "pt"])("localises native analysis failures in %s", async language => {
  await i18n.changeLanguage(language);
  for (const code of ["timeout", "cancelled", "busy", "limit", "failed"]) {
    const key = `errors.library.structure.${code}`;
    const message = localizeError("invalidInput", "Heading analysis stopped. English fallback must not leak.", `library.structure.${code}`);
    expect(i18n.exists(key)).toBe(true);
    expect(message).toBe(i18n.t(key as never));
    expect(message).not.toContain("English fallback");
  }
});
