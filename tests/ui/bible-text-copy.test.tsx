import { act, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { BibleTextCopyPreview } from "@/types/scripture";
import { createMemoryApi } from "../support/memoryApi";
import { openSettingsSection, renderAppInProject as renderApp } from "../support/render";

describe("checking a saved Bible for a separate corrected copy", () => {
  it("shows the comparison and adds only on request, keeping the earlier Bible and its anchored notes", async () => {
    const api = createMemoryApi();
    const original = api.installFixtureBible("OLD", "alpha");
    Object.assign(original, { sourceOrigin: "bundled", isBuiltin: true, publisher: "Fixture publisher", copyrightNotice: "Fixture attribution", licenseType: "Public Domain" });
    await api.annotationCreate({
      reference: "JHN.15.1", kind: "note", content: "Keep this exact quotation.",
      anchor: { translationId: original.id, start: { verse: "JHN.15.1", offset: 8 }, end: { verse: "JHN.15.1", offset: 13 }, quote: "alpha" },
    });
    const before = structuredClone({ bible: original, text: api.state.text.get(original.id), annotations: api.state.annotations });
    const read = api.bibleTextCopyPreview;
    vi.spyOn(api, "bibleTextCopyPreview").mockImplementation(async (id, jobId) => ({
      ...await read(id, jobId), changedVerses: 3, addedVerses: 1, removedVerses: 2, changedSuperscriptions: 1,
    }));
    const commit = vi.spyOn(api, "bibleImportCommit");
    const move = vi.spyOn(api, "bibleSetState");
    const remove = vi.spyOn(api, "bibleRemovePermanently");
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    await user.click(within(section).getByRole("button", { name: "Check saved text…" }));
    const dialog = await screen.findByRole("dialog", { name: "Check Fixture OLD" });
    const counts = await within(dialog).findByLabelText("Differences from the installed Bible");
    expect(within(counts).getByText("Verses with different text").nextElementSibling).toHaveTextContent("3");
    expect(within(counts).getByText("Additional verses").nextElementSibling).toHaveTextContent("1");
    expect(within(counts).getByText("Verses not included in the new copy").nextElementSibling).toHaveTextContent("2");
    expect(within(counts).getByText("Chapter titles with different text").nextElementSibling).toHaveTextContent("1");
    expect(within(dialog).getByText(/They stay linked to the earlier Bible/)).toBeVisible();
    expect(within(dialog).getByRole("textbox", { name: "Short name" })).toHaveValue("OLD-2");
    expect(commit).not.toHaveBeenCalled();

    await user.click(within(dialog).getByRole("button", { name: "Add as a separate Bible" }));
    expect(await within(dialog).findByRole("status")).toHaveTextContent("Added Fixture OLD (OLD-2) as a separate Bible");
    const copy = api.state.bibles.find((b) => b.shortName === "OLD-2")!;
    expect(copy.id).not.toBe(original.id);
    expect(copy).toMatchObject({ sourceOrigin: "bundled", publisher: original.publisher, copyrightNotice: original.copyrightNotice, licenseType: original.licenseType });
    expect(api.state.bibles.find((b) => b.id === original.id)).toEqual(before.bible);
    expect(api.state.text.get(original.id)).toEqual(before.text);
    expect(api.state.annotations).toEqual(before.annotations);
    expect(move).not.toHaveBeenCalled();
    expect(remove).not.toHaveBeenCalled();
    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    const installed = within(section).getByRole("list", { name: "Installed Bibles" });
    expect(within(installed).getByText("(OLD)")).toBeVisible();
    expect(within(installed).getByText("(OLD-2)")).toBeVisible();
  });

  it("offers no new copy when the saved text and chapter titles already match", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("OLD", "alpha");
    const commit = vi.spyOn(api, "bibleImportCommit");
    const cancel = vi.spyOn(api, "bibleImportCancel");
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    await user.click(within(section).getByRole("button", { name: "Check saved text…" }));
    const dialog = await screen.findByRole("dialog", { name: "Check Fixture OLD" });
    expect(await within(dialog).findByText("The Bible text and chapter titles match the saved files. No new copy is needed.")).toBeVisible();
    expect(within(dialog).queryByRole("button", { name: "Add as a separate Bible" })).not.toBeInTheDocument();
    const previewId = api.state.pendingPreview!.id;
    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    expect(cancel).toHaveBeenCalledWith(previewId);
    expect(commit).not.toHaveBeenCalled();
    expect(api.state.bibles).toHaveLength(1);
  });

  it("shows normal import problems and blocks addition even when the comparison counts are zero", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("OLD", "alpha");
    const read = api.bibleTextCopyPreview;
    vi.spyOn(api, "bibleTextCopyPreview").mockImplementation(async (id, jobId) => {
      const result = await read(id, jobId);
      result.preview.canImport = false;
      result.preview.issues = [{ severity: "error", book: "JHN", chapter: 15, message: "John 15:1 appears twice.", code: "bibleImport.verseTwice", params: { bookCode: "JHN", chapter: "15", verse: "1" } }];
      return result;
    });
    const commit = vi.spyOn(api, "bibleImportCommit");
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    await user.click(within(section).getByRole("button", { name: "Check saved text…" }));
    const dialog = await screen.findByRole("dialog", { name: "Check Fixture OLD" });
    expect(await within(dialog).findByText("John 15:1 appears twice.")).toBeVisible();
    expect(within(dialog).getByRole("button", { name: "Add as a separate Bible" })).toBeDisabled();
    expect(within(dialog).queryByText(/No new copy is needed/)).not.toBeInTheDocument();
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(commit).not.toHaveBeenCalled();
    expect(api.state.bibles).toHaveLength(1);
    expect(api.state.pendingPreview).toBeNull();
  });

  it("cancels an ongoing check by job ID and discards any late preview", async () => {
    const api = createMemoryApi();
    const original = api.installFixtureBible("OLD", "alpha");
    const result = await api.bibleTextCopyPreview(original.id);
    let finish!: (result: BibleTextCopyPreview) => void;
    const check = vi.spyOn(api, "bibleTextCopyPreview").mockImplementation(() => new Promise((resolve) => { finish = resolve; }));
    const cancel = vi.spyOn(api, "bibleImportCancel");
    const commit = vi.spyOn(api, "bibleImportCommit");
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    const button = within(section).getByRole("button", { name: "Check saved text…" });
    await user.click(button);
    const dialog = screen.getByRole("dialog", { name: "Check Fixture OLD" });
    expect(within(dialog).getByRole("status")).toHaveTextContent("Reading and checking every book");
    const jobId = check.mock.calls[0]![1];
    expect(jobId).toEqual(expect.any(String));
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(cancel).toHaveBeenCalledWith(jobId);
    await act(async () => finish(result));
    await waitFor(() => expect(cancel).toHaveBeenCalledWith(result.preview.previewId));
    expect(screen.queryByRole("dialog", { name: "Check Fixture OLD" })).not.toBeInTheDocument();
    expect(button).toHaveFocus();
    expect(commit).not.toHaveBeenCalled();
    expect(api.state.bibles).toEqual([original]);
  });

  it("explains unavailable saved files without proposing removal of the installed Bible", async () => {
    const api = createMemoryApi();
    const original = api.installFixtureBible("OLD", "alpha");
    vi.spyOn(api, "bibleTextCopyPreview").mockRejectedValue({ kind: "invalidInput", message: "Missing saved files.", code: "bibleImport.copyUnavailable", params: {} });
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    await user.click(within(section).getByRole("button", { name: "Check saved text…" }));
    const dialog = await screen.findByRole("dialog", { name: "Check Fixture OLD" });
    expect(await within(dialog).findByRole("alert")).toHaveTextContent("Its kept USFM source is unavailable");
    expect(within(dialog).getByRole("alert")).toHaveTextContent("The installed Bible and its notes are unchanged");
    expect(within(dialog).queryByRole("button", { name: "Add as a separate Bible" })).not.toBeInTheDocument();
    expect(api.state.bibles).toEqual([original]);
  });

  it("only offers saved-text comparison for USFM Bibles", async () => {
    const api = createMemoryApi();
    const bible = api.installFixtureBible("EPB", "alpha");
    bible.sourceType = "epub";
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    expect(within(section).queryByRole("button", { name: "Check saved text…" })).not.toBeInTheDocument();
  });
});
