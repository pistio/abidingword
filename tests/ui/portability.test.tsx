import { act, cleanup, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ApiError } from "@/lib/api";
import { databaseRestoreActive } from "@/lib/databaseRestoreLock";
import { guardApiDuringDatabaseRestore } from "@/lib/guardedApi";
import { createMemoryApi } from "../support/memoryApi";
import { openInfo, openSettingsSection, renderAppInProject as renderApp } from "../support/render";

// Settings › Data and backup (v0.19): export one project, import one as a
// new project, back up everything, and restore everything after a clear
// warning. Each is its own action. All text is invented for the tests.

async function setup() {
  const api = createMemoryApi();
  await api.projectCreate({ workspaceId: api.state.workspace.id, title: "John" });
  const romans = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Romans" });
  await api.studyCreate({ projectId: romans.id, title: "Justified" });
  return { api, romansId: romans.id };
}

const data = async (user: Parameters<typeof openSettingsSection>[0]) => openSettingsSection(user, "Data and backup");

describe("Data and backup", () => {
  it("is one section of Settings with four separate actions", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    expect(within(section).getByRole("button", { name: "Data and backup" })).toHaveAccessibleDescription("Export, import and back up your work");
    for (const name of ["Export project…", "Import project…", "Back up everything…", "Restore from backup…"]) {
      expect(within(section).getByRole("button", { name: new RegExp(name) })).toBeVisible();
    }
    // What a backup never holds, and that it is private.
    expect(within(section).getByText(/API keys are never included/)).toBeVisible();
    expect(within(section).getByText(/a private copy for you, not for sharing/)).toBeVisible();
  });

  it("gives an open section a quiet shaded ground and leaves closed ones plain", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    expect(section).toHaveAttribute("data-open", "true");
    expect(section.className).toContain("bg-bg-secondary");
    const language = screen.getByRole("region", { name: "Language" });
    expect(language).not.toHaveAttribute("data-open");
    expect(language.className).not.toContain("bg-bg-secondary");
  });

  it("exports the chosen project to the file the save dialog named", async () => {
    const { api, romansId } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    await user.selectOptions(within(section).getByRole("combobox", { name: "Project to export" }), romansId);
    api.state.portability.nextSave = "/home/test/Documents/Romans";
    await user.click(within(section).getByRole("button", { name: /Export project…/ }));
    expect(await within(section).findByText("“Romans” was saved to /home/test/Documents/Romans.awproject.")).toBeInTheDocument();
    expect(within(section).getByText("Bible Studies: 1")).toBeInTheDocument();
    expect(api.state.portability.exported).toEqual([{ projectId: romansId, path: "/home/test/Documents/Romans.awproject" }]);
  });

  it("does nothing when the save dialog is cancelled", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    const exportProject = vi.spyOn(api, "projectExport");
    api.state.portability.nextSave = null;
    await user.click(within(section).getByRole("button", { name: /Export project…/ }));
    expect(exportProject).not.toHaveBeenCalled();
    expect(within(section).queryByRole("status")).toBeNull();
  });

  it("shows what a project file holds and lacks before importing it as a new project", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    const importProject = vi.spyOn(api, "projectImport");
    await user.click(within(section).getByRole("button", { name: /Import project…/ }));
    const preview = await within(section).findByRole("region", { name: "Project to import" });
    // "John" exists here, so the import is a new project beside it.
    expect(within(preview).getByText("It will be added as a new project, “John (2)”. Nothing on this computer is replaced.")).toBeInTheDocument();
    expect(within(preview).getByText(/Bible Studies: 2 · Bible Teachings: 1/)).toBeInTheDocument();
    expect(within(preview).getByText("Not on this computer")).toBeInTheDocument();
    expect(within(preview).getByText("Library: Commentary on John")).toBeInTheDocument();
    expect(importProject).not.toHaveBeenCalled();

    await user.click(within(preview).getByRole("button", { name: "Import" }));
    expect(await within(section).findByText("“John (2)” was added. You’ll find it under Projects.")).toBeInTheDocument();
    expect(within(section).getByText("Citations kept whose library book is not on this computer: 1")).toBeInTheDocument();
    expect(within(section).queryByText(/could not be read/)).toBeNull();
    expect(api.state.projects.map((p) => p.title)).toEqual(["John", "Romans", "My studies", "John (2)"]);
  });

  it("adds nothing when the preview is cancelled", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    await user.click(within(section).getByRole("button", { name: /Import project…/ }));
    const preview = await within(section).findByRole("region", { name: "Project to import" });
    await user.click(within(preview).getByRole("button", { name: "Cancel" }));
    expect(within(section).queryByRole("region", { name: "Project to import" })).toBeNull();
    expect(api.state.projects).toHaveLength(3);
  });

  it("says why a file cannot be imported, and changes nothing", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    api.state.portability.package = new ApiError("importFailed", "Part of this file does not match its list of contents: it is damaged or was changed. Nothing was changed here.");
    await user.click(within(section).getByRole("button", { name: /Import project…/ }));
    expect(await within(section).findByRole("alert")).toHaveTextContent(/damaged or was changed/);
    expect(within(section).queryByRole("region", { name: "Project to import" })).toBeNull();
    expect(api.state.projects).toHaveLength(3);
  });

  it("backs up everything to the file the save dialog named", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    api.state.portability.nextSave = "/home/test/Backups/All.awbackup";
    await user.click(within(section).getByRole("button", { name: /Back up everything…/ }));
    expect(await within(section).findByText("The backup was saved to /home/test/Backups/All.awbackup (1.2 MB).")).toBeInTheDocument();
    expect(api.state.portability.backups).toEqual(["/home/test/Backups/All.awbackup"]);
  });

  it.each(["backup", "project"] as const)("waits for current Study writing before creating a %s file", async (kind) => {
    const { api, romansId } = await setup();
    api.state.portability.nextSave = kind === "backup" ? "/home/test/All.awbackup" : "/home/test/Romans.awproject";
    let release!: () => void;
    const pending = new Promise<void>(resolve => { release = resolve; });
    const originalSave = api.studyUpdate;
    const save = vi.spyOn(api, "studyUpdate").mockImplementation(async input => {
      await pending;
      return originalSave(input);
    });
    let titleAtCapture: string | undefined;
    const originalBackup = api.backupCreate;
    const originalExport = api.projectExport;
    const capture = kind === "backup"
      ? vi.spyOn(api, "backupCreate").mockImplementation(async path => {
        titleAtCapture = api.state.studies[0]?.title;
        return originalBackup(path);
      })
      : vi.spyOn(api, "projectExport").mockImplementation(async input => {
        titleAtCapture = api.state.studies[0]?.title;
        return originalExport(input);
      });
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Romans/ }));
    await user.click(await screen.findByRole("button", { name: /Justified/ }));
    const title = await screen.findByRole("textbox", { name: "Study title" });
    await user.clear(title);
    await user.type(title, "Invented writing for this backup");
    const section = await data(user);
    await waitFor(() => expect(save).toHaveBeenCalled());
    if (kind === "project") await user.selectOptions(within(section).getByRole("combobox", { name: "Project to export" }), romansId);
    await user.click(within(section).getByRole("button", { name: kind === "backup" ? /Back up everything…/ : /Export project…/ }));
    expect(capture).not.toHaveBeenCalled();
    expect(databaseRestoreActive()).toBe(false);
    await act(async () => { release(); await pending; });
    await waitFor(() => expect(capture).toHaveBeenCalledTimes(1));
    expect(titleAtCapture).toBe("Invented writing for this backup");
    expect(within(section).getByText(/was saved/)).toBeVisible();
  });

  it.each(["backup", "project"] as const)("refuses a %s file while Study writing still fails to save", async (kind) => {
    const { api, romansId } = await setup();
    vi.spyOn(api, "studyUpdate").mockRejectedValue(new ApiError("filesystem", "Invented disk-full save failure"));
    const capture = kind === "backup" ? vi.spyOn(api, "backupCreate") : vi.spyOn(api, "projectExport");
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Romans/ }));
    await user.click(await screen.findByRole("button", { name: /Justified/ }));
    const title = await screen.findByRole("textbox", { name: "Study title" });
    await user.clear(title);
    await user.type(title, "Unsaved invented Study writing");
    const section = await data(user);
    if (kind === "project") await user.selectOptions(within(section).getByRole("combobox", { name: "Project to export" }), romansId);
    await user.click(within(section).getByRole("button", { name: kind === "backup" ? /Back up everything…/ : /Export project…/ }));
    expect(await within(section).findByRole("alert")).toHaveTextContent(/Some changes could not be saved/);
    expect(capture).not.toHaveBeenCalled();
    expect(databaseRestoreActive()).toBe(false);
    expect(api.state.portability.backups).toEqual([]);
    expect(api.state.portability.exported).toEqual([]);
  });

  it.each(["backup", "project"] as const)("handles a rejected %s save dialog without creating a file or locking editing", async (kind) => {
    const { api } = await setup();
    vi.spyOn(api, "choosePortableSaveFile").mockRejectedValueOnce(new ApiError("filesystem", "The invented destination could not be opened."));
    const capture = kind === "backup" ? vi.spyOn(api, "backupCreate") : vi.spyOn(api, "projectExport");
    const { user } = await renderApp(api);
    const section = await data(user);
    await user.click(within(section).getByRole("button", { name: kind === "backup" ? /Back up everything…/ : /Export project…/ }));
    expect(await within(section).findByRole("alert")).toHaveTextContent("The invented destination could not be opened.");
    expect(capture).not.toHaveBeenCalled();
    expect(databaseRestoreActive()).toBe(false);
    expect(within(section).getByRole("button", { name: kind === "backup" ? /Back up everything…/ : /Export project…/ })).toBeEnabled();
  });

  it("restores only after the warning is confirmed, and says where the safety copy is", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    api.state.portability.nextOpen = "/home/test/Backups/All.awbackup";

    // First look, then cancel: nothing restored.
    await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
    let preview = await within(section).findByRole("region", { name: "Backup to restore" });
    expect(within(preview).getByText(/replaces your current work and settings/)).toBeInTheDocument();
    expect(within(preview).getByText(/Projects: 3 · Bible Studies: 12/)).toBeInTheDocument();
    // The safe choice has the focus.
    expect(within(preview).getByRole("button", { name: "Cancel" })).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(api.state.portability.restored).toEqual([]);
    expect(within(section).getByRole("button", { name: /Restore from backup…/ })).toHaveFocus();
    expect(within(section).queryByRole("region", { name: "Backup to restore" })).not.toBeInTheDocument();

    await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
    preview = await within(section).findByRole("region", { name: "Backup to restore" });
    await user.click(within(preview).getByRole("button", { name: "Replace everything with this backup" }));
    const finished = await screen.findByRole("dialog", { name: "The backup was restored." });
    expect(within(finished).getByText(/A safety copy of your previous database is kept at .*pre-restore/)).toBeInTheDocument();
    expect(api.state.portability.restored).toEqual(["/home/test/Backups/All.awbackup"]);
    expect(within(finished).getByRole("button", { name: "Reload Abiding Word" })).toHaveFocus();
    expect(finished).toHaveAccessibleDescription(/Editing stays paused/);
    await user.tab();
    expect(within(finished).getByRole("button", { name: "Reload Abiding Word" })).toHaveFocus();
    await user.tab({ shift: true });
    await user.keyboard("{Escape}");
    expect(finished).toBeInTheDocument();
    expect(within(finished).getByRole("button", { name: "Reload Abiding Word" })).toHaveFocus();
    expect(databaseRestoreActive()).toBe(true);
  });

  it("shows a backup that cannot be read as an error, with nothing restored", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    api.state.portability.backup = new ApiError("importFailed", "This file was saved by a newer version of Abiding Word (0.30.0). Please update Abiding Word first.");
    await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
    expect(await within(section).findByRole("alert")).toHaveTextContent(/newer version of Abiding Word/);
    expect(api.state.portability.restored).toEqual([]);
  });

  it("reports a native backup picker failure and allows a safe retry or cancellation", async () => {
    const { api } = await setup();
    vi.spyOn(api, "choosePortableFile").mockRejectedValueOnce(new ApiError("filesystem", "The invented backup folder could not be opened."));
    const { user } = await renderApp(api);
    const section = await data(user);
    await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
    expect(await within(section).findByRole("alert")).toHaveTextContent("The invented backup folder could not be opened.");
    expect(databaseRestoreActive()).toBe(false);
    await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
    const preview = await within(section).findByRole("region", { name: "Backup to restore" });
    await user.click(within(preview).getByRole("button", { name: "Cancel" }));
    expect(within(section).getByRole("button", { name: /Restore from backup…/ })).toHaveFocus();
    expect(api.state.portability.restored).toEqual([]);
  });

  it("unlocks after a failed full restore with a successful rollback and permits an explicit retry", async () => {
    const { api } = await setup();
    const restore = vi.spyOn(api, "backupRestore").mockRejectedValueOnce(new ApiError("backup", "The backup could not be restored. Your current work was put back as it was.", "portability.restoreFailed"));
    const { user } = await renderApp(api);
    const section = await data(user);
    await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
    await user.click(await within(section).findByRole("button", { name: "Replace everything with this backup" }));
    const failure = await within(section).findByRole("alert");
    expect(failure).toHaveTextContent(/current work was put back/);
    expect(failure.parentElement).toHaveFocus();
    expect(databaseRestoreActive()).toBe(false);
    expect(api.state.studies[0]?.title).toBe("Justified");
    expect(api.state.portability.restored).toEqual([]);
    await user.click(within(section).getByRole("button", { name: "Replace everything with this backup" }));
    await screen.findByRole("dialog", { name: "The backup was restored." });
    expect(restore).toHaveBeenCalledTimes(2);
    expect(api.state.portability.restored).toHaveLength(1);
  });
  it("shows verified daily recovery points, refuses corrupt ones and requires explicit confirmation", async () => {
    const { api } = await setup();
    const good = "abiding-word-v27-2026-11-20T10-20-00.123Z.db";
    const broken = "abiding-word-v26-2026-11-19T10-20-00.123Z.db";
    api.state.portability.dailySnapshots = [
      { id: good, createdAt: "2026-11-20T10:20:00.123Z", schemaVersion: 27, bytes: 1_500_000, usable: true },
      { id: broken, createdAt: "2026-11-19T10:20:00.123Z", schemaVersion: 26, bytes: 900_000, usable: false },
    ];
    const { user } = await renderApp(api);
    const section = await data(user);
    expect(within(section).getByText(/keeps up to seven verified copies/)).toBeVisible();
    await user.click(within(section).getByRole("button", { name: "Show recovery points" }));
    const list = await within(section).findByRole("list", { name: "Automatic recovery points" });
    expect(within(list).getByText("Available")).toBeInTheDocument();
    expect(within(list).getByText(/damaged or incompatible version/)).toBeInTheDocument();
    const buttons = within(list).getAllByRole("button", { name: "Review restore…" });
    expect(buttons).toHaveLength(2);
    expect(buttons[0]).toBeEnabled();
    expect(buttons[1]).toBeDisabled();
    expect(buttons[0]).toHaveAccessibleDescription(/database v27 Available/);
    expect(buttons[1]).toHaveAccessibleDescription(/database v26 Unavailable/);

    // Inspection and cancellation cannot change the native database.
    await user.click(buttons[0]!);
    const preview = await within(section).findByRole("region", { name: "Recovery point to restore" });
    expect(within(preview).getByText(/Imported source files, Bible files and images stay unchanged/)).toBeVisible();
    expect(within(preview).getByText(/verified safety copy/)).toBeVisible();
    expect(within(preview).getByRole("button", { name: "Cancel" })).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(api.state.portability.dailyRestored).toEqual([]);

    const reopened = within(section).getByRole("list", { name: "Automatic recovery points" });
    expect(within(reopened).getAllByRole("button", { name: "Review restore…" })[0]).toHaveFocus();
    await user.click(within(reopened).getAllByRole("button", { name: "Review restore…" })[0]!);
    await user.click(within(section).getByRole("button", { name: "Restore this database copy" }));
    const finished = await screen.findByRole("dialog", { name: /The database was restored/ });
    expect(within(finished).getByText(/pre-restore/)).toBeInTheDocument();
    expect(within(finished).getByRole("button", { name: "Reload Abiding Word" })).toHaveFocus();
    expect(api.state.portability.dailyRestored).toEqual([good]);
  });

  it.each(["backup", "daily"] as const)("%s restore settles an actual Study edit and keeps the successful restore locked after the UI unmounts", async (kind) => {
    const { api } = await setup();
    const id = "abiding-word-v27-2026-11-20T10-00-00.000Z.db";
    api.state.portability.dailySnapshots = [{ id, createdAt: "2026-11-20T10:00:00Z", schemaVersion: 27, bytes: 500_000, usable: true }];
    let release!: () => void;
    const pending = new Promise<void>(resolve => { release = resolve; });
    const originalSave = api.studyUpdate;
    const save = vi.spyOn(api, "studyUpdate").mockImplementation(async input => {
      await pending;
      return originalSave(input);
    });
    const replace = kind === "backup" ? vi.spyOn(api, "backupRestore") : vi.spyOn(api, "dailySnapshotRestore");
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Romans/ }));
    await user.click(await screen.findByRole("button", { name: /Justified/ }));
    const title = await screen.findByRole("textbox", { name: "Study title" });
    await user.clear(title);
    await user.type(title, "Latest invented Study writing");
    const section = await data(user);
    await waitFor(() => expect(save).toHaveBeenCalled());
    if (kind === "backup") {
      await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
      await user.click(await within(section).findByRole("button", { name: "Replace everything with this backup" }));
    } else {
      await user.click(within(section).getByRole("button", { name: "Show recovery points" }));
      await user.click(await within(section).findByRole("button", { name: "Review restore…" }));
      await user.click(within(section).getByRole("button", { name: "Restore this database copy" }));
    }
    const progress = await screen.findByRole("dialog", { name: "Saving current work and restoring…" });
    expect(progress).toHaveAccessibleDescription("Keep Abiding Word open until recovery has finished.");
    expect(replace).not.toHaveBeenCalled();
    expect(databaseRestoreActive()).toBe(true);
    await user.tab();
    await user.keyboard("{Escape}");
    expect(progress).toHaveFocus();
    expect(progress).toBeInTheDocument();

    await act(async () => { release(); await pending; });
    await screen.findByRole("button", { name: "Reload Abiding Word" });
    expect(replace).toHaveBeenCalledTimes(1);
    expect(api.state.studies.find(study => study.title === "Latest invented Study writing")).toBeDefined();
    const writes = save.mock.calls.length;
    cleanup();
    expect(databaseRestoreActive()).toBe(true);
    await expect(guardApiDuringDatabaseRestore(api).studyUpdate({ id: api.state.studies[0]!.id, title: "Stale state must not return" })).rejects.toMatchObject({ message: expect.stringMatching(/Reload Abiding Word/) });
    expect(save).toHaveBeenCalledTimes(writes);
  });

  it.each(["backup", "daily"] as const)("%s rollback failure retains its safety-copy path and never unlocks stale editing", async (kind) => {
    const { api } = await setup();
    const id = "abiding-word-v27-2026-11-20T10-00-00.000Z.db";
    api.state.portability.dailySnapshots = [{ id, createdAt: "2026-11-20T10:00:00Z", schemaVersion: 27, bytes: 500_000, usable: true }];
    const failure = new ApiError("backup", "Recovery could not put your earlier work back. Safety copy: C:\\Invented\\backups\\pre-restore\\safe.db", "portability.rollbackFailed");
    if (kind === "backup") vi.spyOn(api, "backupRestore").mockRejectedValue(failure);
    else vi.spyOn(api, "dailySnapshotRestore").mockRejectedValue(failure);
    const { user } = await renderApp(api);
    const section = await data(user);
    if (kind === "backup") {
      await user.click(within(section).getByRole("button", { name: /Restore from backup…/ }));
      await user.click(await within(section).findByRole("button", { name: "Replace everything with this backup" }));
    } else {
      await user.click(within(section).getByRole("button", { name: "Show recovery points" }));
      await user.click(await within(section).findByRole("button", { name: "Review restore…" }));
      await user.click(within(section).getByRole("button", { name: "Restore this database copy" }));
    }
    const blocked = await screen.findByRole("dialog", { name: "Recovery needs attention" });
    expect(within(blocked).getByRole("alert")).toHaveTextContent(failure.message);
    expect(within(blocked).getByRole("button", { name: "Reload Abiding Word" })).toHaveFocus();
    expect(databaseRestoreActive()).toBe(true);
    await user.keyboard("{Escape}");
    expect(blocked).toBeInTheDocument();
  });

  it("does not offer recovery when there are no snapshots, and fails safely if one changed after review", async () => {
    const { api } = await setup();
    const { user } = await renderApp(api);
    const section = await data(user);
    await user.click(within(section).getByRole("button", { name: "Show recovery points" }));
    expect(await within(section).findByText("No automatic recovery points are available yet.")).toBeInTheDocument();
    const id = "abiding-word-v27-2026-11-20T10-00-00.000Z.db";
    api.state.portability.dailySnapshots = [{ id, createdAt: "2026-11-20T10:00:00Z", schemaVersion: 27, bytes: 500_000, usable: true }];
    await user.click(within(section).getByRole("button", { name: "Refresh recovery points" }));
    const entries = await within(section).findByRole("list", { name: "Automatic recovery points" });
    await user.click(within(entries).getByRole("button", { name: "Review restore…" }));
    // Native revalidates the file after the first preview, before any mutation.
    api.state.portability.dailySnapshots[0]!.usable = false;
    await user.click(within(section).getByRole("button", { name: "Restore this database copy" }));
    expect(await within(section).findByRole("alert")).toHaveTextContent(/damaged or incompatible/);
    expect(api.state.portability.dailyRestored).toEqual([]);
    expect(within(section).queryByText("The database was restored.")).not.toBeInTheDocument();
    expect(databaseRestoreActive()).toBe(false);
    expect(api.state.studies[0]?.title).toBe("Justified");
    expect(within(section).getByRole("button", { name: "Refresh recovery points" })).toBeEnabled();
  });
});

