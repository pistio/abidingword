import type { StudyTemplate } from "@/types/studies";

/** New-study catalogue only. Legacy definitions remain installed for saved studies. */
export const STUDY_TEMPLATE_KEYS = [
  "pozetek-writing-study", "ten-step-writing-study", "biblical-character-study",
  "word-study", "whole-book-study", "topical-study",
] as const;

export function inStudyCatalogue(template: Pick<StudyTemplate, "isBuiltin" | "key">): boolean {
  return !template.isBuiltin || (STUDY_TEMPLATE_KEYS as readonly string[]).includes(template.key);
}
