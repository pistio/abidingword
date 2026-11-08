import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { useSaveTracker } from "@/hooks/SaveTracker";
import { useApi } from "@/lib/ApiContext";
import { toApiError } from "@/lib/api";
import type { StudySection } from "@/types/domain";
import canonical from "../../../resources/templates/pozetek-writing-study.json";
import { sectionTitle } from "./builtinText";

export function PozetekRepair({ studyId, onChanged, hasOpenDrafts = false }: { studyId: string; onChanged: (sections: StudySection[]) => void; hasOpenDrafts?: boolean }) {
  const { t } = useTranslation();
  const root = useRef<HTMLDivElement>(null);
  const draftState = useRef(hasOpenDrafts);
  useEffect(() => { draftState.current = hasOpenDrafts; }, [hasOpenDrafts]);
  const api = useApi();
  const { settled } = useSaveTracker();
  const [preview, setPreview] = useState<StudySection[] | null>(null);
  const [canUndo, setCanUndo] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    let active = true;
    api.studyPozetekRepair(studyId, "preview").then(view => { if (active) setCanUndo(view.canUndo); }, e => { if (active) setError(toApiError(e).message); });
    return () => { active = false; };
  }, [api, studyId]);
  const hasDrafts = () => draftState.current || root.current?.closest("article")?.querySelector("[data-part-editing], [data-capture]");
  const run = async (action: "preview" | "apply" | "undo") => {
    // A note draft or unsubmitted part name exists only in the editor, so the
    // database receipt cannot detect it. Keep it visible until saved/closed.
    if (action === "undo" && hasDrafts()) {
      setError(t("studies.repair.finishDrafts")); return;
    }
    setBusy(true); setError(null);
    try {
      await settled();
      if (action === "undo" && hasDrafts()) throw new Error(t("studies.repair.finishDrafts"));
      const view = await api.studyPozetekRepair(studyId, action, action === "apply" ? preview?.map(s => s.id) : undefined);
      setCanUndo(view.canUndo);
      if (action === "preview") setPreview(view.sections);
      else { onChanged(view.sections); setPreview(null); }
    } catch (e) { setError(toApiError(e).message); }
    finally { setBusy(false); }
  };
  const counts = new Map<string, number>();
  for (const part of preview ?? []) if (part.key) counts.set(part.key, (counts.get(part.key) ?? 0) + 1);
  const duplicates = [...counts].filter(([, count]) => count > 1).map(([key, count]) => `${key} (${count})`).join(", ");
  return <div ref={root} className="basis-full flex flex-wrap items-center gap-2">
    <Button disabled={busy} variant="quiet" onClick={() => void run("preview")}>{t("studies.repair.review")}</Button>
    {canUndo && <Button disabled={busy || hasOpenDrafts} variant="quiet" onClick={() => void run("undo")}>{t("studies.repair.undo")}</Button>}
    {canUndo && hasOpenDrafts && <p className="type-meta">{t("studies.repair.finishDrafts")}</p>}
    {preview && <Dialog wide title={t("studies.repair.review")} onClose={() => { if (!busy) setPreview(null); }} actions={<>
      <Button disabled={busy} onClick={() => setPreview(null)}>{t("common.cancel")}</Button>
      <Button disabled={busy || canUndo} variant="primary" onClick={() => void run("apply")}>{t("studies.repair.apply")}</Button>
    </>}>
      <p>{t("studies.repair.explain")}</p>
      <ol>{canonical.sections.filter(s => !s.optional).map(s => <li key={s.key}>
        {sectionTitle(s, canonical.key)} — {t(counts.has(s.key) ? "studies.repair.keep" : "studies.repair.add")}
      </li>)}</ol>
      {duplicates && <p role="status">{t("studies.repair.duplicates", { parts: duplicates })}</p>}
      <p>{t("studies.repair.safety")}</p>
      {canUndo && <p>{t("studies.repair.undoFirst")}</p>}
      {error && <p role="alert">{error}</p>}
    </Dialog>}
    {error && !preview && <p role="alert">{error}</p>}
  </div>;
}
