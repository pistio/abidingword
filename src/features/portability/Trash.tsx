import { useCallback, useState } from "react";
import { Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { useLoad } from "@/hooks/useLoad";
import { useApi } from "@/lib/ApiContext";
import { toApiError } from "@/lib/api";

export function DeleteControl({ id, title, kind, permanent = false, initiallyConfirm = false, onCancel, onDone }: { id: string; title: string; kind: "project" | "study"; permanent?: boolean; initiallyConfirm?: boolean; onCancel?: () => void; onDone: () => void }) {
  const { t } = useTranslation();
  const api = useApi();
  const [confirm, setConfirm] = useState(initiallyConfirm);
  const [typed, setTyped] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const remove = async () => {
    if (busy || (permanent && typed !== title)) return;
    setBusy(true); setError(null);
    try {
      if (kind === "project") await api.projectDelete(id, permanent);
      else await api.studyDelete(id, permanent);
      onDone();
    } catch (e) { setError(toApiError(e).message); }
    finally { setBusy(false); }
  };
  return <section className="flex flex-col gap-2 border-t border-danger pt-3">
    <h3 className="type-group m-0">{t("settings.trash.danger")}</h3>
    <p className="m-0 text-text-muted">{t(`settings.trash.${kind}Hint`)}</p>
    {!confirm ? <Button variant="quiet" onClick={() => setConfirm(true)}><Trash2 className="size-4" aria-hidden="true" />{t(permanent ? "settings.trash.permanent" : "settings.trash.move")}</Button> : <div role="alertdialog" aria-label={t(permanent ? "settings.trash.permanentConfirm" : "settings.trash.confirm", { title })} className="flex flex-col gap-2">
      <p className="m-0">{t(permanent ? "settings.trash.permanentConfirm" : "settings.trash.confirm", { title })}</p>
      {permanent && <label className="flex flex-col gap-1">{t("settings.trash.typeTitle")}<input className="aw-input px-2" value={typed} onChange={e => setTyped(e.target.value)} /></label>}
      <div className="flex flex-wrap gap-2"><Button variant="danger" disabled={busy || (permanent && typed !== title)} onClick={() => void remove()}>{t(permanent ? "settings.trash.permanent" : "settings.trash.move")}</Button><Button autoFocus disabled={busy} onClick={() => { setConfirm(false); setTyped(""); onCancel?.(); }}>{t("common.cancel")}</Button></div>
    </div>}
    {error && <p role="alert">{error}</p>}
  </section>;
}

export function Trash({ workspaceId }: { workspaceId: string }) {
  const { t } = useTranslation(); const api = useApi();
  const list = useLoad(useCallback(() => api.trashList(workspaceId), [api, workspaceId]));
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  return <section className="flex flex-col gap-3" aria-label={t("settings.trash.title")}>
    <h3 className="type-group m-0">{t("settings.trash.title")}</h3>
    <p className="m-0 text-text-muted">{t("settings.trash.hint")}</p>
    {list.data?.length === 0 && <p>{t("settings.trash.empty")}</p>}
    {list.data?.map(entry => <article key={entry.id} className="flex flex-col gap-2 rounded-aw border border-border p-3">
      <h4 className="type-item m-0">{entry.title}</h4>
      <Button disabled={busy} onClick={async () => { setBusy(true); setError(null); try { if (entry.kind === "project") await api.projectRestore(entry.id); else await api.studyRestore(entry.id); list.reload(); } catch (e) { setError(toApiError(e).message); } finally { setBusy(false); } }}>{t("settings.trash.restore")}</Button>
      <DeleteControl {...entry} permanent onDone={list.reload} />
    </article>)}
    {(error || list.error) && <p role="alert">{error ?? list.error?.message}</p>}
  </section>;
}
