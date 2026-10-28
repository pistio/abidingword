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

it("keeps answers while switching from folded steps to continuous writing", async () => {
  const { screen, within, waitFor } = await import("@testing-library/react");
  const { renderApp } = await import("../support/render");
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Writing" });
  const template = api.state.templates.find(t => t.key === "pozetek-writing-study")!;
  const detail = await api.studyCreate({ projectId: project.id, title: "Continuous fixture", templateId: template.id });
  const { user } = await renderApp(api);
  await user.click(within(await screen.findByRole("list", { name: "Studies" })).getByRole("button", { name: /Continuous fixture/ }));
  const answers = await screen.findAllByRole("textbox", { name: "Your answer" });
  expect(answers).toHaveLength(7);
  expect(answers[0]).toHaveAttribute("rows", "10");
  expect(screen.getAllByText("Guidance (optional)")[0]?.closest("details")).not.toHaveAttribute("open");
  await user.type(answers[0]!, "My preserved observation");
  await user.click(screen.getByRole("button", { name: "Fold all" }));
  expect(screen.queryAllByRole("textbox", { name: "Your answer" })).toHaveLength(0);
  await user.click(screen.getByRole("button", { name: "Continuous writing" }));
  expect(screen.getAllByRole("textbox", { name: "Your answer" })).toHaveLength(7);
  expect(screen.getAllByRole("textbox", { name: "Your answer" })[0]).toHaveValue("My preserved observation");
  await waitFor(async () => expect((await api.studySectionList(detail.study.id))[0]?.blocks[0]?.content).toBe("My preserved observation"));
  await user.click(screen.getByRole("button", { name: "Step view" }));
  expect(screen.queryAllByRole("textbox", { name: "Your answer" })).toHaveLength(0);
});
