import i18n from "i18next";

/** A successful replacement is a one-way boundary for this webview. */
type RestorePhase = "idle" | "settling" | "replacing" | "reload";
let phase: RestorePhase = "idle";
let replacementStarted = false;
let webviewEpoch = 0;
let settlementFailed = false;
const pending = new Set<Promise<void>>();
const failures = new Map<unknown, unknown>();
const versions = new Map<unknown, number>();
const flushers = new Set<() => void>();

function restoreError(key: "restoreAlreadyRunning" | "restoreReloadRequired" | "restoreSaveFailed") {
  return { kind: "conflict", message: i18n.t(`settings.data.${key}`) };
}

export function beginDatabaseRestore() {
  if (phase !== "idle") throw restoreError(phase === "reload" ? "restoreReloadRequired" : "restoreAlreadyRunning");
  phase = "settling";
  replacementStarted = false;
  settlementFailed = false;
  document.getElementById("root")?.setAttribute("inert", "");
}

/** Only an unsuccessful preparation/replacement may resume the old editors. */
export function endDatabaseRestore() {
  if (phase === "reload") return;
  phase = "idle";
  settlementFailed = false;
  document.getElementById("root")?.removeAttribute("inert");
}

export function databaseRestoreActive() {
  return phase !== "idle";
}

/** Settlement may finish already queued writes; replacement/reload may not. */
export function databaseWritesBlocked() {
  return phase === "replacing" || phase === "reload";
}

export function assertDatabaseWritesAllowed() {
  if (databaseWritesBlocked()) throw restoreError("restoreReloadRequired");
}

export function markDatabaseRestoreComplete() {
  phase = "reload";
  document.getElementById("root")?.setAttribute("inert", "");
}

export function registerAutosaveFlusher(flush: () => void) {
  flushers.add(flush);
  return () => { flushers.delete(flush); };
}

/**
 * Observe synchronously, without creating an unhandled rejected child promise.
 * A failure belongs to its writing lane until that same lane is saved again;
 * an unrelated successful write cannot clear it. Reads pass no failure key.
 */
export function observeDatabaseOperation<T>(operation: Promise<T>, failureKey?: unknown): Promise<T> {
  const epoch = webviewEpoch;
  const version = failureKey === undefined ? 0 : (versions.get(failureKey) ?? 0) + 1;
  if (failureKey !== undefined) versions.set(failureKey, version);
  const completion = operation.then(
    () => {
      if (epoch === webviewEpoch && failureKey !== undefined && versions.get(failureKey) === version) failures.delete(failureKey);
    },
    error => {
      if (epoch === webviewEpoch && phase === "settling") settlementFailed = true;
      if (epoch === webviewEpoch && failureKey !== undefined && versions.get(failureKey) === version) failures.set(failureKey, error);
    },
  );
  pending.add(completion);
  void completion.then(() => { pending.delete(completion); });
  return operation;
}

/** Flush pending text before waiting, and include work enqueued while waiting. */
export async function settleDatabaseOperations(): Promise<void> {
  for (const flush of [...flushers]) flush();
  while (pending.size) await Promise.all([...pending]);
  if (failures.size || settlementFailed) throw restoreError("restoreSaveFailed");
  // No await between the empty-queue check and closing the write gate.
  if (phase === "settling") phase = "replacing";
}

/** A restore command is permitted only by the shared, settled restore flow. */
export function assertDatabaseReplacementAllowed() {
  if (phase !== "replacing" || replacementStarted) throw restoreError(phase === "reload" ? "restoreReloadRequired" : "restoreAlreadyRunning");
  replacementStarted = true;
}

/** Test isolation represents a fresh webview; production never clears the latch. */
export function resetDatabaseWriteBarrierForTests() {
  if (!import.meta.env.MODE || import.meta.env.MODE !== "test") throw new Error("Test-only webview reset");
  phase = "idle";
  replacementStarted = false;
  webviewEpoch += 1;
  settlementFailed = false;
  pending.clear();
  failures.clear();
  versions.clear();
  flushers.clear();
  document.getElementById("root")?.removeAttribute("inert");
}
