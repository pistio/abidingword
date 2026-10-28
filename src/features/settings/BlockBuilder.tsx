import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { BibleTranslation } from "@/types/scripture";
import type { FieldKind, TemplateField } from "@/types/domain";
import type { TemplateSection } from "@/types/studies";
import { ownPartKey } from "@/features/studies/templateParts";
import { Button } from "@/design-system/Button";

export const BUILDER_KINDS: FieldKind[] = ["writing", "guidance", "heading", "divider", "passage", "concordance", "cross_references", "word_lookup", "search"];

/** The editor changes definitions only; studies retain their independent snapshots. */
export function BlockBuilder({ section, onChange, editions = [] }: { section: TemplateSection; onChange: (changes: Partial<TemplateSection>) => void; editions?: BibleTranslation[] }) {
  const { t } = useTranslation();
  const [kind, setKind] = useState<FieldKind>("writing");
  const fields = section.fields ?? [];
  const save = (next: TemplateField[]) => onChange({ fields: next, fieldsVersion: Math.max(1, section.fieldsVersion ?? 0) });
  const update = (index: number, patch: Partial<TemplateField>) => save(fields.map((field, i) => i === index ? { ...field, ...patch } : field));
  const move = (index: number, delta: number) => {
    const next = [...fields];
    [next[index], next[index + delta]] = [next[index + delta]!, next[index]!];
    save(next);
  };
  return <details className="rounded-aw border border-border-subtle p-3">
    <summary className="cursor-pointer">{t("studies.builder.title")}</summary>
    <ol className="list-none space-y-3 p-0">
      {fields.map((field, index) => <li key={field.key} className="space-y-2 border-b border-border-subtle pb-3">
        <span>{t(`studies.builder.kinds.${field.kind}`, { defaultValue: field.kind })}</span>
        <label className="flex flex-col">{t("studies.builder.label")}<input className="aw-input px-2" value={field.label ?? ""} onChange={e => update(index, { label: e.target.value || undefined })} /></label>
        <label className="flex flex-col">{t("studies.builder.guidance")}<textarea className="aw-input px-2" value={field.config?.guidance ?? ""} onChange={e => update(index, { config: { ...field.config, guidance: e.target.value } })} /></label>
        {!["text", "entries", "writing", "guidance", "heading", "divider"].includes(field.kind) && <>
          <label className="flex flex-col">{t("studies.builder.query")}<input className="aw-input px-2" value={field.config?.query ?? ""} onChange={e => update(index, { config: { ...field.config, query: e.target.value } })} /></label>
          <label className="flex flex-col">{t("studies.builder.editionDefault")}<select className="aw-input" value={field.config?.translationId ?? ""} onChange={e => update(index, { config: { ...field.config, translationId: e.target.value } })}><option value="">{t("studies.builder.chooseEdition")}</option>{field.config?.translationId && !editions.some(edition => edition.id === field.config?.translationId) && <option value={field.config.translationId}>{field.config.translationId}</option>}{editions.filter(edition => edition.kind === "structured").map(edition => <option key={edition.id} value={edition.id}>{edition.name} ({edition.shortName})</option>)}</select></label>
        </>}
        {field.kind === "concordance" && <label>{t("scripture.concordance.modeLabel")} <select className="aw-input" value={field.config?.mode ?? "word"} onChange={e => update(index, { config: { ...field.config, mode: e.target.value } })}>{(["word", "phrase"] as const).map(mode => <option key={mode} value={mode}>{t(`scripture.concordance.mode.${mode}`)}</option>)}</select></label>}
        <div className="flex flex-wrap gap-2">
          <Button disabled={index === 0} onClick={() => move(index, -1)}>{t("studies.builder.up")}</Button>
          <Button disabled={index === fields.length - 1} onClick={() => move(index, 1)}>{t("studies.builder.down")}</Button>
          <Button onClick={() => save([...fields.slice(0, index + 1), { ...field, key: ownPartKey() }, ...fields.slice(index + 1)])}>{t("studies.builder.duplicate")}</Button>
          <Button onClick={() => save(fields.filter((_, i) => i !== index))}>{t("common.delete")}</Button>
        </div>
      </li>)}
    </ol>
    <label>{t("studies.builder.kind")} <select className="aw-input" value={kind} onChange={e => setKind(e.target.value as FieldKind)}>{BUILDER_KINDS.map(k => <option key={k} value={k}>{t(`studies.builder.kinds.${k}`)}</option>)}</select></label>
    <Button onClick={() => save([...fields, { key: ownPartKey(), kind, label: t(`studies.builder.kinds.${kind}`) }])}>{t("studies.builder.add")}</Button>
  </details>;
}
