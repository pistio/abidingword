// Numbers and dates in the active interface language (Intl, no library).
import i18n from "i18next";
import { normaliseLanguage } from "./languages";

/** Formatting variants; stored interface language IDs stay unchanged. */
export function formatLocale(language = i18n.language): string {
  const locales = { en: "en-GB", de: "de-DE", ko: "ko-KR", fr: "fr-FR", es: "es-ES", pt: "pt-BR" };
  return locales[normaliseLanguage(language) ?? "en"];
}

export function formatNumber(value: number): string {
  return new Intl.NumberFormat(formatLocale()).format(value);
}

/** e.g. "2 Oct 2026", "2. Okt. 2026", "2026. 10. 2." */
export function formatDate(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return "";
  return new Intl.DateTimeFormat(formatLocale(), { dateStyle: "medium" }).format(date);
}
