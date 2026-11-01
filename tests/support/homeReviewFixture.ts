import type { MemoryApi } from "./memoryApi";

/** Identical invented profile shape for baseline/current screenshots. */
export async function homeReviewFixture(api: MemoryApi, ownerShape = false) {
  if (ownerShape) for (const project of [...api.state.projects]) await api.projectDelete(project.id, true);
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "My studies" });
  const study = await api.studyCreate({ projectId: project.id, title: "Root Bible Study", primaryPassage: "JHN.15.1-JHN.15.11" });
  await api.studyBlockUpdate({ id: study.sections[0]!.blocks[0]!.id, content: "Invented root research retained while filing this document." });
  const legacy = await api.teachingCreate({ projectId: project.id, title: "Historical independent message", templateVersion: 2, blocks: [{ blockType: "writing", title: "Main part", participantFacing: false }] });
  await api.teachingBlockUpdate({ id: legacy.blocks![0]!.id, body: "Invented private manuscript for compatibility review." });
}
