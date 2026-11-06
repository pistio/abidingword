import { screen, waitFor, within } from "@testing-library/react";
import { expect, it } from "vitest";
import { createMemoryApi } from "../support/memoryApi";
import { openInfo, openSettingsSection, renderAppInProject as renderApp } from "../support/render";
import { openStudyOptions } from "../support/studyActions";

it("confirms a named Study, exposes durable Trash and restores the same identity without relinking Teaching", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Safety project" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Romans research" });
  const teaching = await api.teachingCreate({ studyId: study.id, title: "Saved teaching" });
  const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: /Romans research/ }));
  await openStudyOptions(user);
  const info = await openInfo(user, "Bible Study information");
  await user.click(within(info).getByRole("button", { name: "Move to Trash" }));
  expect((await api.studyGet(study.id)).study.id).toBe(study.id);
  const confirm = within(info).getByRole("alertdialog", { name: "Move “Romans research” to Trash?" });
  await user.click(within(confirm).getByRole("button", { name: "Move to Trash" }));
  await waitFor(() => expect(info).not.toBeInTheDocument());
  const data = await openSettingsSection(user, "Data and backup");
  const trash = within(data).getByRole("region", { name: "Trash" });
  expect(await within(trash).findByRole("heading", { name: study.title })).toBeVisible();
  await user.click(within(trash).getByRole("button", { name: "Restore" }));
  await waitFor(() => expect(within(trash).queryByRole("heading", { name: study.title })).toBeNull());
  expect((await api.studyGet(study.id)).study.id).toBe(study.id);
  expect((await api.teachingGet(teaching.id)).studyId).toBeNull();
});

it("requires the exact Project title before permanently deleting a complete container", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Disposable graph" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Retained study" });
  await api.teachingCreate({ studyId: study.id, title: "Retained teaching" });
  await api.projectDelete(project.id);
  const { user } = await renderApp(api);
  const data = await openSettingsSection(user, "Data and backup");
  const trash = within(data).getByRole("region", { name: "Trash" });
  await user.click(await within(trash).findByRole("button", { name: "Delete permanently" }));
  const confirm = within(trash).getByRole("alertdialog");
  const remove = within(confirm).getByRole("button", { name: "Delete permanently" });
  expect(remove).toBeDisabled();
  await user.type(within(confirm).getByRole("textbox"), "Wrong title");
  expect(remove).toBeDisabled();
  await user.clear(within(confirm).getByRole("textbox"));
  await user.type(within(confirm).getByRole("textbox"), project.title);
  await user.click(remove);
  await waitFor(() => expect(api.state.teaching.preparations).toHaveLength(0));
  expect(await api.trashList(api.state.workspace.id)).toEqual([]);
});

it("opens native language names directly from the workspace globe shortcut", async () => {
  const { user } = await renderApp();
  await user.click(screen.getByRole("button", { name: "Change language" }));
  const languages = await screen.findByRole("combobox", { name: "Interface language" });
  expect(within(languages).getAllByRole("option").map(option => option.textContent)).toEqual(["English", "Deutsch", "한국어", "Français", "Español", "Português"]);
});

it("opens the exact indexed chapter section ahead of a literal mention in the same work", async () => {
  const api = createMemoryApi();
  api.installFixtureBible("FXA", "alpha");
  api.state.settings.scripturePassage = "JHN.15.5";
  api.installFixtureSource({
    title: "Invented chapter guide", format: "markdown", language: "English",
    segments: [{ text: "A passing mention of John 15:5." }, { text: "The indexed chapter opening.", sectionPath: "John 15" }],
    mentions: [{ segment: 0, reference: "JHN.15.5", matchedText: "John 15:5" }],
    sections: [{ segment: 1, reference: "JHN.15", matchedText: "John 15", heading: "John 15", through: 1 }],
  });
  const { user } = await renderApp(api);
  const scripture = within(screen.getByRole("complementary", { name: "Scripture" }));
  await user.type(scripture.getByLabelText("Go to passage"), "John 15{Enter}");
  await user.click(await scripture.findByRole("button", { name: /^Verse 5\. Show verse tools/ }));
  const sources = within(screen.getByRole("complementary", { name: "Sources" }));
  const results = await sources.findByRole("list", { name: "Sources for John 15:5" });
  const work = within(results).getByRole("button", { name: /Invented chapter guide/ });
  expect(work).toHaveTextContent("Heading names John 15");
  expect(work).not.toHaveTextContent("Mentions");
  await user.click(work);
  const reader = await sources.findByRole("region", { name: "Invented chapter guide" });
  await waitFor(() => expect(reader.querySelector("[data-target]")).toHaveTextContent("The indexed chapter opening."));
});
