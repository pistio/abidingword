import { useCallback, useEffect, useId, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import type { TFunction } from "i18next";
import { ActionMenu } from "@/design-system/ActionMenu";
import { Button } from "@/design-system/Button";
import { ConfirmDialog } from "@/design-system/ConfirmDialog";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { useScripture } from "@/features/scripture/ScriptureContext";
import type { BibleTranslation, WordChoice, WordsPreview } from "@/types/scripture";
import { TRASH_RETENTION_DAYS, type ItemState } from "@/types/library";
import { NO_CHOICE, WordResearch, hasWordResearch, willIndex } from "@/features/scripture/WordResearch";
import { formatDate, formatNumber } from "@/i18n/format";
import { LanguageChoice, type LanguageValue } from "@/features/scripture/LanguageChoice";

/** Settings → Bible Texts: what is installed, details, add, archive, Trash. */
export function BibleTexts({ onAddBible }: { onAddBible: () => void }) {
  const { t } = useTranslation();
  const api = useApi();
  const { allTranslations, reloadTranslations } = useScripture();
  // Archived and trashed Bibles too (v0.24): the Scripture pane only knows the active ones.
  const [all, setAll] = useState<BibleTranslation[] | null>(null);
  const load = useCallback(() => api.bibleListAll().then(setAll, () => undefined), [api]);
  useEffect(() => {
    void load();
  }, [load, allTranslations]);
  const bibles = all ?? allTranslations;
  const active = bibles.filter((b) => b.state === "active");
  const archived = bibles.filter((b) => b.state === "archived");
  const trashed = bibles.filter((b) => b.state === "trashed");
  const [asking, setAsking] = useState<{ action: "trash" | "delete"; bibles: BibleTranslation[] } | null>(null);
  const [done, setDone] = useState<{ message: string; undo?: { ids: string[]; to: ItemState } } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const list = useRef<HTMLDivElement>(null);

  const refresh = async () => {
    await reloadTranslations();
    await load();
  };
  const move = async (targets: BibleTranslation[], to: ItemState, message: string, undo?: ItemState) => {
    setError(null);
    try {
      await api.bibleSetState(targets.map((b) => b.id), to);
      await refresh();
      setDone({ message, undo: undo ? { ids: targets.map((b) => b.id), to: undo } : undefined });
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  const one = (b: BibleTranslation[]) => (b.length === 1 ? b[0]!.name : "");

  return (
    <div ref={list} tabIndex={-1} className="flex flex-col gap-4 outline-none">
      {active.length === 0 ? (
        <p className="m-0 text-text-muted">{t("scripture.bibles.empty")}</p>
      ) : (
        <ul aria-label={t("scripture.bibles.installed")} className="m-0 flex list-none flex-col border-t border-border-subtle p-0">
          {active.map((bible) => (
            <BibleRow
              key={bible.id}
              bible={bible}
              onArchive={() => move([bible], "archived", t("scripture.bibles.lifecycle.archived", { name: bible.name }), "active")}
              onTrash={() => setAsking({ action: "trash", bibles: [bible] })}
            />
          ))}
        </ul>
      )}
      <div>
        <Button onClick={onAddBible}>{t("scripture.bibles.addBible")}</Button>
      </div>

      {archived.length > 0 && (
        <details className="border-t border-border-subtle pt-3">
          <summary className="cursor-pointer font-medium text-text-strong">{t("scripture.bibles.lifecycle.archivedTitle", { count: archived.length, formatted: formatNumber(archived.length) })}</summary>
          <p className="type-meta mb-2 mt-1">{t("scripture.bibles.lifecycle.archivedText")}</p>
          <ul className="m-0 flex list-none flex-col p-0">
            {archived.map((bible) => (
              <KeptRow key={bible.id} bible={bible}>
                <Button variant="quiet" onClick={() => move([bible], "active", t("scripture.bibles.lifecycle.restored", { name: bible.name }))}>
                  {t("common.lifecycle.restore")}
                </Button>
                <Button variant="quiet" onClick={() => setAsking({ action: "trash", bibles: [bible] })}>
                  {t("common.lifecycle.moveToTrash")}
                </Button>
              </KeptRow>
            ))}
          </ul>
        </details>
      )}

      {trashed.length > 0 && (
        <details className="border-t border-border-subtle pt-3">
          <summary className="cursor-pointer font-medium text-text-strong">{t("scripture.bibles.lifecycle.trashTitle", { count: trashed.length, formatted: formatNumber(trashed.length) })}</summary>
          <p className="type-meta mb-2 mt-1">{t("common.lifecycle.trashText", { count: TRASH_RETENTION_DAYS })}</p>
          <ul className="m-0 flex list-none flex-col p-0">
            {trashed.map((bible) => (
              <KeptRow key={bible.id} bible={bible} since={bible.stateChangedAt}>
                <Button variant="quiet" onClick={() => move([bible], "active", t("scripture.bibles.lifecycle.restored", { name: bible.name }))}>
                  {t("common.lifecycle.restore")}
                </Button>
                <Button variant="quiet" className="hover:!text-danger" onClick={() => setAsking({ action: "delete", bibles: [bible] })}>
                  {t("common.lifecycle.deletePermanently")}
                </Button>
              </KeptRow>
            ))}
          </ul>
          {trashed.length > 1 && (
            <Button variant="quiet" className="mt-2 hover:!text-danger" onClick={() => setAsking({ action: "delete", bibles: trashed })}>
              {t("common.lifecycle.emptyTrash")}
            </Button>
          )}
        </details>
      )}

      <p role="status" className="m-0 empty:hidden">
        {done && (
          <>
            {done.message}{" "}
            {done.undo && (
              <Button variant="quiet" onClick={() => move(state(done.undo!.ids), done.undo!.to, t("common.lifecycle.undone"))}>
                {t("common.lifecycle.undo")}
              </Button>
            )}
          </>
        )}
      </p>
      {error && <p role="alert" className="m-0 text-danger">{error}</p>}

      {asking?.action === "trash" && (
        <ConfirmDialog
          title={t("scripture.bibles.lifecycle.trashQuestion", { name: one(asking.bibles) })}
          confirmLabel={t("common.lifecycle.moveToTrash")}
          returnFocus={() => list.current}
          onCancel={() => setAsking(null)}
          onConfirm={async () => {
            await api.bibleSetState(asking.bibles.map((b) => b.id), "trashed");
            setAsking(null);
            await refresh();
            setDone({ message: t("scripture.bibles.lifecycle.trashed", { name: one(asking.bibles) }), undo: { ids: asking.bibles.map((b) => b.id), to: "active" } });
          }}
        >
          <p className="m-0">{t("scripture.bibles.lifecycle.trashText", { count: TRASH_RETENTION_DAYS })}</p>
          <p className="m-0">{t("scripture.bibles.lifecycle.keptText")}</p>
        </ConfirmDialog>
      )}
      {asking?.action === "delete" && (
        <ConfirmDialog
          title={
            asking.bibles.length === 1
              ? t("scripture.bibles.lifecycle.deleteQuestion", { name: one(asking.bibles) })
              : t("scripture.bibles.lifecycle.deleteManyQuestion", { count: asking.bibles.length, formatted: formatNumber(asking.bibles.length) })
          }
          confirmLabel={t("common.lifecycle.deletePermanently")}
          typeToConfirm={asking.bibles.length > 1 ? String(asking.bibles.length) : undefined}
          returnFocus={() => list.current}
          onCancel={() => setAsking(null)}
          onConfirm={async () => {
            await api.bibleRemovePermanently(asking.bibles.map((b) => b.id));
            const count = asking.bibles.length;
            setAsking(null);
            await refresh();
            setDone({ message: t("scripture.bibles.lifecycle.deleted", { count, formatted: formatNumber(count) }) });
          }}
        >
          <p className="m-0">{t("scripture.bibles.lifecycle.deleteText")}</p>
          <p className="m-0">{t("scripture.bibles.lifecycle.keptText")}</p>
          <p className="m-0 font-medium">{t("common.lifecycle.cannotUndo")}</p>
        </ConfirmDialog>
      )}
    </div>
  );

  function state(ids: string[]): BibleTranslation[] {
    return bibles.filter((b) => ids.includes(b.id));
  }
}

/** An archived or trashed Bible: its name, and what can be done with it. */
function KeptRow({ bible, since, children }: { bible: BibleTranslation; since?: string | null; children: ReactNode }) {
  const { t } = useTranslation();
  return (
    <li className="flex flex-wrap items-baseline justify-between gap-2 border-b border-border-subtle py-2">
      <span>
        {bible.name} <span className="text-text-muted">({bible.shortName})</span>
        {since && <span className="type-meta block">{t("common.lifecycle.trashedOn", { date: formatDate(since) })}</span>}
      </span>
      <span className="flex gap-1">{children}</span>
    </li>
  );
}

function BibleRow({ bible, onArchive, onTrash }: { bible: BibleTranslation; onArchive: () => void; onTrash: () => void }) {
  const { t } = useTranslation();
  const source = bible.sourceOrigin === "bundled" ? t("scripture.bibles.source.bundled") : t("scripture.bibles.source.imported");
  const document = bible.kind === "document";
  const status = document ? t("scripture.bibles.status.document") : bible.importStatus === "ready" ? t("scripture.bibles.status.ready") : t("scripture.bibles.status.notStructured");
  const files = bible.originalFileName ? bible.originalFileName.split(", ").length : 1;
  const [open, setOpen] = useState(false);

  return (
    <li className="flex flex-col gap-2 border-b border-border-subtle py-4">
      <div className="flex flex-wrap items-baseline justify-between gap-2">
        <div>
          <p className="type-item m-0">
            {bible.name} <span className="font-normal text-text-muted">({bible.shortName})</span>
          </p>
          <p className="m-0 text-[0.9375rem] text-text-muted">
            {[bible.language, source, status].join(t("common.listSeparator"))}
          </p>
        </div>
        <div className="flex items-center gap-1">
          <Button variant="quiet" aria-expanded={open} onClick={() => setOpen(!open)}>
            {open ? t("scripture.bibles.hideDetails") : t("scripture.bibles.details")}
          </Button>
          <ActionMenu
            label={t("scripture.bibles.lifecycle.menu", { name: bible.name })}
            actions={[
              { label: t("common.lifecycle.archive"), description: t("scripture.bibles.lifecycle.archiveHint"), onSelect: onArchive },
              { label: t("common.lifecycle.moveToTrashEllipsis"), onSelect: onTrash },
            ]}
          />
        </div>
      </div>

      {open && (
        <dl className="m-0 grid grid-cols-1 gap-x-6 gap-y-1 text-[0.9375rem] sm:grid-cols-[10rem_1fr]">
          <dt className="font-medium text-text-strong">{t("scripture.bibles.field.language")}</dt>
          <dd className="m-0">
            <BibleLanguage bible={bible} />
          </dd>
          <dt className="font-medium text-text-strong">{t("scripture.bibles.field.contents")}</dt>
          <dd className="m-0">
            {document
              ? t("scripture.bibles.contentsDocument", { count: files, formatted: formatNumber(files) })
              : t("scripture.bibles.contents", { count: bible.bookCount, verses: formatNumber(bible.verseCount) })}
          </dd>
          {bible.publisher && (<><dt className="font-medium text-text-strong">{t("scripture.bibles.field.publisher")}</dt><dd className="m-0">{bible.publisher}</dd></>)}
          {bible.licenseType && (<><dt className="font-medium text-text-strong">{t("scripture.bibles.field.licence")}</dt><dd className="m-0">{bible.licenseType}</dd></>)}
          {bible.copyrightNotice && (<><dt className="font-medium text-text-strong">{t("scripture.bibles.field.notice")}</dt><dd className="m-0">{bible.copyrightNotice}</dd></>)}
          {bible.originalFileName && (<><dt className="font-medium text-text-strong">{t("scripture.bibles.field.from")}</dt><dd className="m-0 break-all">{bible.originalFileName}</dd></>)}
          <dt className="font-medium text-text-strong">{t("scripture.bibles.field.added")}</dt>
          <dd className="m-0">{formatDate(bible.createdAt)}</dd>
          <dt className="font-medium text-text-strong">{t("scripture.bibles.field.words")}</dt>
          <dd className="m-0">{isDocumentSource(bible) ? t("scripture.bibles.wordsDocument") : <BibleWords bible={bible} />}</dd>
        </dl>
      )}

    </li>
  );
}

/** Whether a Bible was added from a document (v0.23): such files carry no word information. */
function isDocumentSource(bible: BibleTranslation): boolean {
  return ["pdf", "epub", "docx", "markdown", "text"].includes(bible.sourceType);
}

/**
 * A Bible's language, correctable here (v0.18.3): the name shown and the tag
 * typed references use first. Saving changes nothing else about the Bible.
 */
function BibleLanguage({ bible }: { bible: BibleTranslation }) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const { reloadTranslations } = useScripture();
  const [editing, setEditing] = useState(false);
  const [value, setValue] = useState<LanguageValue | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const changeButton = useRef<HTMLButtonElement>(null);
  const returnFocus = useRef(false);

  // Back on "Change language" after Save or Cancel, as from any small editor.
  useEffect(() => {
    if (!editing && returnFocus.current) {
      returnFocus.current = false;
      changeButton.current?.focus();
    }
  }, [editing]);
  const finish = () => {
    returnFocus.current = true;
    setEditing(false);
  };

  const start = () => {
    setValue(bible.languageCode ? { code: bible.languageCode, name: bible.language } : bible.language ? { code: null, name: bible.language } : null);
    setError(null);
    setEditing(true);
  };
  const save = async () => {
    if (!value || !value.name.trim()) return;
    setBusy(true);
    setError(null);
    try {
      await api.bibleSetLanguage({ id: bible.id, language: value.name.trim(), languageCode: value.code });
      await reloadTranslations();
      finish();
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };

  if (!editing) {
    return (
      <span className="inline-flex flex-wrap items-baseline gap-x-3">
        <span>{bible.language || t("scripture.bibles.languageNotSet")}</span>
        <Button ref={changeButton} variant="quiet" onClick={start}>
          {t("scripture.bibles.changeLanguage")}
        </Button>
      </span>
    );
  }
  return (
    <div className="flex max-w-md flex-col gap-2">
      <LanguageChoice id={`${id}-language`} label={t("scripture.bibles.field.language")} value={value} onChange={setValue} />
      <div className="flex gap-2">
        <Button variant="primary" onClick={save} disabled={busy || !value || !value.name.trim()}>
          {t("scripture.bibles.saveLanguage")}
        </Button>
        <Button onClick={finish}>{t("common.cancel")}</Button>
      </div>
      {error && <p role="alert" className="m-0 text-danger">{error}</p>}
    </div>
  );
}

/** A documented scheme's name, or what is shown instead. */
function grammarState(bible: BibleTranslation, t: TFunction): string | null {
  const words = bible.words;
  if (!words || !words.morphology) return null;
  if (words.morphScheme === "raw" || words.morphScheme === "none") return t("scripture.bibles.words.grammarRaw");
  const scheme = t(`scripture.grammar.scheme.${words.morphScheme}`);
  return words.grammarBasis === "person" ? t("scripture.bibles.words.grammarChosen", { scheme }) : t("scripture.bibles.words.grammarStated", { scheme });
}

/**
 * A Bible's word information (v0.22): what its index holds, and reading its
 * kept files again to add or renew the index — the text stays exactly as it
 * is, and nothing is saved until the person confirms. Removing the index
 * leaves the text and the ordinary concordance as they are.
 */
function BibleWords({ bible }: { bible: BibleTranslation }) {
  const { t } = useTranslation();
  const api = useApi();
  const { reloadTranslations } = useScripture();
  const [preview, setPreview] = useState<WordsPreview | null>(null);
  const [choice, setChoice] = useState<WordChoice>(NO_CHOICE);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmRemove, setConfirmRemove] = useState(false);
  const words = bible.words;
  // The choices already made for this Bible's index, so a re-read keeps them.
  const current: WordChoice = {
    numbers: !!words && words.lexicalIds && !words.lemmas && !words.morphology,
    grammar: words?.grammarBasis === "person" && words.morphScheme !== "raw" && words.morphScheme !== "none" ? words.morphScheme : null,
  };
  // A preview left open is dropped when the panel goes.
  const open = useRef<string | null>(null);
  useEffect(() => {
    open.current = preview?.previewId ?? null;
  }, [preview]);
  useEffect(
    () => () => {
      if (open.current) void api.bibleWordsCancel(open.current).catch(() => undefined);
    },
    [api],
  );

  const run = async (work: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await work();
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };
  const check = () =>
    run(async () => {
      setChoice(current);
      setPreview(await api.bibleWordsPreview(bible.id));
    });
  const cancel = () => {
    if (preview) void api.bibleWordsCancel(preview.previewId).catch(() => undefined);
    setPreview(null);
  };
  const apply = () =>
    run(async () => {
      if (!preview) return;
      await api.bibleWordsApply(preview.previewId, choice);
      setPreview(null);
      await reloadTranslations();
    });
  const remove = () =>
    run(async () => {
      await api.bibleWordsRemove(bible.id);
      setConfirmRemove(false);
      await reloadTranslations();
    });

  const grammar = grammarState(bible, t);
  return (
    <div className="flex flex-col gap-2">
      <p className="m-0">
        {words
          ? t("scripture.bibles.words.indexed", { count: words.wordCount, formatted: formatNumber(words.wordCount) })
          : t("scripture.bibles.words.none")}
      </p>
      {words && (
        <p className="m-0 text-text-muted">
          {[
            words.lemmas && t("scripture.research.dictionary"),
            words.morphology && t("scripture.research.grammar"),
            words.lexicalIds && t("scripture.research.number"),
          ]
            .filter(Boolean)
            .join(t("common.listSeparator"))}
          {grammar && <><br />{grammar}</>}
          {words.grammarEvidence && <><br />{t("scripture.bibles.words.evidence", { file: words.grammarEvidence.file, words: words.grammarEvidence.words })}</>}
        </p>
      )}

      {preview ? (
        <div className="flex flex-col gap-3 border-l-[3px] border-border bg-hover px-4 py-3">
          {hasWordResearch(preview.words) ? (
            <WordResearch report={preview.words} choice={choice} onChoice={setChoice} />
          ) : (
            <p className="m-0">{t("scripture.bibles.words.nothing")}</p>
          )}
          {words && willIndex(preview.words, choice) && <p className="m-0 text-text-muted">{t("scripture.bibles.words.replaces")}</p>}
          <div className="flex flex-wrap gap-2">
            <Button variant="primary" onClick={apply} disabled={busy || !willIndex(preview.words, choice)}>
              {t("scripture.bibles.words.save")}
            </Button>
            <Button onClick={cancel} disabled={busy}>{t("common.cancel")}</Button>
          </div>
        </div>
      ) : confirmRemove ? (
        <div role="alertdialog" aria-labelledby={`words-remove-${bible.id}`} className="flex flex-col gap-3 border-l-[3px] border-danger bg-hover px-4 py-3">
          <p id={`words-remove-${bible.id}`} className="m-0">{t("scripture.bibles.words.confirmRemove", { name: bible.name })}</p>
          <div className="flex gap-2">
            <Button variant="danger" onClick={remove} disabled={busy}>{t("scripture.bibles.words.remove")}</Button>
            <Button autoFocus onClick={() => setConfirmRemove(false)}>{t("scripture.bibles.keep")}</Button>
          </div>
        </div>
      ) : (
        <div className="flex flex-wrap gap-x-3">
          <Button variant="quiet" onClick={check} disabled={busy}>
            {busy ? t("scripture.bibles.words.checking") : words ? t("scripture.bibles.words.recheck") : t("scripture.bibles.words.check")}
          </Button>
          {words && (
            <Button variant="quiet" onClick={() => setConfirmRemove(true)} disabled={busy}>
              {t("scripture.bibles.words.removeEllipsis")}
            </Button>
          )}
        </div>
      )}
      {error && <p role="alert" className="m-0 text-danger">{error}</p>}
    </div>
  );
}
