import { act, screen, waitFor, within } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import i18n from "@/i18n";
import { derive } from "@/features/materials/derive";
import { createFakePdfEngine } from "../support/fakePdfEngine";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";

const sources = () => within(screen.getByRole("complementary", { name: "Sources" }));
afterEach(async () => { window.localStorage.clear(); await act(async () => { await i18n.changeLanguage("en"); }); });

it.each([false, true])("offers a complete folder import from Add to library, recursive=%s", async (recursive) => {
  const api = createMemoryApi();
  api.state.batch.nextFolder = { path: "/synthetic", files: ["/synthetic/first.md", "/synthetic/nested/second.md"] };
  const { user } = await renderApp(api);
  await user.click(await sources().findByRole("button", { name: "Add to library…" }));
  await user.click(screen.getByRole("button", { name: "Add many sources" }));
  const dialog = within(screen.getByRole("dialog", { name: "Add many sources" }));
  if (!recursive) await user.click(dialog.getByRole("checkbox", { name: "Include files in subfolders" }));
  await user.click(dialog.getByRole("button", { name: "Choose a folder…" }));
  await dialog.findByRole("list", { name: "Files found" });
  expect(dialog.queryByText("second.md") !== null).toBe(recursive);
  await user.click(dialog.getByRole("button", { name: recursive ? "Add 2 sources" : "Add 1 source" }));
  await dialog.findByText(recursive ? "2 sources were added." : "1 source was added.");
  await user.click(dialog.getByRole("button", { name: "Close" }));
  expect(await sources().findByRole("button", { name: "Open library" })).toBeInTheDocument();
  expect(api.state.library.sources).toHaveLength(recursive ? 2 : 1);
});

it("offers multiple individual files without visiting Settings", async () => {
  const api = createMemoryApi();
  api.state.batch.nextFiles = ["/synthetic/first.md", "/synthetic/second.md"];
  const { user } = await renderApp(api);
  await user.click(await sources().findByRole("button", { name: "Add to library…" }));
  await user.click(screen.getByRole("button", { name: "Add many sources" }));
  const dialog = within(screen.getByRole("dialog", { name: "Add many sources" }));
  await user.click(dialog.getByRole("button", { name: "Choose files…" }));
  expect(await dialog.findByText("first.md")).toBeInTheDocument();
  expect(dialog.getByText("second.md")).toBeInTheDocument();
  expect(api.state.library.sources).toHaveLength(0); // review before importing
});

it("uses physical PDF pages in both modes, keeps printed labels separate, and records jumps in history", async () => {
  const api = createMemoryApi();
  const source = api.installFixtureSource({ title: "Page fixture", format: "pdf", segments: [1, 2, 3].map(pageNumber => ({ text: `Synthetic page ${pageNumber}`, pageNumber })) });
  api.state.library.segments.find(segment => segment.sourceId === source.id && segment.pageNumber === 2)!.pageLabel = "iv";
  const engine = createFakePdfEngine({ pages: 3 });
  const { user } = await renderApp(api, { pdfEngine: engine });
  await user.click(await sources().findByRole("button", { name: "Page fixture" }));
  const nav = () => within(screen.getByRole("navigation", { name: "PDF page navigation" }));
  expect(nav().getByRole("button", { name: "Previous PDF page" })).toBeDisabled();
  await user.click(nav().getByRole("button", { name: "Next PDF page" }));
  expect(nav().getByRole("spinbutton", { name: "PDF page" })).toHaveValue(2);
  expect(nav().getByText(/Printed page label: iv/)).toBeInTheDocument();
  await user.click(screen.getByRole("button", { name: "Original" }));
  await waitFor(() => expect(nav().getByRole("spinbutton", { name: "PDF page" })).toHaveValue(2));
  await user.click(nav().getByRole("button", { name: "Next PDF page" }));
  await waitFor(() => expect(nav().getByRole("spinbutton", { name: "PDF page" })).toHaveValue(3));
  expect(nav().getByRole("button", { name: "Next PDF page" })).toBeDisabled();
  await user.click(screen.getByRole("button", { name: "Reading" }));
  await waitFor(() => expect(nav().getByRole("spinbutton", { name: "PDF page" })).toHaveValue(3));
  const input = nav().getByRole("spinbutton", { name: "PDF page" });
  await user.clear(input); await user.type(input, "99");
  expect(nav().getByRole("button", { name: "Go to page" })).toBeDisabled();
  await user.clear(input); await user.type(input, "1");
  await user.click(nav().getByRole("button", { name: "Go to page" }));
  expect(nav().getByRole("spinbutton", { name: "PDF page" })).toHaveValue(1);
  await user.click(screen.getByRole("button", { name: i18n.t("library.history.back") }));
  await waitFor(() => expect(nav().getByRole("spinbutton", { name: "PDF page" })).toHaveValue(3));
  await user.click(screen.getByRole("button", { name: i18n.t("library.history.forward") }));
  await waitFor(() => expect(nav().getByRole("spinbutton", { name: "PDF page" })).toHaveValue(1));
  expect(api.state.library.segments.find(segment => segment.pageNumber === 2)?.pageLabel).toBe("iv");
});

