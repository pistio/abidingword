import { act, screen, within, waitFor } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import i18n from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp, openSettingsSection } from "../support/render";
import { openTeachingOptions } from "../support/studyActions";

afterEach(async () => { window.localStorage.clear(); await act(async () => { await i18n.changeLanguage("en"); }); });
const pane = () => within(screen.getByRole("main", { name: "Study" }));

it("keeps existing standalone Teaching editable without offering general creation", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Compatibility" });
  const prep = await api.teachingCreate({ projectId: project.id, title: "Existing sermon", templateVersion: 2, blocks: [{ blockType: "writing", title: "Main part" }] });
  await api.teachingBlockUpdate({ id: prep.blocks![0]!.id, body: "An independent message." });
  const { user } = await renderApp(api);
  expect(pane().queryByRole("button", { name: "New Teaching" })).toBeNull();
  const data = await openSettingsSection(user, "Data and backup");
  await user.click(within(data).getByText("Legacy standalone Teaching", { selector: "summary" }));
  expect(screen.queryByRole("button", { name: "New Teaching" })).toBeNull();
  await user.click(await screen.findByRole("button", { name: "Existing sermon" }));
  const main = await screen.findByRole("textbox", { name: "Main part" });
  expect(main).toHaveValue("An independent message.");
  await user.type(main, " Retained edit.");
  await waitFor(() => expect(api.state.teaching.preparations[0]!.blocks![0]!.body).toContain("Retained edit."));
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
  await user.click(await pane().findByRole("button", { name: "Messages Project folder" }));
  await user.click(await pane().findByRole("button", { name: /Vine study/ }));
  await user.click(await pane().findByRole("button", { name: "Bible Teaching" }));
  await openTeachingOptions(user);
  await user.click(await pane().findByText(/^Based on: Bible Study/));
  const select = await pane().findByRole("combobox", { name: "Bible Study (optional)" });
  expect(select).toHaveValue(study.id);
  await waitFor(() => expect(select).not.toBeDisabled());
  await user.selectOptions(select, "");
  await user.click(pane().getByRole("button", { name: "Save study link" }));
  await waitFor(() => expect(api.state.teaching.preparations[0]!.studyId).toBeNull());
  expect((await api.teachingGet(teaching.id)).findings[0]).toMatchObject({ snapshot: true, finding: { content: finding.content } });
  await openTeachingOptions(user);
  await user.click(pane().getByText(/^Link to Bible Study/));
  await user.selectOptions(pane().getByRole("combobox", { name: "Bible Study (optional)" }), study.id);
  await user.click(pane().getByRole("button", { name: "Save study link" }));
  await waitFor(() => expect(api.state.teaching.preparations[0]!.studyId).toBe(study.id));
  expect((await api.teachingGet(teaching.id)).findings).toHaveLength(1);
  await openTeachingOptions(user);
  await user.click(pane().getByRole("button", { name: "Bible Teaching information" }));
  await openTeachingOptions(user);
  const info = await screen.findByRole("dialog", { name: "Bible Teaching information" });
  expect(within(info).getByText(/Exact saved words/)).toBeInTheDocument();
  expect(within(info).queryByRole("button", { name: /Remove.*Synthetic source/ })).toBeNull();
});

it.each(["en", "de", "fr", "es", "pt", "ko"])("opens existing standalone Teaching in %s without creating a Study", async (language) => {
  await act(async () => { await i18n.changeLanguage(language); });
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Existing project" });
  await api.teachingCreate({ projectId: project.id, title: "Existing message" });
  const { user } = await renderApp(api);
  const main = within(screen.getByRole("main", { name: i18n.t("app.panes.name.study") }));
  expect(main.queryByRole("button", { name: i18n.t("studies.teaching.newTeaching") })).toBeNull();
  const data = await openSettingsSection(user, i18n.t("settings.sections.data"));
  await user.click(within(data).getByText(i18n.t("studies.files.legacy"), { selector: "summary" }));
  await user.click(await screen.findByRole("button", { name: "Existing message" }));
  await screen.findByRole("textbox", { name: i18n.t("studies.teaching.titleLabel") });
  await openTeachingOptions(user);
  await user.click(await screen.findByText(new RegExp(`^${i18n.t("studies.teaching.workspace.linkStudy")}`)));
  expect(await screen.findByRole("combobox", { name: i18n.t("studies.teaching.workspace.study") })).toHaveValue("");
  expect(api.state.studies).toHaveLength(0);
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
