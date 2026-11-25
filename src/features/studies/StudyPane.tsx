import { Button } from "@/design-system/Button";
import { useStudyFocus } from "./StudyFocus";
import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { ChevronRight } from "lucide-react";
import { Notice } from "@/design-system/Notice";
import { PaneHeader } from "@/design-system/PaneHeader";
import { SaveTrackerProvider } from "@/hooks/SaveTracker";
import { useLoad } from "@/hooks/useLoad";
import { useApi } from "@/lib/ApiContext";
import { readLocal, writeLocal } from "@/lib/storage";
import type { Project, Workspace } from "@/types/domain";
import { StudyHome } from "./StudyHome";
import { TeachingView } from "@/features/teaching/TeachingView";
import { ProjectView } from "./ProjectView";
import { StudyEditor } from "./StudyEditor";
import { rememberProject } from "./projects";
import { StudySearchResults } from "./StudySearchResults";

export type StudyLocation =
  // The start: the plain list of studies (see StudyHome).
  | { kind: "projects" }
  // The project list, for organising.
  | { kind: "projectList" }
  | { kind: "project"; projectId: string }
  | { kind: "teaching"; projectId: string; create?: boolean }
  | { kind: "study"; projectId: string; studyId: string };

const LOCATION_KEY = "studyPane.location";

interface StudyPaneProps {
  workspace: Workspace;
  onRevealScripture: () => void;
  /** A Study search to show, or null. */
  search: string | null;
  onCloseSearch: () => void;
}

/**
 * The centre pane: browse projects, open a project's studies, edit a study,
 * or show Study search results. The last place is remembered, so reopening
 * the app returns to it.
 *
 * Primary passage rule: when the user opens a study (from a list, search or
 * a related-study link), the Scripture pane moves to its primary passage.
 * Returning to the last study at app start, and anything done inside a
 * study, leave Scripture where the user put it.
 */
export function StudyPane({ workspace, onRevealScripture, search, onCloseSearch }: StudyPaneProps) {
  const { t } = useTranslation();
  const [location, setLocation] = useState<StudyLocation>(() =>
    readLocal<StudyLocation>(LOCATION_KEY, { kind: "projects" }),
  );
  // Set only when the user opens a study; never saved.
  const [revealFor, setRevealFor] = useState<string | null>(null);
  useEffect(() => writeLocal(LOCATION_KEY, location.kind === "teaching" ? { kind: location.kind, projectId: location.projectId } : location), [location]);
  // The Study open here is what "Add to Study" in a source adds to (v0.15); none while browsing.
  const { setOpenStudy } = useStudyFocus();
  const openId = location.kind === "study" ? location.studyId : null;
  useEffect(() => setOpenStudy(openId), [openId, setOpenStudy]);

  const goToProjects = useCallback(() => setLocation({ kind: "projects" }), []);
  const openStudy = useCallback(
    (study: { id: string; projectId: string }) => {
      setRevealFor(study.id);
      setLocation({ kind: "study", projectId: study.projectId, studyId: study.id });
      onCloseSearch();
    },
    [onCloseSearch],
  );
  const projectId = location.kind === "projects" || location.kind === "projectList" ? null : location.projectId;

  return (
    <>
      <PaneHeader id="pane-study-title" title={t("app.panes.name.study")} />
      <SaveTrackerProvider>
        <div data-scroll-root className="min-h-0 flex-1 overflow-y-auto">
          <div className="@container mx-auto flex max-w-[50rem] flex-col gap-3 px-7 pb-24 pt-2">
            {search !== null ? (
              <StudySearchResults query={search} onOpen={openStudy} onClose={onCloseSearch} />
) : projectId === null ? (
              <StudyHome
                workspace={workspace}
                onOpenProject={(id) => setLocation({ kind: "project", projectId: id })}
                onOpenStudy={openStudy}
              />
            ) : (
              <ProjectScope
                workspaceId={workspace.id}
                projectId={projectId}
                location={location}
                setLocation={setLocation}
                onMissing={goToProjects}
                onOpenStudy={openStudy}
                revealFor={revealFor}
                onRevealScripture={onRevealScripture}
              />
            )}
          </div>
        </div>
      </SaveTrackerProvider>
    </>
  );
}