it("shows actual opening history instead of presenting import order as recently opened", async () => {
  const api = createMemoryApi();
  for (const title of ["First imported", "Second imported"]) api.installFixtureSource({ title, format: "text", segments: [{ text: title }] });
  const { user } = await renderApp(api);
  expect(sources().queryByRole("heading", { name: "Recently opened" })).toBeNull();
  await user.click(await sources().findByRole("button", { name: "First imported" }));
  await user.click(await sources().findByRole("button", { name: "Back to Sources" }));
  expect(await sources().findByRole("heading", { name: "Recently opened" })).toBeInTheDocument();
  expect(sources().getByRole("button", { name: "First imported" })).toBeInTheDocument();
  expect(within(sources().getByRole("list", { name: "Recently opened" })).queryByRole("button", { name: "Second imported" })).toBeNull();
});

it.each(["en", "de", "ko", "fr", "es", "pt"])("labels the passage, attributes Ten-Step and avoids a non-German EVA acronym in %s", async (language) => {
  await act(async () => { await i18n.changeLanguage(language); });
  const api = createMemoryApi(); const { user } = await renderApp(api);
  const label = i18n.t("studies.newStudy.subjectLabel.passage");
  expect(await screen.findByRole("textbox", { name: label })).toHaveAttribute("placeholder", i18n.t("studies.newStudy.passagePlaceholder"));
  for (const type of ["word", "topical", "character", "book"] as const) {
    await user.click(screen.getByRole("button", { name: i18n.t(`studies.newStudy.approach.${type}`) }));
    expect(screen.getByRole("textbox", { name: i18n.t(`studies.newStudy.subjectLabel.${type}`) })).toBeVisible();
  }
  await user.click(screen.getByRole("button", { name: i18n.t("studies.newStudy.approach.passage") }));
  const method = await screen.findByRole("combobox", { name: i18n.t("studies.newStudy.template") });
  await waitFor(() => expect(within(method).getAllByRole("option").length).toBeGreaterThan(1));
  await user.selectOptions(method, "ten-step-writing-study");
  expect(screen.getByText(/Helge Stadelmann.*Thomas Richter.*2024/)).toBeInTheDocument();
  if (language === "de") expect(i18n.t("studies.teaching.builtin.eva.name")).toContain("EVA");
  else expect(i18n.t("studies.teaching.builtin.eva.name")).not.toContain("EVA");
});

it("exposes visibility beside the writing while participant exports still omit private content", async () => {
  const api = createMemoryApi(); const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: "New Teaching" }));
  await user.click(await screen.findByRole("button", { name: "Start" }));
  const editor = await screen.findByRole("article", { name: "Sermon" });
  const main = within(editor).getByRole("textbox", { name: "Main part" });
  await user.type(main, "PRIVATE_SENTINEL");
  const block = main.closest('[role="group"]');
  expect(block).not.toBeNull();
  const visibility = within(block as HTMLElement).getByRole("checkbox", { name: "Show this to participants" });
  expect(visibility).toBeVisible();
  if ((visibility as HTMLInputElement).checked) await user.click(visibility);
  await waitFor(() => expect(api.state.teaching.preparations[0]!.blocks!.some(b => b.body === "PRIVATE_SENTINEL" && !b.participantFacing)).toBe(true));
  const prep = await api.teachingGet(api.state.teaching.preparations[0]!.id);
  expect(JSON.stringify(derive(i18n.t.bind(i18n), "worksheet", prep))).not.toContain("PRIVATE_SENTINEL");
});

it("never labels a linked Teaching as independent while the Study list is loading", async () => {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Linked preparation" });
  const { study } = await api.studyCreate({ projectId: project.id, title: "Retained study" });
  const teaching = await api.teachingCreate({ projectId: project.id, studyId: study.id, title: "Linked teaching" });
  const known = await api.studyList({ projectId: project.id });
  const { user } = await renderApp(api);
  await user.click(await screen.findByRole("button", { name: "Teaching sessions" }));
  let release!: (value: typeof known) => void;
  const delayed = new Promise<typeof known>((resolve) => { release = resolve; });
  api.studyList = () => delayed;
  await user.click(await screen.findByRole("button", { name: teaching.title }));
  const editor = await screen.findByRole("article", { name: teaching.title });
  expect(within(editor).getByText("Bible Study (optional) · Bible Study")).toBeVisible();
  expect(within(editor).queryByText("Bible Study (optional) · Independent teaching")).toBeNull();
  await act(async () => release(known));
  expect(await within(editor).findByText("Bible Study (optional) · Retained study")).toBeVisible();
  expect((await api.teachingGet(teaching.id)).studyId).toBe(study.id);
});
