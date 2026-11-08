import { Fragment, useCallback, useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowLeft, Bookmark, BookOpenText, FileText, Highlighter, Info, List, ListTree, Maximize2, Redo2, Search, StickyNote, Undo2, ZoomIn, ZoomOut, MoveHorizontal } from "lucide-react";
import { ActionMenu } from "@/design-system/ActionMenu";
import { Button } from "@/design-system/Button";
import { Notice } from "@/design-system/Notice";
import { Dialog } from "@/design-system/Dialog";
import { useLoad } from "@/hooks/useLoad";
import { formatNumber } from "@/i18n/format";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { hostOf } from "@/lib/externalLink";
import type { PdfPlace } from "@/lib/pdfEngine";
import type { PdfHeading, ContentsItem, SegmentWindow, SourceAnnotation, SourceLink, SourceMatch, SourceReference, SourceSegment } from "@/types/library";
import { PHRASE_CLASS } from "@/lib/annotationRender";
import type { ScriptureRef } from "@/types/scripture";
import { citationText, formatLabel, locatorLabel, quotationWithCitation, sourceTypeLabel } from "./labels";
import { SourceAbout } from "./SourceAbout";
import { useLibraryNav, type QuotedWords } from "./LibraryNav";
import { PassageLinkNote } from "./PassageLinkNote";
import { ScriptureReference } from "./ScriptureReference";
import { ContentsPanel, useContentsLayout } from "./ContentsPanel";
import { EpubViewer } from "./formatted/EpubViewer";
import { activeMatch, FindBar, useSourceFind } from "./FindBar";
import { placeOfPosition, positionOfPlace, type EpubPlace } from "./formatted/epubLocation";
import { useEpubStructure } from "./formatted/useEpubStructure";
import { PdfStructurePanel } from "./PdfStructurePanel";
import { PdfPageNavigation } from "./PdfPageNavigation";
import { ReaderToolbar, type ReaderAction } from "./ReaderToolbar";
import { PdfViewer, type PdfFind, type PdfControls, type PdfControlState, type Zoom } from "./PdfViewer";
import { canGoBack, canGoForward, NO_HISTORY, stepBack, stepForward, visit, type ReaderHistory, type SourceLocation } from "./readerHistory";
import { currentEntry, useSourceContents } from "./useSourceContents";
import { MarksInspector, MarksMarker, type MarkActions } from "./annotations/ParagraphMarks";
import { AddToStudyPanel, type FromSource } from "@/features/studies/AddToStudyPanel";
import { copyText } from "@/lib/clipboard";
import { SourceMarkStrip, type PlaceRequest } from "./annotations/SourceMarkStrip";
import { ComparisonTray, CompareHelp, SourceHelp } from "./SourceHelp";
import { useComparison } from "@/features/ai/Comparison";
import { SourceMarksPanel } from "./annotations/SourceMarksPanel";
import { selectionInside, type SourceSelection } from "./annotations/sourceSelection";
import { useSourceAnnotations, type MarkedRange } from "./annotations/useSourceAnnotations";

export type ReaderMode = "reading" | "original";

/** The view last chosen for a source (v0.13), kept on this computer only. Never a global default. */
const modeKey = (sourceId: string) => `abidingword.reader.mode.${sourceId}`;
function rememberedMode(sourceId: string): ReaderMode | null {
  try {
    const value = window.localStorage.getItem(modeKey(sourceId));
    return value === "reading" || value === "original" ? value : null;
  } catch {
    return null;
  }
}
function rememberMode(sourceId: string, mode: ReaderMode) {
  try {
    window.localStorage.setItem(modeKey(sourceId), mode);
  } catch {
    // Remembering the view is a convenience; without storage the reader opens in Reading.
  }
}

/**
 * Where a reader is: which view, and the physical page (PDF) that view is
 * at. Kept by the shell for the current session so that moving between the
 * Sources pane and the full reader (and back) does not lose the place.
 */
export interface ReaderPosition {
  mode: ReaderMode;
  /** Physical page, or null for a source that has none (or before it is known). */
  page: number | null;
  /** The Back / Forward history of this reading session, kept while moving between the pane and the full reader. */
  history?: ReaderHistory;
  /** Exact text offset, PDF page fraction or formatted document place, to continue there. */
  place?: SourceLocation;
}

interface SourceReaderProps {
  sourceId: string;
  segmentId?: string;
  structureNodeId?: string;
  /** Leaves the reader. Told where it was, so the next reader can open there. */
  onClose: (position: ReaderPosition) => void;
  /**
   * "page" (default): the full-width reader that replaces the workspace.
   * "pane": inside the Sources pane, beside Scripture and the Study, with
   * "Open full reader" to switch to the page.
   */
  variant?: "page" | "pane";
  /** Pane only: open this source in the full-width reader, at the same place. */
  onExpand?: (position: ReaderPosition) => void;
  /** Where to open (view and page), when continuing from another reader. */
  initialPosition?: ReaderPosition;
  /** Cited words to show marked in Reading (a Study finding's citation, found exactly). */
  quotedWords?: QuotedWords;
  onPositionReader?: (sourceId: string, read: (() => ReaderPosition | null) | null) => void;
  backLabel?: string;
}

/**
 * A plain reader for a library source, for context and checking citations:
 * extracted text with page markers (PDF) or section headings (EPUB,
 * Markdown). Opens at an exact segment and loads more around it on demand.
 */
function scrollInReader(viewport: HTMLElement | null, element: Element | null | undefined, block: "start" | "center") {
  if (!viewport || !element || !viewport.contains(element)) return;
  // Keep native target alignment, but prevent it scrolling the workspace,
  // neighbouring panes or the document. Restore ancestors before painting.
  const ancestors: { element: HTMLElement; top: number; left: number }[] = [];
  for (let parent = viewport.parentElement; parent; parent = parent.parentElement) {
    ancestors.push({ element: parent, top: parent.scrollTop, left: parent.scrollLeft });
  }
  element.scrollIntoView?.({ block });
  for (const saved of ancestors) { saved.element.scrollTop = saved.top; saved.element.scrollLeft = saved.left; }
}

