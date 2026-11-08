import { useCallback, useEffect, useLayoutEffect, useRef, useState, type ReactNode, type Ref } from "react";
import { useTranslation } from "react-i18next";
import { Settings2 } from "lucide-react";
import { ActionMenu, type MenuAction } from "@/design-system/ActionMenu";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { readLocal, writeLocal } from "@/lib/storage";

/**
 * A.4.2 (#26): one compact, adaptive reader toolbar for the Sources pane and
 * the full reader. Back, the Reading/Original switch and the page indicator
 * always stay in the row; the other actions are icons that can be pinned to
 * the row or left in the "…" menu (per device, with a reset). When the row is
 * too narrow, pinned icons move into the menu too: the toolbar never wraps
 * into extra rows that push the text down. Nothing is hidden from keyboard or
 * screen-reader users: every action is in the row or in the menu, by name.
 */
export type ReaderActionId =
  | "find" | "contents" | "history" | "bookmark" | "note" | "marks" | "structure" | "full" | "about" | "zoomIn" | "zoomOut" | "fitWidth" | "fitPage";

export interface ReaderAction {
  /** The pin it belongs to (Back and Forward share "history"). */
  id: ReaderActionId;
  /** Unique within the toolbar (for React and tests). */
  key: string;
  label: string;
  icon: ReactNode;
  onSelect: (opener: HTMLElement | null) => void;
  pressed?: boolean;
  expanded?: boolean;
  controls?: string;
  disabled?: boolean;
  group?: string;
}

/** Shown in the row unless the person changes it: the two the owner reaches for most. */
export const DEFAULT_PINS: ReaderActionId[] = ["find", "contents"];
export const PIN_ORDER: ReaderActionId[] = ["find", "contents", "history", "bookmark", "note", "marks", "structure", "full", "about", "zoomOut", "zoomIn", "fitWidth", "fitPage"];
const STORAGE_KEY = "reader.toolbar.pins";
/** One icon button, including its gap (44 px target + 4 px). */
const SLOT = 48;

export function readPins(): ReaderActionId[] {
  const stored = readLocal<unknown>(STORAGE_KEY, null);
  if (!Array.isArray(stored)) return DEFAULT_PINS;
  return stored.filter((x): x is ReaderActionId => typeof x === "string" && (PIN_ORDER as string[]).includes(x));
}

export function usePins() {
  const [pins, setPins] = useState<ReaderActionId[]>(readPins);
  const save = useCallback((next: ReaderActionId[] | null) => {
    writeLocal(STORAGE_KEY, next);
    setPins(next ?? DEFAULT_PINS);
  }, []);
  return { pins, save };
}

/** An icon button whose name shows as a tooltip on hover, keyboard focus and a long press (never the only way in). */
export function IconAction({ action, buttonRef }: { action: ReaderAction; buttonRef?: Ref<HTMLButtonElement> }) {
  const [hint, setHint] = useState(false);
  const timer = useRef<number | null>(null);
  const stop = () => { if (timer.current !== null) window.clearTimeout(timer.current); timer.current = null; };
  useEffect(() => stop, []);
  return (
    <span className="group relative inline-flex">
      <button
        ref={buttonRef}
        type="button"
        data-reader-action={action.key}
        aria-label={action.label}
        aria-pressed={action.pressed}
        aria-expanded={action.expanded}
        aria-controls={action.controls}
        disabled={action.disabled}
        title={action.label}
        onClick={(e) => { setHint(false); action.onSelect(e.currentTarget); }}
        onPointerDown={(e) => { if (e.pointerType === "touch") { stop(); timer.current = window.setTimeout(() => setHint(true), 500); } }}
        onPointerUp={stop}
        onPointerLeave={() => { stop(); setHint(false); }}
        onBlur={() => setHint(false)}
        className="inline-flex size-11 shrink-0 items-center justify-center rounded-aw text-text-strong hover:bg-hover disabled:cursor-not-allowed disabled:opacity-50 aria-pressed:bg-selected"
      >
        {action.icon}
      </button>
      <span
        aria-hidden="true"
        data-visible={hint}
        className="pointer-events-none absolute left-1/2 top-full z-40 mt-1 hidden -translate-x-1/2 whitespace-nowrap rounded-aw bg-text-strong px-2 py-1 text-[0.8125rem] text-bg group-hover:block group-has-[:focus-visible]:block data-[visible=true]:block"
      >
        {action.label}
      </span>
    </span>
  );
}

