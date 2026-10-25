import { useCallback, useEffect, useId, useMemo, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, Plus } from "lucide-react";
import { ActionMenu } from "@/design-system/ActionMenu";
import { Button } from "@/design-system/Button";
import { ConfirmDialog } from "@/design-system/ConfirmDialog";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { formatDate, formatNumber } from "@/i18n/format";
import { BatchImport } from "@/features/library/BatchImport";
import { useLibraryNav } from "@/features/library/LibraryNav";
import { formatLabel, sourceTypeLabel } from "@/features/library/labels";
import {
  LICENCE_STATUSES,
  SOURCE_TYPES,
  TRASH_RETENTION_DAYS,
  type ItemState,
  type ManagedSource,
  type ManagePage,
  type ManageSort,
  type Source,
  type SourceCollection,
  type SourceFormat,
  type SourceType,
} from "@/types/library";

const PAGE = 50;
const FORMATS = ["pdf", "epub", "docx", "markdown", "text"] as const;
const SORTS: ManageSort[] = ["added", "title", "author", "category", "language"];

/** A collection tree, flattened with each one's depth, for lists and choices. */
function flatten(collections: SourceCollection[]): (SourceCollection & { depth: number })[] {
  const out: (SourceCollection & { depth: number })[] = [];
  const walk = (parent: string | null, depth: number) => {
    for (const c of collections.filter((x) => x.parentId === parent).sort((a, b) => a.name.localeCompare(b.name))) {
      out.push({ ...c, depth });
      if (depth < 20) walk(c.id, depth + 1);
    }
  };
  walk(null, 0);
  return out;
}

const indent = (depth: number) => " ".repeat(depth);

/**
 * Settings → Sources (v0.24): every source in the library, found by words,
 * category, language, format, collection or tag; their details; collections,
 * tags; archive, Trash and deleting for good, one at a time or many at once.
 * The list comes in pages, so a library of thousands stays quick.
 */
