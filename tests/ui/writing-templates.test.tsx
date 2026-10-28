import { expect, it } from "vitest";
import { createMemoryApi } from "../support/memoryApi";
import { partsFromStudy } from "@/features/studies/templateParts";

it("snapshots generous single-answer templates without changing legacy structured studies", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Preservation" });
  for (const [legacy, current, count] of [["pozetek-study", "pozetek-writing-study", 7], ["ten-step-exegesis", "ten-step-writing-study", 10]] as const) {
    const old = api.state.templates.find(t => t.key === legacy)!;
    const study = await api.studyCreate({ projectId: project.id, title: legacy, templateId: old.id });
    const before = await api.studySectionList(study.study.id);
    const writing = api.state.templates.find(t => t.key === current)!;
    const next = await api.studyCreate({ projectId: project.id, title: current, templateId: writing.id });
    const sections = await api.studySectionList(next.study.id);
    expect(sections).toHaveLength(count);
    for (const section of sections) {
      expect(section.fields?.fields).toEqual([{ key: "answer", kind: "writing" }]);
      expect(section.blocks).toHaveLength(1);
      expect(section.blocks[0]?.fieldKey).toBe("answer");
    }
    expect(partsFromStudy(sections).every(s => s.fields?.[0]?.kind === "writing")).toBe(true);
    expect(await api.studySectionList(study.study.id)).toEqual(before);
    expect(api.state.templates.find(t => t.key === legacy)).toEqual(old);
  }
});
