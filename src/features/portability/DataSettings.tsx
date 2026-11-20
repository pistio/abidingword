import { useCallback, useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { ArchiveRestore, HardDriveDownload, Sprout, Upload } from "lucide-react";
import { Button } from "@/design-system/Button";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { runDatabaseRestore, settleOutstandingWrites } from "@/hooks/SaveTracker";
import { formatDate } from "@/i18n/format";
import type { BackupPreview, DailySnapshot, DailySnapshotRestore, ProjectImportReport, ProjectPackagePreview, RestoreReport, SampleReport } from "@/types/portability";
import { LegacyTeaching } from "./LegacyTeaching";
import { Trash } from "./Trash";
import { ProjectExport } from "./ProjectExport";
import { backupCountsLine, countsLine, megabytes } from "./format";

/**
 * Settings › Data and backup (v0.19): four separate actions. Export one
 * project, import one (always as a new project), back up everything, and
 * restore everything from a backup after a clear warning.
 */
export function DataSettings() {
  const { t } = useTranslation();
  const api = useApi();
  const workspace = useLoad(useCallback(() => api.workspaceGetDefault(), [api]));
  const workspaceId = workspace.data?.id ?? null;
  return (
    <div className="flex flex-col gap-8">
      {workspaceId && <><Trash workspaceId={workspaceId} /><LegacyTeaching workspaceId={workspaceId} /></>}
      <div className="flex flex-col gap-4">
        <h3 className="type-group m-0">{t("settings.data.projects")}</h3>
        <p className="m-0 max-w-prose text-[0.9375rem] text-text-muted">{t("settings.data.projectsHint")}</p>
        {workspaceId && <ExportPicker workspaceId={workspaceId} />}
        <ProjectImport />
      </div>
      <div className="flex flex-col gap-4 border-t border-border-subtle pt-6">
        <h3 className="type-group m-0">{t("settings.data.backup")}</h3>
        <p className="m-0 max-w-prose text-[0.9375rem] text-text-muted">{t("settings.data.backupHint")}</p>
        <BackupCreate />
        <BackupRestore />
        <DailyRecovery />
      </div>
      <div className="flex flex-col gap-4 border-t border-border-subtle pt-6">
        <h3 className="type-group m-0">{t("settings.data.sample")}</h3>
        <p className="m-0 max-w-prose text-[0.9375rem] text-text-muted">{t("settings.data.sampleHint")}</p>
        <SampleInstall />
      </div>
    </div>
  );
}

function ExportPicker({ workspaceId }: { workspaceId: string }) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const projects = useLoad(useCallback(() => api.projectList(workspaceId), [api, workspaceId]));
  const [chosen, setChosen] = useState<string>("");
  const list = projects.data ?? [];
  const selected = list.find((p) => p.id === chosen) ?? list[0] ?? null;
  if (list.length === 0) return null;
  return (
    <div className="flex flex-col gap-2">
      <label htmlFor={id} className="text-[0.9375rem] font-medium text-text-muted">
        {t("settings.data.exportLabel")}
      </label>
      <select id={id} value={selected?.id ?? ""} onChange={(e) => setChosen(e.target.value)} className="h-11 max-w-sm aw-input px-2 text-text-strong">
        {list.map((p) => (
          <option key={p.id} value={p.id}>
            {p.title}
          </option>
        ))}
      </select>
      <ProjectExport key={selected?.id} projectId={selected?.id ?? null} title={selected?.title ?? ""} />
    </div>
  );
}

function MissingList({ missing }: { missing: ProjectPackagePreview["missing"] }) {
  const { t } = useTranslation();
  const rows = [
    [t("settings.data.missingSources"), missing.sources],
    [t("settings.data.missingTemplates"), missing.presentationTemplates],
    [t("settings.data.missingBibles"), missing.bibles],
  ] as const;
  const any = rows.some(([, items]) => items.length > 0) || missing.images > 0;
  if (!any) return null;
  return (
    <div className="flex flex-col gap-1">
      <p className="m-0 font-medium text-text-strong">{t("settings.data.missingHeading")}</p>
      <ul className="m-0 pl-5">
        {rows
          .filter(([, items]) => items.length > 0)
          .map(([label, items]) => (
            <li key={label}>
              {label}: {items.join(", ")}
            </li>
          ))}
        {missing.images > 0 && <li>{t("settings.data.missingImages", { n: missing.images })}</li>}
      </ul>
      <p className="m-0 type-meta">{t("settings.data.missingHint")}</p>
    </div>
  );
}

