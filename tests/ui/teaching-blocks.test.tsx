import { describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { derive } from "@/features/materials/derive";
import type { TeachingPreparation } from "@/types/teaching";

const prep = (): TeachingPreparation => ({
  id: "p",
  studyId: "s",
  title: "Sunday",
  kind: "bible_study",
  audience: null,
  purpose: null,
  centralMessage: null,
  introduction: "",
  conclusion: "",
  notes: "",
  points: [],
  findings: [],
  passages: [],
  materials: [],
  templateKey: "eva",
  templateVersion: 2,
  blocks: [
    { id: "q", parentId: null, position: 0, blockType: "discussion_questions", blockKey: null, title: "Questions", body: "What do you see?", participantFacing: true, role: null, guidanceKey: null, pointId: null },
    { id: "n", parentId: null, position: 1, blockType: "teaching_notes", blockKey: null, title: "Leader", body: "Listen for the vine.", participantFacing: false, role: null, guidanceKey: null, pointId: null },
  ],
  createdAt: "",
  updatedAt: "",
  revision: 1,
});

describe("block teachings keep leader notes private", () => {
  const t = i18n.t.bind(i18n);

  it("leaves leader notes off the question sheet and the worksheet unless the leader's version is asked for", () => {
    const group = JSON.stringify(derive(t, "questions", prep()));
    const sheet = JSON.stringify(derive(t, "worksheet", prep()));
    const leader = JSON.stringify(derive(t, "questions", prep(), { leader: true }));
    const manuscript = JSON.stringify(derive(t, "manuscript", prep()));
    const withNotes = JSON.stringify(derive(t, "manuscript", prep(), { includeNotes: true }));
    expect(group).toContain("What do you see?");
    expect(group).not.toContain("Listen for the vine.");
    expect(sheet).toContain("What do you see?");
    expect(sheet).not.toContain("Listen for the vine.");
    expect(leader).toContain("Listen for the vine.");
    expect(manuscript).not.toContain("Listen for the vine.");
    expect(withNotes).toContain("Listen for the vine.");
  });
});
