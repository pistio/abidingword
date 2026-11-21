import { useEffect, useState } from "react";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SaveTrackerProvider, runDatabaseRestore, settleOutstandingWrites, useSaveTracker } from "@/hooks/SaveTracker";
import { useAutosave } from "@/hooks/useAutosave";
import { beginDatabaseRestore, databaseRestoreActive, endDatabaseRestore } from "@/lib/databaseRestoreLock";
import { guardApiDuringDatabaseRestore } from "@/lib/guardedApi";
import { ApiError, type Api } from "@/lib/api";
import { EditableTitle } from "@/features/studies/EditableTitle";

function deferred<T = void>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: Error) => void;
  const promise = new Promise<T>((done, fail) => { resolve = done; reject = fail; });
  return { promise, resolve, reject };
}

function PendingSave({ promise, lane }: { promise: Promise<void>; lane?: unknown }) {
  const { track } = useSaveTracker();
  useEffect(() => { track(promise, lane); }, [promise, lane, track]);
  return null;
}

function Editor({ save, label = "Writing", saveKey }: { save: (value: string) => Promise<unknown>; label?: string; saveKey?: string }) {
  const [value, setValue] = useState("");
  const { schedule, flush } = useAutosave(save, undefined, saveKey, setValue);
  const { status } = useSaveTracker();
  return <>
    <textarea aria-label={label} value={value} onChange={event => { setValue(event.target.value); schedule(event.target.value); }} onBlur={flush} />
    <output>{status.state}</output>
  </>;
}

const saveFailure = { kind: "conflict", message: expect.stringContaining("could not be saved") };

afterEach(() => endDatabaseRestore());

