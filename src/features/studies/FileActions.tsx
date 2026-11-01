import { useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Dialog } from "@/design-system/Dialog";
import { Button } from "@/design-system/Button";
import { ListingLifecycle, type TrashedRecord } from "@/features/portability/ListingLifecycle";
import { useApi } from "@/lib/ApiContext";
import { toApiError } from "@/lib/api";
import type { Project } from "@/types/domain";

export function NameDialog({ title, value = "", onClose, onSave }: { title: string; value?: string; onClose: () => void; onSave: (title: string) => Promise<void> }) {
  const { t } = useTranslation();
  const [name, setName] = useState(value); const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);
  const submit = async (event: FormEvent) => { event.preventDefault(); setBusy(true); try { await onSave(name.trim()); onClose(); } catch (e) { setError(toApiError(e).message); setBusy(false); } };
  return <Dialog title={title} onClose={onClose} actions={null}><form onSubmit={submit} className="flex flex-col gap-4">
    <label className="flex flex-col gap-1">{t("studies.files.name")}<input autoFocus value={name} onChange={e => setName(e.target.value)} className="h-11 min-w-0 aw-input px-3" /></label>
    {error && <p role="alert">{error}</p>}
    <div className="flex flex-wrap gap-2"><Button type="submit" variant="primary" disabled={busy || !name.trim()}>{t("common.save")}</Button><Button onClick={onClose}>{t("common.cancel")}</Button></div>
  </form></Dialog>;
}

export function FileActions({ record, projects, rootId, projectId, onChanged, onTrashed }: { record: TrashedRecord; projects?: Project[]; rootId?: string; projectId?: string; onChanged: () => void; onTrashed: () => void }) {
  const { t } = useTranslation(); const api = useApi();
  const [rename, setRename] = useState(false); const [moving, setMoving] = useState(false);
  const [destination, setDestination] = useState(projectId ?? ""); const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);
  const move = async () => { setBusy(true); try { await api.studyMove(record.id, destination); setMoving(false); onChanged(); } catch (e) { setError(toApiError(e).message); } finally { setBusy(false); } };
  return <>
    <ListingLifecycle record={record} label={t("studies.files.actions")} onDone={onTrashed} actions={[
      { label: t("studies.files.rename"), onSelect: () => setRename(true) },
      ...(record.kind === "study" && projects && rootId ? [{ label: t("studies.files.move"), onSelect: () => { setError(null); setDestination(projectId ?? rootId); setMoving(true); } }] : []),
    ]} />
    {rename && <NameDialog title={t("studies.files.rename")} value={record.title} onClose={() => setRename(false)} onSave={async title => { if (record.kind === "study") await api.studyUpdate({ id: record.id, title }); else await api.projectUpdate({ id: record.id, title }); onChanged(); }} />}
    {moving && <Dialog title={t("studies.files.moveTitle", { title: record.title })} onClose={() => setMoving(false)} actions={<><Button variant="primary" disabled={busy || !destination || destination === projectId} onClick={() => void move()}>{t("studies.files.move")}</Button><Button onClick={() => setMoving(false)}>{t("common.cancel")}</Button></>}>
      <label className="flex flex-col gap-1">{t("studies.files.destination")}<select autoFocus className="h-11 min-w-0 aw-input px-2" value={destination} onChange={e => setDestination(e.target.value)}>{projects?.map(p => <option key={p.id} value={p.id}>{p.id === rootId ? t("studies.home.title") : p.title}</option>)}</select></label>
      <p className="type-meta">{t("studies.files.moveHint")}</p>{error && <p role="alert">{error}</p>}
    </Dialog>}
  </>;
}