function ProjectImport() {
  const { t } = useTranslation();
  const api = useApi();
  const [picked, setPicked] = useState<{ path: string; preview: ProjectPackagePreview } | null>(null);
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState<ProjectImportReport | null>(null);
  const [error, setError] = useState<string | null>(null);

  const choose = async () => {
    setError(null);
    setDone(null);
    const path = await api.choosePortableFile({ kind: "project", title: t("settings.data.openProject"), filterName: t("settings.data.projectFilter") });
    if (!path) return;
    setBusy(true);
    try {
      setPicked({ path, preview: await api.projectPackageInspect(path) });
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };
  const confirm = async () => {
    if (!picked || busy) return;
    setBusy(true);
    setError(null);
    try {
      setDone(await api.projectImport(picked.path));
      setPicked(null);
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div>
        <Button disabled={busy} onClick={() => void choose()}>
          <Upload className="size-4" aria-hidden="true" />
          {t("settings.data.import")}
        </Button>
      </div>
      {picked && (
        <section aria-label={t("settings.data.importPreview")} className="flex flex-col gap-3 rounded-aw border border-border-subtle bg-raised p-4 text-[0.9375rem]">
          <div>
            <p className="m-0 font-medium text-text-strong">{picked.preview.title}</p>
            {picked.preview.description && <p className="m-0 text-text-muted">{picked.preview.description}</p>}
            <p className="m-0 type-meta">{t("settings.data.savedOn", { date: formatDate(picked.preview.createdAt), version: picked.preview.appVersion })}</p>
            <p className="m-0 type-meta">{countsLine(t, picked.preview.counts)}</p>
          </div>
          <p className="m-0">{t("settings.data.importAs", { title: picked.preview.importTitle })}</p>
          <MissingList missing={picked.preview.missing} />
          <div className="flex flex-wrap gap-2">
            <Button variant="primary" disabled={busy} onClick={() => void confirm()}>
              {busy ? t("settings.data.working") : t("settings.data.importConfirm")}
            </Button>
            <Button disabled={busy} onClick={() => setPicked(null)}>
              {t("common.cancel")}
            </Button>
          </div>
        </section>
      )}
      {done && (
        <Notice tone="success">
          <p className="m-0">{t("settings.data.imported", { title: done.title })}</p>
          <p className="m-0 mt-1 type-meta">{countsLine(t, done.counts)}</p>
          {done.citationsKept > 0 && <p className="m-0 mt-1">{t("settings.data.citationsKept", { n: done.citationsKept })}</p>}
          {done.citationsReconnected > 0 && <p className="m-0 mt-1">{t("settings.data.citationsReconnected", { n: done.citationsReconnected })}</p>}
          {done.leftOut > 0 && <p className="m-0 mt-1">{t("settings.data.leftOut", { n: done.leftOut })}</p>}
        </Notice>
      )}
      {error && <Notice tone="error">{error}</Notice>}
    </div>
  );
}

function BackupCreate() {
  const { t, i18n } = useTranslation();
  const api = useApi();
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState<{ path: string; size: number } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const run = async () => {
    if (busy) return;
    setBusy(true);
    setError(null);
    setDone(null);
    const stamp = new Date().toISOString().slice(0, 10);
    try {
      const path = await api.choosePortableSaveFile({ kind: "backup", defaultName: `Abiding Word ${stamp}.awbackup`, filterName: t("settings.data.backupFilter") });
      if (!path) return;
      await settleOutstandingWrites();
      const [saved, report] = await api.backupCreate(path);
      setDone({ path: saved, size: report.size });
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="flex flex-col gap-3">
      <div>
        <Button disabled={busy} onClick={() => void run()}>
          <HardDriveDownload className="size-4" aria-hidden="true" />
          {busy ? t("settings.data.working") : t("settings.data.backupCreate")}
        </Button>
      </div>
      {busy && <p role="status" className="m-0 type-meta">{t("settings.data.workingLong")}</p>}
      {done && <Notice tone="success">{t("settings.data.backedUp", { path: done.path, size: megabytes(done.size, i18n.language) })}</Notice>}
      {error && <Notice tone="error">{error}</Notice>}
    </div>
  );
}

function BackupRestore() {
  const { t, i18n } = useTranslation();
  const api = useApi();
  const [picked, setPicked] = useState<{ path: string; preview: BackupPreview } | null>(null);
  const [busy, setBusy] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [done, setDone] = useState<RestoreReport | null>(null);
  const [blocked, setBlocked] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const chooseButton = useRef<HTMLButtonElement>(null);
  const warningId = useId();

  const cancel = () => {
    if (busy) return;
    setPicked(null);
    chooseButton.current?.focus();
  };

  const choose = async () => {
    if (busy || done || blocked) return;
    setBusy(true);
    setError(null);
    try {
      const path = await api.choosePortableFile({ kind: "backup", title: t("settings.data.openBackup"), filterName: t("settings.data.backupFilter") });
      if (!path) return;
      setPicked({ path, preview: await api.backupInspect(path) });
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };
  const confirm = async () => {
    if (!picked || busy) return;
    setBusy(true);
    setRestoring(true);
    setError(null);
    try {
      setDone(await runDatabaseRestore(() => api.backupRestore(picked.path)));
      setPicked(null);
    } catch (e) {
      const failure = toApiError(e);
      if (failure.code === "portability.rollbackFailed") setBlocked(failure.message);
      else setError(failure.message);
    } finally {
      setBusy(false);
      setRestoring(false);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div>
        <Button ref={chooseButton} disabled={busy || done !== null} onClick={() => void choose()}>
          <ArchiveRestore className="size-4" aria-hidden="true" />
          {t("settings.data.restore")}
        </Button>
      </div>
      {picked && (
        <section aria-label={t("settings.data.restorePreview")} aria-describedby={warningId}
          className="flex min-w-0 flex-col gap-3 rounded-aw border border-border-subtle bg-raised p-4 text-[0.9375rem]"
          onKeyDown={(event) => {
            if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); cancel(); }
          }}>
          <div>
            <p className="m-0 type-meta">{t("settings.data.savedOn", { date: formatDate(picked.preview.createdAt), version: picked.preview.appVersion })}</p>
            <p className="m-0 type-meta">
              {backupCountsLine(t, picked.preview.counts)} · {t("settings.data.size", { size: megabytes(picked.preview.size, i18n.language) })}
            </p>
          </div>
          <div id={warningId}><Notice tone="warning">{t("settings.data.restoreWarning")}</Notice></div>
          <div className="flex flex-wrap gap-2">
            <Button disabled={busy} autoFocus onClick={cancel}>
              {t("common.cancel")}
            </Button>
            <Button variant="danger" disabled={busy} onClick={() => void confirm()}>
              {busy ? t("settings.data.working") : t("settings.data.restoreConfirm")}
            </Button>
          </div>
          {busy && <p role="status" className="m-0 type-meta">{t("settings.data.workingLong")}</p>}
        </section>
      )}
      {(restoring || done || blocked) && (
        <RestoreStatusDialog
          message={blocked ? t("settings.data.restoreNeedsAttention") : done ? t("settings.data.restored") : t("settings.data.restoreInProgress")}
          safetyCopy={done?.safetyCopy}
          filesSetAside={done?.filesSetAside}
          failureMessage={blocked ?? undefined}
        />
      )}
      {error && <RestoreError message={error} />}
    </div>
  );
}

/** The example project, added only on this explicit request; never twice. */
function SampleInstall() {
  const { t } = useTranslation();
  const api = useApi();
  const [busy, setBusy] = useState(false);
  const [done, setDone] = useState<SampleReport | null>(null);
  const [error, setError] = useState<string | null>(null);
  const run = async () => {
    setBusy(true);
    setError(null);
    try {
      setDone(await api.sampleProjectInstall());
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="flex flex-col gap-3">
      <div>
        <Button disabled={busy} onClick={() => void run()}>
          <Sprout className="size-4" aria-hidden="true" />
          {busy ? t("settings.data.working") : t("settings.data.sampleAdd")}
        </Button>
      </div>
      {done && <Notice tone="success">{t(done.added ? "settings.data.sampleAdded" : "settings.data.sampleThere", { title: done.title })}</Notice>}
      {error && <Notice tone="error">{error}</Notice>}
    </div>
  );
}


/**
 * Daily snapshots are a *database-only* fallback, not a second way to restore
 * source files. Every choice is checked again by Rust, by basename and schema.
 * The destructive action is available only after the explicit preview.
 */
function DailyRecovery() {
  const { t, i18n } = useTranslation();
  const api = useApi();
  const [items, setItems] = useState<DailySnapshot[] | null>(null);
  const [selected, setSelected] = useState<DailySnapshot | null>(null);
  const [busy, setBusy] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [done, setDone] = useState<DailySnapshotRestore | null>(null);
  const [blocked, setBlocked] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const listId = useId();
  const warningId = useId();
  const returnToPoint = useRef<string | null>(null);

  useEffect(() => {
    if (selected || !returnToPoint.current) return;
    document.getElementById(returnToPoint.current)?.focus();
    returnToPoint.current = null;
  }, [selected]);

  const cancel = () => {
    if (busy || !selected) return;
    returnToPoint.current = `${listId}-${selected.id}-choose`;
    setSelected(null);
  };

  const reloadList = async () => {
    if (busy || done) return;
    setBusy(true);
    setError(null);
    try {
      setItems(await api.dailySnapshotList());
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };

  const restore = async () => {
    if (!selected || !selected.usable || busy || done) return;
    setBusy(true);
    setRestoring(true);
    setError(null);
    try {
      const result = await runDatabaseRestore(() => api.dailySnapshotRestore(selected.id));
      setDone(result);
      setSelected(null);
      setItems(null);
    } catch (e) {
      setSelected(null);
      const failure = toApiError(e);
      if (failure.code === "portability.rollbackFailed") setBlocked(failure.message);
      else setError(failure.message);
    } finally {
      setBusy(false);
      setRestoring(false);
    }
  };

  const description = (item: DailySnapshot) => t("settings.data.dailyWhen", {
    date: formatDate(item.createdAt),
    size: megabytes(item.bytes, i18n.language),
    version: item.schemaVersion,
  });

  return (
    <section className="flex min-w-0 flex-col gap-3 border-t border-border-subtle pt-5">
      <h4 className="m-0 font-medium text-text-strong">{t("settings.data.dailyTitle")}</h4>
      <p className="m-0 max-w-prose text-[0.9375rem] text-text-muted">{t("settings.data.dailyHint")}</p>
      <div>
        <Button disabled={busy || done !== null || selected !== null} onClick={() => void reloadList()}>
          {t(items === null ? "settings.data.dailyShow" : "settings.data.dailyRefresh")}
        </Button>
      </div>
      {busy && !restoring && <p role="status" className="m-0 type-meta">{t("settings.data.working")}</p>}
      {items?.length === 0 && <p className="m-0 text-text-muted">{t("settings.data.dailyEmpty")}</p>}
      {items && items.length > 0 && !selected && !done && (
        <ul className="m-0 flex list-none flex-col gap-2 p-0" aria-label={t("settings.data.dailyTitle")}>
          {items.map((item) => (
            <li key={item.id} className="flex flex-wrap items-center justify-between gap-3 rounded-aw border border-border-subtle p-3">
              <div id={`${listId}-${item.id}`} className="flex min-w-0 flex-col gap-1 break-words">
                <span className="text-[0.9375rem] text-text-strong">{description(item)}</span>
                <span className="type-meta text-text-muted">{t(item.usable ? "settings.data.dailyReady" : "settings.data.dailyUnavailable")}</span>
              </div>
              <Button id={`${listId}-${item.id}-choose`} aria-describedby={`${listId}-${item.id}`} disabled={!item.usable || busy} onClick={() => setSelected(item)}>
                {t("settings.data.dailyChoose")}
              </Button>
            </li>
          ))}
        </ul>
      )}
      {selected && (
        <section aria-label={t("settings.data.dailyPreview")} aria-describedby={warningId}
          className="flex min-w-0 flex-col gap-3 rounded-aw border border-border-subtle bg-raised p-4"
          onKeyDown={(event) => {
            if (event.key === "Escape") { event.preventDefault(); event.stopPropagation(); cancel(); }
          }}>
          <p className="m-0 font-medium text-text-strong">{description(selected)}</p>
          <div id={warningId}><Notice tone="warning">{t("settings.data.dailyWarning")}</Notice></div>
          <div className="flex flex-wrap gap-2">
            <Button disabled={busy} autoFocus onClick={cancel}>{t("common.cancel")}</Button>
            <Button variant="danger" disabled={busy} onClick={() => void restore()}>
              {busy ? t("settings.data.working") : t("settings.data.dailyConfirm")}
            </Button>
          </div>
        </section>
      )}
      {(restoring || done || blocked) && <RestoreStatusDialog message={blocked ? t("settings.data.restoreNeedsAttention") : done ? t("settings.data.dailyRestored") : t("settings.data.restoreInProgress")} safetyCopy={done?.safetyCopy} failureMessage={blocked ?? undefined} />}
      {error && <RestoreError message={error} />}
    </section>
  );
}


/** Keep recovery progress and the explicit reload action reachable at narrow
 * widths and large text sizes. The write barrier owns the permanent success
 * latch: unmounting this presentation must never make stale editors writable.
 */
function RestoreStatusDialog({ message, safetyCopy, filesSetAside = 0, failureMessage }: {
  message: string;
  safetyCopy?: string;
  filesSetAside?: number;
  failureMessage?: string;
}) {
  const { t } = useTranslation();
  const titleId = useId();
  const hintId = useId();
  const overlay = useRef<HTMLDivElement>(null);
  const panel = useRef<HTMLElement>(null);
  const reload = useRef<HTMLButtonElement>(null);
  const complete = safetyCopy !== undefined || failureMessage !== undefined;

  useEffect(() => {
    // #root is controlled by the write barrier. Other portals must also stop
    // accepting interaction while recovery is running or awaiting a reload.
    const hidden = new Set<HTMLElement>();
    const freezeBackground = () => {
      for (const element of document.body.children) {
        if (!(element instanceof HTMLElement) || element === overlay.current || element.id === "root" || element.hasAttribute("inert")) continue;
        element.setAttribute("inert", "");
        hidden.add(element);
      }
    };
    freezeBackground();
    const observer = new MutationObserver(freezeBackground);
    observer.observe(document.body, { childList: true });
    const focusHere = () => (reload.current ?? panel.current)?.focus();
    const keepFocus = (event: FocusEvent) => {
      if (event.target instanceof Node && !panel.current?.contains(event.target)) focusHere();
    };
    const keepKeys = (event: KeyboardEvent) => {
      if (event.key !== "Tab" && event.key !== "Escape") return;
      event.preventDefault();
      event.stopImmediatePropagation();
      focusHere();
    };
    document.addEventListener("focusin", keepFocus, true);
    document.addEventListener("keydown", keepKeys, true);
    return () => {
      observer.disconnect();
      document.removeEventListener("focusin", keepFocus, true);
      document.removeEventListener("keydown", keepKeys, true);
      for (const element of hidden) element.removeAttribute("inert");
    };
  }, []);

  useEffect(() => { (reload.current ?? panel.current)?.focus(); }, [complete]);

  return createPortal(
    <div ref={overlay} className="fixed inset-0 z-[200] flex items-center justify-center bg-black/55 p-4">
      <section ref={panel} role="dialog" aria-modal="true" aria-labelledby={titleId} aria-describedby={hintId} tabIndex={-1}
        className="flex max-h-[calc(100dvh-2rem)] w-full min-w-0 max-w-lg flex-col gap-3 overflow-y-auto rounded-aw border border-border-subtle bg-bg p-4 shadow-xl sm:p-6">
        <h3 id={titleId} className="m-0 type-group text-text-strong">{message}</h3>
        <p id={hintId} role={complete ? undefined : "status"} className="m-0 text-[0.9375rem]">
          {t(failureMessage ? "settings.data.restoreFailureReloadHint" : complete ? "settings.data.restoreReloadHint" : "settings.data.restoreInProgressHint")}
        </p>
        {safetyCopy && <p className="m-0 break-words text-[0.9375rem] [overflow-wrap:anywhere]">{t("settings.data.safetyCopy", { path: safetyCopy })}</p>}
        {failureMessage && <div className="min-w-0 [overflow-wrap:anywhere]"><Notice tone="error">{failureMessage}</Notice></div>}
        {filesSetAside > 0 && <p className="m-0">{t("settings.data.setAside", { n: filesSetAside })}</p>}
        {complete && <div className="flex shrink-0 justify-end">
          <Button ref={reload} variant="primary" onClick={() => window.location.reload()}>
            {t("settings.data.restoredContinue")}
          </Button>
        </div>}
      </section>
    </div>,
    document.body,
  );
}

function RestoreError({ message }: { message: string }) {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => { ref.current?.focus(); }, [message]);
  return <div ref={ref} tabIndex={-1}><Notice tone="error">{message}</Notice></div>;
}
