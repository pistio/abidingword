import { act, screen, within, waitFor } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import i18n from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";

afterEach(async () => { window.localStorage.clear(); await act(async () => { await i18n.changeLanguage("en"); }); });
const pane = () => within(screen.getByRole("main", { name: "Study" }));

it("starts Teaching with a project and no Study, using the existing block editor", async () => {
  const api = createMemoryApi();
  const { user } = await renderApp(api);
  await user.click(await pane().findByRole("button", { name: "New Teaching" }));
  expect(await pane().findByRole("form", { name: "New Teaching" })).toBeInTheDocument();
  expect(api.state.studies).toHaveLength(0);
  expect(api.state.projects).toHaveLength(1);
  expect(pane().getByRole("combobox", { name: "Bible Study (optional)" })).toHaveValue("");
  const radios = pane().getAllByRole("radio");
  expect(radios).toHaveLength(4); // EVA, sermon/talk, children/youth, blank
  await user.click(pane().getByRole("radio", { name: "Sermon / Talk" }));
  await user.click(pane().getByRole("button", { name: "Start" }));
  const article = await pane().findByRole("article", { name: "Sermon" });
  await user.type(within(article).getByRole("textbox", { name: "Main part" }), "An independent message.");
  await waitFor(() => expect(api.state.teaching.preparations[0]!.blocks!.some(b => b.body === "An independent message.")).toBe(true));
  expect(api.state.teaching.preparations[0]!.studyId).toBeNull();
  await user.click(pane().getByRole("button", { name: "Back to Teaching list" }));
  await user.click(await pane().findByRole("button", { name: "Sermon" }));
  expect(await pane().findByRole("textbox", { name: "Main part" })).toHaveValue("An independent message.");
  expect(api.state.studies).toHaveLength(0);
});

it("links and unlinks an existing Teaching without replacing its manuscript or saved notes", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Messages" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Vine study" });
  const finding = await api.findingCreate({ studyId: study.id, findingType: "observation", content: "Saved vine observation" });
  api.state.findings[0]!.sources.push({id:"saved-citation",findingId:finding.id,sourceId:null,segmentId:null,title:"Synthetic source",author:null,locator:"p. 7",removed:true,unavailable:true,quote:"Exact saved words",section:null,annotationId:null});
  const teaching = await api.teachingCreate({ projectId: project.id, studyId: study.id, title: "Independent manuscript" });
  await api.teachingFindingLink({ preparationId: teaching.id, findingId: finding.id });
  const { user } = await renderApp(api);
  await user.click(await pane().findByRole("button", { name: "Teaching sessions" }));
  await user.click(await pane().findByRole("button", { name: teaching.title }));
  await user.click(pane().getByText(/^Bible Study \(optional\) ·/));
  const select = await pane().findByRole("combobox", { name: "Bible Study (optional)" });
  expect(select).toHaveValue(study.id);
  await user.selectOptions(select, "");
  await user.click(pane().getByRole("button", { name: "Save study link" }));
  await waitFor(() => expect(api.state.teaching.preparations[0]!.studyId).toBeNull());
  expect((await api.teachingGet(teaching.id)).findings[0]).toMatchObject({ snapshot: true, finding: { content: finding.content } });
  await user.click(pane().getByText(/^Bible Study \(optional\) ·/));
  await user.selectOptions(pane().getByRole("combobox", { name: "Bible Study (optional)" }), study.id);
  await user.click(pane().getByRole("button", { name: "Save study link" }));
  await waitFor(() => expect(api.state.teaching.preparations[0]!.studyId).toBe(study.id));
  expect((await api.teachingGet(teaching.id)).findings).toHaveLength(1);
  await user.click(pane().getByRole("button", { name: "Bible Teaching information" }));
  const info = await screen.findByRole("dialog", { name: "Bible Teaching information" });
  expect(within(info).getByText(/Exact saved words/)).toBeInTheDocument();
  expect(within(info).queryByRole("button", { name: /Remove.*Synthetic source/ })).toBeNull();
});

it.each(["en", "de", "fr", "es", "pt", "ko"])("opens standalone Teaching in %s without creating a Study", async (language) => {
  await act(async () => { await i18n.changeLanguage(language); });
  const api = createMemoryApi(); const { user } = await renderApp(api);
  const main = within(screen.getByRole("main", { name: i18n.t("app.panes.name.study") }));
  await user.click(await main.findByRole("button", { name: i18n.t("studies.teaching.newTeaching") }));
  expect(await main.findByRole("combobox", { name: i18n.t("studies.teaching.workspace.study") })).toHaveValue("");
  expect(main.getAllByRole("radio")).toHaveLength(4);
  expect(api.state.studies).toHaveLength(0);
  expect(api.state.projects).toHaveLength(1);
});

it("keeps trashed Teaching after Study deletion and removes it when a trashed project is permanently deleted", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({workspaceId:api.state.workspace.id,title:"Disposable project"});
  const {study} = await api.studyCreate({projectId:project.id,title:"Disposable study"});
  const t = await api.teachingCreate({projectId:project.id,studyId:study.id,title:"Retained teaching"});
  await api.teachingDelete(t.id);
  await api.studyDelete(study.id);
  expect((await api.teachingRestore(t.id)).studyId).toBeNull();
  await api.studyRestore(study.id);
  await api.teachingSetStudy(t.id,study.id);
  await api.teachingDelete(t.id);
  await api.projectDelete(project.id);
  await expect(api.studyGet(study.id)).rejects.toThrow();
  await api.projectDelete(project.id,true);
  await expect(api.teachingRestore(t.id)).rejects.toThrow();
  expect(api.state.teaching.deletedPreparations.size).toBe(0);
});
