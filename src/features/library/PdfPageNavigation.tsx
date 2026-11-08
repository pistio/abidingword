import { createPortal } from "react-dom";
import { useAnchoredPopover } from "@/design-system/useAnchoredPopover";
import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { formatNumber } from "@/i18n/format";

/**
 * Physical PDF pages, shared by Reading and Original. Printed labels are
 * evidence, never offsets. A.4.2 (#26): previous/next icons and a quiet
 * "3 / 400" indicator that opens a small jump-to-page form; the difference
 * between physical and printed numbers is explained there, not permanently.
 */
export function PdfPageNavigation({ page, total, printedLabel, onGo, compact = false }: {
  compact?: boolean; page: number; total: number; printedLabel?: string | null; onGo: (page: number) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const [open, setOpen] = useState(false);
  const [text, setText] = useState(String(page));
  const [invalid, setInvalid] = useState(false);
  const indicator = useRef<HTMLButtonElement>(null);
  const field = useRef<HTMLInputElement>(null);
  const box = useRef<HTMLDivElement>(null);
  const position = useAnchoredPopover(indicator, open, 280);
  useEffect(() => {
    if (!open) return;
    field.current?.focus();
    field.current?.select();
    const away = (e: MouseEvent) => {
      const target = e.target as Node;
      if (!box.current?.contains(target) && !indicator.current?.contains(target)) setOpen(false);
    };
    document.addEventListener("mousedown", away);
    return () => document.removeEventListener("mousedown", away);
  }, [open]);
  const close = () => { setOpen(false); setInvalid(false); indicator.current?.focus(); };
  const go = (target: number) => { onGo(target); };
  const submit = () => {
    const wanted = Number(text);
    if (!Number.isInteger(wanted) || wanted < 1 || wanted > total) { setInvalid(true); return; }
    setInvalid(false);
    setOpen(false);
    go(wanted);
    indicator.current?.focus();
  };
  const where = t("library.pageNav.indicator", { page: formatNumber(page), total: formatNumber(total) });
  const printed = printedLabel ? t("library.pageNav.printed", { label: printedLabel }) : "";
  return (
    <nav aria-label={t("library.pageNav.label")} className="relative flex shrink-0 items-center">
      <button type="button" aria-label={t("library.pageNav.previous")} title={t("library.pageNav.previous")} disabled={page <= 1} onClick={() => go(page - 1)}
        className="inline-flex size-11 items-center justify-center rounded-aw text-text-strong hover:bg-hover disabled:opacity-40 @max-xs:hidden">
        <ChevronLeft className="size-4" aria-hidden="true" />
      </button>
      <button
        ref={indicator}
        type="button"
        data-testid="pdf-page-indicator"
        aria-haspopup="dialog"
        aria-expanded={open}
        aria-controls={open ? `${id}-jump` : undefined}
        aria-label={`${where}${printed ? ` ${printed}` : ""}. ${t("library.pageNav.jump")}`}
        title={printed || t("library.pageNav.jump")}
        onClick={() => { setText(String(page)); setInvalid(false); setOpen((v) => !v); }}
        className="inline-flex min-h-11 min-w-11 justify-center items-center rounded-aw px-1.5 font-ui text-[0.875rem] tabular-nums text-text-muted hover:bg-hover hover:text-text-strong"
      >
        {compact ? formatNumber(page) : `${formatNumber(page)} / ${formatNumber(total)}`}
      </button>
      <button type="button" aria-label={t("library.pageNav.next")} title={t("library.pageNav.next")} disabled={page >= total} onClick={() => go(page + 1)}
        className="inline-flex size-11 items-center justify-center rounded-aw text-text-strong hover:bg-hover disabled:opacity-40 @max-xs:hidden">
        <ChevronRight className="size-4" aria-hidden="true" />
      </button>
      {open && createPortal(
        <div
          ref={box}
          id={`${id}-jump`}
          role="dialog"
          aria-label={t("library.pageNav.jump")}
          onKeyDown={(e) => { if (e.key === "Escape") { e.preventDefault(); e.stopPropagation(); close(); } }}
          style={position}
          className="aw-popover flex flex-col gap-2 p-3 font-ui"
        >
          <form noValidate className="flex items-center gap-2" onSubmit={(e) => { e.preventDefault(); submit(); }}>
            <label htmlFor={id} className="text-[0.9375rem]">{t("library.pageNav.page")}</label>
            <input
              ref={field}
              id={id}
              type="number"
              min={1}
              max={total}
              step={1}
              value={text}
              aria-invalid={invalid}
              aria-describedby={`${id}-hint`}
              onChange={(e) => { setText(e.target.value); setInvalid(false); }}
              className="aw-input h-10 w-20 px-2"
            />
            <span className="type-meta">{t("library.pageNav.total", { total: formatNumber(total) })}</span>
            <button type="submit" className="inline-flex min-h-10 items-center rounded-aw border border-border px-3 text-[0.9375rem] font-medium hover:bg-hover">{t("library.pageNav.go")}</button>
          </form>
          {/* In a very narrow pane the arrows beside the indicator fold away; they are here instead. */}
          <div className="hidden gap-2 @max-xs:flex">
            <button type="button" disabled={page <= 1} onClick={() => go(page - 1)} className="inline-flex min-h-10 items-center gap-1 rounded-aw border border-border px-2 text-[0.875rem] disabled:opacity-40"><ChevronLeft className="size-4" aria-hidden="true" />{t("library.pageNav.previous")}</button>
            <button type="button" disabled={page >= total} onClick={() => go(page + 1)} className="inline-flex min-h-10 items-center gap-1 rounded-aw border border-border px-2 text-[0.875rem] disabled:opacity-40">{t("library.pageNav.next")}<ChevronRight className="size-4" aria-hidden="true" /></button>
          </div>
          {invalid && <p role="alert" className="m-0 text-[0.875rem] text-danger">{t("library.pageNav.invalid", { total: formatNumber(total) })}</p>}
          <p id={`${id}-hint`} className="type-meta m-0">{t("library.pageNav.physical")}{printed ? ` ${printed}` : ""}</p>
        </div>, document.body
      )}
    </nav>
  );
}
