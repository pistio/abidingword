import { useCallback, useEffect, useId, useRef, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { ArrowLeft, Plus, Trash2 } from "lucide-react";
import { Button } from "@/design-system/Button";
import { ConfirmDialog } from "@/design-system/ConfirmDialog";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { highlighted } from "@/lib/highlight";
import { formatDate, formatNumber } from "@/i18n/format";
import { SOURCE_TYPES, TRASH_RETENTION_DAYS, type LibraryResults, type ManagePage, type SourceFormat, type SourceType } from "@/types/library";
import { formatLabel, locatorLabel, sourceTypeLabel } from "./labels";
import { useLibraryNav } from "./LibraryNav";

/** The everyday list pages at the same size as Settings → Sources. */
const PAGE = 50;

type ArchiveFilter = "active" | "archived" | "both";

interface SearchOutcome {
  names: ManagePage;
  passages: LibraryResults;
}

/** The library: every source, filters, search by name and by the text, add and remove. */
export function LibraryView({
  initialQuery,
  onSearched,
  onClose,
  onRemoved,
  onManage,
}: {
  initialQuery?: string;
  onSearched?: (query: string) => void;
  onClose: () => void;
  onRemoved?: (sourceId: string) => void;
  /** Opens Settings → Sources, where collections, tags and Trash are managed. */
  onManage: () => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const { openSource, addSource } = useLibraryNav();
  const id = useId();
  const heading = useRef<HTMLHeadingElement>(null);
  const [sourceType, setSourceType] = useState<SourceType | "">("");
  const [language, setLanguage] = useState("");
  const [archive, setArchive] = useState<ArchiveFilter>("both");
  const [offset, setOffset] = useState(0);
  const state = archive === "both" ? undefined : archive;
  const page = useLoad(
    useCallback(
      () =>
        api.libraryManageList({
          sourceType: sourceType || null,
          language: language || null,
          state: state ?? null,
          sort: "added",
          descending: true,
          offset,
          limit: PAGE,
        }),
      [api, sourceType, language, state, offset],
    ),
  );
  const languages = useLoad(useCallback(() => api.libraryLanguages(), [api]));
  const collections = useLoad(useCallback(() => api.libraryCollections(), [api]));
  const [query, setQuery] = useState(initialQuery ?? "");
  const [submitted, setSubmitted] = useState(initialQuery ?? "");
  const [outcome, setOutcome] = useState<SearchOutcome | null>(null);
  const [searchError, setSearchError] = useState<string | null>(null);
  const [confirming, setConfirming] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => heading.current?.focus(), []);

  const fetchSearch = useCallback(
    (text: string, passageOffset = 0) =>
      Promise.all([
        api.librarySearch({
          query: text,
          offset: passageOffset,
          sourceType: sourceType || undefined,
          language: language || undefined,
          state,
        }),
        passageOffset === 0
          ? api.libraryManageList({
              text,
              sourceType: sourceType || null,
              language: language || null,
              state: state ?? null,
              sort: "title",
              descending: false,
              offset: 0,
              limit: PAGE,
            })
          : Promise.resolve(null),
      ]),
    [api, sourceType, language, state],
  );
  const acceptSearch = (passageOffset: number, passages: LibraryResults, names: ManagePage | null) => {
    setSearchError(null);
    setOutcome((previous) => ({
      names: names ?? previous?.names ?? { total: 0, offset: 0, items: [], active: 0, archived: 0, trashed: 0 },
      passages:
        passageOffset > 0 && previous ? { ...passages, hits: [...previous.passages.hits, ...passages.hits] } : passages,
    }));
  };
  const failSearch = (e: unknown, passageOffset: number) => {
    setSearchError(toApiError(e).message);
    if (passageOffset === 0) setOutcome(null);
  };
  // Opening from the app bar, returning from a source, or changing a filter.
  // Results are stored when the request finishes, not while this effect runs.
  useEffect(() => {
    if (!submitted.trim()) return;
    let active = true;
    fetchSearch(submitted, 0).then(
      ([passages, names]) => {
        if (active) acceptSearch(0, passages, names);
      },
      (e: unknown) => {
        if (active) failSearch(e, 0);
      },
    );
    return () => {
      active = false;
    };
  }, [fetchSearch, submitted]);

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (!query.trim()) return;
    onSearched?.(query);
    if (query === submitted) {
      fetchSearch(query, 0).then(
        ([passages, names]) => acceptSearch(0, passages, names),
        (e: unknown) => failSearch(e, 0),
      );
    } else setSubmitted(query);
  };

  const collectionName = (collectionId: string) => collections.data?.find((c) => c.id === collectionId)?.name;

  // How many marks the source being removed has (v0.14): said in the confirmation, so nothing is lost unawares.
  const [markCount, setMarkCount] = useState<{ sourceId: string; count: number } | null>(null);
  const confirm = (sourceId: string) => {
    setConfirming(sourceId);
    setMarkCount(null);
    api.sourceMarksSources().then(
      (all) => setMarkCount({ sourceId, count: all.find((x) => x.sourceId === sourceId)?.count ?? 0 }),
      () => undefined,
    );
  };

  // Moved to Trash (v0.24): recoverable, so the confirmation says so and an Undo follows.
  const [trashed, setTrashed] = useState<{ id: string; title: string } | null>(null);
  const trash = async (sourceId: string, title: string) => {
    await api.librarySetState([sourceId], "trashed");
    setConfirming(null);
    onRemoved?.(sourceId);
    page.reload();
    setOutcome(null);
    setTrashed({ id: sourceId, title });
  };
  const undo = async () => {
    if (!trashed) return;
    setError(null);
    try {
      await api.librarySetState([trashed.id], "active");
      setTrashed(null);
      page.reload();
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  const listRef = useRef<HTMLUListElement>(null);
  const narrow = (value: string) => {
    setOffset(0);
    return value;
  };

  const select = "aw-input";
  const items = page.data?.items ?? [];
  const filtered = Boolean(sourceType || language || archive !== "both");
  const names = outcome?.names.items ?? [];
  const passages = outcome?.passages;
  const nothing = outcome !== null && names.length === 0 && (passages?.hits.length ?? 0) === 0;
  return (
    <main aria-labelledby={`${id}-title`} className="min-h-0 flex-1 overflow-y-auto bg-bg">
      {/* Back stays in reach while the page scrolls: the same full-width bar as Settings (v0.18.5). */}
      <div className="sticky top-0 z-10 border-b border-border-subtle bg-bg">
        <div className="mx-auto max-w-[52rem] px-4 py-2 sm:px-8">
          <Button variant="quiet" onClick={onClose}>
            <ArrowLeft className="size-4" aria-hidden="true" /> {t("library.view.backToWorkspace")}
          </Button>
        </div>
      </div>
      <div className="mx-auto flex max-w-[52rem] flex-col gap-8 px-4 pb-24 pt-6 sm:px-8">
        <div className="flex flex-col items-start gap-4">
          <div className="flex w-full flex-wrap items-end justify-between gap-3">
            <h1 id={`${id}-title`} ref={heading} tabIndex={-1} className="type-title m-0">
              {t("library.view.title")}
            </h1>
            <div className="flex flex-wrap gap-2">
              <Button onClick={onManage}>{t("library.view.manage")}</Button>
              <Button variant="primary" onClick={addSource}>
                <Plus className="size-4" aria-hidden="true" /> {t("library.addToLibraryEllipsis")}
              </Button>
            </div>
          </div>
          <p className="m-0 text-text-muted">{t("library.view.intro")}</p>
        </div>

        <section aria-labelledby={`${id}-search`} className="flex flex-col gap-3">
          <h2 id={`${id}-search`} className="type-section m-0">{t("library.search.heading")}</h2>
          <form onSubmit={submit} className="flex gap-2" noValidate role="search">
            <label htmlFor={`${id}-q`} className="sr-only">{t("library.search.label")}</label>
            <input id={`${id}-q`} value={query} onChange={(e) => setQuery(e.target.value)} placeholder={t("library.search.placeholder")} className="aw-input flex-1" />
            <Button type="submit">{t("common.search")}</Button>
          </form>
          {searchError && <Notice tone="error">{searchError}</Notice>}
          {nothing && (
            <p role="status" className="m-0 text-[0.9375rem] text-text-muted">{t("library.search.nothingFound")}</p>
          )}
          {names.length > 0 && (
            <div className="flex flex-col gap-2">
              <h3 className="type-item m-0">{t("library.search.byName")}</h3>
              <ul aria-label={t("library.search.byNameLabel")} className="m-0 flex list-none flex-col p-0">
                {names.map((hit) => (
                  <li key={hit.id} className="border-t border-border-subtle">
                    <button type="button" onClick={() => openSource(hit.id)} className="flex w-full flex-col items-start gap-1 px-1 py-3 text-left hover:bg-hover">
                      <span className="font-medium text-text-strong">
                        {hit.author ? `${hit.author}, ` : ""}
                        <cite className="not-italic">{hit.title}</cite>
                      </span>
                      {hit.state === "archived" && <ArchiveBadge label={t("common.lifecycle.state.archived")} />}
                    </button>
                  </li>
                ))}
              </ul>
              {outcome !== null && outcome.names.total > names.length && (
                <p className="m-0 text-[0.9375rem] text-text-muted">
                  {t("library.manage.range", {
                    from: formatNumber(1),
                    to: formatNumber(names.length),
                    total: formatNumber(outcome.names.total),
                  })}
                </p>
              )}
            </div>
          )}
          {passages && passages.hits.length > 0 && (
            <div className="flex flex-col gap-2">
              <h3 className="type-item m-0">{t("library.search.inText")}</h3>
              <p role="status" className="m-0 text-[0.9375rem] text-text-muted">
                {t(passages.hasMore ? "library.search.resultCountMore" : "library.search.resultCount", {
                  count: passages.hits.length,
                  number: formatNumber(passages.hits.length),
                })}
              </p>
              <ol aria-label={t("library.search.resultsLabel")} className="m-0 flex list-none flex-col p-0">
                {passages.hits.map((hit) => (
                  <li key={hit.segmentId} className="border-t border-border-subtle">
                    <button type="button" onClick={() => openSource(hit.sourceId, hit.segmentId)} className="flex w-full flex-col items-start gap-1 px-1 py-3 text-left hover:bg-hover">
                      <span className="font-medium text-text-strong">
                        {hit.author ? `${hit.author}, ` : ""}
                        <cite className="not-italic">{hit.sourceTitle}</cite>
                        {hit.locator && <span className="font-normal text-text-muted">{t("common.listSeparator")}{locatorLabel(t, hit.locator)}</span>}
                      </span>
                      {hit.archived && <ArchiveBadge label={t("common.lifecycle.state.archived")} />}
                      <span className="reading text-[1rem]">{highlighted(hit.snippet)}</span>
                    </button>
                  </li>
                ))}
              </ol>
              {passages.hasMore && (
                <div>
                  <Button onClick={() => fetchSearch(submitted, passages.hits.length).then(
                    ([found]) => acceptSearch(passages.hits.length, found, null),
                    (e: unknown) => failSearch(e, passages.hits.length),
                  )}>{t("library.search.showMore")}</Button>
                </div>
              )}
            </div>
          )}
        </section>

        <section aria-labelledby={`${id}-all`} className="flex flex-col gap-3">
          <h2 id={`${id}-all`} className="type-section m-0">{t("library.filter.heading")}</h2>
          <div role="group" aria-label={t("library.filter.label")} className="flex flex-wrap gap-x-4 gap-y-2 text-[0.9375rem]">
            <label className="flex items-center gap-2">
              {t("library.filter.type")}
              <select value={sourceType} onChange={(e) => setSourceType(narrow(e.target.value) as SourceType | "")} className={select}>
                <option value="">{t("library.filter.allTypes")}</option>
                {SOURCE_TYPES.map((type) => <option key={type} value={type}>{sourceTypeLabel(t, type)}</option>)}
              </select>
            </label>
            <label className="flex items-center gap-2">
              {t("library.filter.language")}
              <select value={language} onChange={(e) => setLanguage(narrow(e.target.value))} className={select}>
                <option value="">{t("library.filter.allLanguages")}</option>
                {(languages.data ?? []).map((l) => <option key={l} value={l}>{l}</option>)}
              </select>
            </label>
            <label className="flex items-center gap-2">
              {t("library.filter.archive")}
              <select
                value={archive}
                aria-describedby={`${id}-archive-help`}
                onChange={(e) => setArchive(narrow(e.target.value) as ArchiveFilter)}
                className={select}
              >
                <option value="both">{t("library.filter.both")}</option>
                <option value="active">{t("library.filter.inUse")}</option>
                <option value="archived">{t("common.lifecycle.state.archived")}</option>
              </select>
            </label>
          </div>
          <p id={`${id}-archive-help`} className="m-0 max-w-[40rem] text-[0.9375rem] text-text-muted">{t("library.filter.archiveHelp")}</p>
          {page.error && <Notice tone="error">{page.error.message}</Notice>}
          {error && <Notice tone="error">{error}</Notice>}
          <p role="status" className="m-0 empty:hidden">
            {trashed && (
              <>
                {t("library.trash.done", { title: trashed.title })}{" "}
                <Button variant="quiet" onClick={undo}>
                  {t("common.lifecycle.undo")}
                </Button>
              </>
            )}
          </p>
          {page.data?.total === 0 && (
            <p className="m-0 text-text-muted">{filtered ? t("library.filter.noMatch") : t("library.filter.empty")}</p>
          )}
          {items.length > 0 && (
            <ul ref={listRef} tabIndex={-1} aria-label={t("library.filter.listLabel")} className="m-0 list-none border-t border-border-subtle p-0 outline-none">
              {items.map((s) => {
                const collectionNames = s.collections.map(collectionName).filter((name): name is string => Boolean(name));
                const tagNames = s.tags.map((tag) => tag.name);
                return (
                  <li key={s.id} className="flex flex-col gap-2 border-b border-border-subtle py-2.5">
                    <div className="flex flex-wrap items-baseline justify-between gap-2">
                      <button type="button" onClick={() => openSource(s.id)} className="min-w-0 text-left">
                        <span className="type-item block hover:underline">{s.title}</span>
                        <span className="type-meta block">
                          {[
                            s.author,
                            sourceTypeLabel(t, s.sourceType),
                            s.format ? formatLabel(t, s.format as SourceFormat) : null,
                            s.language,
                            t("library.view.addedOn", { date: formatDate(s.createdAt) }),
                          ]
                            .filter(Boolean)
                            .join(t("common.listSeparator"))}
                        </span>
                        {(collectionNames.length > 0 || tagNames.length > 0 || s.state === "archived") && (
                          <span className="mt-1 flex flex-wrap items-center gap-1">
                            {s.state === "archived" && <ArchiveBadge label={t("common.lifecycle.state.archived")} />}
                            {collectionNames.length > 0 && (
                              <span className="type-meta">
                                {t("library.manage.field.collections")}
                                {t("common.listSeparator")}
                                {collectionNames.join(t("common.listSeparator"))}
                              </span>
                            )}
                            {tagNames.length > 0 && (
                              <span className="type-meta">
                                {t("library.manage.field.tags")}
                                {t("common.listSeparator")}
                                {tagNames.join(t("common.listSeparator"))}
                              </span>
                            )}
                          </span>
                        )}
                      </button>
                      {/* Rarely needed: a quiet icon on each row, its name saying which source. */}
                      <button
                        type="button"
                        onClick={() => confirm(s.id)}
                        aria-label={t("library.trash.label", { title: s.title })}
                        title={t("library.trash.label", { title: s.title })}
                        className="aw-icon-button size-11 hover:!text-danger"
                      >
                        <Trash2 className="size-4" aria-hidden="true" />
                      </button>
                    </div>
                    {confirming === s.id && (
                      <ConfirmDialog
                        title={t("library.trash.question", { title: s.title })}
                        confirmLabel={t("common.lifecycle.moveToTrash")}
                        returnFocus={() => listRef.current}
                        onCancel={() => setConfirming(null)}
                        onConfirm={() => trash(s.id, s.title)}
                      >
                        <p className="m-0">{t("library.trash.text", { count: TRASH_RETENTION_DAYS })}</p>
                        <p className="m-0">{t("library.trash.kept")}</p>
                        {markCount?.sourceId === s.id && markCount.count > 0 && <p className="m-0">{t("library.trash.marks", { count: markCount.count })}</p>}
                      </ConfirmDialog>
                    )}
                  </li>
                );
              })}
            </ul>
          )}
          {page.data && page.data.total > PAGE && (
            <nav aria-label={t("library.manage.pages")} className="flex flex-wrap items-center gap-2">
              <Button variant="quiet" disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE))}>
                {t("library.manage.previous")}
              </Button>
              <span>
                {t("library.manage.range", {
                  from: formatNumber(offset + 1),
                  to: formatNumber(Math.min(offset + PAGE, page.data.total)),
                  total: formatNumber(page.data.total),
                })}
              </span>
              <Button variant="quiet" disabled={offset + PAGE >= page.data.total} onClick={() => setOffset(offset + PAGE)}>
                {t("library.manage.next")}
              </Button>
            </nav>
          )}
        </section>
      </div>
    </main>
  );
}

/** The same quiet badge Settings uses for an archived source. */
function ArchiveBadge({ label }: { label: string }) {
  return <span className="rounded-full border border-border-subtle px-2 py-0.5 text-[0.8125rem] text-text-muted">{label}</span>;
}
