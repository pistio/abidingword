import { afterEach, expect, it } from "vitest";
import i18n from "@/i18n";
import { formatDate, formatLocale, formatNumber } from "@/i18n/format";

afterEach(async () => { await i18n.changeLanguage("en"); });
it("uses British dates and Brazilian Portuguese formatting without changing language IDs", async () => {
  await i18n.changeLanguage("en");
  expect(formatDate("2026-11-08T12:00:00Z")).toBe("8 Nov 2026");
  expect(formatNumber(1234.5)).toBe("1,234.5");
  expect(formatLocale("pt")).toBe("pt-BR");
  expect(formatLocale("pt-PT")).toBe("pt-BR");
  expect(formatLocale("unknown")).toBe("en-GB");
  expect(formatDate("not a date")).toBe("");
  await i18n.changeLanguage("pt");
  expect(formatNumber(1234.5)).toBe("1.234,5");
  expect(i18n.language).toBe("pt");
});
