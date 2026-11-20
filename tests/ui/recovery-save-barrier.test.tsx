import { useEffect } from "react";
import { act, render, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SaveTrackerProvider, settleOutstandingWrites, useSaveTracker } from "@/hooks/SaveTracker";
import { useAutosave } from "@/hooks/useAutosave";
import { beginDatabaseRestore, databaseRestoreActive, endDatabaseRestore } from "@/lib/databaseRestoreLock";

// No real Study, Teaching, keys, source files or external API calls.
function PendingSave({ promise }: { promise: Promise<void> }) {
  const { track } = useSaveTracker();
  useEffect(() => { track(promise); }, [promise, track]);
  return null;
}

afterEach(() => endDatabaseRestore());

describe("database replacement save barrier", () => {
  it("waits for an in-flight Study save even after its editor provider unmounts", async () => {
    let resolve!: () => void;
    const saving = new Promise<void>((done) => { resolve = done; });
    const view = render(
      <SaveTrackerProvider><PendingSave promise={saving} /></SaveTrackerProvider>,
    );
    view.unmount();

    let settled = false;
    const pending = settleOutstandingWrites().then(() => { settled = true; });
    await Promise.resolve();
    expect(settled).toBe(false);
    await act(async () => { resolve(); await pending; });
    expect(settled).toBe(true);
  });

  it("does not claim that a still-pending failed save was safely settled", async () => {
    let reject!: (reason: Error) => void;
    const saving = new Promise<void>((_, fail) => { reject = fail; });
    const view = render(
      <SaveTrackerProvider><PendingSave promise={saving} /></SaveTrackerProvider>,
    );
    const settlement = settleOutstandingWrites();
    await act(async () => { reject(new Error("Invented save failure")); });
    await expect(settlement).rejects.toThrow("Invented save failure");
    view.unmount();
  });

  it("refuses new autosaves while restoring, but allows writing when an unsuccessful restore unlocks", async () => {
    const save = vi.fn(async (_text: string) => undefined);
    let edits!: ReturnType<typeof useAutosave>;

    function Editor() {
      edits = useAutosave(save);
      return null;
    }
    const view = render(<SaveTrackerProvider><Editor /></SaveTrackerProvider>);
    act(() => {
      beginDatabaseRestore();
      edits.schedule("must not overwrite the restored database");
      edits.flush();
    });
    expect(databaseRestoreActive()).toBe(true);
    expect(save).not.toHaveBeenCalled();

    act(() => {
      endDatabaseRestore();
      edits.schedule("still safe to write after failed recovery");
      edits.flush();
    });
    await waitFor(() => expect(save).toHaveBeenCalledTimes(1));
    expect(save).toHaveBeenCalledWith("still safe to write after failed recovery");
    view.unmount();
  });
});
