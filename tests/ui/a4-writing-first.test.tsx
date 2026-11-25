// A.4.3 (#25): Bible Study and Bible Teaching open as writing-first editors.
// The Study title, two tabs, the first section heading and a spacious field
// lead; everything else is one quiet control away and keeps its behaviour.
import { act, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";
import { openBlockOptions, openPartOptions, openStudyOptions, openTeachingOptions } from "../support/studyActions";

const studyPane = () => screen.getByRole("main", { name: i18n.t("app.panes.name.study") });

async function pozetekStudy(language = "de") {
  await act(async () => { await i18n.changeLanguage(language); });
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Organisatorisches Projekt" });
  const template = api.state.templates.find((t) => t.key === "pozetek-writing-study")!;
  await api.studyCreate({ projectId: project.id, title: "Römer 8", templateId: template.id });
  const { user } = await renderApp(api);
  await user.click(within(await screen.findByRole("list", { name: i18n.t("studies.list.listLabel") })).getByRole("button", { name: /Römer 8/ }));
  await within(studyPane()).findByRole("textbox", { name: i18n.t("studies.editor.titleLabel") });
  return { api, user };
}

describe("A.4.3 writing-first Bible Study", () => {
  it("leads with the Study title, two tabs, the first section heading and its writing field (German POZETEK)", async () => {
    await pozetekStudy("de");
    const pane = within(studyPane());
    expect(pane.getByRole("textbox", { name: i18n.t("studies.editor.titleLabel") })).toHaveValue("Römer 8");
    expect(pane.queryByText("Organisatorisches Projekt")).toBeNull(); // the project is not the title
    const tabs = within(pane.getByRole("group", { name: i18n.t("studies.view.label") }));
    expect(tabs.getAllByRole("button").map((b) => b.textContent)).toEqual(["Bibelstudium", "Bibellehre"]);
    // Nothing else is drawn before the writing.
    for (const hidden of ["Angaben", "Schritt für Schritt", "Fortlaufend schreiben", "Deine Antwort", "Hinweise (optional)", "Hinzufügen"]) {
      expect(pane.queryByText(hidden, { exact: true })).toBeNull();
    }
    expect(pane.queryByRole("navigation", { name: "Studienübersicht" })).toBeNull();
    const first = pane.getAllByRole("heading", { level: 2 })[0]!;
    expect(first).toHaveTextContent("Personen");
    const part = first.closest("section")!;
    const field = within(part).getByRole("textbox");
    expect(field).toHaveAccessibleName(/^P — Personen — Deine Antwort$/);
    expect(field).toHaveAttribute("rows", "5");
    // Its only other control is one quiet "…".
    expect(within(part).getByRole("button", { name: i18n.t("studies.sections.options", { title: "P — Personen" }) })).toBeInTheDocument();
    expect(within(part).getByRole("button", { name: i18n.t("studies.flow.hideGuidance") })).toBeInTheDocument();
    expect(field).toHaveClass("resize-y");
    expect(pane.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toContain("O — Ort");
  });

  it("keeps every POZETEK section, writing continuously by default, and its attribution and hints", async () => {
    const { user } = await pozetekStudy("de");
    const pane = within(studyPane());
    expect(pane.getAllByRole("heading", { level: 2 })).toHaveLength(7);
    expect(pane.getAllByRole("textbox", { name: /Deine Antwort$/ })).toHaveLength(7); // continuous: all at once
    await openPartOptions(user, "P — Personen");
    const hints = pane.getByRole("button", { name: i18n.t("studies.flow.guidance") });
    if (hints.getAttribute("aria-pressed") !== "true") await user.click(hints);
    expect(studyPane().querySelector("[data-part-hints]")).not.toBeNull();
    expect(studyPane().querySelector("[data-part-hints]")!.textContent!.length).toBeGreaterThan(10);
  });

  it("puts details, passages, the note filter and the writing mode behind one Study options control", async () => {
    const { user } = await pozetekStudy("en");
    const pane = within(studyPane());
    const toggle = pane.getByRole("button", { name: "Study options" });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
    expect(pane.queryByRole("button", { name: "Bible Study information" })).toBeNull();
    await openStudyOptions(user);
    const options = pane.getByRole("group", { name: "Study options" });
    expect(within(options).getByRole("button", { name: "Bible Study information" })).toBeInTheDocument();
    expect(within(options).getByRole("button", { name: "Continuous writing" })).toHaveAttribute("aria-pressed", "true");
    await user.click(within(options).getByRole("button", { name: "Step-by-step" }));
    expect(pane.getAllByRole("textbox", { name: /Your answer$/ })).toHaveLength(1);
    await user.keyboard("{Escape}");
    await waitFor(() => expect(pane.queryByRole("group", { name: "Study options" })).toBeNull());
    expect(toggle).toHaveFocus();
  });

  it("keeps the save state quiet when saved and visible while saving or failing", async () => {
    const { api, user } = await pozetekStudy("en");
    const pane = within(studyPane());
    const status = studyPane().querySelector<HTMLElement>("[data-save-state]")!;
    expect(status).toHaveTextContent("");
    const original = api.studyBlockUpdate;
    api.studyBlockUpdate = async () => { throw new Error("Disk is full"); };
    await user.type(pane.getAllByRole("textbox", { name: /Your answer$/ })[0]!, "A");
    await waitFor(() => expect(status).toHaveAttribute("data-save-state", "error"), { timeout: 5000 });
    expect(status.className).toMatch(/text-danger/);
    api.studyBlockUpdate = original;
  });
});

describe("A.4.3 writing-first Bible Teaching", () => {
  async function evaTeaching() {
    await act(async () => { await i18n.changeLanguage("en"); });
    const api = createMemoryApi();
    const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "John" });
    await api.studyCreate({ projectId: project.id, title: "Abiding" });
    const { user } = await renderApp(api);
    await user.click(within(await screen.findByRole("list", { name: "Studies" })).getByRole("button", { name: /Abiding/ }));
    await within(studyPane()).findByRole("textbox", { name: "Study title" });
    await user.click(within(studyPane()).getByRole("button", { name: "Bible Teaching" }));
    const form = within(studyPane()).getByRole("form", { name: "Prepare a teaching from this study" });
    await user.click(within(form).getByRole("radio", { name: "Group Bible Study — Observation, Understanding, Application" }));
    await user.click(within(form).getByRole("button", { name: "Start" }));
    const article = await within(studyPane()).findByRole("article", { name: "Group Bible study · Abiding" });
    return { api, user, article };
  }

  it("leads with the session title and the first section heading, with one quiet control per block", async () => {
    const { article } = await evaTeaching();
    const view = within(article);
    // Header: title, Materials icon, one options control; no view switch, no details line.
    expect(view.getByRole("button", { name: "Materials" })).toBeInTheDocument();
    expect(view.getByRole("button", { name: "Teaching options" })).toHaveAttribute("aria-expanded", "false");
    expect(view.queryByRole("radiogroup", { name: "How you write" })).toBeNull();
    expect(view.queryByRole("button", { name: "Bible Teaching information" })).toBeNull();
    expect(view.getAllByRole("heading", { level: 3 })[0]).toHaveTextContent("Observation");
    // No per-block move/duplicate/delete icons, visibility checkboxes or Add bars on screen.
    expect(view.queryByRole("checkbox", { name: "Show this to participants" })).toBeNull();
    expect(view.queryByRole("button", { name: /^Move .* up$/ })).toBeNull();
    expect(view.queryByRole("button", { name: /^Add a block in/ })).toBeNull();
    expect(view.getAllByRole("button", { name: /^Options for / }).length).toBeGreaterThanOrEqual(3);
  });

  it("marks leader-only content persistently and changes visibility only when asked", async () => {
    const { api, user, article } = await evaTeaching();
    const view = within(article);
    const marks = () => article.querySelectorAll("[data-leader-only]").length;
    const before = marks();
    const privateBlocks = () => api.state.teaching.preparations[0]!.blocks!.filter((b) => !b.participantFacing).length;
    const stored = privateBlocks();
    await openBlockOptions(user);
    const boxes = view.getAllByRole("checkbox", { name: "Show this to participants" });
    const shared = boxes.find((b) => (b as HTMLInputElement).checked);
    if (shared) {
      await user.click(shared);
      await waitFor(() => expect(privateBlocks()).toBe(stored + 1));
      await waitFor(() => expect(marks()).toBe(before + 1));
    } else {
      // Every block starts for the leader: making one public removes exactly one mark.
      await user.click(boxes[0]!);
      await waitFor(() => expect(privateBlocks()).toBe(stored - 1));
      await waitFor(() => expect(marks()).toBe(before - 1));
    }
  });

  it("asks before deleting a block, by name, and keeps it on Keep", async () => {
    const { api, user, article } = await evaTeaching();
    const count = api.state.teaching.preparations[0]!.blocks!.length;
    await openBlockOptions(user);
    const remove = within(article).getAllByRole("button", { name: /^Remove / })[0]!;
    const name = remove.getAttribute("aria-label")!.replace(/^Remove /, "");
    await user.click(remove);
    const confirm = within(article).getByRole("alertdialog");
    expect(confirm).toHaveTextContent(name);
    await user.click(within(confirm).getByRole("button", { name: "Keep" }));
    expect(api.state.teaching.preparations[0]!.blocks!).toHaveLength(count);
  });

  it("keeps the view choice and Study link in the teaching's options", async () => {
    const { user, article } = await evaTeaching();
    await openTeachingOptions(user);
    const options = within(article).getByRole("group", { name: "Teaching options" });
    expect(within(options).getByRole("radiogroup", { name: "How you write" })).toBeInTheDocument();
    expect(within(options).getByRole("button", { name: "Bible Teaching information" })).toBeInTheDocument();
  });
});
