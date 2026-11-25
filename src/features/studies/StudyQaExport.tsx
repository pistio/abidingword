import { useId, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import type { StudySection } from "@/types/domain";
import { sectionTitle } from "./builtinText";
import { qaOf } from "./qa";

export type QaExportMode = "questions" | "answers" | "full";

/**
 * A.4.4 (#27): a Study as Markdown: its questions only, its questions with
 * their answers, or the whole study's writing. Only the person's writing in
 * the study's parts; its separate notes are never included.
 */
export function studyMarkdown(t: TFunction, title: string, sections: StudySection[], mode: QaExportMode, templateKey?: string): string {
  const out: string[] = [`# ${title.trim()}`];
  let number = 0;
  for (const section of sections) {
    const part: string[] = [];
    for (const block of section.blocks) {
      const pair = block.blockType === "question" ? qaOf(block.data) : null;
      if (pair) {
        if (!pair.question.trim()) continue;
        number += 1;
        if (mode === "questions") part.push(`${number}. ${pair.question.trim()}`);
        else {
          part.push(`**${number}. ${pair.question.trim()}**`);
          if (pair.answer.trim()) part.push(pair.answer.trim());
        }
      } else if (mode === "full" && block.content.trim()) {
        part.push(block.content.trim());
      }
    }
    if (part.length) out.push(`## ${sectionTitle(section, templateKey)}`, ...part);
  }
  if (out.length === 1) out.push(t("studies.qaExport.empty"));
  return `${out.join("\n\n")}\n`;
}

export function StudyQaExport({ title, sections, templateKey, onClose }: { title: string; sections: StudySection[]; templateKey?: string; onClose: () => void }) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const [mode, setMode] = useState<QaExportMode>("questions");
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const text = useMemo(() => studyMarkdown(t, title, sections, mode, templateKey), [t, title, sections, mode, templateKey]);
  const save = async () => {
    setError(null);
    setStatus(null);
    try {
      const path = await api.chooseMaterialFile({ defaultName: `${title.trim() || t("studies.view.study")}.md`, filterName: t("studies.qaExport.fileType") });
      if (!path) return;
      setStatus(t("studies.qaExport.saved", { path: await api.materialSaveMarkdown({ path, text }) }));
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  return (
    <Dialog
      wide
      title={t("studies.qaExport.title")}
      onClose={onClose}
      actions={<>
        <Button onClick={onClose}>{t("common.close")}</Button>
        <Button variant="primary" onClick={() => void save()}>{t("studies.qaExport.save")}</Button>
      </>}
    >
      <p className="m-0 type-meta">{t("studies.qaExport.help")}</p>
      <div role="radiogroup" aria-label={t("studies.qaExport.mode")} className="flex flex-wrap gap-1">
        {(["questions", "answers", "full"] as const).map((m) => (
          <label key={m} className="aw-choice aw-choice-card min-h-11 cursor-pointer gap-2 px-3">
            <input type="radio" name={`${id}-mode`} checked={mode === m} onChange={() => setMode(m)} className="sr-only" />
            {t(`studies.qaExport.modes.${m}`)}
          </label>
        ))}
      </div>
      {error && <Notice tone="error">{error}</Notice>}
      {status && <p role="status" className="type-meta m-0 break-words">{status}</p>}
      <pre data-testid="qa-export-preview" className="m-0 max-h-[45vh] overflow-auto whitespace-pre-wrap rounded-aw border border-border-subtle bg-bg-secondary p-3 text-[0.875rem]">{text}</pre>
    </Dialog>
  );
}
