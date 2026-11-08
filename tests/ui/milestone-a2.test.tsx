import { act, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";

afterEach(async () => { localStorage.clear(); await act(async () => { await i18n.changeLanguage("en"); }); });

it.each(["en", "de", "ko", "fr", "es", "pt"])("restores Zeit with optional guidance and its writing field in %s", async language => {
  await act(async () => { await i18n.changeLanguage(language); });
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Recovery" });
  const { study, sections } = await api.studyCreate({ projectId: project.id, templateId: api.state.templates.find(template => template.key === "pozetek-writing-study")!.id, title: "Synthetic method" });
  const time = sections.find(part => part.key === "time")!;
  const kept = sections[0]!;
  await api.studyBlockUpdate({ id: kept.blocks[0]!.id, content: "Retained user writing" });
  await api.studySectionDelete({ id: time.id, force: false });
  const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: /^Synthetic method/ }));
  await user.click(await screen.findByText(i18n.t("studies.parts.restoreMissing")));
  await user.click(screen.getByRole("button", { name: i18n.t("studies.parts.restoreOne", { title: i18n.t("studies.builtin.templates.pozetek-writing-study.sections.time.title") }) }));
  await waitFor(() => expect(api.state.sections.filter(part => part.studyId === study.id && part.key === "time")).toHaveLength(1));
  const result = await api.studyGet(study.id);
  const restored = result.sections.find(part => part.key === "time")!;
  expect(restored.id).not.toBe(time.id);
  expect(restored.fields).toEqual(time.fields);
  expect(restored.guidance).toEqual(time.guidance);
  expect(restored.blocks).toHaveLength(1);
  expect(restored.blocks[0]).toMatchObject({ fieldKey: "answer", content: "" });
  expect(result.sections.find(part => part.id === kept.id)!.blocks[0]!.content).toBe("Retained user writing");
});

it.each(["pozetek-study", "pozetek-writing-study", "ten-step-exegesis", "ten-step-writing-study"])("can restore every original section after all were removed from %s", async templateKey => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Recovery" });
  const original = await api.studyCreate({ projectId: project.id, templateId: api.state.templates.find(template => template.key === templateKey)!.id, title: "Entire method removed" });
  for (const section of original.sections) await api.studySectionDelete({ id: section.id, force: true });
  const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: /^Entire method removed/ }));
  await user.click(await screen.findByText("Restore missing sections"));
  await user.click(screen.getByRole("button", { name: "Restore all missing sections" }));
  await waitFor(() => expect(api.state.sections.filter(part => part.studyId === original.study.id)).toHaveLength(original.sections.length));
  const restored = (await api.studyGet(original.study.id)).sections;
  expect(restored.map(part => part.key)).toEqual(original.sections.map(part => part.key));
  expect(restored.every(part => part.blocks.every(block => block.content === ""))).toBe(true);
});

it("confirms Study deletion from its row, offers Undo and exposes workspace Trash", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Rows" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Named row" });
  const teaching = await api.teachingCreate({ studyId: study.id, title: "Retained teaching" });
  const { user } = await renderApp(api);
  const actions = await screen.findByRole("group", { name: "Actions for “Named row”" });
  await user.click(within(actions).getByRole("button", { name: "File actions" }));
  await user.click(screen.getByRole("menuitem", { name: "Move to Trash" }));
  const confirm = await screen.findByRole("alertdialog", { name: "Move “Named row” to Trash?" });
  expect((await api.studyGet(study.id)).study.id).toBe(study.id);
  await user.click(within(confirm).getByRole("button", { name: "Move to Trash" }));
  const undo = await screen.findByRole("button", { name: "Undo" });
  await waitFor(() => expect(api.state.studies.some(item => item.id === study.id)).toBe(false));
  await user.click(undo);
  await waitFor(() => expect(api.state.studies.some(item => item.id === study.id)).toBe(true));
  expect((await api.teachingGet(teaching.id)).studyId).toBeNull();
  await user.click(screen.getByRole("button", { name: "Trash" }));
  expect(await screen.findByRole("dialog", { name: "Trash" })).toBeVisible();
});

it("offers Project row confirmation with descendants explained and no one-click deletion", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Named container" });
  await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Other container" });
  await api.studyCreate({ projectId: project.id, title: "Descendant row" });
  const { user } = await renderApp(api);
  const actions = await screen.findByRole("group", { name: "Actions for “Named container”" });
  await user.click(within(actions).getByRole("button", { name: "File actions" }));
  await user.click(screen.getByRole("menuitem", { name: "Move to Trash" }));
  const dialog = await screen.findByRole("dialog", { name: "Move “Named container” to Trash?" });
  expect(dialog).toHaveTextContent("The Project and its Studies and Teaching are hidden together");
  await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
  expect((await api.projectGet(project.id)).id).toBe(project.id);
  expect(screen.queryByRole("button", { name: /Descendant row/ })).toBeNull();
  await user.click(within(actions).getByRole("button", { name: "File actions" }));
  await user.click(screen.getByRole("menuitem", { name: "Move to Trash" }));
  const confirmed = await screen.findByRole("alertdialog", { name: "Move “Named container” to Trash?" });
  await user.click(within(confirmed).getByRole("button", { name: "Move to Trash" }));
  await waitFor(() => expect(screen.queryByRole("button", { name: /Descendant row/ })).toBeNull());
  await user.click(screen.getByRole("button", { name: "Undo" }));
  await user.click(await screen.findByRole("button", { name: "Named container Project folder" }));
  expect(await screen.findByRole("button", { name: /Descendant row/ })).toBeVisible();
});

