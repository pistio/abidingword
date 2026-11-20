import { createContext, useCallback, useContext, useMemo, useRef, useState, type ReactNode } from "react";
import { toApiError } from "@/lib/api";
import {
  beginDatabaseRestore, endDatabaseRestore, markDatabaseRestoreComplete,
  observeDatabaseOperation, settleDatabaseOperations,
} from "@/lib/databaseRestoreLock";

/** Includes pending debounce values, detached editors and direct API writes. */
export const settleOutstandingWrites = settleDatabaseOperations;

/** Shared by full-backup and database-only replacement after user confirmation. */
export async function runDatabaseRestore<T>(replace: () => Promise<T>): Promise<T> {
  beginDatabaseRestore();
  try {
    await settleOutstandingWrites();
    const result = await replace();
    markDatabaseRestoreComplete();
    return result;
  } catch (error) {
    // A failed rollback leaves the database state uncertain: stale editing must
    // not resume. The native error retains the safety-copy recovery location.
    if (toApiError(error).code === "portability.rollbackFailed") markDatabaseRestoreComplete();
    else endDatabaseRestore();
    throw error;
  }
}

export type SaveStatus =
  | { state: "idle" }
  | { state: "saving" }
  | { state: "saved" }
  | { state: "error"; message: string };

interface SaveTrackerValue {
  status: SaveStatus;
  track: (save: Promise<unknown>, lane?: unknown) => void;
  /** Rejects when writing is still unsaved, including a previously failed save. */
  settled: () => Promise<void>;
}

const SaveTrackerContext = createContext<SaveTrackerValue | null>(null);

/** Collects the outcome of every save below it, for one status message. */
export function SaveTrackerProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<SaveStatus>({ state: "idle" });
  const inFlight = useRef(0);
  const failures = useRef(new Map<unknown, string>());
  const versions = useRef(new Map<unknown, number>());

  const track = useCallback((save: Promise<unknown>, lane: unknown = save) => {
    observeDatabaseOperation(save, lane);
    const version = (versions.current.get(lane) ?? 0) + 1;
    versions.current.set(lane, version);
    inFlight.current += 1;
    setStatus({ state: "saving" });
    const finish = (message?: string) => {
      inFlight.current -= 1;
      if (versions.current.get(lane) === version) {
        if (message === undefined) failures.current.delete(lane);
        else failures.current.set(lane, message);
      }
      if (inFlight.current === 0) {
        const error = failures.current.values().next().value;
        setStatus(error ? { state: "error", message: error } : { state: "saved" });
      }
    };
    void save.then(() => finish(), error => finish(toApiError(error).message));
  }, []);

  const value = useMemo(() => ({ status, track, settled: settleOutstandingWrites }), [status, track]);
  return <SaveTrackerContext.Provider value={value}>{children}</SaveTrackerContext.Provider>;
}

export function useSaveTracker(): SaveTrackerValue {
  const value = useContext(SaveTrackerContext);
  if (!value) throw new Error("useSaveTracker must be used inside <SaveTrackerProvider>");
  return value;
}
