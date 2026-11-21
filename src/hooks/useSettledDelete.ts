import { useCallback, useRef, useState } from "react";
import { settleOutstandingWrites } from "./SaveTracker";

/** Keep this editor and its controls still until a confirmed deletion settles. */
export function useSettledDelete(pauseInput?: () => () => void) {
  const root = useRef<HTMLDivElement>(null);
  const running = useRef(false);
  const [deleting, setDeleting] = useState(false);
  const runDeletion = useCallback(async (remove: () => Promise<void>) => {
    if (running.current) return false;
    running.current = true;
    // Freeze synchronously, before promise continuations or React's next render.
    root.current?.setAttribute("inert", "");
    setDeleting(true);
    let resumeInput: (() => void) | undefined;
    try {
      resumeInput = pauseInput?.();
      await settleOutstandingWrites();
      await remove();
      return true;
    } finally {
      resumeInput?.();
      running.current = false;
      root.current?.removeAttribute("inert");
      setDeleting(false);
    }
  }, [pauseInput]);
  return { root, deleting, runDeletion };
}