it.each(["source", "segment"])("rejects a mismatched indexed %s instead of displaying it under the requested citation", async mismatch => {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  api.state.settings.scripturePassage = "JHN.15";
  const intended = api.installFixtureSource({ title: "Intended source", format: "text", segments: [{ text: "A reference to John 15:5." }, { text: "UNRELATED_SEGMENT_SENTINEL" }], mentions: [{ segment: 0, reference: "JHN.15.5", matchedText: "John 15:5" }] });
  const other = api.installFixtureSource({ title: "Other source", format: "text", segments: [{ text: "WRONG_SOURCE_SENTINEL" }] });
  const open = api.libraryOpenSegment;
  api.libraryOpenSegment = async id => { const window = await open(id); return mismatch === "source" ? { ...window, sourceId: other.id } : { ...window, targetSegmentId: window.segments[1]!.id }; };
  const { user } = await renderApp(api);
  const sources = within(screen.getByRole("complementary", { name: "Sources" }));
  expect(await sources.findByText(/Only references or mentions are indexed here/)).toBeVisible();
  await user.click(sources.getByRole("button", { name: new RegExp(intended.title) }));
  expect(await sources.findByRole("alert")).toHaveTextContent("The indexed location does not belong to this source or is unavailable.");
  expect(sources.queryByText("WRONG_SOURCE_SENTINEL")).toBeNull();
  expect(sources.queryByText("A reference to John 15:5.")).toBeNull();
});

it("offers Undo only after the backend confirms a removed method section was empty", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Empty method Undo" });
  const template = api.state.templates.find(template => template.key === "pozetek-writing-study")!;
  const original = await api.studyCreate({ projectId: project.id, templateId: template.id, title: "Undo method" });
  const time = original.sections.find(part => part.key === "time")!;
  const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: /^Undo method/ }));
  await user.click(await screen.findByRole("button", { name: i18n.t("studies.sections.options", { title: "Time" }) }));
  await user.click(screen.getByRole("button", { name: i18n.t("studies.sections.delete", { title: "Time" }) }));
  const confirmation = await screen.findByRole("alertdialog", { name: "Delete part Time?" });
  await user.click(within(confirmation).getByRole("button", { name: "Delete part" }));
  await user.click(await screen.findByRole("button", { name: "Undo" }));
  await waitFor(() => expect(api.state.sections.filter(part => part.studyId === original.study.id && part.key === "time")).toHaveLength(1));
  const restored = (await api.studyGet(original.study.id)).sections.find(part => part.key === "time")!;
  expect(restored.id).not.toBe(time.id);
  expect(restored.blocks.every(block => block.content === "")).toBe(true);
});

it("waits for pending writing saves before restoring method structure", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Pending saves" });
  const template = api.state.templates.find(template => template.key === "pozetek-writing-study")!;
  const original = await api.studyCreate({ projectId: project.id, templateId: template.id, title: "Pending writing" });
  await api.studySectionDelete({ id: original.sections.find(section => section.key === "time")!.id, force: false });
  const save = api.studyBlockUpdate;
  let release!: () => void;
  api.studyBlockUpdate = async input => { await new Promise<void>(resolve => { release = resolve; }); return save(input); };
  const restore = vi.spyOn(api, "studySectionRestoreMissing");
  const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: /^Pending writing/ }));
  const answer = (await screen.findAllByRole("textbox", { name: /Your answer$/ }))[0]!;
  await user.type(answer, "Pending research sentinel");
  await user.click(screen.getByText("Restore missing sections"));
  await user.click(screen.getByRole("button", { name: "Restore “Time”" }));
  await waitFor(() => expect(release).toBeTypeOf("function"));
  expect(restore).not.toHaveBeenCalled();
  await act(async () => release());
  await waitFor(() => expect(restore).toHaveBeenCalledTimes(1));
  await waitFor(async () => expect((await api.studyGet(original.study.id)).sections).toHaveLength(7));
  expect((await api.studyGet(original.study.id)).sections[0]!.blocks[0]!.content).toBe("Pending research sentinel");
  expect(screen.getAllByRole("textbox", { name: /Your answer$/ })[0]).toHaveValue("Pending research sentinel");
});
