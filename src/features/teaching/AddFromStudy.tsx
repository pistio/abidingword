import { useCallback, useId, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { referenceLabel } from "@/i18n/scripture";
import { sectionTitle } from "@/features/studies/builtinText";
import { findingLabel } from "@/features/studies/labels";
import { qaOf } from "@/features/studies/qa";
import type { StudyItemRef, TeachingPreparation } from "@/types/teaching";

type Group = "writing" | "qa" | "finding" | "citation" | "passage";
interface Choice extends StudyItemRef {
  group: Group;
  preview: string;
}

const short = (text: string, max = 140) => {
  const flat = text.replace(/\s+/g, " ").trim();
  return flat.length > max ? `${flat.slice(0, max - 1)}…` : flat;
};

/**
 * A.4.4 (#27): "Add from Bible Study". The person chooses what to copy from
 * the teaching's own study: its writing, questions and answers, notes,
 * quotations and passages. Each becomes a new, editable block with its origin
 * named; nothing is synchronised, nothing existing is replaced, and every copy
 * starts for the leader only until the person shares it.
 */
export function AddFromStudy({ prep, onAdded, onClose }: {
  prep: TeachingPreparation;
  onAdded: (next: TeachingPreparation, count: number) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const studyId = prep.studyId!;
  const detail = useLoad(useCallback(() => api.studyGet(studyId), [api, studyId]));
  const findings = useLoad(useCallback(() => api.findingList(studyId), [api, studyId]));
  const passages = useLoad(useCallback(() => api.studyPassageList(studyId), [api, studyId]));
  const [chosen, setChosen] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const choices = useMemo<Choice[]>(() => {
    const out: Choice[] = [];
    for (const section of detail.data?.sections ?? []) {
      const name = sectionTitle(section);
      for (const block of section.blocks) {
        const pair = block.blockType === "question" ? qaOf(block.data) : null;
        if (pair) {
          if (pair.question.trim() || pair.answer.trim()) out.push({ group: "qa", kind: "qa", id: block.id, label: short(pair.question || name, 80), preview: short(pair.answer) });
        } else if (block.content.trim()) {
          out.push({ group: "writing", kind: "writing", id: block.id, label: name, preview: short(block.content) });
        }
      }
    }
    for (const finding of findings.data ?? []) {
      if (finding.content.trim()) out.push({ group: "finding", kind: "finding", id: finding.id, label: findingLabel(t, finding.findingType), preview: short(finding.content) });
      for (const source of finding.sources) {
        if (source.quote?.trim()) out.push({ group: "citation", kind: "citation", id: source.id, label: t("studies.teaching.fromStudy.quote"), preview: short(`„${source.quote}“ — ${[source.title, source.author, source.locator].filter(Boolean).join(", ")}`) });
      }
    }
    for (const passage of passages.data ?? []) out.push({ group: "passage", kind: "passage", id: passage.id, label: referenceLabel(passage.reference), preview: "" });
    return out;
  }, [detail.data, findings.data, passages.data, t]);

  const loading = !detail.data || !findings.data || !passages.data;
  const loadError = detail.error ?? findings.error ?? passages.error;
  const groups: Group[] = ["writing", "qa", "finding", "citation", "passage"];
  const toggle = (key: string) => setChosen((now) => {
    const next = new Set(now);
    if (next.has(key)) next.delete(key);
    else next.add(key);
    return next;
  });
  const add = async () => {
    setBusy(true);
    setError(null);
    try {
      const items = choices.filter((c) => chosen.has(`${c.kind}:${c.id}`)).map(({ kind, id, label }) => ({ kind, id, label }));
      const next = await api.teachingAddFromStudy({ preparationId: prep.id, items });
      onAdded(next, items.length);
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      wide
      title={t("studies.teaching.fromStudy.title")}
      onClose={onClose}
      actions={<>
        <Button onClick={onClose}>{t("common.cancel")}</Button>
        <Button variant="primary" disabled={busy || chosen.size === 0} onClick={() => void add()}>{t("studies.teaching.fromStudy.add")}</Button>
      </>}
    >
      <p className="m-0 type-meta">{t("studies.teaching.fromStudy.help")}</p>
      {loadError && <Notice tone="error">{loadError.message}</Notice>}
      {error && <Notice tone="error">{error}</Notice>}
      {loading && !loadError && <p role="status">{t("common.loading")}</p>}
      {!loading && choices.length === 0 && <p className="m-0">{t("studies.teaching.fromStudy.none")}</p>}
      <div className="flex max-h-[55vh] flex-col gap-3 overflow-y-auto">
        {groups.map((group) => {
          const here = choices.filter((c) => c.group === group);
          if (!here.length) return null;
          return (
            <fieldset key={group} className="m-0 flex flex-col gap-1 border-0 p-0">
              <legend className="type-subhead mb-1">{t(`studies.teaching.fromStudy.groups.${group}`)}</legend>
              {here.map((c) => {
                const key = `${c.kind}:${c.id}`;
                return (
                  <label key={key} className="flex min-h-11 items-start gap-3 rounded-aw px-1 py-1 hover:bg-hover">
                    <input type="checkbox" className="mt-1 size-5 shrink-0" checked={chosen.has(key)} onChange={() => toggle(key)} aria-describedby={c.preview ? `${id}-${key}` : undefined} />
                    <span className="flex min-w-0 flex-col">
                      <span className="font-medium">{c.label}</span>
                      {c.preview && <span id={`${id}-${key}`} className="type-meta break-words">{c.preview}</span>}
                    </span>
                  </label>
                );
              })}
            </fieldset>
          );
        })}
      </div>
    </Dialog>
  );
}