export function SourceReader({ sourceId, segmentId, structureNodeId, onClose, variant = "page", onExpand, initialPosition, quotedWords, onPositionReader, backLabel }: SourceReaderProps) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const source = useLoad(useCallback(() => api.libraryGetSource(sourceId), [api, sourceId]));
  // Help with a passage and comparing passages (v0.20) are offered only when the assistant is on.
  const assistant = useLoad(useCallback(() => api.assistantSettingsGet(), [api]));
  const helpOn = assistant.data?.config.enabled === true;
  const comparison = useComparison();
  const [helping, setHelping] = useState<SourceSelection | null>(null);
  const [comparing, setComparing] = useState(false);
  const [window, setWindow] = useState<SegmentWindow | null>(null);
  const [error, setError] = useState<string | null>(null);
  // A web link the system could not open: shown above the formatted view, which has no body to show `error` in.
  const [linkFailed, setLinkFailed] = useState(false);
  const target = useRef<HTMLElement | null>(null);
  const heading = useRef<HTMLHeadingElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const [mode, setMode] = useState<ReaderMode>(() => initialPosition?.mode ?? (quotedWords ? "reading" : rememberedMode(sourceId)) ?? "reading");
  // The page this reader was told to open at (continuing from another reader).
  const [initialPage] = useState(initialPosition?.page ?? null);
  const [initialPlace] = useState(initialPosition?.place);
  const focusedHeading = useRef(initialPosition?.place?.headingId ?? structureNodeId);
  const startedInOriginal = initialPosition?.mode === "original";
  // The physical page the original is showing, kept for the way back to Reading.
  const originalPage = useRef<number | null>(null);
  // A page Reading should scroll to after coming back from the original.
  const landing = useRef<number | null>(null);
  // Where the original opens; until chosen, the page of the passage opened at.
  const [openPage, setOpenPage] = useState<number | null>(null);
  const [structureOpen, setStructureOpen] = useState(false);
  const [structureRevision, setStructureRevision] = useState(0);
  // The source's own contents (an EPUB's, a PDF's outline), and the panel that shows them.
  const { items: contents, generated: contentsGenerated, generation: contentsGeneration, adopt: adoptContents } = useSourceContents(source.data, structureRevision);
  const stage = useRef<HTMLDivElement>(null);
  const [compactControls, setCompactControls] = useState(false);
  useLayoutEffect(() => {
    const element=stage.current; if (!element) return;
    const measure=()=> { if(element.clientWidth>0) setCompactControls(element.clientWidth<300); };
    measure();
    if (typeof ResizeObserver === "undefined") return;
    const observer=new ResizeObserver(measure); observer.observe(element);
    return ()=>observer.disconnect();
  }, [sourceId]);
  const opener = useRef<HTMLButtonElement>(null);
  const contentsLayout = useContentsLayout(stage, contents.length > 0);
  const [panel, setPanel] = useState<{ currentId: string | null } | null>(null);
  // The source's marks, listed beside the text like its contents (v0.14).
  const [marksPanel, setMarksPanel] = useState(false);
  // A mark chosen in the Marks list while the formatted original is shown: brought into view there.
  const [revealMark, setRevealMark] = useState<{ id: string; nonce: number } | null>(null);
  const marksPanelOpener = useRef<HTMLButtonElement>(null);
  const [jump, setJump] = useState<{ page: number; nonce: number; fraction?: number } | null>(null);
  // A segment position Reading should scroll to after a contents entry or link was chosen.
  // With an `offset`, the segment is put exactly that far from the top of the view (returning by Back / Forward).
  // With `find`, the current "Find in this source" match is shown instead, and focus stays in the find bar.
  const landingPosition = useRef<{ position: number; offset: number | null; find?: boolean; headingId?: string } | null>(null);
  const jumps = useRef(0);

  // Back / Forward: deliberate jumps only. Kept in a ref as well as state, so a handler always sees the latest.
  const [history, setHistory] = useState<ReaderHistory>(initialPosition?.history ?? NO_HISTORY);
  const historyRef = useRef(history);
  // Closing the panel gives focus back to its button, a frame later. A jump that lands
  // on a paragraph cancels that, so the place chosen keeps focus whatever the timing.
  const refocusOpener = useRef<number | null>(null);
  const commit = (next: ReaderHistory) => {
    historyRef.current = next;
    setHistory(next);
  };
  // Where the original is, on request (page and how far down it).
  const pdfNow = useRef<(() => PdfPlace) | null>(null);
  const pdfControls = useRef<PdfControls | null>(null);
  const [pdfControlsState, setPdfControlsState] = useState<PdfControlState | null>(null);
  const [pdfZoom, setPdfZoom] = useState<Zoom>({kind:"fit-width"});
  const [pdfTotal, setPdfTotal] = useState<{id:string; total:number} | null>(null);
  const pdfControlsChanged = useCallback((next: PdfControlState | null) => {
    if (next) { setPdfZoom(next.zoom); if (next.total) setPdfTotal({id:sourceId,total:next.total}); }
    setPdfControlsState(next);
  }, [sourceId]);
  // The original PDF's page, as state too: the bookmark button says whether this page has one.
  const [pdfPage, setPdfPage] = useState<number | null>(null);
  const [textPage, setTextPage] = useState<number | null>(initialPosition?.page ?? null);
  // Reads a selection on an original PDF page as an anchor in the stored text (v0.14).
  const pdfSelection = useRef<((selection: Selection) => SourceSelection | null) | null>(null);
  // The links in an EPUB's text, by the segment they are in.
  const links = useSourceLinks(source.data);
  // The Scripture references written in the text, by segment (from the stored index).
  const references = useSourceReferences(source.data?.id);
  // The person's highlights, notes and bookmarks in this source (v0.14).
  const marks = useSourceAnnotations(source.data?.id);
  // The paragraph whose marks are shown below it, and the marker that opened them (focus returns there).
  const [marksOpen, setMarksOpen] = useState<number | null>(null);
  const marksOpener = useRef<HTMLButtonElement | null>(null);
  // "Add note here": the paragraph to write on; the toolbar button gets focus back afterwards.
  const [placeNote, setPlaceNote] = useState<PlaceRequest | null>(null);
  const noteHereButton = useRef<HTMLButtonElement>(null);
  const placeNonce = useRef(0);
  // The paragraph the reader is at: the one last clicked or focused while it is in view, else the top of the view.
  const [here, setHere] = useState<number | null>(null);
  const chosenParagraph = useRef<number | null>(null);
  const { openInBible } = useLibraryNav();
  // An EPUB's formatted view: its reading order, where it was told to open, and where it is on request.
  const epub = useEpubStructure(source.data);
  const [epubStart] = useState<EpubPlace | null>(
    initialPosition?.place?.item ? { item: initialPosition.place.item, fragment: initialPosition.place.fragment, offset: initialPosition.place.itemOffset } : null,
  );
  const [epubOpen, setEpubOpen] = useState<EpubPlace | null>(null);
  const [epubJump, setEpubJump] = useState<(EpubPlace & { nonce: number }) | null>(null);
  const epubNow = useRef<(() => EpubPlace) | null>(null);

  useEffect(() => {
    let active = true;
    let load: Promise<SegmentWindow>;
    let requestedTarget: string | undefined;
    if (!startedInOriginal && initialPlace?.position !== null && initialPlace?.position !== undefined) {
      const position = initialPlace.position;
      landingPosition.current = { position, offset: initialPlace.offset };
      load = api.sourceGetSegments({ sourceId, offset: Math.max(0, position - 20), limit: 150 }).then((w) => ({
        ...w, targetSegmentId: segmentId && w.segments.some((segment) => segment.id === segmentId) ? segmentId : null,
      }));
    } else if (initialPage !== null) {
      // Continuing from another reader: open the text where that page begins.
      // The marked passage stays marked only if it is on that page.
      load = api.sourceGetSegments({ sourceId, aroundPage: initialPage }).then((w) => {
        const marked = segmentId ? w.segments.find((x) => x.id === segmentId) : undefined;
        const keep = marked !== undefined && marked.pageNumber === initialPage;
        if (!keep && !startedInOriginal) landing.current = initialPage;
        return { ...w, targetSegmentId: keep ? segmentId! : null };
      });
    } else {
      requestedTarget = segmentId;
      load = segmentId ? api.libraryOpenSegment(segmentId) : api.sourceGetSegments({ sourceId });
    }
    load.then((w) => {
      if (!active) return;
      if (w.sourceId !== sourceId || (requestedTarget && w.targetSegmentId !== requestedTarget) || (w.targetSegmentId && !w.segments.some(segment => segment.id === w.targetSegmentId))) {
        setError(t("library.reader.targetUnavailable"));
        return;
      }
      setWindow(w);
    }, (e) => active && setError(toApiError(e).message));
    return () => {
      active = false;
    };
  }, [api, sourceId, segmentId, initialPage, startedInOriginal, initialPlace, t]);

  // Opened at a place: show and focus that paragraph. Otherwise focus the
  // title, so keyboard and screen-reader users know the reader opened.
  const targetId = window?.targetSegmentId;
  const loaded = !!window && !!source.data;
  useEffect(() => {
    if (!loaded) return;
    if (target.current) {
      scrollInReader(bodyRef.current, [...target.current.querySelectorAll<HTMLElement>("[data-pdf-heading]")].find(el=>el.dataset.pdfHeading===(structureNodeId ?? focusedHeading.current)) ?? target.current.querySelector<HTMLElement>("[data-pdf-heading]") ?? target.current, "center");
      target.current.focus({ preventScroll: true });
    } else {
      heading.current?.focus({ preventScroll: true });
    }
  }, [loaded, targetId, structureNodeId]);

  const viewKind = source.data?.originalView?.kind;
  const isPdf = viewKind === "pdf";
  // An EPUB has an Original view unless its structure could not be read or it has no text.
  const isEpubView = (viewKind === "epub" || viewKind === "docx") && epub.status !== "failed" && !(epub.status === "ready" && epub.items.length === 0);
  const hasOriginal = isPdf || isEpubView;
  const targetPage = window?.segments.find((x) => x.id === window.targetSegmentId)?.pageNumber ?? null;
  const startPage = openPage ?? initialPage ?? targetPage ?? 1;
  const showOriginal = mode === "original" && hasOriginal;

  /** The physical page Reading is at: the marked passage if it is in view, else the top of the view. */
  const readingPage = (): number => {
    const body = bodyRef.current;
    const paragraphs = body ? [...body.querySelectorAll<HTMLElement>("[data-seg-page]")] : [];
    const area = body?.getBoundingClientRect();
    const targetPage = window?.segments.find((x) => x.id === window.targetSegmentId)?.pageNumber ?? null;
    const marked = body?.querySelector<HTMLElement>("[data-target]");
    if (marked && area && targetPage !== null) {
      const box = marked.getBoundingClientRect();
      if (box.bottom > area.top && box.top < area.bottom) return targetPage;
    }
    const top = paragraphs.find((el) => area && el.getBoundingClientRect().bottom > area.top);
    return Number(top?.dataset.segPage ?? targetPage ?? 1);
  };

  /** Where this reader is now, for the next reader to continue from. */
  const positionNow = (): ReaderPosition => ({
    mode: showOriginal ? "original" : "reading",
    page: showOriginal ? (isEpubView ? null : (originalPage.current ?? startPage)) : (isPdf && window ? (textPage ?? readingPage()) : null),
    history: historyRef.current,
    place: placeNow(),
  });

  const choose = async (next: ReaderMode) => {
    if (next === mode) return;
    rememberMode(sourceId, next);
    // A request made in the other view, and a panel that named a place there, are finished.
    setPanel(null);
    setJump(null);
    setEpubJump(null);
    if (isEpubView) {
      // Same place in the other view: the document and element for the text on top, and the other way round.
      if (next === "original") {
        const at = topSegment();
        setEpubOpen(at ? placeOfPosition(epub.items, at.position) : null);
        setMode("original");
      } else {
        const position = (() => {
          const here = epubNow.current?.();
          return here ? positionOfPlace(epub.items, here) : null;
        })();
        setMode("reading");
        if (position !== null) void goToPosition(position);
      }
      return;
    }
    if (next === "original") {
      const here = topSegment();
      const body=bodyRef.current, area=body?.getBoundingClientRect();
      const visible=(element:Element | null | undefined)=>{const box=element?.getBoundingClientRect();return !!(box && area && box.bottom>area.top && box.top<area.bottom);};
      const focused=body ? [...body.querySelectorAll<HTMLElement>("[data-pdf-heading]")].find(el=>el.dataset.pdfHeading===focusedHeading.current) : undefined;
      // A centred section target may leave preceding context at the viewport's
      // top. Keep the selected heading while it is visible, just as the page
      // navigator keeps the visible marked target's physical page.
      const selected=visible(focused) ? window?.segments.find(s=>s.presentation?.headings.some(h=>h.id===focusedHeading.current)) : undefined;
      const marked=visible(body?.querySelector("[data-target]")) ? window?.segments.find(s=>s.id===window.targetSegmentId) : undefined;
      const segment=selected ?? marked ?? window?.segments.find(s=>s.position===here?.position);
      const anchor = segment?.presentation?.headings.find(h=>h.id===focusedHeading.current)?.anchor ?? segment?.presentation?.headings[0]?.anchor ?? segment?.presentation?.anchors[0];
      const page = anchor?.page ?? textPage ?? readingPage();
      setOpenPage(page);
      if (anchor) setJump({page,fraction:Math.max(0,(anchor.baseline-anchor.size)/Math.max(1,anchor.pageHeight)),nonce:++jumps.current});
      setMode("original");
      return;
    }
    // Back to Reading: open the text where the original was.
    const page = originalPage.current;
    const at = pdfNow.current?.();
    setMode("reading");
    if (at) {
      try {
        const w = await api.sourceGetSegments({sourceId,aroundPage:at.page});
        const candidates = w.segments.filter(s=>s.pageNumber===at.page && s.presentation?.anchors.length);
        const nearest = candidates.reduce<SourceSegment | null>((best,s) => {
          const distance = (p: SourceSegment) => Math.min(...p.presentation!.anchors.map(a=>Math.abs((a.baseline-a.size)/Math.max(1,a.pageHeight)-at.fraction)));
          return !best || distance(s) < distance(best) ? s : best;
        },null);
        if(nearest) {
          const h=nearest.presentation?.headings.reduce<PdfHeading | undefined>((best,h)=>!best || Math.abs((h.anchor.baseline-h.anchor.size)/Math.max(1,h.anchor.pageHeight)-at.fraction)<Math.abs((best.anchor.baseline-best.anchor.size)/Math.max(1,best.anchor.pageHeight)-at.fraction) ? h : best,undefined);
          focusedHeading.current=h?.id;
          landingPosition.current={position:nearest.position,offset:null,headingId:h?.id}; setWindow({...w,targetSegmentId:null}); return;
        }
      } catch(e) {setError(toApiError(e).message); return;}
    }
    if (page === null || page === startPage) {
      // Still where it was opened: keep the window (and any marked passage).
      landing.current = startPage;
      setWindow((w) => (w ? { ...w } : w));
      return;
    }
    try {
      const w = await api.sourceGetSegments({ sourceId, aroundPage: page });
      landing.current = page;
      setWindow({ ...w, targetSegmentId: null });
    } catch (e) {
      setError(toApiError(e).message);
    }
  };

  // Scroll Reading to the page the original was on, or to the entry chosen in the contents.
  useEffect(() => {
    if (mode !== "reading" || !window) return;
    const body = bodyRef.current;
    if (landing.current !== null) {
      // The page itself, or (a page with no text of its own) the next one that has some.
      const wanted = landing.current;
      setTextPage(wanted);
      const spots = body ? [...body.querySelectorAll<HTMLElement>("[data-seg-page]")] : [];
      scrollInReader(body, spots.find((el) => Number(el.dataset.segPage) >= wanted), "start");
      landing.current = null;
    }
    if (landingPosition.current?.find) {
      // A find match: centred, and the find bar keeps focus (typing and Enter go on there).
      const mark = body?.querySelector<HTMLElement>("[data-find-active]");
      scrollInReader(body, mark ?? body?.querySelector<HTMLElement>(`[data-seg-position="${landingPosition.current.position}"]`), "center");
      landingPosition.current = null;
    }
    if (landingPosition.current !== null) {
      const { position, offset, headingId } = landingPosition.current;
      const spot = body?.querySelector<HTMLElement>(`[data-seg-position="${position}"]`);
      if (offset !== null) {
        // Back / Forward: the very place, as far from the top of the view as it was.
        scrollInReader(body, spot, "start");
        if (body) body.scrollTop -= offset;
      } else {
        // Show the chapter or section heading that precedes the text, too.
        const before = spot?.previousElementSibling;
        scrollInReader(body, [...(spot?.querySelectorAll<HTMLElement>("[data-pdf-heading]") ?? [])].find(el=>el.dataset.pdfHeading===headingId) ?? spot?.querySelector<HTMLElement>("[data-pdf-heading]") ?? (before?.hasAttribute("data-section-marker") ? before : spot), "start");
      }
      spot?.setAttribute("tabindex", "-1");
      if (spot && refocusOpener.current !== null) {
        cancelAnimationFrame(refocusOpener.current);
        refocusOpener.current = null;
      }
      spot?.focus({ preventScroll: true });
      landingPosition.current = null;
    }
  }, [mode, window]);

  /** The contents entry the reader is in now: by page for a PDF, by segment position otherwise. */
  const currentEntryNow = (): string | null => {
    if (isPdf && !showOriginal && focusedHeading.current) {
      const body=bodyRef.current, area=body?.getBoundingClientRect();
      const selected=body ? [...body.querySelectorAll<HTMLElement>("[data-pdf-heading]")].find(el=>el.dataset.pdfHeading===focusedHeading.current) : undefined;
      if(selected && (!area || selected.getBoundingClientRect().bottom >= area.top)) return focusedHeading.current;
    }
    const pageContents = contents.some(item => item.target?.kind === "page");
    if (isPdf && pageContents) return currentEntry(contents, "page", showOriginal ? (originalPage.current ?? startPage) : readingPage());
    if(isPdf && showOriginal) {
      const page = originalPage.current ?? startPage;
      const fraction = pdfNow.current?.().fraction ?? 0;
      const candidates = window?.segments.filter(s=>s.pageNumber===page && (s.presentation?.anchors[0]?.baseline ?? 0)/(s.presentation?.anchors[0]?.pageHeight ?? 1) <= fraction + .05);
      return currentEntry(contents,"segment",candidates?.at(-1)?.position ?? window?.segments.find(s=>s.pageNumber===page)?.position ?? 0);
    }
    if (showOriginal && isEpubView) {
      const here = epubNow.current?.();
      return currentEntry(contents, "segment", (here ? positionOfPlace(epub.items, here) : null) ?? 0);
    }
    const body = bodyRef.current;
    const area = body?.getBoundingClientRect();
    const top = body ? [...body.querySelectorAll<HTMLElement>("[data-seg-position]")].find((el) => area && el.getBoundingClientRect().bottom > area.top) : undefined;
    return currentEntry(contents, "segment", Number(top?.dataset.segPosition ?? window?.offset ?? 0));
  };

  const openContents = () => {
    setMarksPanel(false);
    setPanel({ currentId: currentEntryNow() });
  };
  const closeContents = () => {
    setPanel(null);
    if (refocusOpener.current !== null) cancelAnimationFrame(refocusOpener.current);
    refocusOpener.current = requestAnimationFrame(() => {
      refocusOpener.current = null;
      opener.current?.focus({ preventScroll: true });
    });
  };

  const openMarksPanel = () => {
    setPanel(null);
    setMarksPanel(true);
  };
  const closeMarksPanel = () => {
    setMarksPanel(false);
    if (refocusOpener.current !== null) cancelAnimationFrame(refocusOpener.current);
    refocusOpener.current = requestAnimationFrame(() => {
      refocusOpener.current = null;
      marksPanelOpener.current?.focus({ preventScroll: true });
    });
  };
  /** Contents and Marks share the place beside the text: one switch moves between them. */
  const switchPanels = (to: "contents" | "marks") => {
    if (to === "marks") openMarksPanel();
    else {
      setMarksPanel(false);
      openContents();
    }
  };

  /** Reading, at a physical page: stay in Reading and show that page's text. */
  const goToPage = async (page: number) => {
    setTextPage(page);
    landing.current = page;
    if (window?.segments.some((x) => x.pageNumber === page)) {
      setWindow({ ...window });
      return;
    }
    try {
      const w = await api.sourceGetSegments({ sourceId, aroundPage: page });
      setWindow({ ...w, targetSegmentId: null });
    } catch (e) {
      landing.current = null;
      setError(toApiError(e).message);
    }
  };

  /** Reading, at a segment of the extracted text (EPUB). With `find`, at the current find match there. */
  const goToPosition = async (position: number, offset: number | null = null, find = false, headingId?: string) => {
    focusedHeading.current=headingId;
    landingPosition.current = { position, offset, find, headingId };
    if (window && position >= window.offset && position < window.offset + window.segments.length) {
      setWindow({ ...window });
      return;
    }
    try {
      const w = await api.sourceGetSegments({ sourceId, offset: Math.max(0, position - 20), limit: 150 });
      setWindow({ ...w, targetSegmentId: null });
    } catch (e) {
      landingPosition.current = null;
      setError(toApiError(e).message);
    }
  };

  /** The segment on top of the Reading view (or the one a link was followed from), and how far its top is from the top of the view. */
  const topSegment = (anchor?: number): { position: number; offset: number } | null => {
    const body = bodyRef.current;
    const area = body?.getBoundingClientRect();
    const all = body ? [...body.querySelectorAll<HTMLElement>("[data-seg-position]")] : [];
    // The paragraph a link was followed from, if there is one: Back returns to exactly that. Otherwise the top of the view.
    const top =
      (anchor !== undefined ? all.find((el) => el.dataset.segPosition === String(anchor)) : undefined) ??
      all.find((el) => area && el.getBoundingClientRect().bottom > area.top) ??
      all[0];
    if (!top) return null;
    return { position: Number(top.dataset.segPosition), offset: area ? Math.round(top.getBoundingClientRect().top - area.top) : 0 };
  };

  /** The stored text of a paragraph on screen (Reading), to anchor a selection in. */
  const storedText = (segmentId: string) => {
    const segment = window?.segments.find((x) => x.id === segmentId);
    return segment ? { position: segment.position, text: segment.text } : undefined;
  };

  /** The paragraph the reader is at (see `here`), worked out now. */
  const hereNow = (): number | null => {
    const body = bodyRef.current;
    const area = body?.getBoundingClientRect();
    const chosen = chosenParagraph.current;
    if (body && area && chosen !== null) {
      const box = body.querySelector<HTMLElement>(`[data-seg-position="${chosen}"]`)?.getBoundingClientRect();
      if (box && box.bottom > area.top && box.top < area.bottom) return chosen;
    }
    return topSegment()?.position ?? null;
  };

  // Keep `here` current as the reader scrolls, clicks or tabs through the text.
  useEffect(() => {
    const body = bodyRef.current;
    if (!body || showOriginal) return;
    let frame: number | null = null;
    const update = () => {
      if (frame !== null) return;
      frame = requestAnimationFrame(() => {
        frame = null;
        setHere(hereNow());
      });
    };
    const choose = (event: Event) => {
      const paragraph = (event.target as Element | null)?.closest?.("[data-seg-position]");
      if (paragraph instanceof HTMLElement) chosenParagraph.current = Number(paragraph.dataset.segPosition);
      update();
    };
    update();
    body.addEventListener("scroll", update, { passive: true });
    body.addEventListener("click", choose);
    body.addEventListener("focusin", choose);
    return () => {
      if (frame !== null) cancelAnimationFrame(frame);
      body.removeEventListener("scroll", update);
      body.removeEventListener("click", choose);
      body.removeEventListener("focusin", choose);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps -- the body element and the text on screen are what matter
  }, [window, showOriginal]);

  const hereSegment = window?.segments.find((x) => x.position === here) ?? null;
  const onPdfPage = showOriginal && isPdf;
  // In the original PDF, "here" is the page in view: a bookmark there (made in either view) is this page's.
  const pageBookmark = (page: number | null) => (page === null ? undefined : marks.all.find((m) => m.kind === "bookmark" && m.state !== "unresolved" && m.pageNumber === page));
  const hereBookmark = onPdfPage
    ? pageBookmark(pdfPage)
    : hereSegment
      ? (marks.byParagraph.get(hereSegment.position) ?? []).find((m) => m.kind === "bookmark")
      : undefined;
  /** The original PDF's place in view, and the first paragraph of that page (if it has text). */
  const pdfHere = async (): Promise<{ page: number; fraction: number; segmentId?: string }> => {
    const at = pdfNow.current?.() ?? { page: originalPage.current ?? startPage, fraction: 0 };
    const page = at.page;
    const fraction = Math.min(1, Math.max(0, at.fraction));
    const w = await api.sourceGetSegments({ sourceId, aroundPage: page, limit: 20 });
    return { page, fraction, segmentId: w.segments.find((x) => x.pageNumber === page)?.id };
  };
  const [markError, setMarkError] = useState<string | null>(null);

  // "Add to Study" (v0.15): what is being added, and where focus goes back to afterwards.
  const [adding, setAdding] = useState<{ from: FromSource; returnTo: HTMLElement | null; position: number | null } | null>(null);
  // A short confirmation ("Added to …", "Copied"), read out politely.
  const [notice, setNotice] = useState<string | null>(null);
  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(null), 6000);
    return () => clearTimeout(timer);
  }, [notice]);
  /** The source's details for a citation, with the place of the paragraph the words begin in. */
  const citationOf = (segmentId: string | null, fallbackPage?: number | null) => {
    const segment = window?.segments.find((x) => x.id === segmentId);
    return {
      title: source.data?.title ?? "",
      author: source.data?.author ?? null,
      locator: segment?.locator ?? (fallbackPage ? `p. ${fallbackPage}` : null),
      section: segment?.sectionPath ?? null,
    };
  };
  const fromSelection = (words: SourceSelection): FromSource => ({
    cited: { sourceId, start: words.start, end: words.end, quote: words.quote },
    quote: words.quote,
    note: "",
    ...citationOf(words.start.segmentId, words.page ?? (onPdfPage ? pdfPage : null)),
  });
  const fromMark = (mark: SourceAnnotation): FromSource => ({
    cited: { annotationId: mark.id },
    quote: mark.quote,
    note: mark.kind === "note" ? mark.content : "",
    title: source.data?.title ?? "",
    author: source.data?.author ?? null,
    locator: mark.locator,
    section: mark.sectionPath,
  });
  const startAdding = (from: FromSource, returnTo: HTMLElement | null, position: number | null) => {
    setMarksPanel(false);
    setNotice(null);
    setAdding({ from, returnTo, position });
  };
  const stopAdding = () => {
    const was = adding;
    setAdding(null);
    requestAnimationFrame(() => {
      // Back where it was asked for: the button if it is still there, else the paragraph, else the text.
      if (was?.returnTo?.isConnected) return was.returnTo.focus({ preventScroll: true });
      const paragraph = was?.position !== null && was?.position !== undefined ? bodyRef.current?.querySelector<HTMLElement>(`[data-seg-position="${was.position}"]`) : null;
      const target = paragraph ?? bodyRef.current ?? stage.current;
      if (!target) return;
      if (!target.hasAttribute("tabindex")) {
        target.setAttribute("tabindex", "-1");
        target.addEventListener("blur", () => target.removeAttribute("tabindex"), { once: true });
      }
      target.focus({ preventScroll: true });
    });
  };
  const copy = async (quote: string, details: { title: string; author: string | null; locator: string | null; section: string | null }) => {
    const done = await copyText(quotationWithCitation(t, quote, details));
    setNotice(done ? t("library.copy.done") : t("library.copy.failed"));
  };
  const markActions: MarkActions = {
    onAddToStudy: (mark, from) => startAdding(fromMark(mark), from, mark.start?.position ?? null),
    onCopy: (mark) => {
      if (mark.quote !== null) void copy(mark.quote, fromMark(mark));
    },
  };

  /** Bookmark this place, or remove the bookmark that is here. */
  const toggleBookmark = async () => {
    if (onPdfPage) {
      setMarkError(null);
      try {
        const at = await pdfHere();
        const existing = pageBookmark(at.page);
        if (existing) await api.sourceAnnotationDelete(existing.id);
        else await api.sourceAnnotationCreate({ sourceId, kind: "bookmark", place: at });
        await marks.reload();
      } catch (e) {
        setMarkError(toApiError(e).message);
      }
      return;
    }
    const position = hereNow();
    const segment = window?.segments.find((x) => x.position === position);
    if (!segment) return;
    setMarkError(null);
    try {
      const existing = (marks.byParagraph.get(segment.position) ?? []).find((m) => m.kind === "bookmark");
      if (existing) await api.sourceAnnotationDelete(existing.id);
      else await api.sourceAnnotationCreate({ sourceId, kind: "bookmark", place: { segmentId: segment.id } });
      setHere(segment.position);
      await marks.reload();
    } catch (e) {
      setMarkError(toApiError(e).message);
    }
  };

  /** "Add note here": a note on the paragraph the reader is at, with no words selected. */
  const noteHere = () => {
    if (onPdfPage) {
      void pdfHere().then(
        (at) => setPlaceNote({ ...at, nonce: ++placeNonce.current }),
        (e: unknown) => setMarkError(toApiError(e).message),
      );
      return;
    }
    const position = hereNow();
    const segment = window?.segments.find((x) => x.position === position);
    if (segment) setPlaceNote({ segmentId: segment.id, nonce: ++placeNonce.current });
  };

  const toggleMarks = (position: number, button: HTMLButtonElement) => {
    marksOpener.current = button;
    setMarksOpen((open) => (open === position ? null : position));
  };
  /**
   * A mark in a paragraph's panel changed. When the paragraph has none left, its panel and marker
   * go: focus moves to the paragraph itself rather than being lost (and the panel does not reopen
   * by itself when a mark is made there later).
   */
  const marksChanged = async (position: number) => {
    const all = await marks.reload();
    if (all.some((m) => m.state !== "unresolved" && m.start?.position === position)) return;
    setMarksOpen((open) => (open === position ? null : open));
    requestAnimationFrame(() => {
      const paragraph = bodyRef.current?.querySelector<HTMLElement>(`[data-seg-position="${position}"]`);
      if (!paragraph) return;
      if (!paragraph.hasAttribute("tabindex")) {
        paragraph.setAttribute("tabindex", "-1");
        paragraph.addEventListener("blur", () => paragraph.removeAttribute("tabindex"), { once: true });
      }
      paragraph.focus({ preventScroll: true });
    });
  };
  const closeMarks = () => {
    const position = marksOpen;
    setMarksOpen(null);
    requestAnimationFrame(() => {
      // The marker that opened the panel, or (opened from the Marks list) the marker of that paragraph.
      const marker = marksOpener.current?.isConnected
        ? marksOpener.current
        : bodyRef.current?.querySelector<HTMLElement>(`[data-seg-position="${position}"] [data-source-marker]`);
      marker?.focus({ preventScroll: true });
    });
  };

  /** A formatted EPUB place with both of its faces: document and element, and the matching segment of the text. */
  const epubPlace = (at: EpubPlace | null, offsetOfText = 0, position?: number | null): SourceLocation => ({
    page: null,
    fraction: 0,
    position: position ?? (at ? positionOfPlace(epub.items, at) : null),
    offset: offsetOfText,
    item: at?.item ?? null,
    fragment: at?.fragment ?? null,
    itemOffset: at?.offset ?? 0,
  });

  /** Where the reader is now, as a place Back and Forward can return to. */
  const placeNow = (anchor?: number | EpubPlace): SourceLocation => {
    if (showOriginal && isEpubView) {
      const first = epub.items[0] ? { item: epub.items[0].id, fragment: null, offset: 0 } : null;
      return epubPlace(typeof anchor === "object" ? anchor : (epubNow.current?.() ?? epubOpen ?? epubStart ?? first));
    }
    if (showOriginal) {
      const at = pdfNow.current?.() ?? { page: originalPage.current ?? startPage, fraction: 0 };
      return { page: at.page, fraction: at.fraction, position: null, offset: 0, item: null, fragment: null, itemOffset: 0 };
    }
    const top = topSegment(typeof anchor === "number" ? anchor : undefined);
    const text = isEpubView && top ? placeOfPosition(epub.items, top.position) : null;
    return {
      page: isPdf && window ? (textPage ?? readingPage()) : null,
      fraction: 0,
      position: top?.position ?? null,
      headingId: focusedHeading.current,
      offset: top?.offset ?? 0,
      item: text?.item ?? null,
      fragment: text?.fragment ?? null,
      itemOffset: 0,
    };
  };

  // The shell reads this while the DOM is still mounted, before a route or pane changes.
  // In particular, a PDF's own cleanup must not replace its exact place with page-top.
  const readPosition = useRef<(() => ReaderPosition | null) | null>(null);
  useLayoutEffect(() => { readPosition.current = () => loaded ? positionNow() : null; });
  useLayoutEffect(() => {
    onPositionReader?.(sourceId, () => readPosition.current?.() ?? null);
    return () => onPositionReader?.(sourceId, null);
  }, [sourceId, onPositionReader]);

  /** Moves the view to a place, in the mode the reader is in (the mode is never changed by a jump). */
  const goTo = (place: SourceLocation) => {
    focusedHeading.current=place.headingId;
    if (!showOriginal && isPdf && place.page !== null) setTextPage(place.page);
    if (showOriginal && isEpubView) {
      const at: EpubPlace | null =
        place.item !== null
          ? { item: place.item, fragment: place.fragment, offset: place.itemOffset }
          : place.position !== null
            ? placeOfPosition(epub.items, place.position)
            : null;
      if (at) setEpubJump({ ...at, nonce: ++jumps.current });
    } else if (showOriginal) {
      if (place.position !== null && isPdf) {
        void api.sourceGetSegments({sourceId,offset:place.position,limit:1}).then(w=>{
          const s = w.segments[0]; const anchor=s?.presentation?.headings.find(h=>h.id===place.headingId)?.anchor ?? s?.presentation?.headings[0]?.anchor ?? s?.presentation?.anchors[0];
          if(s?.pageNumber) setJump({page:s.pageNumber,fraction:anchor ? Math.max(0,(anchor.baseline-anchor.size)/Math.max(1,anchor.pageHeight)) : 0,nonce:++jumps.current});
        }).catch(e=>setError(toApiError(e).message));
      } else if (place.page !== null) setJump({ page: place.page, fraction: place.fraction, nonce: ++jumps.current });
    } else if (place.position !== null) {
      void goToPosition(place.position, place.offset, false, place.headingId);
    } else if (place.item !== null) {
      const position = positionOfPlace(epub.items, { item: place.item, fragment: place.fragment });
      if (position !== null) void goToPosition(position);
    } else if (place.page !== null) {
      void goToPage(place.page);
    }
  };

  /** A deliberate jump (a link, a contents entry): Back returns to where the reader was. */
  const jumpTo = (target: { page?: number; fraction?: number; position?: number; headingId?: string; item?: string; fragment?: string | null }, from?: number | EpubPlace) => {
    // A link that leads outside the text leaves the reader where it is.
    if (target.position !== undefined && window && (target.position < 0 || target.position >= window.total)) {
      void api.logError("A link in the source could not be followed: it leads outside the text").catch(() => undefined);
      return;
    }
    let place: SourceLocation;
    if (isEpubView && target.item !== undefined) {
      place = epubPlace({ item: target.item, fragment: target.fragment ?? null, offset: 0 });
    } else if (isEpubView && target.position !== undefined) {
      place = epubPlace(placeOfPosition(epub.items, target.position), 0, target.position);
    } else {
      place = { page: target.page ?? null, fraction: target.fraction ?? 0, position: target.position ?? null, headingId:target.headingId, offset: 0, item: null, fragment: null, itemOffset: 0 };
    }
    commit(visit(historyRef.current, placeNow(from), place));
    if (!showOriginal && place.position !== null) void goToPosition(place.position,null,false,place.headingId);
    else goTo(place);
  };

  const goBack = () => {
    const step = stepBack(historyRef.current, placeNow());
    if (!step) return;
    commit(step.history);
    goTo(step.to);
  };
  const goForward = () => {
    const step = stepForward(historyRef.current, placeNow());
    if (!step) return;
    commit(step.history);
    goTo(step.to);
  };

  // Find in this source (v0.13). Showing a match is not a jump: Back / Forward are left alone.
  const [finding, setFinding] = useState(false);
  const [findFocus, setFindFocus] = useState(0);
  const findOpener = useRef<HTMLButtonElement>(null);
  const showMatch = (match: SourceMatch) => {
    if (showOriginal && isEpubView) {
      const at = placeOfPosition(epub.items, match.position);
      if (at) setEpubJump({ ...at, nonce: ++jumps.current });
    } else if (showOriginal) {
      // The page the match is on; the original has no exact place for it yet.
      if (match.pageNumber !== null) setJump({ page: match.pageNumber, nonce: ++jumps.current });
    } else {
      void goToPosition(match.position, null, true);
    }
  };
  const finder = useSourceFind(sourceId, showMatch);
  const openFind = () => {
    setFinding(true);
    setFindFocus((n) => n + 1);
  };
  const closeFind = () => {
    setFinding(false);
    finder.clear();
    requestAnimationFrame(() => findOpener.current?.focus({ preventScroll: true }));
  };
  const onReaderKey = (event: React.KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "f") {
      event.preventDefault();
      openFind();
    }
  };
  // The find marks in each segment, with the current one flagged.
  const current = activeMatch(finder.find);
  // In the original PDF: the current match on its page, marked there only if the page's own text agrees.
  const pdfFind: PdfFind | null = (() => {
    if (!current || current.pageNumber === null || !finder.find.result) return null;
    const onPage = finder.find.result.matches.filter((m) => m.pageNumber === current.pageNumber);
    return { page: current.pageNumber, query: finder.find.searched, index: onPage.indexOf(current), count: onPage.length };
  })();
  const findMarks = new Map<number, FindMark[]>();
  for (const m of finder.find.result?.matches ?? []) {
    const list = findMarks.get(m.position) ?? [];
    list.push({ start: m.start, end: m.end, active: m === current });
    findMarks.set(m.position, list);
  }
  // The words a Study finding cites, shown where they are (opened from its citation).
  if (quotedWords && window) {
    const first = window.segments.find((x) => x.id === quotedWords.startSegmentId);
    const last = window.segments.find((x) => x.id === quotedWords.endSegmentId);
    if (first && last) {
      for (const segment of window.segments) {
        if (segment.position < first.position || segment.position > last.position) continue;
        const start = segment.position === first.position ? quotedWords.startOffset : 0;
        const end = segment.position === last.position ? quotedWords.endOffset : segment.text.length;
        if (start < end) findMarks.set(segment.position, [...(findMarks.get(segment.position) ?? []), { start, end, active: false, quoted: true }]);
      }
    }
  }

  /** A mark chosen in the Marks panel: to its place (a jump, so Back returns), with its paragraph's marks shown in Reading. */
  const chooseMark = (mark: SourceAnnotation) => {
    setMarksPanel(false);
    if (showOriginal && isPdf) {
      // The original shows its page; the page's marks are there.
      if (mark.pageNumber !== null) jumpTo({ page: mark.pageNumber, fraction: mark.start ? 0 : (mark.fraction ?? 0) });
    } else if (mark.start) {
      jumpTo({ position: mark.start.position });
      if (showOriginal && isEpubView) setRevealMark({ id: mark.id, nonce: ++jumps.current });
      if (!showOriginal) {
        marksOpener.current = null;
        setMarksOpen(mark.start.position);
      }
    } else if (mark.pageNumber !== null) {
      jumpTo({ page: mark.pageNumber, fraction: mark.fraction ?? 0 });
    }
  };

  const chooseEntry = (item: ContentsItem) => {
    const target = item.target;
    if (!target) return;
    closeContents();
    // The mode is kept: Original moves its page (zoom unchanged), Reading shows that page's text.
    if (target.kind === "page") jumpTo({ page: target.page });
    else jumpTo({ position: target.position, headingId: isPdf ? item.id : undefined });
  };

  const more = async (direction: "earlier" | "later") => {
    if (!window) return;
    const limit = 150;
    const offset = direction === "earlier" ? Math.max(0, window.offset - limit) : window.offset + window.segments.length;
    try {
      const next = await api.sourceGetSegments({ sourceId, offset, limit: direction === "earlier" ? window.offset - offset : limit });
      setWindow({
        ...window,
        offset: direction === "earlier" ? next.offset : window.offset,
        segments: direction === "earlier" ? [...next.segments, ...window.segments] : [...window.segments, ...next.segments],
        targetSegmentId: window.targetSegmentId,
      });
    } catch (e) {
      setError(toApiError(e).message);
    }
  };

  const inPane = variant === "pane";
  const navigablePage = showOriginal ? (pdfPage ?? startPage) : (textPage ?? targetPage ?? 1);
  const physicalPages = pdfTotal?.id === sourceId ? pdfTotal.total : source.data?.pageCount;
  const pageNavigation = source.data?.format === "pdf" && physicalPages ? <PdfPageNavigation
    compact={compactControls}
    page={navigablePage}
    total={physicalPages}
    printedLabel={window?.segments.find((segment) => segment.pageNumber === navigablePage)?.presentation?.printedLabel ?? window?.segments.find((segment) => segment.pageNumber === navigablePage)?.pageLabel}
    onGo={(page) => jumpTo({ page })}
  /> : null;
  const s = source.data;
  const Title = inPane ? "h2" : "h1";
  const body = (
    <>
      {source.error && <Notice tone="error">{source.error.message}</Notice>}
      {error && <Notice tone="error">{error}</Notice>}
      {markError && <Notice tone="error">{markError}</Notice>}
      {s && (
        <header className="flex flex-col gap-1 border-b border-border-subtle pb-4">
          {inPane && (
            <Title
              id={`${id}-title`}
              ref={heading}
              tabIndex={-1}
              className="type-worktitle m-0"
            >
              {s.title}
            </Title>
          )}
          {s.author && <p className="reading m-0">{s.author}</p>}
          <p className="m-0 text-[0.9375rem] text-text-muted">
            {[
              sourceTypeLabel(t, s.sourceType),
              s.format ? formatLabel(t, s.format) : null,
              s.pageCount ? t("library.size.pages", { count: s.pageCount, number: formatNumber(s.pageCount) }) : null,
              s.language,
              s.publicationYear,
            ]
              .filter(Boolean)
              .join(t("common.listSeparator"))}
          </p>
        </header>
      )}
      {window && (
        <div className="flex flex-col gap-3">
          {window.offset > 0 && <Button className="self-start" onClick={() => more("earlier")}>{t("library.reader.showEarlier")}</Button>}
          <div className="reading flex flex-col gap-3">
            {window.segments.map((segment, index) => (
              <SegmentView
                key={segment.id}
                segment={segment}
                previous={window.segments[index - 1]}
                isTarget={segment.id === window.targetSegmentId}
                targetRef={segment.id === window.targetSegmentId ? target : undefined}
                links={links.get(segment.position)}
                references={references.get(segment.position)}
                marks={findMarks.get(segment.position)}
                ranges={marks.ranges.get(segment.position)}
                paragraphMarks={marks.byParagraph.get(segment.position)}
                marksOpen={marksOpen === segment.position}
                marksId={`${id}-marks-${segment.position}`}
                onToggleMarks={(button) => toggleMarks(segment.position, button)}
                onMarksChanged={() => void marksChanged(segment.position)}
                onCloseMarks={closeMarks}
                markActions={markActions}
                onFollow={(position) => jumpTo({ position }, segment.position)}
                onOpenInBible={(reference) => openInBible(reference, positionNow())}
              />
            ))}
          </div>
          {window.offset + window.segments.length < window.total && (
            <Button className="self-start" onClick={() => more("later")}>{t("library.reader.showMore")}</Button>
          )}
        </div>
      )}
    </>
  );

  // The original, as its own view under the (never scrolling) header. It
  // mounts as soon as the page to open at is known.
  const original = showOriginal && isPdf ? (
    !window && !error && openPage === null && initialPage === null ? null : (
      <PdfViewer
        key={`${sourceId}-${startPage}`}
        sourceId={sourceId}
        initialPage={startPage}
        initialFraction={initialPlace?.fraction ?? (()=>{const s=window?.segments.find(x=>x.id===window.targetSegmentId);const a=s?.presentation?.headings.find(h=>h.id===structureNodeId)?.anchor ?? s?.presentation?.headings[0]?.anchor ?? s?.presentation?.anchors[0];return a ? Math.max(0,(a.baseline-a.size)/Math.max(1,a.pageHeight)) : 0;})()}
        onPageChange={(page) => {
          originalPage.current = page;
          setPdfPage(page);
        }}
        jumpTo={jump}
        onOutline={adoptContents}
        onLink={(place) => jumpTo({ page: place.page, fraction: place.fraction })}
        controlsRef={pdfControls}
        initialZoom={pdfZoom}
        onControlsState={pdfControlsChanged}
        pageControls={false}
        locationRef={pdfNow}
        onOpenInBible={(reference) => openInBible(reference, positionNow())}
        find={pdfFind}
        marks={marks.all}
        onMarksChanged={marks.reload}
        markActions={markActions}
        selectionRef={pdfSelection}
      />
    )
  ) : null;

  // A web link in the formatted view: opened outside the app, on the click. It is neither a jump nor history.
  const openExternal = (url: string) => {
    setLinkFailed(false);
    api.openExternalLink(url).catch((e: unknown) => {
      // Technical detail goes to the log (the host only, never the path or query); the person sees a plain notice.
      void api.logError(`A web link could not be opened (${hostOf(url)}): ${e instanceof Error ? e.message : String(e)}`).catch(() => undefined);
      setLinkFailed(true);
    });
  };

  // Opened straight into the original (the view last chosen for this source): at the passage opened at, once it is known.
  const targetPosition = window?.segments.find((x) => x.id === window.targetSegmentId)?.position;
  const targetPlace = targetPosition !== undefined && epub.status === "ready" ? placeOfPosition(epub.items, targetPosition) : null;
  const epubStartKnown = epubOpen !== null || epubStart !== null || !segmentId || window !== null || error !== null;
  const epubViewer =
    showOriginal && isEpubView ? (
      epub.status === "ready" && epub.items.length > 0 && epubStartKnown ? (
        <>
          {linkFailed && (
            <div className="shrink-0 px-3 pt-2">
              <Notice tone="error">{t("library.formatted.linkFailed")}</Notice>
            </div>
          )}
          <EpubViewer
            key={sourceId}
            sourceId={sourceId}
            kind={viewKind === "docx" ? "docx" : "epub"}
            structure={epub.items}
            start={epubOpen ?? epubStart ?? targetPlace ?? { item: epub.items[0]!.id, fragment: null, offset: 0 }}
            jumpTo={epubJump}
            onLink={(target, from) => jumpTo({ item: target.item, fragment: target.id }, from)}
            onExternal={openExternal}
            locationRef={epubNow}
            onOpenInBible={(reference) => openInBible(reference, positionNow())}
            marks={marks.all}
            revealMark={revealMark}
          />
        </>
      ) : (
        <p className="m-0 p-6 text-text-muted">{t("library.original.loading")}</p>
      )
    ) : null;

  const modeSwitch = hasOriginal ? <ModeSwitch compact={compactControls} mode={mode} onChange={(m) => void choose(m)} /> : null;
  const structurePanel = isPdf ? <>
    {structureOpen && <PdfStructurePanel sourceId={sourceId} segments={window?.segments ?? []}
      onGo={(position,headingId)=>{jumpTo({position,headingId}); setStructureOpen(false);}} onPage={page=>{jumpTo({page});setStructureOpen(false);}} onClose={()=>setStructureOpen(false)}
      observed={()=>{
        // Identifiers, mode and page numbers only: what the person sees now.
        const page = showOriginal ? (originalPage.current ?? startPage) : (textPage ?? readingPage());
        const printed = window?.segments.find(s=>s.pageNumber===page && s.presentation?.printedLabel)?.presentation?.printedLabel ?? null;
        return { mode: showOriginal ? "original" : "reading", physicalPage: page, printedPage: printed, openedSegmentId: segmentId ?? null, openedStructureNodeId: structureNodeId ?? null, focusedHeadingId: focusedHeading.current ?? null, variant };
      }}
      onChanged={async()=>{
        const w = await api.sourceGetSegments({sourceId,offset:window?.offset ?? 0,limit:window?.segments.length ?? 150});
        setWindow({...w,targetSegmentId:window?.targetSegmentId ?? null}); setStructureRevision(v=>v+1);
      }}/>}</> : null;

  // Marking the original PDF: a selection on a page, read through the stored text placed there.
  const closeHelp = () => {
    setHelping(null);
    setComparing(false);
    stopAdding();
  };
  const toCompare = (words: SourceSelection) => {
    const segment = window?.segments.find((x) => x.id === words.start.segmentId);
    const added = comparison.add({
      excerpt: { sourceId, start: words.start, end: words.end, quote: words.quote },
      title: source.data?.title ?? "",
      author: source.data?.author ?? null,
      locator: segment?.locator ?? (words.page ? `p. ${words.page}` : null),
    });
    setNotice(added ? t("library.compare.added", { count: comparison.passages.length + 1 }) : t("library.compare.notAdded"));
  };
  const addPanel = adding ? (
    <AddToStudyPanel
      from={adding.from}
      onCancel={stopAdding}
      onAdded={(where) => {
        stopAdding();
        setNotice(t("studies.fromSource.added", where));
      }}
    />
  ) : helping ? (
    <div className="max-h-[75vh] overflow-y-auto">
      <SourceHelp excerpt={{ sourceId, start: helping.start, end: helping.end, quote: helping.quote }} onClose={closeHelp} />
    </div>
  ) : comparing ? (
    <div className="max-h-[75vh] overflow-y-auto">
      <CompareHelp onClose={closeHelp} />
    </div>
  ) : null;
  const tray = helpOn ? (
    <ComparisonTray
      onCompare={() => {
        setAdding(null);
        setHelping(null);
        setComparing(true);
      }}
    />
  ) : null;
  // Below the pages, never over them: the page area gets smaller while it is open.
  const pdfStrip = onPdfPage ? (
    <div className="max-h-[50%] shrink-0 overflow-y-auto border-t border-border-subtle bg-bg px-2 pt-2 empty:hidden">
      {addPanel}
      <SourceMarkStrip
        root={stage}
        sourceId={sourceId}
        read={(selection) => pdfSelection.current?.(selection) ?? null}
        cannotPlace={t("library.marks.cannotPlace")}
        place={placeNote}
        onPlaceDone={() => {
          setPlaceNote(null);
          requestAnimationFrame(() => noteHereButton.current?.focus({ preventScroll: true }));
        }}
        onSaved={() => void marks.reload()}
        onAddToStudy={(words) => startAdding(fromSelection(words), null, null)}
        onCopy={(words) => void copy(words.quote, citationOf(words.start.segmentId, words.page ?? pdfPage))}
        onHelp={helpOn ? (words) => setHelping(words) : undefined}
        onCompare={helpOn ? toCompare : undefined}
      />
    </div>
  ) : null;
  const markStrip =
    !showOriginal && window ? (
      <SourceMarkStrip
        root={bodyRef}
        sourceId={sourceId}
        read={(selection, root) => selectionInside(selection, root, storedText)}
        stored={storedText}
        place={placeNote}
        onPlaceDone={() => {
          setPlaceNote(null);
          requestAnimationFrame(() => noteHereButton.current?.focus({ preventScroll: true }));
        }}
        onSaved={() => void marks.reload()}
        onAddToStudy={(words) => startAdding(fromSelection(words), null, words.firstPosition)}
        onCopy={(words) => void copy(words.quote, citationOf(words.start.segmentId))}
        onHelp={helpOn ? (words) => setHelping(words) : undefined}
        onCompare={helpOn ? toCompare : undefined}
      />
    ) : null;
  // Always present, so a confirmation is read out when it appears.
  const noticeLine = (
    <p aria-live="polite" aria-atomic="true" data-testid="reader-notice" className={notice ? "m-0 shrink-0 border-b border-border-subtle bg-bg-secondary px-4 py-1.5 font-ui text-[0.9375rem] text-text-strong" : "sr-only"}>
      {notice ?? ""}
    </p>
  );
  const citedSegmentId = window?.targetSegmentId ?? window?.segments[0]?.id ?? null;
  // About this source is in the toolbar menu (collapsed by default, #26); an
  // archived source or a missing original file is still said here, always.
  const provenance = s && (s.state === "archived" || !s.filePresent) ? (
    <div className="flex shrink-0 flex-col gap-1 border-b border-border-subtle px-4 py-2">
      <div className="flex flex-wrap items-center gap-2">
        {s.state === "archived" && (
          <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[0.8125rem] text-text-muted">
            {t("common.lifecycle.state.archived")}
          </span>
        )}
        {!s.filePresent && <p className="m-0 text-danger">{t("library.about.fileMissing")}</p>}
      </div>
      {s.state === "archived" && <p className="m-0 text-[0.9375rem] text-text-muted">{t("library.about.archivedNote")}</p>}
    </div>
  ) : null;
  const findBar = finding ? (
    <FindBar find={finder.find} focusKey={findFocus} onType={finder.type} onSubmit={finder.submit} onStep={finder.step} onClose={closeFind} />
  ) : null;
  const panelSwitch =
    contents.length > 0 ? (
      <div role="group" aria-label={t("library.marks.panels")} className="flex shrink-0 rounded-aw border border-border p-0.5">
        {(["contents", "marks"] as const).map((which) => (
          <button
            key={which}
            type="button"
            aria-pressed={which === "marks" ? marksPanel : panel !== null}
            onClick={() => switchPanels(which)}
            className="aw-tab"
          >
            {which === "marks" ? t("library.marks.title") : t("library.contents.title")}
          </button>
        ))}
      </div>
    ) : null;
  const contentsPanel = panel ? (
    <ContentsPanel
      id={`${id}-contents`}
      title={s?.title ?? ""}
      items={contents}
      generated={contentsGenerated}
      currentId={panel.currentId}
      layout={contentsLayout}
      onSelect={chooseEntry}
      onClose={closeContents}
      switcher={panelSwitch}
    />
  ) : marksPanel ? (
    <SourceMarksPanel
      id={`${id}-marks`}
      title={s?.title ?? ""}
      marks={marks.all}
      layout={contentsLayout}
      switcher={panelSwitch}
      onSelect={chooseMark}
      onChanged={() => void marks.reload()}
      onClose={closeMarksPanel}
      actions={markActions}
    />
  ) : null;
  // A.4.2 (#26): every reader action, once, for the shared toolbar. Visible
  // icons keep the names the separate buttons had; the rest are in "…".
  const [aboutOpen, setAboutOpen] = useState(false);
  const canMarkPlace = (!showOriginal && !!window) || onPdfPage;
  const readerActions: ReaderAction[] = [
    { id: "find", key: "find", label: t("library.find.button"), icon: <Search className="size-4" aria-hidden="true" />, expanded: finding, onSelect: () => (finding ? closeFind() : openFind()) },
    ...(contents.length > 0 ? [{ id: "contents" as const, key: "contents", label: t("library.contents.title"), icon: <List className="size-4" aria-hidden="true" />, expanded: panel !== null, controls: panel ? `${id}-contents` : undefined, onSelect: () => (panel ? closeContents() : openContents()) }] : []),
    { id: "history", key: "back", label: t("library.history.back"), icon: <Undo2 className="size-4" aria-hidden="true" />, disabled: !canGoBack(history), onSelect: goBack },
    { id: "history", key: "forward", label: t("library.history.forward"), icon: <Redo2 className="size-4" aria-hidden="true" />, disabled: !canGoForward(history), onSelect: goForward },
    ...(canMarkPlace ? [
      { id: "bookmark" as const, key: "bookmark", label: hereBookmark ? t("library.marks.removeBookmarkHere") : t("library.marks.bookmarkHere"), icon: <Bookmark className={`size-4 ${hereBookmark ? "fill-current" : ""}`} aria-hidden="true" />, pressed: !!hereBookmark, onSelect: () => void toggleBookmark() },
      { id: "note" as const, key: "note", label: t("library.marks.addNoteHere"), icon: <StickyNote className="size-4" aria-hidden="true" />, onSelect: noteHere },
    ] : []),
    { id: "marks", key: "marks", label: t("library.marks.title"), icon: <Highlighter className="size-4" aria-hidden="true" />, expanded: marksPanel, controls: marksPanel ? `${id}-marks` : undefined, onSelect: () => (marksPanel ? closeMarksPanel() : openMarksPanel()) },
    ...(isPdf ? [{ id: "structure" as const, key: "structure", label: t("library.structure.title"), icon: <ListTree className="size-4" aria-hidden="true" />, onSelect: () => setStructureOpen(true) }] : []),
    ...(inPane && onExpand ? [{ id: "full" as const, key: "full", label: t("library.reader.openFull"), icon: <Maximize2 className="size-4" aria-hidden="true" />, onSelect: () => onExpand(positionNow()) }] : []),
    ...(onPdfPage ? [
      {id:"zoomOut" as const,key:"zoomOut",label:t("library.original.zoomOut"),icon:<ZoomOut className="size-4"/>,disabled:!pdfControlsState?.total,group:t("library.original.zoomValue",{percent:formatNumber(pdfControlsState?.percent ?? 100)}),onSelect:(opener: HTMLElement | null)=>{pdfControls.current?.zoomBy(-1);opener?.focus({preventScroll:true});}},
      {id:"zoomIn" as const,key:"zoomIn",label:t("library.original.zoomIn"),icon:<ZoomIn className="size-4"/>,disabled:!pdfControlsState?.total,group:t("library.original.zoomValue",{percent:formatNumber(pdfControlsState?.percent ?? 100)}),onSelect:(opener: HTMLElement | null)=>{pdfControls.current?.zoomBy(1);opener?.focus({preventScroll:true});}},
      {id:"fitWidth" as const,key:"fitWidth",label:t("library.original.fitWidth"),icon:<MoveHorizontal className="size-4"/>,disabled:!pdfControlsState?.total,group:t("library.original.zoomValue",{percent:formatNumber(pdfControlsState?.percent ?? 100)}),pressed:pdfControlsState?.fit==="fit-width",onSelect:(opener: HTMLElement | null)=>{pdfControls.current?.fitWidth();opener?.focus({preventScroll:true});}},
      {id:"fitPage" as const,key:"fitPage",label:t("library.original.fitPage"),icon:<Maximize2 className="size-4"/>,disabled:!pdfControlsState?.total,group:t("library.original.zoomValue",{percent:formatNumber(pdfControlsState?.percent ?? 100)}),pressed:pdfControlsState?.fit==="fit-page",onSelect:(opener: HTMLElement | null)=>{pdfControls.current?.fitPage();opener?.focus({preventScroll:true});}},
    ] : []),
    ...(s ? [{ id: "about" as const, key: "about", label: t("library.about.summary"), icon: <Info className="size-4" aria-hidden="true" />, onSelect: () => setAboutOpen(true) }] : []),
  ];
  const contentsProgress = isPdf && contentsGeneration ? <div role="status" className="flex shrink-0 flex-wrap items-center gap-1 border-b border-border-subtle px-3 py-1 text-[0.875rem] text-text-muted">
    <span>{t(contentsGeneration === "running" ? "library.structure.automaticRunning" : "library.structure.automaticFailed")}</span>
    <Button variant="quiet" onClick={() => setStructureOpen(true)}>{t("library.structure.automaticDetails")}</Button>
  </div> : null;
  const toolbarRefs = { find: findOpener, contents: opener, note: noteHereButton, marks: marksPanelOpener };
  // Reading has the compact page indicator here; Original has the viewer's own (one per view, never both).
  const fixedControls = <>{modeSwitch}{pageNavigation}{onPdfPage && <span data-testid="pdf-zoom" className="sr-only">{t("library.original.zoomValue",{percent:formatNumber(pdfControlsState?.percent ?? 100)})}</span>}</>;
  const aboutDialog = aboutOpen && s ? (
    <Dialog wide title={t("library.about.summary")} onClose={() => setAboutOpen(false)} actions={<Button onClick={() => setAboutOpen(false)}>{t("common.close")}</Button>}>
      <SourceAbout source={s} citation={citationText(t, citationOf(citedSegmentId))} />
    </Dialog>
  ) : null;

  if (inPane) {
    // A column: controls that never scroll away, then Reading (which scrolls
    // inside its own body) or the original (which has its own toolbar and scroller).
    return (
      <section aria-label={s?.title} className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden" onKeyDown={onReaderKey}>
        <div data-testid="reader-header" className="shrink-0">
          <ReaderToolbar
            label={t("library.toolbar.label")}
            start={
              <button type="button" onClick={() => onClose(positionNow())} aria-label={backLabel ?? t("library.reader.backToSources")} title={backLabel ?? t("library.reader.backToSources")}
                className="inline-flex min-h-11 min-w-11 shrink-0 items-center justify-center gap-1.5 rounded-aw px-2 text-[0.9375rem] font-medium text-text-strong hover:bg-hover">
                <ArrowLeft className="size-4" aria-hidden="true" /><span aria-hidden="true" className="@max-lg:sr-only">{t("common.back")}</span>
              </button>
            }
            fixed={fixedControls}
            actions={readerActions}
            refs={toolbarRefs}
          />
        </div>
        {contentsProgress}
        {provenance}
        {structurePanel}
        {aboutDialog}
        {findBar}
        {noticeLine}
        {tray}
        <div ref={stage} className="relative flex min-h-0 flex-1 flex-col min-w-0 overflow-hidden">
          {showOriginal ? (
            <>
              {epubViewer ?? original}
              {pdfStrip}
            </>
          ) : (
            <div ref={bodyRef} data-testid="reader-body" onScroll={() => { if (isPdf) setTextPage(readingPage()); }} className="min-h-0 flex-1 overflow-y-auto">
              <div className="flex flex-col gap-5 px-5 pb-16 pt-5">
                {body}
                {addPanel ? <div className="sticky bottom-2 z-10">{addPanel}</div> : markStrip}
              </div>
            </div>
          )}
          {contentsPanel}
        </div>
      </section>
    );
  }

  // The full reader is a column too: a header that never scrolls (Back stays
  // reachable however far the text is scrolled) above the content.
  return (
    <main aria-labelledby={`${id}-title`} className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden bg-bg" onKeyDown={onReaderKey}>
      {/* Chrome recedes around the book: one quiet bar, the title in the interface voice. */}
      <div data-testid="reader-header" className="shrink-0">
        <ReaderToolbar
          label={t("library.toolbar.label")}
          start={
            <button type="button" onClick={() => onClose(positionNow())} aria-label={t("common.back")} title={t("common.back")}
              className="inline-flex min-h-11 min-w-11 shrink-0 items-center justify-center gap-1.5 rounded-aw px-2 text-[0.9375rem] font-medium text-text-strong hover:bg-hover">
              <ArrowLeft className="size-4" aria-hidden="true" /><span aria-hidden="true" className="@max-lg:sr-only">{t("common.back")}</span>
            </button>
          }
          title={
            <h1 id={`${id}-title`} ref={heading} tabIndex={-1} title={s?.title} className="type-item m-0 min-w-0 flex-1 truncate">
              {s?.title}
            </h1>
          }
          fixed={fixedControls}
          actions={readerActions}
          refs={toolbarRefs}
        />
      </div>
      {contentsProgress}
      {provenance}
      {structurePanel}
      {aboutDialog}
      {findBar}
      {noticeLine}
        {tray}
      <div ref={stage} className="relative flex min-h-0 flex-1 flex-col min-w-0 overflow-hidden">
        {showOriginal ? (
          <>
            {epubViewer ?? original}
            {pdfStrip}
          </>
        ) : (
          <div ref={bodyRef} data-testid="reader-body" onScroll={() => { if (isPdf) setTextPage(readingPage()); }} className="min-h-0 flex-1 overflow-y-auto">
            <div className="mx-auto flex max-w-[46rem] flex-col gap-6 px-8 pb-24 pt-6">
              {body}
              {addPanel ? <div className="sticky bottom-2 z-10">{addPanel}</div> : markStrip}
            </div>
          </div>
        )}
        {contentsPanel}
      </div>
    </main>
  );
}

