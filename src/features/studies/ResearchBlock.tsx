import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { useApi } from "@/lib/ApiContext";
import { toApiError } from "@/lib/api";
import { useLoad } from "@/hooks/useLoad";
import { Button } from "@/design-system/Button";
import type { StudySection, TemplateField } from "@/types/domain";
import type { LemmaEntry } from "@/types/scripture";
import { structuredText } from "./builtinText";

export interface ResearchEvidence {
  kind: string;
  query: string;
  translationId: string;
  translationName: string;
  reference: string;
  text: string;
  lemma?: string;
  strong?: string;
}

/** All retained evidence carries the edition and query used to obtain it, independently of current controls. */
export function evidenceContent(evidence: ResearchEvidence): string {
  return `${evidence.reference} (${evidence.translationName})\n${evidence.text}`;
}

export function ResearchBlock({ section, field, onChange, onError }: {
  section: StudySection; field: TemplateField;
  onChange: (section: StudySection) => void; onError: (message: string | null) => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const editions = useLoad(useCallback(() => api.bibleList(), [api]));
  const [edition, setEdition] = useState(field.config?.translationId ?? "");
  const [query, setQuery] = useState(field.config?.query ?? "");
  const [busy, setBusy] = useState(false);
  const [saving, setSaving] = useState(false);
  const [hits, setHits] = useState<ResearchEvidence[]>([]);
  const [entries, setEntries] = useState<LemmaEntry[]>([]);
  const [incomplete, setIncomplete] = useState(false);
  const [searched, setSearched] = useState(false);
  const label = field.label ?? structuredText(section, `fields.${field.key}.label`, t(`studies.builder.kinds.${field.kind}`));
  const guidance = field.config?.guidance ?? structuredText(section, `fields.${field.key}.help`, "");
  if (field.kind === "divider") return <hr />;
  if (field.kind === "heading") return <h3 className="type-section">{label}</h3>;
  if (field.kind === "guidance") return <details><summary>{label}</summary><p className="whitespace-pre-wrap">{guidance}</p></details>;

  const run = async (entry?: LemmaEntry) => {
    const translation = editions.data?.find(item => item.id === edition && item.kind === "structured");
    if (!translation || !query.trim()) return;
    setBusy(true); setHits([]); setEntries([]); setIncomplete(false); setSearched(false); onError(null);
    const evidence = (reference: string, text: string): ResearchEvidence => ({ kind: field.kind, query, translationId: translation.id, translationName: `${translation.name} (${translation.shortName})`, reference, text, ...(entry ? { lemma: entry.lemma ?? "", strong: entry.strong ?? "" } : {}) });
    try {
      if (field.kind === "passage" || field.kind === "cross_references") {
        // Cross-references are passages explicitly supplied by the reader, never inferred connections.
        const references = field.kind === "cross_references" ? query.split(";").map(s => s.trim()).filter(Boolean) : [query];
        const results: ResearchEvidence[] = [];
        for (const reference of references) {
          const passage = await api.scriptureGetPassage({ translationId: translation.id, reference });
          if (passage.translationId !== translation.id) throw new Error(t("studies.builder.editionMismatch"));
          if (passage.verses.length) results.push(evidence(passage.reference.canonical, passage.verses.map(v => `${v.verse} ${v.text}`).join("\n")));
        }
        setHits(results);
      } else if (field.kind === "concordance") {
        const result = await api.scriptureConcordance({ translationId: translation.id, query, mode: field.config?.mode === "phrase" ? "phrase" : "word" });
        setHits(result.hits.map(hit => evidence(hit.reference.canonical, hit.text)));
        setIncomplete(!result.hitsComplete);
      } else if (field.kind === "word_lookup") {
        if (!entry) setEntries(await api.scriptureLemmaEntries({ translationId: translation.id, text: query }));
        else {
          const result = await api.scriptureLemmaSearch({ translationId: translation.id, selection: { kind: "entry", lemma: entry.lemma, strong: entry.strong } });
          setHits(result.hits.map(hit => evidence(hit.reference.canonical, hit.text)));
          setIncomplete(!result.hitsComplete);
        }
      } else {
        const result = await api.scriptureSearch({ translationId: translation.id, query });
        setHits(result.hits.filter(hit => hit.translationId === translation.id).map(hit => evidence(hit.reference.canonical, hit.text)));
        setIncomplete(result.truncated);
      }
      setSearched(true);
    } catch (error) { onError(toApiError(error).message); }
    finally { setBusy(false); }
  };
  const keep = async (evidence: ResearchEvidence) => {
    setSaving(true); onError(null);
    try {
      const block = await api.studyBlockCreate({ sectionId: section.id, blockType: "quotation", fieldKey: field.key });
      // Show the created block even if its subsequent write fails; a retry never hides it.
      const blocks = [...section.blocks, block];
      onChange({ ...section, blocks });
      const saved = await api.studyBlockUpdate({ id: block.id, content: evidenceContent(evidence), data: JSON.stringify(evidence) });
      onChange({ ...section, blocks: blocks.map(item => item.id === saved.id ? saved : item) });
    } catch (error) { onError(toApiError(error).message); }
    finally { setSaving(false); }
  };
  const remove = async (id: string) => {
    setSaving(true);
    try { await api.studyBlockDelete(id); onChange({ ...section, blocks: section.blocks.filter(block => block.id !== id) }); }
    catch (error) { onError(toApiError(error).message); }
    finally { setSaving(false); }
  };
  return <section className="space-y-3 rounded-aw border border-border p-4" aria-label={label}>
    <h3 className="m-0 font-medium">{label}</h3>
    {guidance && <details><summary>{t("studies.flow.guidance")}</summary><p>{guidance}</p></details>}
    <p className="text-text-muted">{t(field.kind === "cross_references" ? "studies.builder.crossReferenceNote" : "studies.builder.evidenceNote")}</p>
    <label className="flex flex-col">{t("studies.builder.edition")}<select className="aw-input" disabled={busy} value={edition} onChange={e => { setEdition(e.target.value); setEntries([]); }}><option value="">{t("studies.builder.chooseEdition")}</option>{editions.data?.filter(item => item.kind === "structured").map(item => <option key={item.id} value={item.id}>{item.name} ({item.shortName})</option>)}</select></label>
    {editions.error && <p role="alert">{editions.error.message}</p>}
    <label className="flex flex-col">{t("studies.builder.query")}<input className="aw-input px-2" disabled={busy} value={query} onChange={e => { setQuery(e.target.value); setEntries([]); }} /></label>
    <Button disabled={busy || !query.trim() || !editions.data?.some(item => item.id === edition && item.kind === "structured")} onClick={() => void run()}>{t("studies.builder.run")}</Button>
    {incomplete && <p role="status">{t("studies.builder.incomplete")}</p>}
    {searched && !hits.length && !entries.length && !incomplete && <p role="status">{t("studies.builder.empty")}</p>}
    {entries.map((entry, index) => <Button key={index} disabled={busy} onClick={() => void run(entry)}>{entry.lemma} {entry.strong} ({entry.occurrences})</Button>)}
    <ul className="list-none space-y-3 p-0">{hits.map((hit, index) => <li key={index}><blockquote className="m-0 whitespace-pre-wrap">{evidenceContent(hit)}</blockquote><Button disabled={saving} onClick={() => void keep(hit)}>{t("studies.builder.keep")}</Button></li>)}</ul>
    <h4>{t("studies.builder.evidence")}</h4>
    {section.blocks.filter(block => block.fieldKey === field.key).map(block => <div key={block.id}><blockquote className="whitespace-pre-wrap">{block.content}</blockquote><Button disabled={saving} onClick={() => void remove(block.id)}>{t("common.delete")}</Button></div>)}
  </section>;
}