export function SourcesSettings() {
  const { t } = useTranslation();
  const api = useApi();
  const { addSource, openSource } = useLibraryNav();
  // Adding many at once (v0.24), and imports left unfinished.
  const [batch, setBatch] = useState<{ resume: string | undefined } | null>(null);
  const [jobsVersion, setJobsVersion] = useState(0);
  const openJobs = useLoad(
    // eslint-disable-next-line react-hooks/exhaustive-deps -- jobsVersion asks again after an import closes.
    useCallback(() => api.libraryBatchJobs(), [api, jobsVersion]),
  );
  const id = useId();
  const [text, setText] = useState("");
  const [typed, setTyped] = useState("");
  const [state, setState] = useState<ItemState | null>(null);
  const [sourceType, setSourceType] = useState<SourceType | "">("");
  const [language, setLanguage] = useState("");
  const [format, setFormat] = useState("");
  const [collectionId, setCollectionId] = useState("");
  const [tagId, setTagId] = useState("");
  const [withoutText, setWithoutText] = useState(false);
  const [duplicates, setDuplicates] = useState(false);
  const [sort, setSort] = useState<ManageSort>("added");
  const [descending, setDescending] = useState(true);
  const [offset, setOffset] = useState(0);
  const [page, setPage] = useState<ManagePage | null>(null);
  const [error, setError] = useState<string | null>(null);
  // What is selected belongs to one view of the list: another search or filter starts with nothing selected.
  const view = JSON.stringify([text, state, sourceType, language, format, collectionId, tagId, withoutText, duplicates]);
  const [selection, setSelection] = useState<{ view: string; ids: ReadonlySet<string> }>({ view, ids: new Set() });
  const selected = selection.view === view ? selection.ids : new Set<string>();
  const setSelected = (next: ReadonlySet<string> | ((current: ReadonlySet<string>) => ReadonlySet<string>)) =>
    setSelection((current) => {
      const base = current.view === view ? current.ids : new Set<string>();
      return { view, ids: typeof next === "function" ? next(base) : next };
    });
  const [editing, setEditing] = useState<string | null>(null);
  const [asking, setAsking] = useState<null | { kind: "trash" | "delete" | "collect" | "tag" | "untag"; ids: string[] }>(null);
  const [done, setDone] = useState<{ message: string; undo?: { ids: string[]; to: ItemState } } | null>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const languages = useLoad(useCallback(() => api.libraryLanguages(), [api]));
  const [collections, setCollections] = useState<SourceCollection[]>([]);
  const [tags, setTags] = useState<{ id: string; name: string; count: number }[]>([]);

  const loadOrganisation = useCallback(
    () =>
      Promise.all([api.libraryCollections(), api.libraryTags()]).then(([c, g]) => {
        setCollections(c);
        setTags(g);
      }),
    [api],
  );

  // Words are looked for once typing pauses.
  useEffect(() => {
    const timer = window.setTimeout(() => {
      setText(typed);
      setOffset(0);
    }, 250);
    return () => window.clearTimeout(timer);
  }, [typed]);

  const load = useCallback(
    () =>
      api
        .libraryManageList({
        text: text || null,
        state,
        sourceType: sourceType || null,
        language: language || null,
        format: format || null,
        collectionId: collectionId || null,
        tagId: tagId || null,
        withoutText,
        duplicates,
        sort,
        descending,
          offset,
          limit: PAGE,
        })
        .then(
          (found) => {
            setPage(found);
            setError(null);
          },
          (e) => setError(toApiError(e).message),
        ),
    [api, text, state, sourceType, language, format, collectionId, tagId, withoutText, duplicates, sort, descending, offset],
  );

  useEffect(() => {
    void load();
  }, [load]);
  useEffect(() => {
    void loadOrganisation().catch(() => undefined);
  }, [loadOrganisation]);

  const refresh = async () => {
    await Promise.all([load(), loadOrganisation()]);
  };
  const flat = useMemo(() => flatten(collections), [collections]);
  const items = page?.items ?? [];
  const ids = [...selected];
  const allOnPage = items.length > 0 && items.every((s) => selected.has(s.id));
  const toggle = (sourceId: string) =>
    setSelected((current) => {
      const next = new Set(current);
      if (next.has(sourceId)) next.delete(sourceId);
      else next.add(sourceId);
      return next;
    });

  const move = async (targets: string[], to: ItemState, message: string, undo?: ItemState) => {
    setError(null);
    try {
      await api.librarySetState(targets, to);
      setSelected(new Set());
      await refresh();
      setDone({ message, undo: undo ? { ids: targets, to: undo } : undefined });
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  const count = (n: number) => ({ count: n, formatted: formatNumber(n) });
  const titleOf = (sourceId: string) => items.find((s) => s.id === sourceId)?.title ?? "";
  const viewing: ItemState = state ?? "active";

  const filtersOn = [sourceType, language, format, collectionId, tagId].some(Boolean) || withoutText || duplicates;

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <p className="m-0 text-text-muted">
          {page ? t("library.manage.counts", { active: formatNumber(page.active), archived: formatNumber(page.archived), trashed: formatNumber(page.trashed) }) : " "}
        </p>
        <div className="flex flex-wrap gap-2">
          <Button onClick={addSource}>
            <Plus className="size-4" aria-hidden="true" />
            {t("library.manage.add")}
          </Button>
          <Button onClick={() => setBatch({ resume: undefined })}>{t("library.batch.open")}</Button>
        </div>
      </div>
      {openJobs.data?.map((job) => (
        <Notice key={job.id}>
          <span className="flex flex-wrap items-center gap-2">
            {t("library.batch.unfinished", { added: formatNumber(job.counts.imported), total: formatNumber(job.counts.total) })}
            <Button variant="quiet" onClick={() => setBatch({ resume: job.id })}>
              {t("library.batch.resume")}
            </Button>
            <Button
              variant="quiet"
              onClick={async () => {
                await api.libraryBatchDiscard(job.id);
                setJobsVersion((v) => v + 1);
              }}
            >
              {t("library.batch.discard")}
            </Button>
          </span>
        </Notice>
      ))}
      {batch && (
        <BatchImport
          resume={batch.resume}
          onClose={(added) => {
            setBatch(null);
            setJobsVersion((v) => v + 1);
            if (added > 0) void refresh();
          }}
        />
      )}

      <div className="flex flex-col gap-3">
        <label htmlFor={`${id}-search`} className="sr-only">
          {t("library.manage.searchLabel")}
        </label>
        <input
          id={`${id}-search`}
          type="search"
          value={typed}
          onChange={(e) => setTyped(e.target.value)}
          placeholder={t("library.manage.searchPlaceholder")}
          className="h-10 aw-input px-3 text-base"
        />
        <div role="radiogroup" aria-label={t("library.manage.showLabel")} className="flex flex-wrap gap-1">
          {([null, "archived", "trashed"] as const).map((value) => {
            const label =
              value === null
                ? t("library.manage.show.current", { formatted: formatNumber((page?.active ?? 0) + (page?.archived ?? 0)) })
                : value === "archived"
                  ? t("library.manage.show.archived", { formatted: formatNumber(page?.archived ?? 0) })
                  : t("library.manage.show.trashed", { formatted: formatNumber(page?.trashed ?? 0) });
            return (
              <button
                key={value ?? "current"}
                type="button"
                role="radio"
                aria-checked={state === value}
                onClick={() => {
                  setState(value);
                  setOffset(0);
                }}
                className={`rounded-aw border px-3 py-1.5 text-[0.9375rem] ${state === value ? "border-primary bg-selected font-medium text-text-strong" : "border-border text-text hover:bg-hover"}`}
              >
                {label}
              </button>
            );
          })}
        </div>
        {state === "trashed" && <p className="type-meta m-0">{t("common.lifecycle.trashText", { count: TRASH_RETENTION_DAYS })}</p>}

        <details open={filtersOn || undefined} className="rounded-aw border border-border-subtle px-3 py-2">
          <summary className="cursor-pointer font-medium text-text-strong">{t("library.manage.filters")}</summary>
          <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
            <Choice label={t("library.manage.filter.category")} value={sourceType} onChange={(v) => (setSourceType(v as SourceType | ""), setOffset(0))}>
              <option value="">{t("library.manage.filter.any")}</option>
              {SOURCE_TYPES.map((x) => (
                <option key={x} value={x}>
                  {sourceTypeLabel(t, x)}
                </option>
              ))}
            </Choice>
            <Choice label={t("library.manage.filter.language")} value={language} onChange={(v) => (setLanguage(v), setOffset(0))}>
              <option value="">{t("library.manage.filter.any")}</option>
              {(languages.data ?? []).map((x) => (
                <option key={x} value={x}>
                  {x}
                </option>
              ))}
            </Choice>
            <Choice label={t("library.manage.filter.format")} value={format} onChange={(v) => (setFormat(v), setOffset(0))}>
              <option value="">{t("library.manage.filter.any")}</option>
              {FORMATS.map((x) => (
                <option key={x} value={x}>
                  {formatLabel(t, x)}
                </option>
              ))}
            </Choice>
            <Choice label={t("library.manage.filter.collection")} value={collectionId} onChange={(v) => (setCollectionId(v), setOffset(0))}>
              <option value="">{t("library.manage.filter.any")}</option>
              {flat.map((c) => (
                <option key={c.id} value={c.id}>
                  {indent(c.depth)}
                  {c.name} ({formatNumber(c.count)})
                </option>
              ))}
            </Choice>
            <Choice label={t("library.manage.filter.tag")} value={tagId} onChange={(v) => (setTagId(v), setOffset(0))}>
              <option value="">{t("library.manage.filter.any")}</option>
              {tags.map((g) => (
                <option key={g.id} value={g.id}>
                  {g.name} ({formatNumber(g.count)})
                </option>
              ))}
            </Choice>
            <div className="flex flex-col justify-end gap-1">
              <label className="flex items-center gap-2">
                <input type="checkbox" checked={withoutText} onChange={(e) => (setWithoutText(e.target.checked), setOffset(0))} />
                {t("library.manage.filter.withoutText")}
              </label>
              <label className="flex items-center gap-2">
                <input type="checkbox" checked={duplicates} onChange={(e) => (setDuplicates(e.target.checked), setOffset(0))} />
                {t("library.manage.filter.duplicates")}
              </label>
            </div>
          </div>
        </details>

        <div className="flex flex-wrap items-end gap-2">
          <Choice label={t("library.manage.sortLabel")} value={sort} onChange={(v) => (setSort(v as ManageSort), setDescending(v === "added"), setOffset(0))}>
            {SORTS.map((x) => (
              <option key={x} value={x}>
                {t(`library.manage.sort.${x}`)}
              </option>
            ))}
          </Choice>
          <Button variant="quiet" onClick={() => (setDescending(!descending), setOffset(0))} aria-label={descending ? t("library.manage.descending") : t("library.manage.ascending")}>
            {descending ? <ArrowDown className="size-4" aria-hidden="true" /> : <ArrowUp className="size-4" aria-hidden="true" />}
          </Button>
        </div>
      </div>

      {selected.size > 0 && (
        <div role="region" aria-label={t("library.manage.selectionLabel")} className="sticky top-14 z-[5] flex flex-wrap items-center gap-2 rounded-aw border border-border bg-raised px-3 py-2">
          <span className="font-medium">{t("library.manage.selected", count(selected.size))}</span>
          {viewing !== "trashed" && (
            <ActionMenu
              label={t("library.manage.organise")}
              trigger={<span>{t("library.manage.organise")}</span>}
              align="start"
              actions={[
                { label: t("library.manage.collect"), onSelect: () => setAsking({ kind: "collect", ids }) },
                { label: t("library.manage.tag"), onSelect: () => setAsking({ kind: "tag", ids }) },
                ...(tags.length > 0 ? [{ label: t("library.manage.untag"), onSelect: () => setAsking({ kind: "untag", ids }) }] : []),
                ...(collectionId
                  ? [
                      {
                        label: t("library.manage.uncollect"),
                        onSelect: async () => {
                          await api.libraryCollect(collectionId, ids, false).catch((e) => setError(toApiError(e).message));
                          setSelected(new Set());
                          await refresh();
                        },
                      },
                    ]
                  : []),
              ]}
            />
          )}
          {viewing === "active" && (
            <Button variant="quiet" onClick={() => move(ids, "archived", t("library.manage.done.archived", count(ids.length)), "active")}>
              {t("common.lifecycle.archive")}
            </Button>
          )}
          {viewing !== "active" && (
            <Button variant="quiet" onClick={() => move(ids, "active", t("library.manage.done.restored", count(ids.length)))}>
              {t("common.lifecycle.restore")}
            </Button>
          )}
          {viewing !== "trashed" ? (
            <Button variant="quiet" className="hover:!text-danger" onClick={() => setAsking({ kind: "trash", ids })}>
              {t("common.lifecycle.moveToTrashEllipsis")}
            </Button>
          ) : (
            <Button variant="quiet" className="hover:!text-danger" onClick={() => setAsking({ kind: "delete", ids })}>
              {t("common.lifecycle.deletePermanently")}
            </Button>
          )}
          <Button variant="quiet" onClick={() => setSelected(new Set())}>
            {t("library.manage.clearSelection")}
          </Button>
        </div>
      )}

      <p role="status" className="m-0 empty:hidden">
        {done && (
          <>
            {done.message}{" "}
            {done.undo && (
              <Button variant="quiet" onClick={() => move(done.undo!.ids, done.undo!.to, t("common.lifecycle.undone"))}>
                {t("common.lifecycle.undo")}
              </Button>
            )}
          </>
        )}
      </p>
      {error && <Notice tone="error">{error}</Notice>}

      <div ref={listRef} tabIndex={-1} className="outline-none">
        {page && items.length === 0 ? (
          <p className="m-0 text-text-muted">{text || filtersOn ? t("library.manage.noMatches") : state === "trashed" ? t("library.manage.trashEmpty") : state === "archived" ? t("library.manage.archiveEmpty") : t("library.manage.empty")}</p>
        ) : (
          <>
            <label className="flex items-center gap-2 border-b border-border-subtle pb-2 text-[0.9375rem]">
              <input
                type="checkbox"
                checked={allOnPage}
                onChange={() =>
                  setSelected((current) => {
                    const next = new Set(current);
                    for (const s of items) {
                      if (allOnPage) next.delete(s.id);
                      else next.add(s.id);
                    }
                    return next;
                  })
                }
              />
              {t("library.manage.selectPage")}
            </label>
            <ul aria-label={t("library.manage.listLabel")} className="m-0 list-none p-0">
              {items.map((s) => (
                <Row
                  key={s.id}
                  source={s}
                  collections={flat}
                  checked={selected.has(s.id)}
                  onToggle={() => toggle(s.id)}
                  open={editing === s.id}
                  onOpen={() => setEditing(editing === s.id ? null : s.id)}
                  onRead={() => openSource(s.id)}
                  onSaved={refresh}
                  onArchive={() => move([s.id], "archived", t("library.manage.done.archivedOne", { title: s.title }), "active")}
                  onRestore={() => move([s.id], "active", t("library.manage.done.restoredOne", { title: s.title }))}
                  onTrash={() => setAsking({ kind: "trash", ids: [s.id] })}
                  onDelete={() => setAsking({ kind: "delete", ids: [s.id] })}
                />
              ))}
            </ul>
          </>
        )}
      </div>

      {page && page.total > PAGE && (
        <nav aria-label={t("library.manage.pages")} className="flex flex-wrap items-center gap-2">
          <Button variant="quiet" disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE))}>
            {t("library.manage.previous")}
          </Button>
          <span>{t("library.manage.range", { from: formatNumber(offset + 1), to: formatNumber(Math.min(offset + PAGE, page.total)), total: formatNumber(page.total) })}</span>
          <Button variant="quiet" disabled={offset + PAGE >= page.total} onClick={() => setOffset(offset + PAGE)}>
            {t("library.manage.next")}
          </Button>
        </nav>
      )}

      <OrganiseSection flat={flat} tags={tags} onChanged={refresh} />

      {asking?.kind === "trash" && (
        <ConfirmDialog
          title={asking.ids.length === 1 ? t("library.trash.question", { title: titleOf(asking.ids[0]!) }) : t("library.manage.trashMany", count(asking.ids.length))}
          confirmLabel={t("common.lifecycle.moveToTrash")}
          returnFocus={() => listRef.current}
          onCancel={() => setAsking(null)}
          onConfirm={async () => {
            const targets = asking.ids;
            await api.librarySetState(targets, "trashed");
            setAsking(null);
            setSelected(new Set());
            await refresh();
            setDone({
              message: targets.length === 1 ? t("library.trash.done", { title: titleOf(targets[0]!) }) : t("library.manage.done.trashed", count(targets.length)),
              undo: { ids: targets, to: viewing },
            });
          }}
        >
          <p className="m-0">{t("library.trash.text", { count: TRASH_RETENTION_DAYS })}</p>
          <p className="m-0">{t("library.trash.kept")}</p>
        </ConfirmDialog>
      )}
      {asking?.kind === "delete" && (
        <ConfirmDialog
          title={asking.ids.length === 1 ? t("library.delete.question", { title: titleOf(asking.ids[0]!) }) : t("library.delete.questionMany", count(asking.ids.length))}
          confirmLabel={t("common.lifecycle.deletePermanently")}
          typeToConfirm={asking.ids.length > 1 ? String(asking.ids.length) : undefined}
          returnFocus={() => listRef.current}
          onCancel={() => setAsking(null)}
          onConfirm={async () => {
            const n = asking.ids.length;
            await api.libraryRemovePermanently(asking.ids);
            setAsking(null);
            setSelected(new Set());
            await refresh();
            setDone({ message: t("library.delete.done", count(n)) });
          }}
        >
          <p className="m-0">{t("library.delete.text")}</p>
          <p className="m-0">{t("library.delete.kept")}</p>
          <p className="m-0 font-medium">{t("common.lifecycle.cannotUndo")}</p>
        </ConfirmDialog>
      )}
      {asking?.kind === "collect" && (
        <CollectDialog
          flat={flat}
          count={asking.ids.length}
          onCancel={() => setAsking(null)}
          onDone={async (collection) => {
            await api.libraryCollect(collection.id, asking.ids, true);
            setAsking(null);
            await refresh();
            setDone({ message: t("library.manage.done.collected", { ...count(asking.ids.length), name: collection.name }) });
          }}
        />
      )}
      {asking?.kind === "tag" && (
        <TagDialog
          tags={tags}
          count={asking.ids.length}
          onCancel={() => setAsking(null)}
          onDone={async (names) => {
            await api.libraryRetag(asking.ids, names, []);
            setAsking(null);
            await refresh();
            setDone({ message: t("library.manage.done.tagged", count(asking.ids.length)) });
          }}
        />
      )}
      {asking?.kind === "untag" && (
        <UntagDialog
          tags={tags}
          count={asking.ids.length}
          onCancel={() => setAsking(null)}
          onDone={async (tag) => {
            await api.libraryRetag(asking.ids, [], [tag]);
            setAsking(null);
            await refresh();
            setDone({ message: t("library.manage.done.untagged", count(asking.ids.length)) });
          }}
        />
      )}
    </div>
  );
}