/** Reading | Original, one control for both readers. */
function ModeSwitch({ mode, onChange, compact }: { compact: boolean; mode: ReaderMode; onChange: (mode: ReaderMode) => void }) {
  const { t } = useTranslation();
  if (compact) return <ActionMenu portal label={t("library.original.modesLabel")} align="start"
    trigger={<span title={`${t("library.original.modesLabel")}: ${t(`library.original.${mode}`)}`} className="inline-flex w-7 justify-center"><span className="sr-only">{t("library.original.modesLabel")}: {t(`library.original.${mode}`)}</span>{mode==="reading"?<BookOpenText className="size-4" aria-hidden="true"/>:<FileText className="size-4" aria-hidden="true"/>}</span>}
    actions={(["reading","original"] as const).map(m=>({label:t(`library.original.${m}`),onSelect:()=>onChange(m)}))}/>;
  // A.4.2: in a narrow pane the two views are icons; their names stay for screen readers and as tooltips.
  return (
    <div role="group" aria-label={t("library.original.modesLabel")} className="flex shrink-0">
      {(["reading", "original"] as const).map((m) => (
        <button
          key={m}
          type="button"
          aria-pressed={mode === m}
          title={t(`library.original.${m}`)}
          onClick={() => onChange(m)}
          className="aw-tab inline-flex items-center gap-1"
        >
          {m === "reading" ? <BookOpenText className="hidden size-4 @max-md:inline" aria-hidden="true" /> : <FileText className="hidden size-4 @max-md:inline" aria-hidden="true" />}
          <span className="@max-md:sr-only">{t(`library.original.${m}`)}</span>
        </button>
      ))}
    </div>
  );
}

