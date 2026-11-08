import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { useApi } from "@/lib/ApiContext";
import { toApiError } from "@/lib/api";
import { usePdfEngine } from "./PdfEngineContext";
import type { ContentsItem, PdfBuildProgress, PdfCorrection, PdfDiagnostic, PdfResolvedHeading, ReaderObserved, SourceSegment } from "@/types/library";

/** App-owned metadata only. The original file and citation text remain intact. */
export function PdfStructurePanel({ sourceId, segments, onChanged, onGo, onPage, onClose, observed }: {
  sourceId: string; segments: SourceSegment[]; onChanged: () => Promise<void>;
  onGo: (position: number, headingId: string) => void; onPage: (page:number)=>void; onClose: () => void;
  /** The Reader's state now (mode, page, the place it was opened at), for the diagnostic report. */
  observed?: () => ReaderObserved;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const structure = useLoad(useCallback(() => api.libraryPdfStructure(sourceId), [api, sourceId]));
  const [editingBusy, setBusy] = useState(false);
  const [progress, setProgress] = useState<PdfBuildProgress | null>(null);
  const [progressError, setProgressError] = useState<string | null>(null);
  const [cancelling, setCancelling] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const [refreshMs, setRefreshMs] = useState<number | null>(null);
  const busy = editingBusy || progress?.status === "running";
  useEffect(() => {
    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    const poll = async () => {
      try {
        const next = await api.libraryPdfStructureProgress(sourceId);
        if (active) { setProgress(next); setProgressError(null); }
      } catch (e) { if (active) setProgressError(toApiError(e).message); }
      finally { if (active) timer = setTimeout(() => void poll(), 500); }
    };
    void poll();
    return () => { active = false; clearTimeout(timer); };
  }, [api, sourceId]);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [originalOpen,setOriginalOpen]=useState(false);
  const [draft, setDraft] = useState<PdfCorrection | null>(null);
  const data = structure.data;
  const unsavedAttempt = progress && ["failed", "cancelled"].includes(progress.status);
  const run = async (work: () => Promise<unknown>) => {
    setBusy(true); setError(null);
    try { await work(); structure.reload(); setDraft(null); await onChanged(); }
    catch (e) { setError(toApiError(e).message); }
    finally { setBusy(false); }
  };
  const build = async () => {
    setBusy(true); setError(null); setCancelling(false); setRefreshMs(null);
    try {
      const next = await api.libraryPdfStructureBuild(sourceId);
      structure.setData(next);
      setDraft(null); setRefreshing(true);
      const began = performance.now();
      try { await onChanged(); } finally { setRefreshMs(Math.round(performance.now() - began)); }
    } catch (e) { setError(toApiError(e).message); }
    finally {
      setBusy(false); setRefreshing(false);
      try { setProgress(await api.libraryPdfStructureProgress(sourceId)); }
      catch (e) { setProgressError(toApiError(e).message); }
    }
  };
  const cancel = async () => {
    try { setCancelling(await api.libraryPdfStructureCancel(sourceId)); }
    catch (e) { setError(toApiError(e).message); }
  };
  const edit = (h: PdfResolvedHeading) => setDraft({id:h.id,position:h.segmentPosition,title:h.title,level:h.level,mapping:h.canonical,scope:h.scope === "unmapped" ? null : h.scope,disabled:false,anchorId:`p${h.anchor.page}-c${h.anchor.firstChar}`,start:h.start,end:h.end});
  const add = (segment: SourceSegment, index: number) => {
    const anchor = segment.presentation?.anchors[index];
    if (!anchor) return;
    const text = anchor.originalText.split(/\s+/u).join(" ").trim();
    const start = segment.text.indexOf(text);
    if (start < 0 || segment.text.indexOf(text, start + text.length) >= 0) return;
    setDraft({id:`manual-${crypto.randomUUID()}`,position:segment.position,title:text.slice(0,300),level:2,mapping:null,scope:null,disabled:false,anchorId:`p${anchor.page}-c${anchor.firstChar}`,start,end:start+text.length});
  };
  const fieldClass = "w-full rounded-md border border-border bg-surface px-3 py-2 text-text";
  const headings = (data?.headings ?? []).filter(h => h.path.toLocaleLowerCase().includes(search.toLocaleLowerCase()));
  return <Dialog wide title={t("library.structure.title")} onClose={onClose} actions={<Button onClick={onClose}>{t("common.close")}</Button>}>
    <p>{t("library.structure.explanation")}</p>
    {progressError && <Notice tone="error">{progressError}</Notice>}
    {(busy || progress) && <div className="flex flex-col gap-2" aria-live="polite">
      <p role="status">{t("library.structure.processing.stage", { stage: t(`library.structure.processing.stages.${refreshing ? "refresh" : progress?.stage ?? "file"}`), seconds: Math.floor((progress?.elapsedMs ?? 0) / 1000) })}</p>
      {progress?.status === "running" && <>
        <progress aria-label={t("library.structure.processing.label")} {...(progress.total ? { value: progress.done, max: progress.total } : {})} />
        {progress.total > 0 && <span className="type-meta">{progress.done} / {progress.total}</span>}
        <p className="type-meta">{t("library.structure.processing.background")}</p>
        <Button disabled={!progress.canCancel || cancelling} onClick={() => void cancel()}>{t(cancelling ? "library.structure.processing.cancelling" : "common.cancel")}</Button>
      </>}
      {progress?.status === "succeeded" && <Notice tone="success">{t("library.structure.processing.success", { count: progress.headingCount ?? 0, links: progress.indexedLinks ?? 0 })}</Notice>}
      {progress?.status === "empty" && <Notice tone="warning">{t("library.structure.processing.empty")}</Notice>}
      {progress?.status === "cancelled" && <Notice>{t("library.structure.cancelled")}</Notice>}
      {progress?.status === "failed" && <Notice tone="error">{t(progress.errorCode === "library.structure.timeout" ? "library.structure.timeout" : "library.structure.processing.failed")}</Notice>}
      {progress && progress.status !== "running" && <details><summary>{t("library.structure.processing.timings")}</summary><ul>{progress.timings.map((m, i) => <li key={i}>{t(`library.structure.processing.stages.${m.stage as PdfBuildProgress["stage"]}`)}: {t("library.structure.processing.duration", { milliseconds: m.milliseconds })}</li>)}{refreshMs !== null && <li>{t("library.structure.processing.stages.refresh")}: {t("library.structure.processing.duration", { milliseconds: refreshMs })}</li>}</ul></details>}
    </div>}
    {(error || structure.error) && <Notice tone="error">{error ?? structure.error?.message}</Notice>}
    {!data && !structure.error && <p role="status">{t("common.loading")}</p>}
    <details onToggle={e=>setOriginalOpen(e.currentTarget.open)}><summary>{t("library.structure.originalBookmarks")}</summary>{originalOpen && <OriginalPdfBookmarks sourceId={sourceId} onGo={onPage}/>}</details>
    {data && <div className="flex flex-col gap-3">
      <div className="flex flex-wrap gap-2">
        <Button disabled={busy} onClick={() => void build()}>{t(data.status === "missing" ? "library.structure.build" : "library.structure.rebuild")}</Button>
        <Button disabled={busy || !data.canUndo || !data.fileHash} onClick={() => void run(() => api.libraryPdfStructureUndo(sourceId,data.fileHash!,false))}>{t("library.structure.undo")}</Button>
        <Button disabled={busy || !data.corrections.length || !data.fileHash} onClick={() => void run(() => api.libraryPdfStructureUndo(sourceId,data.fileHash!,true))}>{t("library.structure.reset")}</Button>
      </div>
      {data.status === "missing" && <Notice>{t("library.structure.processing.missing")}</Notice>}
      {data.status === "ready" && !unsavedAttempt && !data.headings.length && progress?.status !== "empty" && <Notice tone="warning">{t("library.structure.processing.empty")}</Notice>}
      {data.status === "ready" && data.rebuildRecommended && <Notice>{t("library.structure.rebuildRecommended")}</Notice>}
      {unsavedAttempt && data.status === "ready" && <Notice>{t("library.structure.processing.previous")}</Notice>}
      {data.status === "ready" && <>
        <p className="type-meta">{t("library.structure.status", {count:data.headings.length})}</p>
        {(!data.complete || data.uncertainPages > 0 || data.unalignedTags > 0) && <Notice>{t("library.structure.uncertain")}</Notice>}
        <label>{t("library.structure.search")}<input className={fieldClass} value={search} onChange={e=>setSearch(e.target.value)} /></label>
        <ul className="m-0 max-h-64 list-none overflow-auto p-0">
          {headings.slice(0,50).map(h=><li key={h.id} className="border-b border-border-subtle py-2">
            <p className="m-0 break-words">{h.path}</p>
            <p className="type-meta m-0">{t(`library.structure.basis.${["outline","printed","tagged","layout","user"].includes(h.basis) ? h.basis as "outline" | "printed" | "tagged" | "layout" | "user" : "layout"}`)} · {t("library.structure.physical",{page:h.anchor.page})} · {h.canonical ?? t("library.structure.unmapped")}</p>
            <Button variant="quiet" onClick={()=>onGo(h.segmentPosition,h.id)}>{t("library.structure.open")}</Button>
            <Button variant="quiet" disabled={busy} onClick={()=>edit(h)}>{t("library.structure.edit")}</Button>
          </li>)}
        </ul>
        {headings.length > 50 && <p className="type-meta">{t("library.structure.refine")}</p>}
        <details><summary>{t("library.structure.add")}</summary>
          <p className="type-meta">{t("library.structure.addHelp")}</p>
          <div className="max-h-48 overflow-auto">{segments.flatMap(segment => (segment.presentation?.anchors ?? []).map((anchor,index) => <Button key={`${segment.id}-${index}`} variant="quiet" className="block max-w-full truncate" onClick={()=>add(segment,index)}>{anchor.originalText}</Button>))}</div>
        </details>
      </>}
      <StructureDiagnostic key={sourceId} sourceId={sourceId} observed={observed} />
      {draft && <form className="flex flex-col gap-3 border-t border-border pt-3" onSubmit={e=>{e.preventDefault(); if(data.fileHash) void run(()=>api.libraryPdfStructureCorrect(sourceId,data.fileHash!,draft));}}>
        <label>{t("library.structure.heading")}<input required maxLength={300} className={fieldClass} value={draft.title} onChange={e=>setDraft({...draft,title:e.target.value})}/></label>
        <label>{t("library.structure.level")}<input required type="number" min={1} max={12} className={fieldClass} value={draft.level} onChange={e=>setDraft({...draft,level:Number(e.target.value)})}/></label>
        <label>{t("library.structure.mapping")}<input className={fieldClass} placeholder={t("library.structure.mappingExample")} value={draft.mapping ?? ""} onChange={e=>setDraft({...draft,mapping:e.target.value})}/></label>
        <p className="type-meta m-0">{t("library.structure.mappingHelp")}</p>
        <label>{t("library.structure.scope")}<select className={fieldClass} value={draft.scope ?? "passage"} onChange={e=>setDraft({...draft,scope:e.target.value})}>{(["passage","chapter","book"] as const).map(scope=><option key={scope} value={scope}>{t(`library.structure.scopes.${scope}`)}</option>)}</select></label>
        <label><input type="checkbox" checked={draft.disabled} onChange={e=>setDraft({...draft,disabled:e.target.checked})}/> {t("library.structure.disable")}</label>
        <div className="flex gap-2"><Button type="submit" disabled={busy}>{t("common.save")}</Button><Button type="button" onClick={()=>setDraft(null)}>{t("common.cancel")}</Button></div>
      </form>}
    </div>}
  </Dialog>;
}

/** A.4.1: a local, metadata-only report of how the structure was read and
 * which place a passage opens, to copy or save for the owner's own check. */
function StructureDiagnostic({ sourceId, observed }: { sourceId: string; observed?: () => ReaderObserved }) {
  const { t } = useTranslation();
  const api = useApi();
  const [passage, setPassage] = useState("");
  const [report, setReport] = useState<PdfDiagnostic | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const input = () => passage.trim() || null;
  const create = async () => {
    setBusy(true); setError(null); setStatus(null);
    try { setReport(await api.libraryPdfStructureDiagnostic(sourceId, input(), observed?.() ?? null)); }
    catch (e) { setError(toApiError(e).message); }
    finally { setBusy(false); }
  };
  const copy = async () => {
    if (!report) return;
    try { await navigator.clipboard.writeText(JSON.stringify(report, null, 2)); setStatus(t("library.structure.diagnostic.copied")); }
    catch { setError(t("library.structure.diagnostic.copyFailed")); }
  };
  const saveFile = async () => {
    setBusy(true); setError(null); setStatus(null);
    const query = input(); const observation = observed?.() ?? null;
    try {
      const path = await api.chooseDiagnosticFile("abidingword-structure-diagnostic.json", t("library.structure.diagnostic.fileType"));
      if (!path) return;
      const saved = await api.libraryPdfStructureDiagnosticSave(sourceId, query, observation, path);
      setReport(saved.report);
      setStatus(t("library.structure.diagnostic.saved", { path: saved.path }));
    } catch (e) { setError(toApiError(e).message); }
    finally { setBusy(false); }
  };
  const q = report?.query;
  return <details className="border-t border-border pt-3" data-structure-diagnostic>
    <summary>{t("library.structure.diagnostic.title")}</summary>
    <div className="flex flex-col gap-2 pt-2">
      <p className="type-meta m-0">{t("library.structure.diagnostic.help")}</p>
      <label>{t("library.structure.diagnostic.passage")}
        <input className="w-full rounded-md border border-border bg-surface px-3 py-2 text-text" value={passage} placeholder={t("library.structure.mappingExample")} disabled={busy} onChange={e => { setPassage(e.target.value); setReport(null); setStatus(null); }} />
      </label>
      <p className="type-meta m-0">{t("library.structure.diagnostic.queryHelp")}</p>
      <div className="flex flex-wrap gap-2">
        <Button disabled={busy} onClick={() => void create()}>{t("library.structure.diagnostic.create")}</Button>
        <Button disabled={!report} onClick={() => void copy()}>{t("library.structure.diagnostic.copy")}</Button>
        <Button disabled={busy} onClick={() => void saveFile()}>{t("library.structure.diagnostic.save")}</Button>
      </div>
      {busy && <p role="status">{t("common.loading")}</p>}
      {error && <Notice tone="error">{error}</Notice>}
      {status && <p role="status" className="type-meta m-0 break-words">{status}</p>}
      {report && <div className="flex flex-col gap-1" aria-live="polite">
        <p className="m-0 type-meta break-all">{t("library.structure.diagnostic.file", { hash: report.source.fileSha256 ?? "—", pages: report.structure.pageCount ?? "—", headings: report.structure.headingCount })}</p>
        {!q && <Notice>{t("library.structure.diagnostic.general")}</Notice>}
        {report.structure.rebuildRecommended && <Notice>{t(report.structure.status === "none" ? "library.structure.processing.missing" : "library.structure.rebuildRecommended")}</Notice>}
        {q && <>
          <p className="m-0">{q.verdict}</p>
          <ol className="m-0 pl-5 type-meta">
            {q.candidates.slice(0, 5).map(c => <li key={`${c.rank}-${c.segmentPosition}`}>{c.evidence} · {c.matched} · {t("library.structure.physical", { page: c.physicalPage ?? "?" })}{c.printedPage ? ` (${c.printedPage})` : ""}</li>)}
          </ol>
        </>}
      </div>}
    </div>
  </details>;
}

/** Supplied page-only bookmarks stay available even when no exact stored-text
 * alignment exists. They are explicitly separate from corrected section maps. */
function OriginalPdfBookmarks({sourceId,onGo}:{sourceId:string;onGo:(page:number)=>void}) {
  const {t}=useTranslation();const api=useApi();const engine=usePdfEngine();
  const outline=useLoad(useCallback(()=>engine.outline(api,sourceId),[engine,api,sourceId]));
  const [search,setSearch]=useState("");
  const entries:{id:string;title:string;page:number}[]=[];
  const visit=(items:ContentsItem[],parents:string[])=>{for(const item of items){const path=[...parents,item.title];if(item.target?.kind==="page")entries.push({id:item.id,title:path.join(" › "),page:item.target.page});visit(item.children,path);}};
  visit(outline.data ?? [],[]);
  const shown=entries.filter(e=>e.title.toLocaleLowerCase().includes(search.toLocaleLowerCase()));
  return <div className="flex flex-col gap-2">
    <p className="type-meta">{t("library.structure.originalBookmarksHelp")}</p>
    {outline.error && <Notice tone="error">{outline.error.message}</Notice>}
    <label>{t("library.structure.search")}<input className="aw-input w-full" value={search} onChange={e=>setSearch(e.target.value)}/></label>
    <ul className="m-0 max-h-64 list-none overflow-auto p-0">{shown.slice(0,50).map(entry=><li key={entry.id}><Button variant="quiet" className="w-full text-left" onClick={()=>onGo(entry.page)}>{entry.title} · {t("library.structure.physical",{page:entry.page})}</Button></li>)}</ul>
    {shown.length>50 && <p className="type-meta">{t("library.structure.refine")}</p>}
  </div>;
}
