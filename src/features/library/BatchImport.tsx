import { useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { formatNumber } from "@/i18n/format";
import { localizeMessage } from "@/i18n/messages";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { SOURCE_TYPES, type ImportItemEdit, type ImportJob, type ImportJobItem, type SourceCollection, type SourceType } from "@/types/library";
import { sourceTypeLabel } from "./labels";

type Phase = "choose" | "scanning" | "reading" | "review" | "adding" | "done";

/**
 * Adding many sources at once (v0.24): choose files or a folder, let
 * Abiding Word read and check them one at a time, review and correct the
 * list on one screen, then add the chosen ones. Every step can be stopped;
 * what was added stays. An unfinished import (`resume`) continues here.
 */
export function BatchImport({ resume, onClose }: { resume?: string; onClose: (added: number) => void }) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const [phase, setPhase] = useState<Phase>(resume ? "review" : "choose");
  const [job, setJob] = useState<ImportJob | null>(null);
  const [recursive, setRecursive] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [collections, setCollections] = useState<SourceCollection[]>([]);
  const [collectionChoice, setCollectionChoice] = useState("");
  const [newCollection, setNewCollection] = useState("");
  const [tags, setTags] = useState("");
  const stop = useRef(false);
  // One loop at a time; corrections still being saved are awaited before adding.
  const [working, setWorking] = useState(false);
  const running = useRef(false);
  const edits = useRef(new Set<Promise<unknown>>());
  // Closing the dialog stops any loop at the next file; the job stays (it can be continued).
  useEffect(
    () => () => {
      stop.current = true;
    },
    [],
  );
  const exclusive = async (work: () => Promise<void>) => {
    if (running.current) return;
    running.current = true;
    setWorking(true);
    try {
      await work();
    } finally {
      running.current = false;
      setWorking(false);
    }
  };

  useEffect(() => {
    let active = true;
    api.libraryCollections().then(
      (c) => active && setCollections(c),
      () => undefined,
    );
    if (resume) {
      api.libraryBatchGet(resume).then(
        (j) => {
          if (!active) return;
          setJob(j);
          setTags(j.tags.join(", "));
          setCollectionChoice(j.collectionId ?? "");
          setPhase(j.counts.waiting > 0 ? "review" : j.state === "finished" ? "done" : "review");
        },
        (e) => active && setError(toApiError(e).message),
      );
    }
    return () => {
      active = false;
    };
  }, [api, resume]);

  const read = async (start: ImportJob) => {
    setPhase("reading");
    let current = start;
    setJob(current);
    try {
      while (current.counts.waiting > 0 && !stop.current) {
        current = await api.libraryBatchStep(current.id);
        setJob(current);
      }
    } catch (e) {
      setError(toApiError(e).message);
    }
    setPhase("review");
  };

  const begin = (paths: string[] | null) =>
    exclusive(async () => {
      if (!paths || paths.length === 0) return;
      setError(null);
      stop.current = false;
      setPhase("scanning");
      try {
        const found = await api.libraryBatchScan(paths, recursive);
        if (stop.current) {
          setJob(found);
          setPhase("review");
          return;
        }
        await read(found);
      } catch (e) {
        setError(toApiError(e).message);
        setPhase("choose");
      }
    });

  const add = () =>
    exclusive(async () => {
    if (!job) return;
    setError(null);
    stop.current = false;
    try {
      await Promise.all([...edits.current]);
      let current = await api.libraryBatchShared(job.id, {
        collectionId: collectionChoice && collectionChoice !== "new" ? collectionChoice : null,
        newCollection: collectionChoice === "new" ? newCollection : null,
        tags: tags
          .split(",")
          .map((x) => x.trim())
          .filter(Boolean),
      });
      setPhase("adding");
      setJob(current);
      while (current.state !== "finished" && current.state !== "cancelled") {
        if (stop.current) {
          current = await api.libraryBatchCancel(current.id);
          break;
        }
        current = await api.libraryBatchImportStep(current.id);
        setJob(current);
      }
      setJob(current);
      setPhase("done");
    } catch (e) {
      setError(toApiError(e).message);
      setPhase("review");
    }
    });

  const retry = () =>
    exclusive(async () => {
      if (!job) return;
      const failed = job.items.filter((i) => i.status === "failed" || i.status === "problem").map((i) => i.id);
      stop.current = false;
      try {
        await read(await api.libraryBatchRetry(job.id, failed));
      } catch (e) {
        setError(toApiError(e).message);
      }
    });

  const continueReading = () =>
    exclusive(async () => {
      if (!job) return;
      stop.current = false;
      await read(job);
    });

  // Closing keeps the job: it is offered again in Settings → Sources.
  const close = () => onClose(job?.counts.imported ?? 0);
  const discard = async () => {
    if (job) await api.libraryBatchDiscard(job.id).catch(() => undefined);
    onClose(job?.counts.imported ?? 0);
  };

  const updateItem = (item: ImportJobItem) =>
    setJob((current) =>
      current
        ? {
            ...current,
            items: current.items.map((i) => (i.id === item.id ? item : i)),
            counts: recount(current, item),
          }
        : current,
    );

  const edit = async (itemId: string, change: Omit<ImportItemEdit, "id">) => {
    if (!job) return;
    const saving = api.libraryBatchEdit(job.id, { ...change, id: itemId }).then(updateItem, (e) => setError(toApiError(e).message));
    edits.current.add(saving);
    await saving;
    edits.current.delete(saving);
  };

  const n = (value: number) => ({
    count: value,
    formatted: formatNumber(value),
  });
  const counts = job?.counts;
  const read_ = counts ? counts.total - counts.waiting : 0;

  const actions =
    phase === "choose" ? (
      <Button onClick={() => onClose(0)}>{t("common.cancel")}</Button>
    ) : phase === "scanning" || phase === "reading" || phase === "adding" ? (
      <Button onClick={() => (stop.current = true)}>{t("library.batch.stop")}</Button>
    ) : phase === "review" ? (
      <>
        <Button variant="quiet" onClick={discard} disabled={working}>
          {t("library.batch.discardImport")}
        </Button>
        <Button onClick={close}>{t("common.close")}</Button>
        {counts && counts.waiting > 0 ? (
          <Button variant="primary" onClick={continueReading} disabled={working}>
            {t("library.batch.continueReading")}
          </Button>
        ) : (
          <Button variant="primary" onClick={add} disabled={working || !counts || counts.toAdd === 0}>
            {t("library.batch.add", n(counts?.toAdd ?? 0))}
          </Button>
        )}
      </>
    ) : (
      <>
        {counts && counts.failed + counts.problem > 0 && job?.state !== "cancelled" && (
          <Button onClick={retry} disabled={working}>
            {t("library.batch.retry")}
          </Button>
        )}
        <Button variant="primary" onClick={() => onClose(counts?.imported ?? 0)}>
          {t("common.close")}
        </Button>
      </>
    );

  return (
    <Dialog wide title={t("library.batch.title")} onClose={phase === "scanning" || phase === "reading" || phase === "adding" ? () => (stop.current = true) : close} actions={actions}>
      {error && <Notice tone="error">{error}</Notice>}
      {phase === "scanning" && (
        <p role="status" className="m-0">
          {t("library.batch.scanning")}
        </p>
      )}
      {phase === "choose" && (
        <div className="flex flex-col gap-3">
          <p className="m-0">{t("library.batch.intro")}</p>
          <div className="flex flex-wrap gap-2">
            <Button disabled={working} onClick={async () => begin(await api.chooseSourceFiles())}>
              {t("library.batch.chooseFiles")}
            </Button>
            <Button disabled={working} onClick={async () => begin(await api.chooseSourceFolder().then((f) => (f ? [f] : null)))}>
              {t("library.batch.chooseFolder")}
            </Button>
          </div>
          <label className="flex items-center gap-2">
            <input type="checkbox" checked={recursive} onChange={(e) => setRecursive(e.target.checked)} />
            {t("library.batch.recursive")}
          </label>
          <p className="m-0 type-meta">{t("library.batch.limits", { max: formatNumber(2000) })}</p>
        </div>
      )}

      {(phase === "reading" || phase === "adding") && counts && (
        <div className="flex flex-col gap-2">
          <p role="status" className="m-0">
            {phase === "reading"
              ? t("library.batch.reading", {
                  done: formatNumber(read_),
                  total: formatNumber(counts.total),
                })
              : t("library.batch.adding", {
                  done: formatNumber(counts.imported),
                  total: formatNumber(counts.imported + counts.toAdd),
                })}
          </p>
          <progress
            className="w-full"
            max={phase === "reading" ? counts.total : counts.imported + counts.toAdd}
            value={phase === "reading" ? read_ : counts.imported}
            aria-label={t("library.batch.progress")}
          />
        </div>
      )}

      {phase === "review" && job && counts && (
        <div className="flex flex-col gap-4">
          {job.state === "interrupted" && (
            <Notice>
              {t("library.batch.interrupted", {
                added: formatNumber(counts.imported),
              })}
            </Notice>
          )}
          <Summary job={job} />
          <fieldset className="m-0 flex flex-col gap-2 border-0 p-0">
            <legend className="mb-1 font-medium">{t("library.batch.shared")}</legend>
            <label htmlFor={`${id}-collection`} className="flex flex-col gap-1 text-[0.9375rem]">
              <span>{t("library.batch.collection")}</span>
              <select id={`${id}-collection`} value={collectionChoice} onChange={(e) => setCollectionChoice(e.target.value)} className="h-9 aw-input px-2">
                <option value="">{t("library.batch.noCollection")}</option>
                {collections.map((c) => (
                  <option key={c.id} value={c.id}>
                    {c.name}
                  </option>
                ))}
                <option value="new">{t("library.manage.newCollection")}</option>
              </select>
            </label>
            {collectionChoice === "new" && (
              <label htmlFor={`${id}-new`} className="flex flex-col gap-1 text-[0.9375rem]">
                <span>{t("library.manage.collectionName")}</span>
                <input id={`${id}-new`} value={newCollection} onChange={(e) => setNewCollection(e.target.value)} className="h-9 aw-input px-3" />
              </label>
            )}
            <label htmlFor={`${id}-tags`} className="flex flex-col gap-1 text-[0.9375rem]">
              <span>{t("library.manage.addTags")}</span>
              <input id={`${id}-tags`} value={tags} onChange={(e) => setTags(e.target.value)} placeholder={t("library.manage.addTagsPlaceholder")} className="h-9 aw-input px-3" />
            </label>
          </fieldset>
          <ul aria-label={t("library.batch.files")} className="m-0 flex list-none flex-col gap-2 p-0">
            {job.items.map((item) => (
              <ItemRow key={item.id} item={item} onEdit={(change) => edit(item.id, change)} />
            ))}
          </ul>
        </div>
      )}

      {phase === "done" && job && counts && (
        <div className="flex flex-col gap-3">
          <p role="status" className="m-0">
            {job.state === "cancelled" ? t("library.batch.stopped", n(counts.imported)) : t("library.batch.done", n(counts.imported))}
          </p>
          {counts.failed > 0 && (
            <ul aria-label={t("library.batch.failedTitle")} className="m-0 flex list-disc flex-col gap-1 pl-5">
              {job.items
                .filter((i) => i.status === "failed")
                .map((i) => (
                  <li key={i.id}>
                    {i.fileName}: {problemText(i)}
                  </li>
                ))}
            </ul>
          )}
        </div>
      )}
    </Dialog>
  );
}