interface SegmentViewProps {
  segment: SourceSegment;
  previous?: SourceSegment;
  isTarget: boolean;
  targetRef?: React.MutableRefObject<HTMLElement | null>;
  /** The links in this segment's text, in order (internal ones only). */
  links?: SourceLink[];
  /** The Scripture references in this segment's text, in order. */
  references?: SourceReference[];
  /** "Find in this source" matches in this segment's text, in order. */
  marks?: FindMark[];
  /** The person's highlighted and noted words in this segment's text. */
  ranges?: MarkedRange[];
  /** The marks that begin in this paragraph (shown by its marker). */
  paragraphMarks?: SourceAnnotation[];
  marksOpen: boolean;
  marksId: string;
  onToggleMarks: (button: HTMLButtonElement) => void;
  onMarksChanged: () => void;
  onCloseMarks: () => void;
  markActions: MarkActions;
  onFollow: (position: number) => void;
  onOpenInBible: (reference: ScriptureRef) => void;
}

/** A find match in a segment's text (UTF-16 offsets); `active` for the current one. */
interface FindMark {
  start: number;
  end: number;
  active: boolean;
  /** Words a Study finding cites, rather than a find match. */
  quoted?: boolean;
}

/** What is drawn over a segment's text: find matches and the person's own marks. */
interface Paint {
  marks?: FindMark[];
  ranges?: MarkedRange[];
}

