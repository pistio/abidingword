import { useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { PaneHeader } from "@/design-system/PaneHeader";
import { Button } from "@/design-system/Button";
import { useScripture } from "@/features/scripture/ScriptureContext";
import { useLoad } from "@/hooks/useLoad";
import { referenceLabel } from "@/i18n/scripture";
import { useApi } from "@/lib/ApiContext";
import type { ScriptureRef } from "@/types/scripture";
import { useLibraryNav, type QuotedWords } from "./LibraryNav";
import { PassageSourceList } from "./PassageSourceList";
import { usePassageSourceExpansion } from "./usePassageSourceExpansion";
import { SourceReader, type ReaderPosition } from "./SourceReader";

/** A source open inside the Sources pane (kept by the shell, so it survives leaving the workspace). */
export interface PaneSource {
  sourceId: string;
  returnToLibrary?: boolean;
  segmentId?: string;
  structureNodeId?: string;
  /** Changes on every open, so opening the same place again re-targets it. */
  key: number;
  /** What had focus when the source was opened; focus returns there on Back. */
  opener: HTMLElement | null;
  /** Where to open (view and page) when continuing from the full reader. */
  position?: ReaderPosition;
  /** Cited words to show marked (opened from a Study finding's citation). */
  words?: QuotedWords;
}

interface SourcesPaneProps {
  reading?: PaneSource | null;
  onCloseReader?: () => void;
  onPositionReader?: (sourceId: string, read: (() => ReaderPosition | null) | null) => void;
  onExpandReader?: (position: ReaderPosition) => void;
}

/**
 * Sources pane: the personal library beside the study. Shows which sources
 * fit the passage being read (a literal mention, or a section whose heading
 * names it; both found when the source was imported, so turning a page only
 * asks the index), grouped by source with the reason for each place, recent
 * sources, and the way into the full Library. A source opened from the
 * workspace is read here, beside Scripture and the Study.
 */
export function SourcesPane({ reading, onCloseReader, onExpandReader, onPositionReader }: SourcesPaneProps = {}) {
  const { t } = useTranslation();
  const api = useApi();
  const { openLibrary, openSource: navigateSource, addSource, recentSourceIds = [] } = useLibraryNav();
  const { currentPassage, location, books } = useScripture();
  // The selected verse or passage; otherwise the chapter on screen.
  const reference = currentPassage?.canonical ?? `${location.book}.${location.chapter}`;
  const chapterCount = books.find((b) => b.code === (currentPassage?.book ?? location.book))?.chapterCount;
  const label = referenceLabel(
    currentPassage ?? {
      canonical: reference,
      display: "",
      book: location.book,
      startChapter: location.chapter,
      startVerse: null,
      endChapter: location.chapter,
      endVerse: null,
    },
    chapterCount,
  );
  const sources = useLoad(useCallback(() => api.libraryPassageSources(reference), [api, reference]));
  const reloadSources = sources.reload;
  useEffect(() => { if (!reading) reloadSources(); }, [reading, reloadSources]);
  const { expansion, toggle } = usePassageSourceExpansion(reference);
  const passageLabel = (ref: ScriptureRef) =>
    referenceLabel(ref, books.find((b) => b.code === ref.book)?.chapterCount);
  // Only sources that share a verse with the passage are offered (the core never returns a chapter-only match).
  const direct = sources.data ?? [];
  const recent = useLoad(useCallback(() => api.libraryList({}), [api]));
  const empty = recent.data?.length === 0;
  const opened = recentSourceIds.flatMap((id) => recent.data?.find((s) => s.id === id) ?? []);
  const otherSources = (recent.data ?? []).filter((source) => !recentSourceIds.includes(source.id) && !direct.some(group => group.sourceId === source.id));

  // Back from the reader: focus returns to what opened it (a citation in the
  // study, say) or, if that is gone, to the first heading of the overview.
  const overview = useRef<HTMLDivElement>(null);
  const lastOverview = useRef<{ reference: string; top: number } | null>(null);
  const openSource: ReturnType<typeof useLibraryNav>["openSource"] = (...args) => {
    lastOverview.current = { reference, top: overview.current?.scrollTop ?? 0 };
    navigateSource(...args);
  };
  const returning = useRef<{ opener: HTMLElement | null; sourceId: string; segmentId?: string } | null>(null);
  useEffect(() => {
    if (!returning.current || reading) return;
    const { opener, sourceId, segmentId } = returning.current;
    returning.current = null;
    if (overview.current) overview.current.scrollTop = lastOverview.current?.reference === reference ? lastOverview.current.top : 0;
    const candidates = [...(overview.current?.querySelectorAll<HTMLElement>("[data-source-result], [data-source-link]") ?? [])];
    const replacement = candidates.find(node => node.dataset.sourceResult === sourceId && node.dataset.sourceSegment === segmentId)
      ?? candidates.find(node => (node.dataset.sourceResult ?? node.dataset.sourceLink) === sourceId);
    if (opener?.isConnected) opener.focus({ preventScroll: true });
    else (replacement ?? overview.current?.querySelector<HTMLElement>("h3"))?.focus({ preventScroll: true });
  }, [reading, reference]);

  if (reading) {
    return (
      <>
        <PaneHeader id="pane-sources-title" title={t("app.panes.name.sources")} />
        <SourceReader
          key={reading.key}
          variant="pane"
          sourceId={reading.sourceId}
          segmentId={reading.segmentId}
          structureNodeId={reading.structureNodeId}
          initialPosition={reading.position}
          quotedWords={reading.words}
          onPositionReader={onPositionReader}
          backLabel={reading.returnToLibrary ? t("library.navigation.backToLibrary") : undefined}
          onClose={() => {
            returning.current = { opener: reading.opener, sourceId: reading.sourceId, segmentId: reading.segmentId };
            onCloseReader?.();
          }}
          onExpand={onExpandReader}
        />
      </>
    );
  }

  return (
    <>
      <PaneHeader id="pane-sources-title" title={t("app.panes.name.sources")} />
      <div ref={overview} className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-4 py-4">
        {/* Nothing until the library has loaded, so the pane never flickers
            from one layout to the other. */}
        {recent.error && <p className="m-0 text-[0.9375rem] text-danger">{recent.error.message}</p>}
        {recent.data === undefined ? null : empty ? (
          <section aria-labelledby="library-heading" className="flex flex-col items-start gap-3">
            <h3 id="library-heading" tabIndex={-1} className="type-group m-0">{t("library.pane.libraryHeading")}</h3>
            <p className="m-0 text-[0.9375rem] text-text-muted">{t("library.pane.empty")}</p>
            <Button onClick={addSource}>{t("library.addToLibraryEllipsis")}</Button>
          </section>
        ) : (
          <>
            <section aria-labelledby="mentions-heading" className="flex flex-col gap-2">
              <h3 id="mentions-heading" tabIndex={-1} className="type-group m-0">
                {t("library.pane.sourcesFor", { passage: label })}
              </h3>
              <p className="type-meta m-0">{t("library.pane.matchEvidence")}</p>
              {direct.length > 0 && direct.every(group => group.locations.every(place => place.origin === "explicit")) && <p role="status" className="type-meta m-0">{t("library.pane.mentionsOnly")}</p>}
              {sources.error && <p className="m-0 text-[0.9375rem] text-danger">{sources.error.message}</p>}
              {sources.data && sources.data.length === 0 && (
                <p className="m-0 text-[0.9375rem] text-text-muted">{t("library.pane.noSources")}</p>
              )}
              {direct.length > 0 && (
                <PassageSourceList
                  groups={direct}
                  label={t("library.pane.sourcesFor", { passage: label })}
                  passageLabel={passageLabel}
                  onOpen={(id,segment,node)=>openSource(id,segment,undefined,undefined,node)}
                  expansion={expansion}
                  onToggleMore={toggle}
                />
              )}
            </section>
            <section aria-labelledby="recent-heading" className="flex flex-col gap-2">
              <h3 id="recent-heading" className="type-group m-0">{t(opened.length > 0 ? "library.pane.recent" : "library.pane.libraryHeading")}</h3>
              {opened.length === 0 && <p className="type-meta m-0">{t("library.pane.noRecent")}</p>}
              <ul aria-label={t(opened.length > 0 ? "library.pane.recent" : "library.pane.libraryHeading")} className="m-0 flex list-none flex-col p-0">
                {(opened.length > 0 ? opened : otherSources).slice(0, 5).map((s) => (
                  <li key={s.id}>
                    <button type="button" data-source-link={s.id} aria-label={s.title} aria-describedby={opened.length > 0 ? `source-continue-${s.id}` : undefined} onClick={() => openSource(s.id)} className="w-full py-1 text-left text-[0.9375rem] text-text hover:underline">
                      {s.title}{opened.length > 0 && <span id={`source-continue-${s.id}`} className="block type-meta">{t("library.pane.continueReading")}</span>}{s.author ? <span className="text-text-muted">{t("common.listSeparator")}{s.author}</span> : null}
                    </button>
                  </li>
                ))}
              </ul>
              {opened.length > 0 && otherSources.length > 0 && <details className="text-[0.9375rem]">
                <summary className="cursor-pointer text-text-muted">{t("library.pane.otherSources")}</summary>
                <ul className="m-0 mt-1 list-none p-0">
                  {otherSources.slice(0, 5).map((source) => <li key={source.id}><button type="button" data-source-link={source.id} className="min-h-10 w-full py-1 text-left hover:underline" onClick={() => openSource(source.id)}>{source.title}</button></li>)}
                </ul>
              </details>}
            </section>
            <div className="flex flex-wrap gap-2 border-t border-border-subtle pt-4">
              <Button onClick={() => openLibrary()}>{t("library.pane.openLibrary")}</Button>
              <Button variant="quiet" onClick={addSource}>{t("library.addToLibraryEllipsis")}</Button>
            </div>
          </>
        )}
      </div>
    </>
  );
}
