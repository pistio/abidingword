import { useLayoutEffect, useState, type CSSProperties, type RefObject } from "react";

/** Follow the trigger without contributing to pane sizing or being clipped by
 * reader scroll containers. Recompute CSS pixels for resize, scroll and zoom. */
export function useAnchoredPopover(anchor: RefObject<HTMLElement | null>, open: boolean, preferredWidth = 280): CSSProperties {
  const [style, setStyle] = useState<CSSProperties>({ position: "fixed" });
  useLayoutEffect(() => {
    if (!open) return;
    const measure = () => {
      const rect = anchor.current?.getBoundingClientRect();
      if (!rect) return;
      const width = Math.min(preferredWidth, Math.max(160, window.innerWidth - 24));
      const below = window.innerHeight - rect.bottom - 16;
      const above = rect.top - 16;
      const up = below < 160 && above > below;
      setStyle({
        position: "fixed",
        left: Math.max(12, Math.min(rect.right - width, window.innerWidth - width - 12)),
        width,
        ...(up ? { bottom: window.innerHeight - rect.top + 4 } : { top: rect.bottom + 4 }),
        maxHeight: Math.max(100, Math.min(480, up ? above : below)),
        overflowY: "auto",
        zIndex: 60,
      });
    };
    measure();
    window.addEventListener("resize", measure);
    document.addEventListener("scroll", measure, true);
    const observer = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(measure);
    if (anchor.current) observer?.observe(anchor.current);
    return () => {
      window.removeEventListener("resize", measure);
      document.removeEventListener("scroll", measure, true);
      observer?.disconnect();
    };
  }, [anchor, open, preferredWidth]);
  return style;
}