/**
 * `text.slice(from, to)`, with find matches and the person's highlights and
 * noted words drawn in it (anything crossing the edges is cut to them). A
 * highlight is a `<mark>` in its colour; noted words are underlined with
 * dots; a find match is drawn over both. Colour is never the only sign: the
 * paragraph's marker says what is there.
 */
function painted(text: string, from: number, to: number, paint: Paint): React.ReactNode {
  const { marks, ranges } = paint;
  if ((!marks || marks.length === 0) && (!ranges || ranges.length === 0)) return text.slice(from, to);
  const cuts = new Set<number>([from, to]);
  for (const m of marks ?? []) [m.start, m.end].forEach((x) => x > from && x < to && cuts.add(x));
  for (const r of ranges ?? []) [r.start, r.end ?? text.length].forEach((x) => x > from && x < to && cuts.add(x));
  const points = [...cuts].sort((a, b) => a - b);
  const out: React.ReactNode[] = [];
  for (let i = 0; i < points.length - 1; i++) {
    const start = points[i]!;
    const end = points[i + 1]!;
    if (start >= end) continue;
    const piece = text.slice(start, end);
    const find = marks?.find((m) => m.start <= start && m.end >= end);
    const over = (ranges ?? []).filter((r) => r.start <= start && (r.end ?? text.length) >= end);
    const colors = [...new Set(over.map((r) => r.color).filter((c): c is NonNullable<typeof c> => c !== null))];
    const noted = over.some((r) => r.note);
    if (!find && over.length === 0) {
      out.push(piece);
      continue;
    }
    const noteClass = noted ? " underline decoration-accent decoration-dotted decoration-2 underline-offset-4" : "";
    if (find) {
      out.push(
        <mark
          key={`f${start}`}
          data-find-active={find.active || undefined}
          data-quoted={find.quoted || undefined}
          className={(find.active ? "rounded-sm bg-hl-amber font-semibold text-text-strong outline outline-2 outline-offset-1 outline-accent" : "rounded-sm bg-hl-amber text-text") + noteClass}
        >
          {piece}
        </mark>,
      );
    } else if (colors.length > 0) {
      // Highlights never overlap (the core lays them like a highlighter); should two ever meet, the last is shown.
      const color = colors[colors.length - 1]!;
      out.push(
        <mark key={`h${start}`} data-source-highlight={color} data-source-note={noted || undefined} className={`${PHRASE_CLASS[color]} text-text${noteClass}`}>
          {piece}
        </mark>,
      );
    } else {
      out.push(
        <span key={`n${start}`} data-source-note={noted || undefined} className={noteClass.trim() || undefined}>
          {piece}
        </span>,
      );
    }
  }
  return out.length === 1 ? out[0] : out;
}