describe("Project information", () => {
  it("offers to export that project", async () => {
    const { api, romansId } = await setup();
    const { user } = await renderApp(api);
    await user.click(await screen.findByRole("button", { name: /Romans/ }));
    const panel = await openInfo(user, "Project information");
    api.state.portability.nextSave = "/home/test/Documents/Romans.awproject";
    await user.click(within(panel).getByRole("button", { name: /Export project…/ }));
    await waitFor(() => expect(api.state.portability.exported).toEqual([{ projectId: romansId, path: "/home/test/Documents/Romans.awproject" }]));
    expect(within(panel).getByText(/“Romans” was saved to/)).toBeInTheDocument();
  });
});

describe("the example project", () => {
  it("is added only when asked, with its four pieces of work, and never twice", async () => {
    const api = createMemoryApi();
    const { user } = await renderApp(api);
    await waitFor(() => expect(api.state.projects).toHaveLength(1));
    const section = await data(user);
    await user.click(within(section).getByRole("button", { name: /Add the example project/ }));
    expect(await within(section).findByText("“Example: Abiding in Christ — John 15:1–11” was added. You’ll find it under Projects.")).toBeInTheDocument();
    expect(api.state.projects.map((p) => p.title)).toEqual(["My studies", "Example: Abiding in Christ — John 15:1–11"]);
    expect(api.state.studies.map((s) => s.title)).toEqual(["Abiding in the True Vine"]);
    expect(api.state.teaching.preparations.map((p) => p.kind)).toEqual(["sermon", "bible_study", "lesson"]);

    await user.click(within(section).getByRole("button", { name: /Add the example project/ }));
    expect(await within(section).findByText("“Example: Abiding in Christ — John 15:1–11” is already here, so nothing was added.")).toBeInTheDocument();
    expect(api.state.projects).toHaveLength(2);
    expect(api.state.studies).toHaveLength(1);
  });
});
