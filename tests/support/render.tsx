import { render, screen, waitFor, within } from "@testing-library/react";
import { expect } from "vitest";
import userEvent from "@testing-library/user-event";
import { App } from "@/app/App";
import i18n from "@/i18n";
import type { PdfEngine } from "@/lib/pdfEngine";
import type { Updater } from "@/lib/updater";
import { createMemoryApi, type MemoryApi } from "./memoryApi";

/** Window width the app sees; a typical desktop unless a test says otherwise. */
export function setWindowWidth(width: number) {
  Object.defineProperty(window, "innerWidth", { configurable: true, writable: true, value: width });
}

/** Renders the whole app against an in-memory API and waits for the shell. */
export async function renderApp(api: MemoryApi = createMemoryApi(), { width = 1440, updater, pdfEngine }: { width?: number; updater?: Updater; pdfEngine?: PdfEngine } = {}) {
  setWindowWidth(width);
  const user = userEvent.setup();
  render(<App api={api} updater={updater} pdfEngine={pdfEngine} />);
  if (api.state.info.ready) {
    // The Study pane's name is in the interface language.
    await screen.findByRole("main", { name: new RegExp(i18n.t("app.panes.name.study"), "i") });
  }
  return { api, user };
}

/**
 * Opens Settings → one section (by its title). Sections start closed; the
 * section's heading is the button that opens it.
 */
export async function openSettingsSection(user: ReturnType<typeof userEvent.setup>, title: string) {
  if (!screen.queryByRole("heading", { level: 1, name: i18n.t("settings.title") })) {
    await user.click(screen.getByRole("button", { name: i18n.t("app.toolbar.settings") }));
  }
  const region = await screen.findByRole("region", { name: title });
  const toggle = within(region).getByRole("button", { name: title });
  if (toggle.getAttribute("aria-expanded") !== "true") await user.click(toggle);
  return region;
}

/**
 * Opens an information panel (v0.19: Project, Bible Study or Bible Teaching
 * information) from its compact control, named by the panel's title, and
 * returns the panel.
 */
export async function openInfo(user: ReturnType<typeof userEvent.setup>, title: string | RegExp) {
  await user.click(await screen.findByRole("button", { name: title }));
  return screen.findByRole("dialog", { name: title });
}

/** Saves an information panel's draft and waits for the panel to close. */
export async function saveInfo(user: ReturnType<typeof userEvent.setup>, panel: HTMLElement) {
  await user.click(within(panel).getByRole("button", { name: "Save" }));
  await waitFor(() => expect(panel).not.toBeInTheDocument());
}

/** Editor regressions explicitly enter the one named folder they seeded.
 * Home acceptance tests use renderApp directly and assert the actual root view.
 */
export async function renderAppInProject(api: MemoryApi = createMemoryApi(), options: Parameters<typeof renderApp>[1] = {}) {
  const project = api.state.projects.length === 1 ? api.state.projects[0] : null;
  const saved = window.localStorage.getItem("studyPane.location");
  const result = await renderApp(api, options);
  if (project && !saved && !["My studies", "Meine Bibelstudien", "내 성경공부", "Mes études", "Mis estudios", "Meus estudos"].includes(project.title)) {
    const folder = await screen.findByRole("button", { name: `${project.title} ${i18n.t("studies.files.folder")}` });
    await result.user.click(folder);
    await screen.findByRole("heading", { level: 1, name: project.title });
    if (!api.state.studies.some(study => study.projectId === project.id)) await result.user.click(screen.getByRole("button", { name: i18n.t("studies.newStudy.formLabel") }));
  }
  return result;
}
