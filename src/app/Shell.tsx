import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useApi } from "@/lib/ApiContext";
import type { SavedReaderPosition } from "@/lib/api";
import { useScripture } from "@/features/scripture/ScriptureContext";
import { useTranslation } from "react-i18next";
import { AddBibleView } from "@/features/scripture/AddBibleView";
import { ScripturePane } from "@/features/scripture/ScripturePane";
import { AddSourceView } from "@/features/library/AddSourceView";
import { LibraryNavProvider, type LibraryNav, type QuotedWords } from "@/features/library/LibraryNav";
import { LibraryView, type LibraryBrowseState } from "@/features/library/LibraryView";
import { SourceReader, type ReaderPosition } from "@/features/library/SourceReader";
import { SourcesPane, type PaneSource } from "@/features/library/SourcesPane";
import { UpdateNotice } from "@/features/updates/UpdateNotice";
import { Sheet } from "@/design-system/Sheet";
import { SettingsView } from "@/features/settings/SettingsView";
import { TeachingTemplateEditor } from "@/features/settings/TeachingTemplateEditor";
import { TemplateEditor } from "@/features/settings/TemplateEditor";
import { StudyPane } from "@/features/studies/StudyPane";
import { AppBar } from "@/layouts/AppBar";
import { NARROW_WIDTH, useWorkspaceLayout } from "@/layouts/layoutPrefs";
import { WorkspaceLayout } from "@/layouts/WorkspaceLayout";
import type { AppInfo, Workspace } from "@/types/domain";

type View = "workspace" | "settings" | "library" | "source" | "addSource";
/** Tasks that open over the current page, which stays as it was behind them. */
type Overlay = { kind: "addBible" } | { kind: "template"; templateId: string | null } | { kind: "teachingTemplate"; templateId: string | null } | null;