/**
 * The segment's text with its links as ordinary inline links (calm, plainly links, wrapping with the
 * text, reachable by keyboard) and its Scripture references as quiet "Open in Bible" actions. Text
 * between them is kept exactly. A span that overlaps one before it, or runs past the text, is shown as
 * plain text; a reference inside a link is left to the link (no control inside another).
 */
function linked(
  text: string,
  links: SourceLink[] | undefined,
  references: SourceReference[] | undefined,
  marks: Paint,
  title: string,
  onFollow: (position: number) => void,
  onOpenInBible: (reference: ScriptureRef) => void,
  from = 0,
  to = text.length,
): React.ReactNode {
  if ((!links || links.length === 0) && (!references || references.length === 0)) return painted(text, from, to, marks);
  type Span = { start: number; end: number; link?: SourceLink; reference?: SourceReference };
  const spans: Span[] = [
    ...(links ?? []).map((link): Span => ({ start: link.start, end: link.end, link })),
    ...(references ?? []).map((reference): Span => ({ start: reference.start, end: reference.end, reference })),
  ].sort((a, b) => a.start - b.start || (a.link ? -1 : 1) - (b.link ? -1 : 1));
  const out: React.ReactNode[] = [];
  let at = from;
  for (const span of spans) {
    if (span.start < at || span.end > to || span.start >= span.end) continue;
    if (span.start > at) out.push(<Fragment key={`t${at}`}>{painted(text, at, span.start, marks)}</Fragment>);
    const written = painted(text, span.start, span.end, marks);
    if (span.link) {
      const target = span.link.target;
      out.push(
        <a
          key={`l${span.start}-${span.end}`}
          href="#"
          title={title}
          onClick={(event) => {
            event.preventDefault();
            onFollow(target);
          }}
          className="cursor-pointer text-link underline underline-offset-2 hover:opacity-80 focus-visible:outline-2 focus-visible:outline-offset-1 focus-visible:outline-primary"
        >
          {written}
        </a>,
      );
    } else if (span.reference) {
      out.push(
        <ScriptureReference key={`r${span.start}-${span.end}`} reference={span.reference.reference} onOpen={onOpenInBible}>
          {written}
        </ScriptureReference>,
      );
    }
    at = span.end;
  }
  if (at < to) out.push(<Fragment key={`t${at}`}>{painted(text, at, to, marks)}</Fragment>);
  return out;
}