describe("database replacement save barrier", () => {
  it("waits for an in-flight Study save even after its editor provider unmounts", async () => {
    const saving = deferred();
    const view = render(<SaveTrackerProvider><PendingSave promise={saving.promise} /></SaveTrackerProvider>);
    view.unmount();
    let settled = false;
    const pending = settleOutstandingWrites().then(() => { settled = true; });
    await Promise.resolve();
    expect(settled).toBe(false);
    await act(async () => { saving.resolve(); await pending; });
    expect(settled).toBe(true);
  });

  it("rejects a failed save without a temporarily unhandled settlement rejection", async () => {
    const saving = deferred();
    render(<SaveTrackerProvider><PendingSave promise={saving.promise} /></SaveTrackerProvider>);
    // Observe rejection BEFORE rejecting the writer. React act yields to the
    // host; attaching .rejects afterwards produced the original CI error.
    const rejected = expect(settleOutstandingWrites()).rejects.toMatchObject(saveFailure);
    await act(async () => { saving.reject(new Error("Invented save failure")); await rejected; });
    // The completed failure must still block a later recovery attempt.
    await expect(settleOutstandingWrites()).rejects.toMatchObject(saveFailure);
  });

  it("does not clear one failed field when another field saves, but permits the same field's successful retry", async () => {
    const lane = Symbol("failed field");
    const saving = deferred();
    const view = render(<SaveTrackerProvider><PendingSave promise={saving.promise} lane={lane} /></SaveTrackerProvider>);
    await act(async () => { saving.reject(new Error("Invented save failure")); });
    view.rerender(<SaveTrackerProvider><PendingSave promise={Promise.resolve()} lane="other" /></SaveTrackerProvider>);
    await expect(settleOutstandingWrites()).rejects.toMatchObject(saveFailure);
    view.rerender(<SaveTrackerProvider><PendingSave promise={Promise.resolve()} lane={lane} /></SaveTrackerProvider>);
    await expect(settleOutstandingWrites()).resolves.toBeUndefined();
  });

  it("flushes unblurred debounce text before replacement and keeps the success latch after unmount", async () => {
    const calls: string[] = [];
    const save = vi.fn(async (text: string) => { calls.push(`save:${text}`); });
    const replace = vi.fn(async () => { calls.push("replace"); return "restored"; });
    const view = render(<SaveTrackerProvider><Editor save={save} /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "Invented unsaved Study text" } });
    expect(save).not.toHaveBeenCalled();
    await act(async () => { expect(await runDatabaseRestore(replace)).toBe("restored"); });
    expect(calls).toEqual(["save:Invented unsaved Study text", "replace"]);
    view.unmount();
    endDatabaseRestore();
    expect(databaseRestoreActive()).toBe(true);
    await expect(runDatabaseRestore(replace)).rejects.toMatchObject({ message: expect.stringContaining("Reload") });
    expect(replace).toHaveBeenCalledTimes(1);
  });

  it("retains text after a failed save and flushes it successfully even after editor unmount", async () => {
    let failing = true;
    const save = vi.fn(async (_text: string) => { if (failing) throw new Error("Invented save failure"); });
    const replace = vi.fn(async () => undefined);
    const view = render(<SaveTrackerProvider><Editor save={save} /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "Invented retained Teaching text" } });
    await act(async () => { await expect(runDatabaseRestore(replace)).rejects.toMatchObject(saveFailure); });
    expect(replace).not.toHaveBeenCalled();
    expect(databaseRestoreActive()).toBe(false);
    expect(screen.getByLabelText("Writing")).toHaveValue("Invented retained Teaching text");
    view.unmount();
    await act(async () => { await Promise.resolve(); });
    failing = false;
    await act(async () => { await runDatabaseRestore(replace); });
    expect(save).toHaveBeenLastCalledWith("Invented retained Teaching text");
    expect(replace).toHaveBeenCalledTimes(1);
  });

  it("never replays a detached failed draft over newer writing from the same reopened field", async () => {
    let failing = true;
    let stored = "Initial saved text";
    const save = vi.fn(async (text: string) => { if (failing) throw new Error("Invented offline failure"); stored = text; });
    const first = render(<SaveTrackerProvider><Editor save={save} saveKey="same-field" /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "Old failed A" } });
    fireEvent.blur(screen.getByLabelText("Writing"));
    await waitFor(() => expect(screen.getByText("error")).toBeInTheDocument());
    first.unmount();
    await act(async () => { await Promise.resolve(); });
    failing = false;
    render(<SaveTrackerProvider><Editor save={save} saveKey="same-field" /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "New authoritative B" } });
    fireEvent.blur(screen.getByLabelText("Writing"));
    await waitFor(() => expect(stored).toBe("New authoritative B"));
    const replaced = vi.fn(async () => { expect(stored).toBe("New authoritative B"); });
    await act(async () => { await runDatabaseRestore(replaced); });
    expect(stored).toBe("New authoritative B");
    expect(replaced).toHaveBeenCalledOnce();
    expect(save.mock.calls.slice(-1)).toEqual([["New authoritative B"]]);
  });

  it("shows retained unsaved text after failed recovery and reopening the editor", async () => {
    let failing = true;
    const save = vi.fn(async (_text: string) => { if (failing) throw new Error("Invented storage failure"); });
    const first = render(<SaveTrackerProvider><Editor save={save} saveKey="recover-visible" /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "Invented unsaved author text" } });
    first.unmount();
    await act(async () => { await Promise.resolve(); });
    const replace = vi.fn(async () => undefined);
    await act(async () => { await expect(runDatabaseRestore(replace)).rejects.toMatchObject(saveFailure); });
    expect(replace).not.toHaveBeenCalled();
    render(<SaveTrackerProvider><Editor save={save} saveKey="recover-visible" /></SaveTrackerProvider>);
    expect(screen.getByLabelText("Writing")).toHaveValue("Invented unsaved author text");
    failing = false;
    fireEvent.blur(screen.getByLabelText("Writing"));
    await act(async () => { await expect(settleOutstandingWrites()).resolves.toBeUndefined(); });
    expect(save).toHaveBeenLastCalledWith("Invented unsaved author text");
    expect(databaseRestoreActive()).toBe(false);
  });

  it("rehydrates the actual Study title control after failed recovery without overwriting a corrected title", async () => {
    let failing = true;
    let stored = "Invented saved title";
    const save = vi.fn(async (title: string) => { if (failing) throw new Error("Invented failure"); stored = title; });
    const title = () => <SaveTrackerProvider><EditableTitle saveKey="study:invented:title" label="Study title" value={stored} onSave={save} /></SaveTrackerProvider>;
    const first = render(title());
    fireEvent.change(screen.getByLabelText("Study title"), { target: { value: "Invented unsaved title" } });
    first.unmount();
    await act(async () => { await Promise.resolve(); });
    await act(async () => { await expect(runDatabaseRestore(async () => undefined)).rejects.toMatchObject(saveFailure); });
    render(title());
    expect(screen.getByLabelText("Study title")).toHaveValue("Invented unsaved title");
    failing = false;
    fireEvent.change(screen.getByLabelText("Study title"), { target: { value: "Invented revised title" } });
    fireEvent.blur(screen.getByLabelText("Study title"));
    await act(async () => { await settleOutstandingWrites(); });
    expect(stored).toBe("Invented revised title");
    await act(async () => { await runDatabaseRestore(async () => { expect(stored).toBe("Invented revised title"); }); });
  });

  it("serialises a reopened field's new value after its detached in-flight write", async () => {
    const oldWrite = deferred();
    let stored = "Initial";
    const save = vi.fn(async (text: string) => { if (text === "A") await oldWrite.promise; stored = text; });
    const first = render(<SaveTrackerProvider><Editor save={save} saveKey="same-inflight-field" /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "A" } });
    fireEvent.blur(screen.getByLabelText("Writing"));
    await waitFor(() => expect(save).toHaveBeenCalledWith("A"));
    first.unmount();
    render(<SaveTrackerProvider><Editor save={save} saveKey="same-inflight-field" /></SaveTrackerProvider>);
    expect(screen.getByLabelText("Writing")).toHaveValue("A");
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "B" } });
    fireEvent.blur(screen.getByLabelText("Writing"));
    await act(async () => { await Promise.resolve(); });
    expect(save).not.toHaveBeenCalledWith("B");
    const replace = vi.fn(async () => { expect(stored).toBe("B"); });
    const restoring = runDatabaseRestore(replace);
    await act(async () => { oldWrite.resolve(); await restoring; });
    expect(save.mock.calls).toEqual([["A"], ["B"]]);
    expect(stored).toBe("B");
  });

  it("includes a second write started while the first is settling", async () => {
    const first = deferred();
    const second = deferred();
    const raw = { studyBlockUpdate: vi.fn(() => first.promise), teachingUpdate: vi.fn(() => second.promise) } as unknown as Api;
    const api = guardApiDuringDatabaseRestore(raw);
    void api.studyBlockUpdate({ id: "study", content: "Invented first" }).then(() => api.teachingUpdate({ id: "teaching", title: "Invented second" }));
    const replace = vi.fn(async () => undefined);
    const restoring = runDatabaseRestore(replace);
    await act(async () => { first.resolve(); await Promise.resolve(); });
    expect(raw.teachingUpdate).toHaveBeenCalledOnce();
    expect(replace).not.toHaveBeenCalled();
    await act(async () => { second.resolve(); await restoring; });
    expect(replace).toHaveBeenCalledOnce();
  });

  it("blocks direct writing, reader saves and native index reads while replacing and after success", async () => {
    const replacing = deferred();
    const raw = {
      studyBlockUpdate: vi.fn(async () => undefined), sourceReaderPositionSave: vi.fn(async () => undefined),
      libraryGeneratedContents: vi.fn(async () => []), dailySnapshotRestore: vi.fn(() => replacing.promise),
    } as unknown as Api;
    const api = guardApiDuringDatabaseRestore(raw);
    const restoring = runDatabaseRestore(() => api.dailySnapshotRestore("invented.db"));
    await waitFor(() => expect(raw.dailySnapshotRestore).toHaveBeenCalledOnce());
    await expect(api.studyBlockUpdate({ id: "invented", content: "stale" })).rejects.toMatchObject({ kind: "conflict" });
    await expect(api.libraryGeneratedContents("invented")).rejects.toMatchObject({ kind: "conflict" });
    await expect(api.dailySnapshotRestore("second.db")).rejects.toMatchObject({ kind: "conflict" });
    replacing.resolve();
    await restoring;
    await expect(api.sourceReaderPositionSave({ sourceId: "invented", position: {} as never })).rejects.toMatchObject({ kind: "conflict" });
    expect(raw.studyBlockUpdate).not.toHaveBeenCalled();
    expect(raw.libraryGeneratedContents).not.toHaveBeenCalled();
    expect(raw.sourceReaderPositionSave).not.toHaveBeenCalled();
  });

  it("retains direct API field failures until that field, not another field, is saved", async () => {
    const update = vi.fn().mockRejectedValueOnce(new Error("Invented failure")).mockResolvedValue(undefined);
    const api = guardApiDuringDatabaseRestore({ teachingUpdate: update } as unknown as Api);
    await expect(api.teachingUpdate({ id: "teaching", title: "Invented title" })).rejects.toThrow("Invented failure");
    await api.teachingUpdate({ id: "teaching", introduction: "Invented introduction" });
    const replace = vi.fn(async () => undefined);
    await expect(runDatabaseRestore(replace)).rejects.toMatchObject(saveFailure);
    expect(replace).not.toHaveBeenCalled();
    await api.teachingUpdate({ id: "teaching", title: "Invented retry" });
    await runDatabaseRestore(replace);
    expect(replace).toHaveBeenCalledOnce();
  });

  it("does not treat an omitted patch field as a successful retry of failed writing", async () => {
    const update = vi.fn().mockRejectedValueOnce(new Error("Invented failure")).mockResolvedValue(undefined);
    const api = guardApiDuringDatabaseRestore({ teachingUpdate: update } as unknown as Api);
    await expect(api.teachingUpdate({ id: "teaching", title: "Unsaved title" })).rejects.toThrow("Invented failure");
    await api.teachingUpdate({ id: "teaching", title: undefined, introduction: "Other text" });
    await expect(runDatabaseRestore(async () => undefined)).rejects.toMatchObject(saveFailure);
    await api.teachingUpdate({ id: "teaching", title: "Saved title" });
    await expect(runDatabaseRestore(async () => undefined)).resolves.toBeUndefined();
  });

  it("unlocks a failed replacement without losing current writing, and allows its next save", async () => {
    const save = vi.fn(async (_text: string) => undefined);
    const view = render(<SaveTrackerProvider><Editor save={save} /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "Invented current text" } });
    await act(async () => { await expect(runDatabaseRestore(async () => { throw new Error("Invented restore failure"); })).rejects.toThrow("Invented restore failure"); });
    expect(databaseRestoreActive()).toBe(false);
    expect(screen.getByLabelText("Writing")).toHaveValue("Invented current text");
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "Invented continued writing" } });
    fireEvent.blur(screen.getByLabelText("Writing"));
    await waitFor(() => expect(save).toHaveBeenLastCalledWith("Invented continued writing"));
    view.unmount();
  });

  it("keeps writing blocked when rollback failed and the safety copy must be recovered", async () => {
    const error = new ApiError("backup", "Invented retained safety copy: /disposable/safety.db", "portability.rollbackFailed");
    await expect(runDatabaseRestore(async () => { throw error; })).rejects.toBe(error);
    endDatabaseRestore();
    expect(databaseRestoreActive()).toBe(true);
  });

  it("does not latch a rejected creation after the person corrects its draft", async () => {
    const create = vi.fn().mockRejectedValueOnce(new ApiError("invalidInput", "Invented title required")).mockResolvedValue({ id: "created" });
    const api = guardApiDuringDatabaseRestore({ teachingCreate: create } as unknown as Api);
    await expect(api.teachingCreate({ projectId: "invented", title: "" })).rejects.toMatchObject({ kind: "invalidInput" });
    await api.teachingCreate({ projectId: "invented", title: "Invented corrected title" });
    const replace = vi.fn(async () => undefined);
    await runDatabaseRestore(replace);
    expect(replace).toHaveBeenCalledOnce();
  });

  it("blocks a restore when an untracked creation fails during settlement", async () => {
    const creating = deferred();
    const api = guardApiDuringDatabaseRestore({ teachingCreate: () => creating.promise } as unknown as Api);
    const rejectedCreate = expect(api.teachingCreate({ projectId: "invented", title: "Invented pending teaching" })).rejects.toThrow("Invented creation failure");
    const replace = vi.fn(async () => undefined);
    const rejectedRestore = expect(runDatabaseRestore(replace)).rejects.toMatchObject(saveFailure);
    creating.reject(new Error("Invented creation failure"));
    await rejectedCreate;
    await rejectedRestore;
    expect(replace).not.toHaveBeenCalled();
    expect(databaseRestoreActive()).toBe(false);
  });

  it("clears a scalar setting failure when that same setting is saved with a corrected value", async () => {
    const rename = vi.fn().mockRejectedValueOnce(new Error("Invented rename failure")).mockResolvedValue(undefined);
    const api = guardApiDuringDatabaseRestore({ libraryCollectionRename: rename } as unknown as Api);
    await expect(api.libraryCollectionRename("collection", "Invented rejected title")).rejects.toThrow("Invented rename failure");
    await api.libraryCollectionRename("another", "Invented other title");
    const replace = vi.fn(async () => undefined);
    await expect(runDatabaseRestore(replace)).rejects.toMatchObject(saveFailure);
    await api.libraryCollectionRename("collection", "Invented corrected title");
    await runDatabaseRestore(replace);
    expect(replace).toHaveBeenCalledOnce();
  });

  it("uses a replaced API method instead of a stale captured implementation", async () => {
    const first = vi.fn(async () => undefined);
    const second = vi.fn(async () => undefined);
    const raw = { studyBlockUpdate: first } as unknown as Api;
    const api = guardApiDuringDatabaseRestore(raw);
    await api.studyBlockUpdate({ id: "invented", content: "first" });
    raw.studyBlockUpdate = second as unknown as Api["studyBlockUpdate"];
    await api.studyBlockUpdate({ id: "invented", content: "second" });
    expect(first).toHaveBeenCalledOnce();
    expect(second).toHaveBeenCalledOnce();
  });

  it("writes queued revisions in order before the replacement", async () => {
    const first = deferred();
    const calls: string[] = [];
    const save = vi.fn(async (text: string) => { calls.push(text); if (text === "first") await first.promise; });
    render(<SaveTrackerProvider><Editor save={save} /></SaveTrackerProvider>);
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "first" } });
    fireEvent.blur(screen.getByLabelText("Writing"));
    fireEvent.change(screen.getByLabelText("Writing"), { target: { value: "second" } });
    const replace = vi.fn(async () => { calls.push("replace"); });
    const restoring = runDatabaseRestore(replace);
    await waitFor(() => expect(save).toHaveBeenCalledWith("first"));
    expect(replace).not.toHaveBeenCalled();
    await act(async () => { first.resolve(); await restoring; });
    expect(calls).toEqual(["first", "second", "replace"]);
  });

  it("rejects a concurrent restore without unlocking the first operation", async () => {
    beginDatabaseRestore();
    await expect(runDatabaseRestore(async () => undefined)).rejects.toMatchObject({ kind: "conflict" });
    expect(databaseRestoreActive()).toBe(true);
    endDatabaseRestore();
    expect(databaseRestoreActive()).toBe(false);
  });
});
