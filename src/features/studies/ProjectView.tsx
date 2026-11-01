import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { StudyCreationActions } from "./StudyCreationActions";
import { InfoControl } from "@/design-system/InfoControl";
import { WorkspaceTrash } from "@/features/portability/ListingLifecycle";
import { DeleteControl } from "@/features/portability/Trash";
import { ProjectExport } from "@/features/portability/ProjectExport";
import type { Project } from "@/types/domain";
import type { StudySummary } from "@/types/studies";
import { toApiError } from "@/lib/api";
import { Notice } from "@/design-system/Notice";
import { useApi } from "@/lib/ApiContext";
import { EditableTitle } from "./EditableTitle";
import { studyCountLabel } from "./format";
import { NewStudyForm } from "./NewStudyForm";
import { StudyList } from "./StudyList";

interface ProjectViewProps {
  onDeleted: () => void;
  rootId?: string;
  projects?: Project[];
  project: Project;
  onProjectChanged: (project: Project) => void;
  onOpenStudy: (study: { id: string; projectId: string }) => void;
}

export function ProjectView({ onDeleted, project, onProjectChanged, onOpenStudy, rootId, projects }: ProjectViewProps) {
  const { t } = useTranslation();
  const api = useApi();
  const [listVersion, setListVersion] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const refresh = () => { setListVersion(v => v + 1); void api.projectGet(project.id).then(p => { onProjectChanged(p); setError(null); }, e => setError(toApiError(e).message)); };
  const rename = useCallback(
    async (title: string) => onProjectChanged(await api.projectUpdate({ id: project.id, title })),
    [api, project.id, onProjectChanged],
  );
  // The creation form is the main thing in an empty project; otherwise it
  // is put away until it is asked for, and the studies come first.
  const [asked, setAsked] = useState(false);
  const form = useRef<HTMLDivElement>(null);
  const opener = useRef<HTMLButtonElement>(null);
  const putAway = () => {
    setAsked(false);
    requestAnimationFrame(() => opener.current?.focus());
  };

  return (
    <div className="flex flex-col gap-8">
        <div>
          <p className="caps-label m-0 mb-2">{t("studies.projects.project")}</p>
          <EditableTitle key={project.id} label={t("studies.projects.titleLabel")} value={project.title} onSave={rename} />
          <div className="-ml-2 mt-1 flex flex-wrap items-center gap-x-2 text-[0.9375rem] text-text-muted">
            <ProjectInfo onDeleted={onDeleted} project={project} onChanged={onProjectChanged} />
            <span className="px-2">{studyCountLabel(t, project.studyCount)}</span>
          </div>
        </div>

      <StudyCreationActions studyButtonRef={opener} onStudy={() => { setAsked(true); requestAnimationFrame(() => form.current?.querySelector("input")?.focus()); }} />
      <div ref={form}>
        {asked && <NewStudyForm projectId={project.id} autoFocus={asked} onCancel={putAway} onCreated={onOpenStudy} />}
      </div>

        <section aria-labelledby="studies-heading" className="flex flex-col gap-3">
          <h2 id="studies-heading" className="type-section m-0">
            {t("studies.projects.studies")}
          </h2>
          <StudyList version={listVersion} projectId={project.id} rootId={rootId} projects={projects} onChanged={refresh} onOpen={(s: StudySummary) => onOpenStudy(s)} />
        </section>

      {error && <Notice tone="error">{error}</Notice>}
      <footer className="mt-6 border-t border-border-subtle pt-3"><WorkspaceTrash onChanged={refresh} /></footer>
    </div>
  );
}

/**
 * Project information (v0.19): the project's description as a draft saved
 * only with Save. Its studies are the page itself; the title is edited where
 * it stands.
 */
function ProjectInfo({ project, onChanged, onDeleted }: { onDeleted: () => void; project: Project; onChanged: (project: Project) => void }) {
  const { t } = useTranslation();
  const api = useApi();
  return (
    <InfoControl<{ description: string }>
      title={t("studies.info.project")}
      buttonLabel={t("common.info.button")}
      initial={() => ({ description: project.description ?? "" })}
      current={async () => ({ description: (await api.projectGet(project.id)).description ?? "" })}
      save={async (draft) => onChanged(await api.projectUpdate({ id: project.id, description: draft.description }))}
      fields={(draft, setDraft) => (
        <label className="flex flex-col gap-1">
          <span className="text-[0.9375rem] font-medium text-text-muted">{t("studies.info.projectDescription")}</span>
          <textarea
            rows={3}
            value={draft.description}
            onChange={(e) => setDraft({ description: e.target.value })}
            placeholder={t("studies.info.projectDescriptionHint")}
            className="reading resize-y rounded-aw border border-border-subtle bg-transparent px-3 py-1.5 text-[1rem] placeholder:text-text-muted focus:border-border"
          />
        </label>
      )}
      actions={(close) => <><ProjectExport projectId={project.id} title={project.title} quiet /><DeleteControl id={project.id} title={project.title} kind="project" onDone={() => { close(); onDeleted(); }} /></>}
    />
  );
}