function SegmentView({
  segment,
  previous,
  isTarget,
  targetRef,
  links,
  references,
  marks,
  ranges,
  paragraphMarks,
  marksOpen,
  marksId,
  onToggleMarks,
  onMarksChanged,
  onCloseMarks,
  markActions,
  onFollow,
  onOpenInBible,
}: SegmentViewProps) {
  const { t } = useTranslation();
  const metadata = segment.presentation;
  const page = metadata?.printedLabel ?? segment.pageLabel ?? segment.pageNumber;
  const renderRange = (start:number,end:number) => linked(segment.text,links,references,{marks,ranges},t("library.links.follow"),onFollow,onOpenInBible,start,end);
  const structured = () => {
    const headings = metadata?.headings ?? [];
    const boundaries = [...new Set([0,segment.text.length,...headings.flatMap(h=>[h.start,h.end]),...(metadata?.runs ?? []).filter(r=>["header","footer","note","artifact"].includes(r.role)).flatMap(r=>[r.start,r.end])])].filter(n=>n>=0 && n<=segment.text.length).sort((a,b)=>a-b);
    const out: React.ReactNode[] = [];
    for(let i=0;i<boundaries.length-1;i++) {
      const start=boundaries[i]!,end=boundaries[i+1]!;
      const heading=headings.find(h=>h.start<=start && h.end>=end);
      const role=metadata?.runs.find(r=>r.start<=start && r.end>=end)?.role;
      const words=renderRange(start,end);
      if(heading) out.push(<span key={start} role="heading" aria-level={Math.min(6,heading.level)} data-pdf-heading={heading.id} className={`block font-display font-semibold text-text-strong ${heading.level<=1 ? "mb-3 mt-5 text-2xl" : heading.level===2 ? "mb-2 mt-4 text-xl" : "mb-2 mt-3 text-lg"}`}>{words}</span>);
      else if(role === "header" || role === "footer" || role === "note" || role === "artifact") out.push(<span key={start} data-pdf-text-role={role} className={`block text-text-muted ${role === "note" ? "pl-4 text-sm" : "text-xs"}`}>{words}</span>);
      else out.push(<Fragment key={start}>{words}</Fragment>);
    }
    return out;
  };
  const roleClass = metadata?.role === "quote" ? "border-l-2 border-border pl-5 italic" : metadata?.role === "list" ? "pl-5" : metadata?.role === "note" ? "pl-4 text-sm text-text-muted" : ["header","footer","artifact"].includes(metadata?.role ?? "") ? "text-xs text-text-muted" : "";
  const newPage = segment.pageNumber !== null && segment.pageNumber !== previous?.pageNumber;
  const newSection = segment.pageNumber === null && segment.sectionPath && segment.sectionPath !== previous?.sectionPath;
  return (
    <>
      {newPage && (
        <p className="m-0 mt-2 border-t border-border-subtle pt-2 font-ui text-[0.875rem] font-medium text-text-muted" aria-label={t("library.reader.pageMarker", { page: String(page) })}>
          {locatorLabel(t, segment.locator, page)}{metadata?.printedLabel && <> · {t("library.structure.physical",{page:segment.pageNumber})}</>}
        </p>
      )}
      {newSection && (
        <p data-section-marker className="m-0 mt-3 font-display text-[1.125rem] font-heading text-text-strong">
          {segment.sectionPath}
        </p>
      )}
      <p
        ref={targetRef as React.RefObject<HTMLParagraphElement> | undefined}
        tabIndex={isTarget ? -1 : undefined}
        data-target={isTarget || undefined}
        data-seg-page={segment.pageNumber ?? undefined}
        data-seg-position={segment.position}
        data-segment-id={segment.id}
        data-pdf-role={metadata?.role}
        className={`m-0 leading-relaxed ${roleClass} ${isTarget ? "border-l-[3px] border-accent bg-note px-3 py-1" : ""}`}
      >
        {structured()}
        {paragraphMarks && paragraphMarks.length > 0 && <MarksMarker marks={paragraphMarks} open={marksOpen} controls={marksId} onToggle={onToggleMarks} />}
      </p>
      {marksOpen && paragraphMarks && paragraphMarks.length > 0 && <MarksInspector id={marksId} marks={paragraphMarks} onChanged={onMarksChanged} onClose={onCloseMarks} {...markActions} />}
      {isTarget && <PassageLinkNote segmentId={segment.id} />}
    </>
  );
}