function Choice({ label, value, onChange, children }: { label: string; value: string; onChange: (value: string) => void; children: ReactNode }) {
  const id = useId();
  return (
    <div className="flex flex-col gap-1">
      <label htmlFor={id} className="text-[0.9375rem] font-medium text-text-strong">
        {label}
      </label>
      <select id={id} value={value} onChange={(e) => onChange(e.target.value)} className="h-10 aw-input px-2">
        {children}
      </select>
    </div>
  );
}

interface RowProps {
  source: ManagedSource;
  collections: (SourceCollection & { depth: number })[];
  checked: boolean;
  onToggle: () => void;
  open: boolean;
  onOpen: () => void;
  onRead: () => void;
  onSaved: () => Promise<void>;
  onArchive: () => void;
  onRestore: () => void;
  onTrash: () => void;
  onDelete: () => void;
}

function Row({ source: s, collections, checked, onToggle, open, onOpen, onRead, onSaved, onArchive, onRestore, onTrash, onDelete }: RowProps) {
  const { t } = useTranslation();
  const meta = [s.author, sourceTypeLabel(t, s.sourceType), s.format ? formatLabel(t, s.format as SourceFormat) : null, s.language, t("library.view.addedOn", { date: formatDate(s.createdAt) })]
    .filter(Boolean)
    .join(t("common.listSeparator"));
  const actions =
    s.state === "trashed"
      ? [
          { label: t("common.lifecycle.restore"), onSelect: onRestore },
          { label: t("common.lifecycle.deletePermanently"), onSelect: onDelete },
        ]
      : [
          { label: t("library.manage.read"), onSelect: onRead },
          s.state === "archived" ? { label: t("common.lifecycle.restore"), onSelect: onRestore } : { label: t("common.lifecycle.archive"), onSelect: onArchive },
          { label: t("common.lifecycle.moveToTrashEllipsis"), onSelect: onTrash },
        ];
  return (
    <li className="flex flex-col gap-2 border-b border-border-subtle py-2.5">
      <div className="flex items-start gap-3">
        <input type="checkbox" checked={checked} onChange={onToggle} aria-label={t("library.manage.selectOne", { title: s.title })} className="mt-1.5" />
        <div className="min-w-0 flex-1">
          <button type="button" aria-expanded={open} onClick={onOpen} className="min-w-0 text-left">
            <span className="type-item block hover:underline">{s.title}</span>
            <span className="type-meta block">{meta}</span>
          </button>
          <div className="mt-1 flex flex-wrap gap-1">
            {s.state === "archived" && <Badge>{t("common.lifecycle.state.archived")}</Badge>}
            {s.state === "trashed" && s.stateChangedAt && <Badge>{t("common.lifecycle.trashedOn", { date: formatDate(s.stateChangedAt) })}</Badge>}
            {s.extractionStatus === "ocr_required" && <Badge>{t("library.manage.badge.noText")}</Badge>}
            {s.duplicates > 0 && <Badge>{t("library.manage.badge.duplicate", { count: s.duplicates, formatted: formatNumber(s.duplicates) })}</Badge>}
            {s.tags.map((g) => (
              <Badge key={g.id}>#{g.name}</Badge>
            ))}
          </div>
        </div>
        <ActionMenu label={t("library.manage.more", { title: s.title })} actions={actions} />
      </div>
      {open && <SourceDetails sourceId={s.id} summary={s} collections={collections} onSaved={onSaved} />}
    </li>
  );
}

