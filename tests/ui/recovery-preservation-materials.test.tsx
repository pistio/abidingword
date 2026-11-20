import { readFileSync, writeFileSync } from "node:fs";
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { derive, toMarkdown, type MaterialDocument, type MaterialOptions } from "@/features/materials/derive";
import { MaterialPage } from "@/features/materials/MaterialView";
import type { MaterialScripture } from "@/types/scripture";
import type { TeachingPreparation } from "@/types/teaching";

// Native integration tests generate this payload from real SQLite repository
// calls and verify its exact preservation after daily/full restore and reopen.
// Only UUIDs/timestamps are normalised; privacy, writing and provenance are real.
const payload = JSON.parse(readFileSync("tests/fixtures/recovery-preservation.json", "utf8")) as {
  preparation: TeachingPreparation;
  scripture: MaterialScripture;
};
const DOCX_FIXTURE = "src-tauri/tests/fixtures/materials/recovery-documents.json";
const options: [string, MaterialDocument["kind"], MaterialOptions, boolean][] = [
  ["recovery-participant-questions", "questions", { includePassage: true }, true],
  ["recovery-participant-answers", "questions", { includePassage: true, answers: true }, true],
  ["recovery-participant-worksheet", "worksheet", { includePassage: true }, true],
  ["recovery-leader", "questions", { includePassage: true, leader: true, includeNotes: true }, false],
  ["recovery-manuscript", "manuscript", { includePassage: true }, false],
  ["recovery-private-manuscript", "manuscript", { includePassage: true, includeNotes: true }, false],
];
const documents = () => options.map(([name, kind, settings, participant]) => ({
  name,
  participant,
  document: derive(i18n.getFixedT("en"), kind, payload.preparation, settings, payload.scripture),
}));

describe("real recovered Study and Teaching material preservation", () => {
  it("keeps private writing and private Scripture out of every participant model and Markdown export", () => {
    for (const item of documents().filter((item) => item.participant)) {
      const model = JSON.stringify(item.document);
      const markdown = toMarkdown(item.document);
      for (const output of [model, markdown]) {
        expect(output).toContain("PUBLIC QUESTION: What does the invented branch need?");
        expect(output).toContain("PUBLIC SCRIPTURE fixture text.");
        expect(output).not.toContain("PRIVATE");
        expect(output).not.toContain("John 15:2");
      }
      if (item.name === "recovery-participant-answers") {
        expect(markdown).toContain("PUBLIC ANSWER: The invented vine sustains it.");
      } else {
        expect(markdown).not.toContain("PUBLIC ANSWER:");
      }
    }
  });

  it("retains the author's answers, private leader writing and Scripture in the requested leader outputs", () => {
    const leader = documents().find((d) => d.name === "recovery-leader")!;
    const text = toMarkdown(leader.document);
    expect(text).toContain("PUBLIC ANSWER: The invented vine sustains it.");
    expect(text).toContain("PRIVATE LEADER: Pause and check the invented clock.");
    expect(text).toContain("PRIVATE SCRIPTURE fixture text.");
    expect(text).toContain("John 15:2");
    const manuscript = documents().find((d) => d.name === "recovery-manuscript")!;
    expect(toMarkdown(manuscript.document)).toContain("PUBLIC MANUSCRIPT: Words prepared by the author.");
    expect(toMarkdown(manuscript.document)).not.toContain("PRIVATE LEADER:");
    expect(toMarkdown(documents().find((d) => d.name === "recovery-private-manuscript")!.document)).toContain("PRIVATE LEADER:");
  });

  it("renders the participant preview without the private blocks", () => {
    render(<MaterialPage doc={documents()[0]!.document} />);
    expect(screen.getByText("PUBLIC QUESTION: What does the invented branch need?")).toBeVisible();
    expect(screen.getByText("PUBLIC SCRIPTURE fixture text.")).toBeVisible();
    expect(screen.queryByText(/PRIVATE/)).not.toBeInTheDocument();
  });

  it("keeps the generated DOCX input exactly equal to the same frontend material model", () => {
    const bytes = `${JSON.stringify(documents(), null, 1)}\n`;
    if (process.env.UPDATE_RECOVERY_FIXTURES) writeFileSync(DOCX_FIXTURE, bytes);
    expect(readFileSync(DOCX_FIXTURE, "utf8")).toBe(bytes);
  });
});