/** Back / Forward through the places a reader has jumped between: small, never dominant. */
const NO_LINKS = new Map<number, SourceLink[]>();
const NO_REFERENCES = new Map<number, SourceReference[]>();

/**
 * The Scripture references written in a source's text, grouped by segment
 * (in order along it): read once per source from the stored index, never by
 * parsing the text again. A read that fails gives none, and the text reads as before.
 */
function useSourceReferences(sourceId: string | undefined): Map<number, SourceReference[]> {
  const api = useApi();
  const [found, setFound] = useState<{ id: string; references: Map<number, SourceReference[]> } | null>(null);
  useEffect(() => {
    if (!sourceId) return;
    let active = true;
    api.librarySourceReferences(sourceId).then(
      (all) => {
        if (!active) return;
        const by = new Map<number, SourceReference[]>();
        for (const r of all) by.set(r.position, [...(by.get(r.position) ?? []), r]);
        setFound({ id: sourceId, references: by });
      },
      (e) => void api.logError(`Scripture references in the source could not be read: ${e instanceof Error ? e.message : String(e)}`).catch(() => undefined),
    );
    return () => {
      active = false;
    };
  }, [api, sourceId]);
  return found && found.id === sourceId ? found.references : NO_REFERENCES;
}

/**
 * The links in an EPUB's extracted text, grouped by the segment they are in
 * (in order along it). Read once per source; a source without links, another
 * format, or a read that fails gives none, and the text reads as before.
 */
function useSourceLinks(source: { id: string; format: string | null } | null | undefined): Map<number, SourceLink[]> {
  const api = useApi();
  const [found, setFound] = useState<{ id: string; links: Map<number, SourceLink[]> } | null>(null);
  const id = source?.id;
  const isEpub = source?.format === "epub";
  useEffect(() => {
    if (!id || !isEpub) return;
    let active = true;
    api.librarySourceLinks(id).then(
      (all) => {
        if (!active) return;
        const by = new Map<number, SourceLink[]>();
        for (const link of all) by.set(link.position, [...(by.get(link.position) ?? []), link]);
        for (const list of by.values()) list.sort((a, b) => a.start - b.start);
        setFound({ id, links: by });
      },
      (e) => void api.logError(`Links in the source could not be read: ${e instanceof Error ? e.message : String(e)}`).catch(() => undefined),
    );
    return () => {
      active = false;
    };
  }, [api, id, isEpub]);
  return found && found.id === id ? found.links : NO_LINKS;
}