function Badge({ children }: { children: ReactNode }) {
  return <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[0.8125rem] text-text-muted">{children}</span>;
}

/** A source's details, editable; its file, tags and collections. Nothing is saved until Save. */
function SourceDetails({ sourceId, summary, collections, onSaved }: { sourceId: string; summary: ManagedSource; collections: (SourceCollection & { depth: number })[]; onSaved: () => Promise<void> }) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const loaded = useLoad(useCallback(() => api.libraryGetSource(sourceId), [api, sourceId]));
  // The loaded details until something is changed; nothing is saved until Save.
  const [edited, setDraft] = useState<Source | null>(null);
  const draft = edited ?? loaded.data;
  const [member, setMember] = useState<ReadonlySet<string>>(new Set(summary.collections));
  const [newTag, setNewTag] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);
  const [busy, setBusy] = useState(false);
  if (loaded.error) return <Notice tone="error">{loaded.error.message}</Notice>;
  if (!draft) return null;
  const set = <K extends keyof Source>(key: K, value: Source[K]) => {
    setSaved(false);
    setDraft({ ...draft, [key]: value });
  };
  const field = (key: "title" | "subtitle" | "author" | "publisher" | "edition" | "isbn" | "doi" | "language" | "licence", label: string) => (
    <div className="flex flex-col gap-1">
      <label htmlFor={`${id}-${key}`} className="text-[0.9375rem] font-medium text-text-strong">
        {label}
      </label>
      <input id={`${id}-${key}`} value={(draft[key] as string | null) ?? ""} onChange={(e) => set(key, (e.target.value || (key === "title" ? "" : null)) as never)} className="h-10 aw-input px-3" />
    </div>
  );

  const save = async () => {
    setBusy(true);
    setError(null);
    try {
      await api.libraryUpdateSource({
        id: draft.id,
        title: draft.title,
        subtitle: draft.subtitle,
        author: draft.author,
        publisher: draft.publisher,
        edition: draft.edition,
        publicationYear: draft.publicationYear,
        isbn: draft.isbn,
        doi: draft.doi,
        language: draft.language,
        sourceType: draft.sourceType,
        licenceStatus: draft.licenceStatus,
        licence: draft.licence,
        notes: draft.notes,
      });
      for (const c of collections) {
        const was = summary.collections.includes(c.id);
        const now = member.has(c.id);
        if (was !== now) await api.libraryCollect(c.id, [draft.id], now);
      }
      if (newTag.trim()) await api.libraryRetag([draft.id], newTag.split(",").map((x) => x.trim()).filter(Boolean), []);
      setNewTag("");
      setSaved(true);
      await onSaved();
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="ml-7 flex flex-col gap-3 rounded-aw border border-border-subtle bg-bg p-3">
      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        {field("title", t("library.manage.field.title"))}
        {field("author", t("library.manage.field.author"))}
        <div className="flex flex-col gap-1">
          <label htmlFor={`${id}-type`} className="text-[0.9375rem] font-medium text-text-strong">
            {t("library.manage.field.category")}
          </label>
          <select id={`${id}-type`} value={draft.sourceType} onChange={(e) => set("sourceType", e.target.value as SourceType)} className="h-10 aw-input px-2">
            {SOURCE_TYPES.map((x) => (
              <option key={x} value={x}>
                {sourceTypeLabel(t, x)}
              </option>
            ))}
          </select>
        </div>
        {field("language", t("library.manage.field.language"))}
      </div>
      <details>
        <summary className="cursor-pointer text-[0.9375rem] font-medium text-text-strong">{t("library.manage.moreDetails")}</summary>
        <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
          {field("subtitle", t("library.manage.field.subtitle"))}
          {field("publisher", t("library.manage.field.publisher"))}
          {field("edition", t("library.manage.field.edition"))}
          <div className="flex flex-col gap-1">
            <label htmlFor={`${id}-year`} className="text-[0.9375rem] font-medium text-text-strong">
              {t("library.manage.field.year")}
            </label>
            <input id={`${id}-year`} inputMode="numeric" value={draft.publicationYear ?? ""} onChange={(e) => set("publicationYear", e.target.value ? Number(e.target.value.replace(/\D/g, "")) : null)} className="h-10 aw-input px-3" />
          </div>
          {field("isbn", t("library.manage.field.isbn"))}
          {field("doi", t("library.manage.field.doi"))}
          <div className="flex flex-col gap-1">
            <label htmlFor={`${id}-licence-status`} className="text-[0.9375rem] font-medium text-text-strong">
              {t("library.manage.field.licenceStatus")}
            </label>
            <select id={`${id}-licence-status`} value={draft.licenceStatus} onChange={(e) => set("licenceStatus", e.target.value as Source["licenceStatus"])} className="h-10 aw-input px-2">
              {LICENCE_STATUSES.map((x) => (
                <option key={x} value={x}>
                  {t(`library.licence.${x}`)}
                </option>
              ))}
            </select>
          </div>
          {field("licence", t("library.manage.field.licence"))}
        </div>
        <dl className="mt-3 grid grid-cols-1 gap-x-4 gap-y-1 text-[0.9375rem] sm:grid-cols-[10rem_1fr]">
          <dt className="font-medium text-text-strong">{t("library.manage.file.name")}</dt>
          <dd className="m-0 break-all">{summary.originalFilename ?? "—"}</dd>
          <dt className="font-medium text-text-strong">{t("library.manage.file.text")}</dt>
          <dd className="m-0">{t(`library.manage.extraction.${summary.extractionStatus === "ocr_required" ? "scanned" : summary.format === "pdf" ? "extracted" : "digital"}`)}</dd>
          {summary.pageCount !== null && (
            <>
              <dt className="font-medium text-text-strong">{t("library.manage.file.pages")}</dt>
              <dd className="m-0">{formatNumber(summary.pageCount)}</dd>
            </>
          )}
          {summary.duplicates > 0 && (
            <>
              <dt className="font-medium text-text-strong">{t("library.manage.file.duplicates")}</dt>
              <dd className="m-0">{t("library.manage.file.duplicatesText", { count: summary.duplicates, formatted: formatNumber(summary.duplicates) })}</dd>
            </>
          )}
        </dl>
      </details>

      {collections.length > 0 && (
        <fieldset className="m-0 flex flex-col gap-1 border-0 p-0">
          <legend className="mb-1 text-[0.9375rem] font-medium text-text-strong">{t("library.manage.field.collections")}</legend>
          {collections.map((c) => (
            <label key={c.id} className="flex items-center gap-2">
              <span aria-hidden="true">{indent(c.depth)}</span>
              <input
                type="checkbox"
                checked={member.has(c.id)}
                onChange={(e) => {
                  setSaved(false);
                  setMember((current) => {
                    const next = new Set(current);
                    if (e.target.checked) next.add(c.id);
                    else next.delete(c.id);
                    return next;
                  });
                }}
              />
              {c.name}
            </label>
          ))}
        </fieldset>
      )}
      <div className="flex flex-col gap-1">
        <span className="text-[0.9375rem] font-medium text-text-strong">{t("library.manage.field.tags")}</span>
        <div className="flex flex-wrap gap-1">
          {summary.tags.map((g) => (
            <span key={g.id} className="flex items-center gap-1 rounded-full border border-border-subtle px-2 py-0.5 text-[0.875rem]">
              #{g.name}
              <button
                type="button"
                aria-label={t("library.manage.removeTag", { name: g.name })}
                className="text-text-muted hover:text-danger"
                onClick={async () => {
                  await api.libraryRetag([draft.id], [], [g.id]).catch((e) => setError(toApiError(e).message));
                  await onSaved();
                }}
              >
                ×
              </button>
            </span>
          ))}
        </div>
        <label htmlFor={`${id}-tags`} className="sr-only">
          {t("library.manage.addTags")}
        </label>
        <input id={`${id}-tags`} value={newTag} onChange={(e) => (setNewTag(e.target.value), setSaved(false))} placeholder={t("library.manage.addTagsPlaceholder")} className="h-10 aw-input px-3" />
      </div>

      {error && <Notice tone="error">{error}</Notice>}
      <div className="flex flex-wrap items-center gap-2">
        <Button variant="primary" onClick={save} disabled={busy || !draft.title.trim()}>
          {t("library.manage.save")}
        </Button>
        {saved && <span role="status">{t("library.manage.saved")}</span>}
      </div>
    </div>
  );
}

