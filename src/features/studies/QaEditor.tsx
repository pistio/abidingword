import { useCallback, useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, MoreHorizontal, Trash2 } from "lucide-react";
import { Button } from "@/design-system/Button";
import { useAutosave } from "@/hooks/useAutosave";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import type { StudyBlock } from "@/types/domain";
import { qaData, qaOf, type QaPair } from "./qa";

/**
 * A.4.4 (#27): a paired Question + Answer in a Study part. Both are the
 * person's own writing, separately editable, with no short length limit;
 * several can follow one another and be moved. Saved as one block (its
 * question and answer kept apart), so nothing is lost or merged.
 */
export function QaEditor({ block, number, isFirst, isLast, autoFocus, onMove, onDeleted, onChange }: {
  block: StudyBlock;
  number: number;
  isFirst: boolean;
  isLast: boolean;
  autoFocus?: boolean;
  onMove: (delta: -1 | 1) => void;
  onDeleted: (id: string) => void;
  onChange?: (block: StudyBlock) => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const [pair, setPair] = useState<QaPair>(() => qaOf(block.data) ?? { question: "", answer: "" });
  const latest = useRef(pair);
  const [open, setOpen] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const question = useRef<HTMLTextAreaElement>(null);
  const answer = useRef<HTMLTextAreaElement>(null);
  const save = useCallback(
    async (data: string) => {
      try {
        const saved = await api.studyBlockUpdate({ id: block.id, data });
        setError(null);
        onChange?.(saved);
      } catch (e) {
        setError(toApiError(e).message);
        throw e;
      }
    },
    [api, block.id, onChange],
  );
  const { schedule, flush } = useAutosave(save);
  useEffect(() => {
    if (autoFocus) question.current?.focus();
  }, [autoFocus]);
  useEffect(() => {
    for (const el of [question.current, answer.current]) {
      if (!el) continue;
      el.style.height = "auto";
      el.style.height = `${Math.max(el.scrollHeight, el === answer.current ? 120 : 48)}px`;
    }
  }, [pair]);
  const set = (key: keyof QaPair, value: string) => {
    const next = { ...latest.current, [key]: value };
    latest.current = next;
    setPair(next);
    schedule(qaData(next));
  };
  const remove = async () => {
    try {
      await api.studyBlockDelete(block.id);
      onDeleted(block.id);
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  const label = t("studies.qa.label", { number });
  const empty = !pair.question.trim() && !pair.answer.trim();
  const tool = "inline-flex min-h-11 items-center gap-1 rounded-aw px-2 text-[0.9375rem] text-text-muted hover:bg-hover hover:text-text-strong disabled:opacity-40";
  return (
    <div role="group" aria-label={label} data-qa-block={block.id} className="flex flex-col gap-1 border-l-[3px] border-border pl-3">
      <div className="flex items-center gap-1">
        <label htmlFor={`${id}-q`} className="type-meta flex-1">{t("studies.qa.question")}</label>
        <button
          type="button"
          aria-expanded={open}
          aria-label={t("studies.qa.options", { label })}
          title={t("studies.qa.options", { label })}
          onClick={() => setOpen((v) => !v)}
          className="inline-flex size-11 items-center justify-center rounded-aw text-text-muted hover:bg-hover hover:text-text-strong aria-expanded:bg-selected"
        >
          <MoreHorizontal className="size-4" aria-hidden="true" />
        </button>
      </div>
      {open && (
        <div role="group" aria-label={t("studies.qa.options", { label })} className="flex flex-wrap items-center gap-1 rounded-aw bg-bg-secondary px-1">
          <button type="button" className={tool} disabled={isFirst} onClick={() => onMove(-1)} aria-label={t("studies.qa.moveUp", { label })}>
            <ArrowUp className="size-4" aria-hidden="true" />
          </button>
          <button type="button" className={tool} disabled={isLast} onClick={() => onMove(1)} aria-label={t("studies.qa.moveDown", { label })}>
            <ArrowDown className="size-4" aria-hidden="true" />
          </button>
          <button type="button" className={`${tool} hover:text-danger`} onClick={() => (empty ? void remove() : setConfirming(true))}>
            <Trash2 className="size-4" aria-hidden="true" />
            {t("common.delete")}
          </button>
        </div>
      )}
      {confirming && (
        <div role="alertdialog" aria-label={t("studies.qa.confirmDelete", { label })} className="flex flex-wrap items-center gap-2 border-l-[3px] border-danger bg-hover px-3 py-2">
          <span>{t("studies.qa.confirmDelete", { label })}</span>
          <Button variant="danger" onClick={() => void remove()}>{t("common.delete")}</Button>
          <Button autoFocus onClick={() => setConfirming(false)}>{t("common.keep")}</Button>
        </div>
      )}
      <textarea
        id={`${id}-q`}
        ref={question}
        rows={2}
        value={pair.question}
        aria-label={`${label} — ${t("studies.qa.question")}`}
        onChange={(e) => set("question", e.target.value)}
        onBlur={flush}
        className="block w-full resize-none overflow-hidden rounded-aw border border-transparent bg-field px-3 py-2 font-medium text-text hover:border-border-subtle focus:border-border-strong"
      />
      <label htmlFor={`${id}-a`} className="type-meta mt-1">{t("studies.qa.answer")}</label>
      <textarea
        id={`${id}-a`}
        ref={answer}
        rows={5}
        value={pair.answer}
        aria-label={`${label} — ${t("studies.qa.answer")}`}
        onChange={(e) => set("answer", e.target.value)}
        onBlur={flush}
        className="block w-full resize-none overflow-hidden rounded-aw border border-transparent bg-field px-3 py-2 text-text reading hover:border-border-subtle focus:border-border-strong"
      />
      {error && <p role="alert" className="m-0 text-[0.9375rem] text-danger">{error}</p>}
    </div>
  );
}