function recount(job: ImportJob, item: ImportJobItem): ImportJob["counts"] {
  const items = job.items.map((i) => (i.id === item.id ? item : i));
  return {
    ...job.counts,
    toAdd: items.filter((i) => i.chosen && (i.status === "ready" || i.status === "duplicate")).length,
  };
}

function problemText(item: ImportJobItem): string {
  return localizeMessage(item.problemCode, item.problemParams ?? undefined, item.problem ?? "");
}

function Summary({ job }: { job: ImportJob }) {
  const { t } = useTranslation();
  const c = job.counts;
  const n = (value: number) => ({
    count: value,
    formatted: formatNumber(value),
  });
  const parts = [
    t("library.batch.summary.found", n(c.total)),
    c.duplicate > 0 && t("library.batch.summary.duplicates", n(c.duplicate)),
    c.problem > 0 && t("library.batch.summary.problems", n(c.problem)),
    c.imported > 0 && t("library.batch.summary.imported", n(c.imported)),
    job.unsupportedCount > 0 && t("library.batch.summary.unsupported", n(job.unsupportedCount)),
    job.overLimitCount > 0 && t("library.batch.summary.overLimit", n(job.overLimitCount)),
  ].filter(Boolean);
  return <p className="m-0">{parts.join(t("common.listSeparator"))}</p>;
}

