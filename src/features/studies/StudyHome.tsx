import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import { Button } from "@/design-system/Button";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { useApi } from "@/lib/ApiContext";
import type { Workspace } from "@/types/domain";
import { WorkspaceTrash } from "@/features/portability/ListingLifecycle";
import { NewStudyForm } from "./NewStudyForm";
import { StudyList } from "./StudyList";
import { NameDialog } from "./FileActions";

interface StudyHomeProps {
  workspace: Workspace;
  onOpenProject: (projectId: string) => void;
  onOpenStudy: (study: { id: string; projectId: string }) => void;
}

/** One home at every library size. The implicit ownership container stays hidden. */
export function StudyHome({ workspace, onOpenProject, onOpenStudy }: StudyHomeProps) {
  const { t } = useTranslation(); const api = useApi();
  const [rootTitle] = useState(() => t("studies.projects.defaultTitle"));
  const [creating, setCreating] = useState(false); const [folder, setFolder] = useState(false); const [version, setVersion] = useState(0);
  const data = useLoad(useCallback(async () => {
    const root = await api.projectRoot(workspace.id, rootTitle);
    return { root, projects: await api.projectList(workspace.id) };
  }, [api, workspace.id, rootTitle]));
  const changed = () => { data.reload(); setVersion(v => v + 1); };
  return <div className="flex flex-col gap-5">
    <h1 className="type-title m-0">{t("studies.home.title")}</h1>
    <div className="flex flex-wrap gap-2"><Button variant="primary" onClick={() => setCreating(true)}><Plus className="size-4" aria-hidden="true" />{t("studies.newStudy.formLabel")}</Button><Button onClick={() => setFolder(true)}><Plus className="size-4" aria-hidden="true" />{t("studies.files.newProject")}</Button></div>
    {data.error && <Notice tone="error">{data.error.message}</Notice>}
    {creating && data.data && <NewStudyForm projectId={data.data.root.id} autoFocus onCancel={() => setCreating(false)} onCreated={onOpenStudy} />}
    {folder && <NameDialog title={t("studies.files.newProject")} onClose={() => setFolder(false)} onSave={async title => { await api.projectCreate({ workspaceId: workspace.id, title }); changed(); }} />}
    {data.data && <StudyList projectId={data.data.root.id} rootId={data.data.root.id} projects={data.data.projects} folders={data.data.projects.filter(p => p.id !== data.data!.root.id)} onOpenProject={onOpenProject} onOpen={onOpenStudy} version={version} onChanged={changed} />}
    <footer className="mt-6 border-t border-border-subtle pt-3"><WorkspaceTrash onChanged={changed} /></footer>
  </div>;
}
