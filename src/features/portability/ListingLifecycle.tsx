import { useState } from "react";
import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ActionMenu, type MenuAction } from "@/design-system/ActionMenu";
import { Dialog } from "@/design-system/Dialog";
import { Button } from "@/design-system/Button";
import { useApi } from "@/lib/ApiContext";
import { toApiError } from "@/lib/api";
import { DeleteControl, Trash } from "./Trash";

export interface TrashedRecord { id: string; title: string; kind: "study" | "project" }

export function ListingLifecycle({ record, onDone, actions = [], label }: { record: TrashedRecord; onDone: () => void; actions?: MenuAction[]; label?: string }) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  return <span role="group" aria-label={t("settings.trash.actions", { title: record.title })}>
    <ActionMenu label={label ?? t("settings.trash.move")} actions={[...actions, { label: t("settings.trash.move"), icon: <Trash2 className="size-4" aria-hidden="true" />, onSelect: () => setOpen(true) }]} />
    {open && <Dialog title={t("settings.trash.confirm", { title: record.title })} actions={null} onClose={() => setOpen(false)}>
      <DeleteControl {...record} initiallyConfirm onCancel={() => setOpen(false)} onDone={() => { setOpen(false); onDone(); }} />
    </Dialog>}
  </span>;
}

export function UndoTrash({ record, onDone }: { record: TrashedRecord; onDone: () => void }) {
  const { t } = useTranslation(); const api = useApi();
  const [busy, setBusy] = useState(false); const [error, setError] = useState<string | null>(null);
  return <div role="status" className="flex flex-wrap items-center gap-2 rounded-aw border border-border-subtle p-2">
    <span>{t("settings.trash.moved", { title: record.title })}</span>
    <Button disabled={busy} onClick={async () => { setBusy(true); try { if (record.kind === "study") await api.studyRestore(record.id); else await api.projectRestore(record.id); onDone(); } catch (e) { setError(toApiError(e).message); setBusy(false); } }}>{t("common.lifecycle.undo")}</Button>
    {error && <p role="alert">{error}</p>}
  </div>;
}

/** The same durable Trash as Settings, reachable beside the lists it serves. */
export function WorkspaceTrash({ onChanged }: { onChanged: () => void }) {
  const { t } = useTranslation(); const api = useApi();
  const [workspaceId, setWorkspaceId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const close = () => { setWorkspaceId(null); onChanged(); };
  return <div>
    <Button variant="quiet" onClick={async () => { try { setWorkspaceId((await api.workspaceGetDefault()).id); } catch (e) { setError(toApiError(e).message); } }}><Trash2 className="size-4" aria-hidden="true" />{t("settings.trash.title")}</Button>
    {workspaceId && <Dialog wide title={t("settings.trash.title")} onClose={close} actions={<Button onClick={close}>{t("common.close")}</Button>}><Trash workspaceId={workspaceId} /></Dialog>}
    {error && <p role="alert">{error}</p>}
  </div>;
}