function ItemRow({ item, onEdit }: { item: ImportJobItem; onEdit: (change: { title?: string; author?: string; language?: string; sourceType?: SourceType; chosen?: boolean }) => void }) {
  const { t } = useTranslation();
  const id = useId();
  const [title, setTitle] = useState(item.title ?? "");
  const [author, setAuthor] = useState(item.author ?? "");
  const choosable = item.status === "ready" || item.status === "duplicate";
  const name = item.title ?? item.fileName;
  return (
    <li className="flex flex-col gap-2 rounded-aw border border-border-subtle p-3">
      <div className="flex flex-wrap items-center gap-2">
        {choosable ? (
          <label className="flex items-center gap-2 font-medium">
            <input type="checkbox" checked={item.chosen} onChange={(e) => onEdit({ chosen: e.target.checked })} aria-label={t("library.batch.choose", { name })} />
            <span>{item.fileName}</span>
          </label>
        ) : (
          <span className="font-medium">{item.fileName}</span>
        )}
        <span className="type-meta">{t(`library.batch.status.${item.status}`)}</span>
      </div>
      {item.status === "duplicate" && (
        <p className="m-0 type-meta">
          {item.duplicateOf
            ? t("library.batch.duplicateLibrary", {
                title: item.duplicateTitle ?? "",
              })
            : t("library.batch.duplicateHere", {
                title: item.duplicateTitle ?? "",
              })}
        </p>
      )}
      {(item.status === "problem" || item.status === "failed") && <p className="m-0 text-[0.9375rem] text-danger">{problemText(item)}</p>}
      {choosable && (
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-3">
          <label htmlFor={`${id}-title`} className="flex flex-col gap-1 text-[0.9375rem]">
            <span>{t("library.manage.field.title")}</span>
            <input id={`${id}-title`} value={title} onChange={(e) => setTitle(e.target.value)} onBlur={() => (title.trim() ? title !== item.title && onEdit({ title }) : setTitle(item.title ?? ""))} className="h-9 aw-input px-3" />
          </label>
          <label htmlFor={`${id}-author`} className="flex flex-col gap-1 text-[0.9375rem]">
            <span>{t("library.manage.field.author")}</span>
            <input id={`${id}-author`} value={author} onChange={(e) => setAuthor(e.target.value)} onBlur={() => author !== (item.author ?? "") && onEdit({ author })} className="h-9 aw-input px-3" />
          </label>
          <div className="flex flex-col gap-1">
            <label htmlFor={`${id}-type`} className="flex flex-col gap-1 text-[0.9375rem]">
              <span>{t("library.manage.field.category")}</span>
              <select id={`${id}-type`} value={item.sourceType ?? "book"} onChange={(e) => onEdit({ sourceType: e.target.value as SourceType })} className="h-9 aw-input px-2">
                {SOURCE_TYPES.map((type) => (
                  <option key={type} value={type}>
                    {sourceTypeLabel(t, type)}
                  </option>
                ))}
              </select>
            </label>
            {item.typeEvidence && <span className="type-meta">{t(`library.batch.evidence.${item.typeEvidence as "title" | "fileName" | "folder" | "fileType" | "person"}`)}</span>}
          </div>
        </div>
      )}
      <details className="type-meta">
        <summary>{t("library.batch.details")}</summary>
        <p className="m-0 break-all">{item.path}</p>
        {item.format && (
          <p className="m-0">
            {[
              item.format.toUpperCase(),
              item.pageCount
                ? t("library.batch.pages", {
                    count: item.pageCount,
                    formatted: formatNumber(item.pageCount),
                  })
                : null,
              item.language,
            ]
              .filter(Boolean)
              .join(" · ")}
          </p>
        )}
      </details>
    </li>
  );
}