interface ProjectScopeProps {
  workspaceId: string;
  projectId: string;
  location: StudyLocation;
  setLocation: (location: StudyLocation) => void;
  onMissing: () => void;
  onOpenStudy: (study: { id: string; projectId: string }) => void;
  revealFor: string | null;
  onRevealScripture: () => void;
}

function ProjectScope({ workspaceId, projectId, location, setLocation, onMissing, onOpenStudy, revealFor, onRevealScripture }: ProjectScopeProps) {
  const { t } = useTranslation();
  const api = useApi();
  const [rootTitle] = useState(() => t("studies.projects.defaultTitle"));
  const project = useLoad(useCallback(() => api.projectGet(projectId), [api, projectId]));
  const projects = useLoad(useCallback(async () => { await api.projectRoot(workspaceId, rootTitle); return api.projectList(workspaceId); }, [api, workspaceId, rootTitle]));
  const root = useLoad(useCallback(() => api.projectRoot(workspaceId, rootTitle), [api, workspaceId, rootTitle]));
  const atRoot = root.data?.id === projectId;
  useEffect(() => { if (atRoot && location.kind === "project") onMissing(); }, [atRoot, location.kind, onMissing]);
  useEffect(() => rememberProject(projectId), [projectId]);
  const missing = project.error?.kind === "notFound";
  const openProject = useCallback(() => setLocation({ kind: "project", projectId }), [setLocation, projectId]);

  useEffect(() => {
    if (missing) onMissing();
  }, [missing, onMissing]);

  if (missing) return null;
  const error = project.error ?? projects.error ?? root.error;
  if (error) return <Notice tone="error">{error.message}</Notice>;
  if (atRoot && location.kind === "project") return null;
  if (!project.data || !projects.data || !root.data) return <p className="text-text-muted">{t("studies.projects.opening")}</p>;

  const inStudy = location.kind === "study";
  return (
    <>
      {/* In a study the way back is one button beside its title, not a row of its own. */}
      {!inStudy && (
        <Breadcrumb
          project={project.data}
          onProjects={() => setLocation({ kind: "projects" })}
        />
      )}
      {location.kind === "study" ? (
        <StudyEditor
          key={location.studyId}
          back={
            atRoot
              ? { label: t("studies.home.back"), onBack: () => setLocation({ kind: "projects" }) }
              : { label: t("studies.editor.backTo", { place: project.data.title }), onBack: openProject }
          }
          studyId={location.studyId}
          revealPrimary={revealFor === location.studyId}
          onMissing={atRoot ? onMissing : openProject}
          onOpenStudy={onOpenStudy}
          onRevealScripture={onRevealScripture}
        />
      ) : location.kind === "teaching" ? (
        <><Button variant="quiet" onClick={openProject}>{t("studies.teaching.workspace.project")}</Button><TeachingView onOpenStudy={onOpenStudy} projectId={projectId} startNew={location.create} onRevealScripture={onRevealScripture} /></>
      ) : (
        <ProjectView rootId={root.data.id} projects={projects.data} onDeleted={onMissing} project={project.data} onProjectChanged={project.setData} onOpenStudy={onOpenStudy} />
      )}
    </>
  );
}

function Breadcrumb({ project, onProjects }: { project: Project; onProjects: () => void }) {
  const { t } = useTranslation();
  return <nav aria-label={t("studies.projects.breadcrumb")}>
    <ol className="m-0 flex list-none flex-wrap items-center gap-1 p-0 text-[0.875rem]">
      <li><button type="button" className="min-h-11 rounded-aw px-1 text-link hover:bg-hover" onClick={onProjects}>{t("studies.home.title")}</button></li>
      <li aria-hidden="true"><ChevronRight className="size-4 text-text-muted" /></li>
      <li className="min-w-0 break-words"><span aria-current="page" className="px-1 text-text-muted">{project.title}</span></li>
    </ol>
  </nav>;
}
