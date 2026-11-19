/** Parent in the rendered tree, including a document page's shadow host. */
function composedParent(element: Element): Element | null {
  if (element.parentElement) return element.parentElement;
  const root = element.getRootNode();
  return "host" in root ? (root as ShadowRoot).host : null;
}

/**
 * Keep native anchor/table alignment inside a reader without moving its
 * enclosing workspace. scrollIntoView's default crosses scroll containers,
 * including overflow:hidden ancestors and shadow boundaries. Restore only
 * those outside the chosen viewport synchronously, before the next paint.
 *
 * Does not focus anything, reset existing scroll positions to zero, resize
 * the window, change text, or depend on a particular viewport/DPI size.
 */
export function scrollInViewport(
  viewport: HTMLElement | null,
  target: Element | null | undefined,
  block: "start" | "center",
): boolean {
  if (!viewport || !target || !viewport.isConnected || !target.isConnected) return false;
  // Element.contains does not cross a ShadowRoot; EPUB/DOCX pages do.
  let within: Element | null = target;
  while (within && within !== viewport) within = composedParent(within);
  if (!within || typeof target.scrollIntoView !== "function") return false;

  const ancestors: { element: Element; top: number; left: number }[] = [];
  for (let parent = composedParent(viewport); parent; parent = composedParent(parent)) {
    ancestors.push({ element: parent, top: parent.scrollTop, left: parent.scrollLeft });
  }
  const window = viewport.ownerDocument.defaultView;
  const top = window?.scrollY ?? 0;
  const left = window?.scrollX ?? 0;
  try {
    // Explicitly synchronous: a queued smooth scroll could escape restoration.
    target.scrollIntoView({ block, inline: "nearest", behavior: "instant" });
  } finally {
    for (const saved of ancestors) {
      if (saved.element.scrollTop !== saved.top || saved.element.scrollLeft !== saved.left) {
        saved.element.scrollTo({ top: saved.top, left: saved.left, behavior: "instant" });
      }
    }
    if (window && (window.scrollY !== top || window.scrollX !== left)) {
      window.scrollTo({ top, left, behavior: "instant" });
    }
  }
  return true;
}
