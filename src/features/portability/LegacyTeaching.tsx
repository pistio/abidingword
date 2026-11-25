import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { useApi } from "@/lib/ApiContext";
import { useLoad } from "@/hooks/useLoad";
import { SaveTrackerProvider } from "@/hooks/SaveTracker";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { TeachingView } from "@/features/teaching/TeachingView";

/** Compatibility management lives in Data settings, outside ordinary study browsing. */
export function LegacyTeaching({ workspaceId }: { workspaceId: string }) {
  const { t } = useTranslation(); const api = useApi(); const [chosen, setChosen] = useState<{ projectId: string; id: string } | null>(null);
  const data = useLoad(useCallback(async () => {
    const projects = await api.projectList(workspaceId);
    const rows = await Promise.all(projects.map(async project => ({ project, rows: (await api.teachingListProject(project.id)).filter(row => !row.studyId) })));
    return rows.filter(row => row.rows.length > 0);
  }, [api, workspaceId]));
  return <details className="border-t border-border-subtle pt-3"><summary className="cursor-pointer type-group">{t("studies.files.legacy")}</summary>
    <p className="type-meta">{t("studies.files.legacyHint")}</p>
    {data.error && <p role="alert">{data.error.message}</p>}
    {data.data?.length === 0 && <p className="type-meta">{t("studies.files.noLegacy")}</p>}
    {data.data?.map(({ project, rows }) => <div key={project.id}><p className="type-meta">{project.title}</p>{rows.map(row => <Button key={row.id} variant="quiet" onClick={() => setChosen({ projectId: project.id, id: row.id })}>{row.title}</Button>)}</div>)}
    {chosen && <Dialog wide title={t("studies.files.legacy")} onClose={() => { setChosen(null); data.reload(); }} actions={<Button onClick={() => { setChosen(null); data.reload(); }}>{t("common.close")}</Button>}><SaveTrackerProvider><TeachingView key={chosen.id} projectId={chosen.projectId} initialTeachingId={chosen.id} onRevealScripture={() => undefined} /></SaveTrackerProvider></Dialog>}
  </details>;
}
