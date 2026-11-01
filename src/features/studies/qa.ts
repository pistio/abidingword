// A.4.4 (#27): a Study "question" block that is a paired Question + Answer
// keeps both in its JSON data; its text (search, export, older builds) is
// derived from them by the native core.
export interface QaPair {
  question: string;
  answer: string;
}

export function qaOf(data: string | null | undefined): QaPair | null {
  if (!data) return null;
  try {
    const value = JSON.parse(data) as Record<string, unknown>;
    if (value?.kind !== "qa") return null;
    return { question: typeof value.question === "string" ? value.question : "", answer: typeof value.answer === "string" ? value.answer : "" };
  } catch {
    return null;
  }
}

export function qaData(pair: QaPair): string {
  return JSON.stringify({ kind: "qa", question: pair.question, answer: pair.answer });
}

export function qaText(pair: QaPair): string {
  return [pair.question.trim(), pair.answer.trim()].filter(Boolean).join("\n\n");
}
