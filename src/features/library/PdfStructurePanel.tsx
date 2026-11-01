import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { useApi } from "@/lib/ApiContext";
import { toApiError } from "@/lib/api";
import { usePdfEngine } from "./PdfEngineContext";
import type { ContentsItem, PdfCorrection, PdfResolvedHeading, SourceSegment } from "@/types/library";

/** App-owned metadata only. The original file and citation text remain intact. */
export function PdfStructurePanel({ sourceId, segments, onChanged, onGo, onPage, onClose }: {
  sourceId: string; segments: SourceSegment[]; onChanged: () => Promise<void>;
  onGo: (position: number, headingId: string) => void; onPage: (page:number)=>void; onClose: () => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const structure = useLoad(useCallback(() => api.libraryPdfStructure(sourceId), [api, sourceId]));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [search, setSearch] = useState("");
  const [originalOpen,setOriginalOpen]=useState(false);
  const [draft, setDraft] = useState<PdfCorrection | null>(null);
  const data = structure.data;
  const run = async (work: () => Promise<unknown>) => {
    setBusy(true); setError(null);
    try { await work(); structure.reload(); setDraft(null); await onChanged(); }
    catch (e) { setError(toApiError(e).message); }
    finally { setBusy(false); }
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
    {(error || structure.error) && <Notice tone="error">{error ?? structure.error?.message}</Notice>}
    {!data && !structure.error && <p role="status">{t("common.loading")}</p>}
    <details onToggle={e=>setOriginalOpen(e.currentTarget.open)}><summary>{t("library.structure.originalBookmarks")}</summary>{originalOpen && <OriginalPdfBookmarks sourceId={sourceId} onGo={onPage}/>}</details>
    {data && <div className="flex flex-col gap-3">
      <div className="flex flex-wrap gap-2">
        <Button disabled={busy} onClick={() => void run(() => api.libraryPdfStructureBuild(sourceId))}>{t(data.status === "missing" ? "library.structure.build" : "library.structure.rebuild")}</Button>
        <Button disabled={busy || !data.canUndo || !data.fileHash} onClick={() => void run(() => api.libraryPdfStructureUndo(sourceId,data.fileHash!,false))}>{t("library.structure.undo")}</Button>
        <Button disabled={busy || !data.corrections.length || !data.fileHash} onClick={() => void run(() => api.libraryPdfStructureUndo(sourceId,data.fileHash!,true))}>{t("library.structure.reset")}</Button>
      </div>
      {data.status === "ready" && <>
        <p className="type-meta">{t("library.structure.status", {count:data.headings.length})}</p>
        {(!data.complete || data.uncertainPages > 0 || data.unalignedTags > 0) && <Notice>{t("library.structure.uncertain")}</Notice>}
        <label>{t("library.structure.search")}<input className={fieldClass} value={search} onChange={e=>setSearch(e.target.value)} /></label>
        <ul className="m-0 max-h-64 list-none overflow-auto p-0">
          {headings.slice(0,50).map(h=><li key={h.id} className="border-b border-border-subtle py-2">
            <p className="m-0 break-words">{h.path}</p>
            <p className="type-meta m-0">{t(`library.structure.basis.${["outline","tagged","layout","user"].includes(h.basis) ? h.basis as "outline" | "tagged" | "layout" | "user" : "layout"}`)} · {t("library.structure.physical",{page:h.anchor.page})} · {h.canonical ?? t("library.structure.unmapped")}</p>
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