export function Shell({ info, workspace, initialReaderPositions = [] }: { info: AppInfo; workspace: Workspace; initialReaderPositions?: SavedReaderPosition[] }) {
  const { t } = useTranslation();
  const api = useApi();
  const [view, setView] = useState<View>("workspace");
  /** Set when the Library asks for Settings → Sources; cleared for an ordinary visit. */
  const [settingsSection, setSettingsSection] = useState<"sources" | "language" | null>(null);
  const [recentSourceIds, setRecentSourceIds] = useState<string[]>([]);
  const readerRequest = useRef(0);
  // Navigation coordinates are persisted in SQLite, and therefore included in
  // the normal .awbackup. They are never derived from the source's contents.
  const positions = useRef(new Map<string, ReaderPosition>(initialReaderPositions.map(row => [row.sourceId, row.position])));
  const persisted = useRef(new Map(initialReaderPositions.map(row => [row.sourceId, JSON.stringify(row.position)])));
  const inFlight = useRef(new Map<string, Promise<void>>());
  const positionReaders = useRef(new Map<string, () => ReaderPosition | null>());
  const persist = useCallback((sourceId: string, position: ReaderPosition) => {
    const fingerprint = JSON.stringify(position);
    if (persisted.current.get(sourceId) === fingerprint) return;
    persisted.current.set(sourceId, fingerprint);
    // Serialize each source's writes: an earlier scroll can never overwrite
    // a later place, even when the native commands complete out of order.
    const previous = inFlight.current.get(sourceId) ?? Promise.resolve();
    const next = previous.catch(() => undefined).then(() => api.sourceReaderPositionSave({ sourceId, position }));
    inFlight.current.set(sourceId, next);
    void next.catch(() => {
      // Retry the latest place on the next capture, without risking startup.
      if (persisted.current.get(sourceId) === fingerprint) persisted.current.delete(sourceId);
    }).finally(() => {
      if (inFlight.current.get(sourceId) === next) inFlight.current.delete(sourceId);
    });
  }, [api]);
  const rememberPosition = useCallback((sourceId: string, position: ReaderPosition) => {
    positions.current.set(sourceId, position);
    persist(sourceId, position);
  }, [persist]);
  const registerPositionReader = useCallback((sourceId: string, read: (() => ReaderPosition | null) | null) => {
    if (read) positionReaders.current.set(sourceId, read);
    else positionReaders.current.delete(sourceId);
  }, []);
  const capturePositions = useCallback(() => {
    for (const [sourceId, read] of positionReaders.current) {
      const position = read();
      if (position) rememberPosition(sourceId, position);
    }
  }, [rememberPosition]);
  useEffect(() => {
    // Frequent bounded snapshots protect a long reading session from an
    // unexpected application exit; navigation/closing also saves immediately.
    const timer = window.setInterval(capturePositions, 3000);
    const onHide = () => capturePositions();
    window.addEventListener("pagehide", onHide);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("pagehide", onHide);
      capturePositions();
    };
  }, [capturePositions]);
  const [browseState, setBrowseState] = useState<LibraryBrowseState>();
  const [settingsFrom, setSettingsFrom] = useState<View>("workspace");
  const [returnTo, setReturnTo] = useState<View>("workspace");
  const [overlay, setOverlay] = useState<Overlay>(null);
  const [templatesVersion, setTemplatesVersion] = useState(0);
  const [teachingTemplatesVersion, setTeachingTemplatesVersion] = useState(0);
  const [studySearch, setStudySearch] = useState<string | null>(null);
  // Each request to open the library (id) remembers its latest search.
  const [library, setLibrary] = useState<{ id: number; query?: string }>({ id: 0 });
  // The full-width reader, and where its Back returns to.
  const [reading, setReading] = useState<{ request: number; sourceId: string; segmentId?: string; structureNodeId?: string; from: View; position?: ReaderPosition; words?: QuotedWords; libraryOrigin?: boolean } | null>(null);
  // A source read inside the Sources pane. Kept here, not in the pane, so it
  // is still open after a visit to the full reader, Settings or the Library.
  const [paneSource, setPaneSource] = useState<PaneSource | null>(null);
  const layout = useWorkspaceLayout();
  const { goTo } = useScripture();
  const { open, setOpen } = layout;

  const showView = (next: View) => {
    capturePositions();
    if (next === "workspace") setPaneSource((current) => current ? { ...current, position: positions.current.get(current.sourceId) ?? current.position } : current);
    if (next === "source") setReading((current) => current ? { ...current, position: positions.current.get(current.sourceId) ?? current.position } : current);
    setView(next);
  };

  const addBible = () => setOverlay({ kind: "addBible" });
  const revealScripture = () => {
    if (!open.scripture) setOpen("scripture", true);
  };
  const sideOf = (pane: "scripture" | "sources") =>
    layout.order.indexOf(pane) < layout.order.indexOf("study") ? "left" : "right";
  const libraryNav = useMemo<LibraryNav>(
    () => ({
      recentSourceIds,
      openLibrary: (query) => {
        capturePositions();
        if (query !== undefined) setBrowseState(undefined);
        setLibrary((current) => ({ id: current.id + 1, query }));
        setView("library");
      },
      // From the workspace a source opens in the Sources pane, beside
      // Scripture and the Study. On a narrow window with that pane hidden,
      // and from every other screen, it opens in the full-width reader,
      // whose Back returns to where it was opened from (after adding a
      // source: the library).
      openSource: (sourceId, segmentId, position, words, structureNodeId) => {
        setRecentSourceIds((ids) => [sourceId, ...ids.filter((id) => id !== sourceId)].slice(0, 10));
        capturePositions();
        // A citation/search target always wins over a remembered reading place.
        if (!position && !segmentId && !words && !structureNodeId) position = positions.current.get(sourceId);
        if (position) rememberPosition(sourceId, position);
        if (view === "workspace" && (open.sources || window.innerWidth >= NARROW_WIDTH)) {
          if (!open.sources) setOpen("sources", true);
          const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
          setPaneSource((current) => ({ sourceId, segmentId, structureNodeId, position, words, opener, key: (current?.key ?? 0) + 1 }));
          return;
        }
        const from = view === "addSource" ? "library" : view === "source" ? (reading?.from ?? "workspace") : view;
        setReading({ request: ++readerRequest.current, sourceId, segmentId, structureNodeId, from, position, words, libraryOrigin: from === "library" || (view === "source" && reading?.libraryOrigin) });
        setView("source");
      },
      addSource: () => {
        capturePositions();
        setReturnTo(view);
        setView("addSource");
      },
      // A reference read in a source: Scripture goes there; nothing about a Study changes.
      // From the full reader the source moves into the Sources pane, at the same place, beside Scripture.
      openInBible: (reference, position) => {
        goTo(reference);
        if (!open.scripture) setOpen("scripture", true);
        if (view === "source" && reading) {
          if (!open.sources) setOpen("sources", true);
          setPaneSource((current) => ({ sourceId: reading.sourceId, segmentId: reading.segmentId, structureNodeId: reading.structureNodeId, position, words: reading.words, returnToLibrary: reading.libraryOrigin ?? reading.from === "library", opener: null, key: (current?.key ?? 0) + 1 }));
          setView("workspace");
        }
      },
    }),
    [recentSourceIds, view, reading, open.sources, open.scripture, setOpen, goTo, capturePositions, rememberPosition],
  );

  return (
    <LibraryNavProvider value={libraryNav}>
    {/* Pinned to the window, not sized from 100vh: it fills the window at any size or display scale. */}
    <div data-testid="app-shell" className="fixed inset-0 flex min-h-0 flex-col overflow-hidden bg-bg text-text">
      <a
        href="#pane-study"
        className="sr-only focus:not-sr-only focus:absolute focus:left-4 focus:top-2 focus:z-10 focus:rounded-aw focus:bg-primary focus:px-4 focus:py-2 focus:text-on-primary"
      >
        {t("app.skipToStudy")}
      </a>
      <AppBar
        view={
          view === "workspace"
            ? "workspace"
            : view === "library" || view === "source" || view === "addSource"
              ? "library"
              : "settings"
        }
        scriptureOpen={open.scripture}
        sourcesOpen={open.sources}
        scriptureSide={sideOf("scripture")}
        sourcesSide={sideOf("sources")}
        onToggleScripture={() => { capturePositions(); setOpen("scripture", !open.scripture); }}
        onToggleSources={() => {
          capturePositions();
          setPaneSource((current) => current ? { ...current, position: positions.current.get(current.sourceId) ?? current.position } : current);
          setOpen("sources", !open.sources);
        }}
        onOpenLanguage={() => { capturePositions(); setSettingsSection("language"); setSettingsFrom(view === "settings" ? settingsFrom : view); setView("settings"); }}
        onOpenSettings={() => {
          capturePositions();
          if (view === "settings") showView(settingsFrom);
          else {
            setSettingsSection(null);
            setSettingsFrom(view);
            setView("settings");
          }
        }}
        onSearch={revealScripture}
        onSearchStudies={setStudySearch}
        onSearchLibrary={libraryNav.openLibrary}
      />
      <UpdateNotice />
      {view === "settings" && (
        <SettingsView
          key={settingsSection ?? "settings"}
          info={info}
          initialSection={settingsSection}
          onClose={() => showView(settingsFrom)}
          onResetLayout={layout.reset}
          onAddBible={addBible}
          onEditTemplate={(templateId) => setOverlay({ kind: "template", templateId })}
          templatesVersion={templatesVersion}
          onEditTeachingTemplate={(templateId) => setOverlay({ kind: "teachingTemplate", templateId })}
          teachingTemplatesVersion={teachingTemplatesVersion}
        />
      )}
      {view === "library" && (
        <LibraryView
          key={library.id}
          initialQuery={library.query}
          initialState={browseState}
          onRemember={setBrowseState}
          onSearched={(query) => setLibrary((current) => ({ ...current, query }))}
          onClose={() => showView("workspace")}
          onManage={() => {
            setSettingsSection("sources");
            setSettingsFrom("library");
            setView("settings");
          }}
          onRemoved={(sourceId) => setPaneSource((current) => (current?.sourceId === sourceId ? null : current))}
        />
      )}
      {view === "addSource" && <AddSourceView onClose={() => showView(returnTo)} />}
      {view === "source" && reading && (
        <SourceReader
          key={reading.request}
          sourceId={reading.sourceId}
          segmentId={reading.segmentId}
          structureNodeId={reading.structureNodeId}
          initialPosition={reading.position}
          quotedWords={reading.words}
          onPositionReader={registerPositionReader}
          onClose={(position) => {
            // Back to the workspace: the pane continues where this reader was.
            if (reading.from === "workspace") {
              setPaneSource((current) =>
                current?.sourceId === reading.sourceId ? { ...current, position, key: current.key + 1 } : current,
              );
            }
            rememberPosition(reading.sourceId, position);
            setView(reading.from);
          }}
        />
      )}
      {overlay?.kind === "addBible" && (
        <Sheet labelledBy="add-bible-title">
          <AddBibleView onClose={() => setOverlay(null)} />
        </Sheet>
      )}
      {overlay?.kind === "template" && (
        <Sheet labelledBy="template-editor-title">
          <TemplateEditor
            templateId={overlay.templateId}
            onClose={() => {
              setOverlay(null);
              setTemplatesVersion((v) => v + 1);
            }}
          />
        </Sheet>
      )}
      {overlay?.kind === "teachingTemplate" && (
        <Sheet labelledBy="teaching-template-editor-title">
          <TeachingTemplateEditor
            templateId={overlay.templateId}
            onClose={() => {
              setOverlay(null);
              setTeachingTemplatesVersion((v) => v + 1);
            }}
          />
        </Sheet>
      )}
      {view === "workspace" && (
        <WorkspaceLayout
          layout={layout}
          scripture={<ScripturePane onAddBible={addBible} />}
          study={
            <StudyPane
              workspace={workspace}
              onRevealScripture={revealScripture}
              search={studySearch}
              onCloseSearch={() => setStudySearch(null)}
            />
          }
          sources={
            <SourcesPane
              reading={paneSource}
              onPositionReader={registerPositionReader}
              onCloseReader={() => {
                capturePositions();
                if (paneSource?.returnToLibrary) setView("library");
                setPaneSource(null);
              }}
              onExpandReader={(position) => {
                if (!paneSource) return;
                rememberPosition(paneSource.sourceId, position);
                setReading({ request: ++readerRequest.current, sourceId: paneSource.sourceId, segmentId: paneSource.segmentId, structureNodeId: paneSource.structureNodeId, from: "workspace", position, words: paneSource.words, libraryOrigin: paneSource.returnToLibrary });
                setView("source");
              }}
            />
          }
        />
      )}
    </div>
    </LibraryNavProvider>
  );
}
