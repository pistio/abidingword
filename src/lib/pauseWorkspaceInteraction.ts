import i18n from "i18next";
import { databaseRestoreActive } from "./databaseRestoreLock";

/**
 * A short UI-only pause while a confirmed parent deletion settles saves.
 * Portal panels are included: dismissing information must not expose an editor
 * between the last save and removal. This never changes the database barrier.
 */
export function pauseWorkspaceInteraction(): () => void {
  const previousFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const status = document.createElement("div");
  status.setAttribute("role", "status");
  status.setAttribute("aria-live", "polite");
  status.setAttribute("data-writing-operation", "");
  status.tabIndex = -1;
  status.className = "fixed inset-0 z-[250] flex items-center justify-center bg-black/55 p-4 text-text";
  const text = document.createElement("p");
  text.className = "m-0 rounded-aw border border-border bg-bg px-6 py-4 shadow-xl";
  text.textContent = i18n.t("settings.data.working");
  status.append(text);
  document.body.append(status);

  const frozen = new Set<HTMLElement>();
  const freeze = () => {
    for (const element of document.body.children) {
      if (!(element instanceof HTMLElement) || element === status || element.hasAttribute("inert")) continue;
      element.setAttribute("inert", "");
      frozen.add(element);
    }
  };
  freeze();
  const observer = new MutationObserver(freeze);
  observer.observe(document.body, { childList: true });
  const stop = (event: Event) => {
    if (event.cancelable) event.preventDefault();
    event.stopImmediatePropagation();
  };
  const keepFocus = (event: FocusEvent) => {
    if (event.target instanceof Node && !status.contains(event.target)) status.focus();
  };
  // Window capture runs before document/global shortcuts and React handlers.
  const events = ["pointerdown", "mousedown", "touchstart", "click", "dblclick", "keydown", "beforeinput", "input", "change", "submit", "drop", "paste"];
  for (const name of events) window.addEventListener(name, stop, { capture: true, passive: false });
  window.addEventListener("focusin", keepFocus, true);
  status.focus();

  let resumed = false;
  return () => {
    if (resumed) return;
    resumed = true;
    observer.disconnect();
    for (const name of events) window.removeEventListener(name, stop, true);
    window.removeEventListener("focusin", keepFocus, true);
    status.remove();
    for (const element of frozen) {
      if (element.id === "root" && databaseRestoreActive()) continue;
      element.removeAttribute("inert");
    }
    const returnFocus = () => {
      if (document.activeElement !== document.body || !previousFocus?.isConnected || previousFocus.closest("[inert]")) return;
      previousFocus.focus({ preventScroll: true });
    };
    returnFocus();
    // A failed action re-enables its initiating button in React's next render.
    // Retry then, but never override a newer focus choice or a removed control.
    requestAnimationFrame(returnFocus);
  };
}
