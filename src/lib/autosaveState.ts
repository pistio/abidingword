/** Shared writing state has no React, native API or adapter dependencies. */
export interface AutosaveDraft { value: string; revision: number }
export interface WritingLane {
  revision: number;
  savedRevision: number;
  chain: Promise<unknown>;
  draft: AutosaveDraft | null;
  owner: symbol | null;
}

const lanes = new Map<string | symbol, WritingLane>();

/** Called by autosave event handlers/effects, never to mutate state in render. */
export function getWritingLane(key: string | symbol): WritingLane {
  let lane = lanes.get(key);
  if (!lane) {
    lane = { revision: 0, savedRevision: 0, chain: Promise.resolve(), draft: null, owner: null };
    lanes.set(key, lane);
  }
  return lane;
}

/** Test isolation models a fresh webview, not a production failure reset. */
export function resetAutosaveLanesForTests() {
  if (import.meta.env.MODE !== "test") throw new Error("Test-only webview reset");
  lanes.clear();
}