/** Put the chosen sources in a collection: one that exists, or a new one. */
function CollectDialog({ flat, count, onCancel, onDone }: { flat: (SourceCollection & { depth: number })[]; count: number; onCancel: () => void; onDone: (collection: SourceCollection) => Promise<void> }) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const [choice, setChoice] = useState(flat[0]?.id ?? "new");
  const [name, setName] = useState("");
  const [parent, setParent] = useState("");
  const [error, setError] = useState<string | null>(null);
  const go = async () => {
    setError(null);
    try {
      const collection = choice === "new" ? await api.libraryCollectionCreate(name, parent || null) : flat.find((c) => c.id === choice)!;
      await onDone(collection);
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  return (
    <Dialog
      title={t("library.manage.collectTitle", { count, formatted: formatNumber(count) })}
      onClose={onCancel}
      actions={
        <>
          <Button onClick={onCancel}>{t("common.cancel")}</Button>
          <Button variant="primary" onClick={go} disabled={choice === "new" && !name.trim()}>
            {t("library.manage.collectConfirm")}
          </Button>
        </>
      }
    >
      <label htmlFor={`${id}-choice`} className="font-medium text-text-strong">
        {t("library.manage.filter.collection")}
      </label>
      <select id={`${id}-choice`} value={choice} onChange={(e) => setChoice(e.target.value)} className="h-10 aw-input px-2">
        {flat.map((c) => (
          <option key={c.id} value={c.id}>
            {indent(c.depth)}
            {c.name}
          </option>
        ))}
        <option value="new">{t("library.manage.newCollection")}</option>
      </select>
      {choice === "new" && (
        <>
          <label htmlFor={`${id}-name`} className="font-medium text-text-strong">
            {t("library.manage.collectionName")}
          </label>
          <input id={`${id}-name`} value={name} onChange={(e) => setName(e.target.value)} className="h-10 aw-input px-3" />
          {flat.length > 0 && (
            <>
              <label htmlFor={`${id}-parent`} className="font-medium text-text-strong">
                {t("library.manage.inside")}
              </label>
              <select id={`${id}-parent`} value={parent} onChange={(e) => setParent(e.target.value)} className="h-10 aw-input px-2">
                <option value="">{t("library.manage.topLevel")}</option>
                {flat.map((c) => (
                  <option key={c.id} value={c.id}>
                    {indent(c.depth)}
                    {c.name}
                  </option>
                ))}
              </select>
            </>
          )}
        </>
      )}
      <p className="type-meta m-0">{t("library.manage.collectHint")}</p>
      {error && <Notice tone="error">{error}</Notice>}
    </Dialog>
  );
}

function TagDialog({ tags, count, onCancel, onDone }: { tags: { id: string; name: string }[]; count: number; onCancel: () => void; onDone: (names: string[]) => Promise<void> }) {
  const { t } = useTranslation();
  const id = useId();
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const names = value.split(",").map((x) => x.trim()).filter(Boolean);
  return (
    <Dialog
      title={t("library.manage.tagTitle", { count, formatted: formatNumber(count) })}
      onClose={onCancel}
      actions={
        <>
          <Button onClick={onCancel}>{t("common.cancel")}</Button>
          <Button
            variant="primary"
            disabled={names.length === 0}
            onClick={() => onDone(names).catch((e) => setError(toApiError(e).message))}
          >
            {t("library.manage.tagConfirm")}
          </Button>
        </>
      }
    >
      <label htmlFor={id} className="font-medium text-text-strong">
        {t("library.manage.addTags")}
      </label>
      <input id={id} list={`${id}-known`} value={value} onChange={(e) => setValue(e.target.value)} placeholder={t("library.manage.addTagsPlaceholder")} className="h-10 aw-input px-3" />
      <datalist id={`${id}-known`}>
        {tags.map((g) => (
          <option key={g.id} value={g.name} />
        ))}
      </datalist>
      {error && <Notice tone="error">{error}</Notice>}
    </Dialog>
  );
}

function UntagDialog({ tags, count, onCancel, onDone }: { tags: { id: string; name: string }[]; count: number; onCancel: () => void; onDone: (tagId: string) => Promise<void> }) {
  const { t } = useTranslation();
  const id = useId();
  const [tag, setTag] = useState(tags[0]?.id ?? "");
  const [error, setError] = useState<string | null>(null);
  return (
    <Dialog
      title={t("library.manage.untagTitle", { count, formatted: formatNumber(count) })}
      onClose={onCancel}
      actions={
        <>
          <Button onClick={onCancel}>{t("common.cancel")}</Button>
          <Button variant="primary" disabled={!tag} onClick={() => onDone(tag).catch((e) => setError(toApiError(e).message))}>
            {t("library.manage.untagConfirm")}
          </Button>
        </>
      }
    >
      <label htmlFor={id} className="font-medium text-text-strong">
        {t("library.manage.filter.tag")}
      </label>
      <select id={id} value={tag} onChange={(e) => setTag(e.target.value)} className="h-10 aw-input px-2">
        {tags.map((g) => (
          <option key={g.id} value={g.id}>
            {g.name}
          </option>
        ))}
      </select>
      {error && <Notice tone="error">{error}</Notice>}
    </Dialog>
  );
}

/** Collections and tags themselves: make, rename, move, remove (the sources stay). */
function OrganiseSection({ flat, tags, onChanged }: { flat: (SourceCollection & { depth: number })[]; tags: { id: string; name: string; count: number }[]; onChanged: () => Promise<void> }) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const [name, setName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [removing, setRemoving] = useState<{ kind: "collection" | "tag"; id: string; name: string } | null>(null);
  const run = async (action: () => Promise<unknown>) => {
    setError(null);
    try {
      await action();
      await onChanged();
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  const [renaming, setRenaming] = useState<{ kind: "collection" | "tag"; id: string; value: string } | null>(null);
  const rename = (kind: "collection" | "tag", itemId: string, current: string) => setRenaming({ kind, id: itemId, value: current });
  const saveName = () => {
    if (!renaming || !renaming.value.trim()) return;
    const target = renaming;
    void run(async () => {
      await (target.kind === "collection" ? api.libraryCollectionRename(target.id, target.value) : api.libraryTagRename(target.id, target.value));
      setRenaming(null);
    });
  };
  const nameField = (kind: "collection" | "tag", itemId: string, shown: ReactNode) =>
    renaming?.kind === kind && renaming.id === itemId ? (
      <form
        className="flex items-center gap-1"
        onSubmit={(e) => {
          e.preventDefault();
          saveName();
        }}
      >
        <label htmlFor={`${id}-rename`} className="sr-only">
          {t("library.manage.newName")}
        </label>
        <input
          id={`${id}-rename`}
          autoFocus
          value={renaming.value}
          onChange={(e) => setRenaming({ ...renaming, value: e.target.value })}
          onKeyDown={(e) => e.key === "Escape" && (e.stopPropagation(), setRenaming(null))}
          className="h-9 aw-input px-2"
        />
        <Button type="submit">{t("library.manage.save")}</Button>
        <Button variant="quiet" onClick={() => setRenaming(null)}>
          {t("common.cancel")}
        </Button>
      </form>
    ) : (
      shown
    );
  return (
    <details className="border-t border-border-subtle pt-3">
      <summary className="cursor-pointer font-medium text-text-strong">{t("library.manage.organiseTitle")}</summary>
      <div className="mt-3 flex flex-col gap-4">
        <section aria-labelledby={`${id}-collections`} className="flex flex-col gap-2">
          <h3 id={`${id}-collections`} className="type-subhead m-0">
            {t("library.manage.collectionsTitle")}
          </h3>
          <p className="type-meta m-0">{t("library.manage.collectionsText")}</p>
          <ul className="m-0 flex list-none flex-col p-0">
            {flat.map((c) => (
              <li key={c.id} className="flex flex-wrap items-center justify-between gap-2 border-b border-border-subtle py-1.5">
                {nameField(
                  "collection",
                  c.id,
                  <span>
                    {indent(c.depth)}
                    {c.name} <span className="text-text-muted">({formatNumber(c.count)})</span>
                  </span>,
                )}
                <span className="flex items-center gap-1">
                  <label className="sr-only" htmlFor={`${id}-move-${c.id}`}>
                    {t("library.manage.moveCollection", { name: c.name })}
                  </label>
                  <select
                    id={`${id}-move-${c.id}`}
                    value={c.parentId ?? ""}
                    onChange={(e) => void run(() => api.libraryCollectionMove(c.id, e.target.value || null))}
                    className="h-9 aw-input px-2 text-[0.875rem]"
                  >
                    <option value="">{t("library.manage.topLevel")}</option>
                    {flat
                      .filter((o) => o.id !== c.id)
                      .map((o) => (
                        <option key={o.id} value={o.id}>
                          {indent(o.depth)}
                          {o.name}
                        </option>
                      ))}
                  </select>
                  <Button variant="quiet" onClick={() => rename("collection", c.id, c.name)}>
                    {t("library.manage.rename")}
                  </Button>
                  <Button variant="quiet" onClick={() => setRemoving({ kind: "collection", id: c.id, name: c.name })}>
                    {t("library.manage.remove")}
                  </Button>
                </span>
              </li>
            ))}
          </ul>
          <form
            className="flex flex-wrap items-end gap-2"
            onSubmit={(e) => {
              e.preventDefault();
              if (name.trim()) void run(async () => (await api.libraryCollectionCreate(name, null), setName("")));
            }}
          >
            <label htmlFor={`${id}-new`} className="flex flex-col gap-1">
              <span className="text-[0.9375rem] font-medium text-text-strong">{t("library.manage.newCollection")}</span>
              <input id={`${id}-new`} value={name} onChange={(e) => setName(e.target.value)} className="h-10 aw-input px-3" />
            </label>
            <Button type="submit" disabled={!name.trim()}>
              {t("library.manage.create")}
            </Button>
          </form>
        </section>
        <section aria-labelledby={`${id}-tags`} className="flex flex-col gap-2">
          <h3 id={`${id}-tags`} className="type-subhead m-0">
            {t("library.manage.tagsTitle")}
          </h3>
          {tags.length === 0 ? (
            <p className="type-meta m-0">{t("library.manage.noTags")}</p>
          ) : (
            <ul className="m-0 flex list-none flex-col p-0">
              {tags.map((g) => (
                <li key={g.id} className="flex flex-wrap items-center justify-between gap-2 border-b border-border-subtle py-1.5">
                  {nameField(
                    "tag",
                    g.id,
                    <span>
                      #{g.name} <span className="text-text-muted">({formatNumber(g.count)})</span>
                    </span>,
                  )}
                  <span className="flex gap-1">
                    <Button variant="quiet" onClick={() => rename("tag", g.id, g.name)}>
                      {t("library.manage.rename")}
                    </Button>
                    <Button variant="quiet" onClick={() => setRemoving({ kind: "tag", id: g.id, name: g.name })}>
                      {t("library.manage.remove")}
                    </Button>
                  </span>
                </li>
              ))}
            </ul>
          )}
        </section>
        {error && <Notice tone="error">{error}</Notice>}
      </div>
      {removing && (
        <ConfirmDialog
          title={removing.kind === "collection" ? t("library.manage.removeCollectionQuestion", { name: removing.name }) : t("library.manage.removeTagQuestion", { name: removing.name })}
          confirmLabel={t("library.manage.remove")}
          onCancel={() => setRemoving(null)}
          onConfirm={async () => {
            const target = removing;
            if (target.kind === "collection") await api.libraryCollectionRemove(target.id);
            else await api.libraryTagRemove(target.id);
            setRemoving(null);
            await onChanged();
          }}
        >
          <p className="m-0">{removing.kind === "collection" ? t("library.manage.removeCollectionText") : t("library.manage.removeTagText")}</p>
        </ConfirmDialog>
      )}
    </details>
  );
}