export function ReaderToolbar({ start, title, fixed, actions, label, menuLabel, refs = {} }: {
  /** Back: always first, never moved. */
  start: ReactNode;
  /** The full reader's title: takes the space that is left and truncates. */
  title?: ReactNode;
  /** The mode switch and page indicator: always present. */
  fixed?: ReactNode;
  actions: ReaderAction[];
  label: string;
  menuLabel?: string;
  /** Buttons focus returns to (by action key), when the action is in the row. */
  refs?: Partial<Record<string, Ref<HTMLButtonElement>>>;
}) {
  const { t } = useTranslation();
  const { pins, save } = usePins();
  const [customising, setCustomising] = useState(false);
  const row = useRef<HTMLDivElement>(null);
  const startPart = useRef<HTMLDivElement>(null);
  const fixedPart = useRef<HTMLDivElement>(null);
  // How many icons fit beside the fixed controls; null when it cannot be measured (all pinned show).
  const [room, setRoom] = useState<number | null>(null);
  const hasTitle = Boolean(title);
  const measure = useCallback(() => {
    const width = row.current?.getBoundingClientRect().width ?? 0;
    const used = (startPart.current?.getBoundingClientRect().width ?? 0) + (fixedPart.current?.getBoundingClientRect().width ?? 0);
    if (!width) { setRoom(null); return; }
    // The title (full reader) gets at least 6rem before icons are shown.
    const reserve = hasTitle ? 96 : 0;
    setRoom(Math.max(0, Math.floor((width - used - reserve - SLOT - 16) / SLOT)));
  }, [hasTitle]);
  useLayoutEffect(() => {
    measure();
    if (typeof ResizeObserver === "undefined" || !row.current) return;
    const observer = new ResizeObserver(() => measure());
    observer.observe(row.current);
    if (fixedPart.current) observer.observe(fixedPart.current);
    return () => observer.disconnect();
  }, [measure]);
  useLayoutEffect(() => { measure(); });

  const pinned = actions.filter((a) => pins.includes(a.id));
  const shown = room === null ? pinned : pinned.slice(0, room);
  const shownKeys = new Set(shown.map((a) => a.key));
  const rest = actions.filter((a) => !shownKeys.has(a.key));
  const menu: MenuAction[] = [
    ...rest.map((a) => ({ label: a.label, icon: a.icon, group: a.group, disabled: a.disabled, onSelect: (opener: HTMLButtonElement | null) => a.onSelect(opener) })),
    { label: t("library.toolbar.customise"), icon: <Settings2 className="size-4" aria-hidden="true" />, group: t("library.toolbar.settings"), onSelect: () => setCustomising(true) },
  ];
  const available = PIN_ORDER.filter((id) => actions.some((a) => a.id === id));

  return (
    <div
      ref={row}
      role="toolbar"
      aria-label={label}
      data-testid="reader-toolbar"
      className="@container relative z-20 flex min-h-12 shrink-0 flex-nowrap items-center gap-1 border-b border-border-subtle bg-bg px-1 py-0.5"
    >
      <div ref={startPart} className="flex shrink-0 flex-nowrap items-center">{start}</div>
      {title ? <div className="flex min-w-0 flex-1 items-center">{title}</div> : null}
      <div ref={fixedPart} className="flex shrink-0 flex-nowrap items-center gap-1">{fixed}</div>
      {title ? null : <div className="min-w-0 flex-1" />}
      <div className="flex shrink-0 flex-nowrap items-center gap-1">
        {shown.map((a) => <IconAction key={a.key} action={a} buttonRef={refs[a.key]} />)}
        <ActionMenu portal label={menuLabel ?? t("library.toolbar.more")} actions={menu} />
      </div>
      {customising && (
        <Dialog
          title={t("library.toolbar.customiseTitle")}
          onClose={() => setCustomising(false)}
          actions={<>
            <Button onClick={() => save(null)}>{t("library.toolbar.reset")}</Button>
            <Button variant="primary" onClick={() => setCustomising(false)}>{t("common.close")}</Button>
          </>}
        >
          <p className="m-0 type-meta">{t("library.toolbar.customiseHelp")}</p>
          <fieldset className="m-0 flex flex-col gap-1 border-0 p-0">
            <legend className="sr-only">{t("library.toolbar.showInToolbar")}</legend>
            {available.map((id) => {
              const sample = actions.find((a) => a.id === id)!;
              const checked = pins.includes(id);
              return (
                <label key={id} className="flex min-h-11 items-center gap-3">
                  <input
                    type="checkbox"
                    className="size-5"
                    checked={checked}
                    onChange={() => save(checked ? pins.filter((p) => p !== id) : PIN_ORDER.filter((p) => p === id || pins.includes(p)))}
                  />
                  <span className="inline-flex items-center gap-2">{sample.icon}{t(`library.toolbar.items.${id}`)}</span>
                </label>
              );
            })}
          </fieldset>
        </Dialog>
      )}
    </div>
  );
}
