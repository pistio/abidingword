import { useCallback, useEffect, useRef } from "react";
import { useSaveTracker } from "./SaveTracker";
import { assertDatabaseWritesAllowed, databaseRestoreActive, databaseWritesBlocked, registerAutosaveFlusher } from "@/lib/databaseRestoreLock";

export const AUTOSAVE_DELAY_MS = 500;
interface Draft { value: string; revision: number }
interface WritingLane { revision: number; savedRevision: number; chain: Promise<unknown>; draft: Draft | null; owner: symbol | null }
const lanes = new Map<string | symbol, WritingLane>();

/** Debounced, ordered writing with a stable field identity across remounts. */
export function useAutosave(save: (value: string) => Promise<unknown>, delay = AUTOSAVE_DELAY_MS, identity?: string, recover?: (value: string) => void) {
  const { track } = useSaveTracker();
  const saveRef = useRef(save);
  const pending = useRef<Draft | null>(null);
  const recoverRef = useRef(recover);
  const owner = useRef(Symbol("editor"));
  const timer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const fallbackKey = useRef(Symbol("autosave"));
  const key = identity ?? fallbackKey.current;
  const active = useRef(0);
  const mounted = useRef(true);
  const unregister = useRef<(() => void) | undefined>(undefined);

  useEffect(() => { saveRef.current = save; recoverRef.current = recover; }, [save, recover]);

  // Called from event handlers/effects, never by mutating globals in render.
  const writingLane = useCallback(() => {
    let lane = lanes.get(key);
    if (!lane) {
      lane = { revision: 0, savedRevision: 0, chain: Promise.resolve(), draft: null, owner: null };
      lanes.set(key, lane);
    }
    return lane;
  }, [key]);

  const forgetDetached = useCallback(() => {
    if (!mounted.current && active.current === 0 && pending.current === null) {
      unregister.current?.();
      unregister.current = undefined;
    }
  }, []);

  const flush = useCallback(() => {
    clearTimeout(timer.current);
    if (databaseWritesBlocked()) return;
    const draft = pending.current;
    if (draft === null) return;
    const lane = writingLane();
    pending.current = null;
    // Another mount of this field has newer author-written text. The old
    // failed draft must never be replayed over it while preparing recovery.
    if (draft.revision !== lane.revision || lane.owner !== owner.current || draft.revision <= lane.savedRevision) { forgetDetached(); return; }
    const write = saveRef.current;
    active.current += 1;
    const run = lane.chain.catch(() => undefined).then(() => {
      assertDatabaseWritesAllowed();
      if (draft.revision <= lane.savedRevision) return;
      return write(draft.value);
    });
    lane.chain = run;
    track(run, key);
    void run.then(
      () => {
        active.current -= 1;
        lane.savedRevision = Math.max(lane.savedRevision, draft.revision);
        if (lane.draft?.revision === draft.revision) lane.draft = null;
        forgetDetached();
      },
      () => {
        active.current -= 1;
        if (lane.revision === draft.revision && lane.owner === owner.current && pending.current === null) pending.current = draft;
        forgetDetached();
      },
    );
  }, [track, key, writingLane, forgetDetached]);

  const schedule = useCallback((value: string) => {
    if (databaseRestoreActive()) return;
    const lane = writingLane();
    lane.owner = owner.current;
    lane.draft = { value, revision: ++lane.revision };
    pending.current = lane.draft;
    clearTimeout(timer.current);
    timer.current = setTimeout(flush, delay);
  }, [flush, delay, writingLane]);

  useEffect(() => {
    mounted.current = true;
    const lane = writingLane();
    lane.owner = owner.current;
    if (lane.draft) {
      pending.current = lane.draft;
      recoverRef.current?.(lane.draft.value);
    }
    unregister.current ??= registerAutosaveFlusher(flush);
    return () => {
      mounted.current = false;
      flush();
      forgetDetached();
    };
  }, [flush, forgetDetached, writingLane]);

  return { schedule, flush };
}

/** Test isolation models a fresh webview, not a production failure reset. */
export function resetAutosaveLanesForTests() {
  if (import.meta.env.MODE !== "test") throw new Error("Test-only webview reset");
  lanes.clear();
}
