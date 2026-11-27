import { useEffect, useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { formatNumber } from "@/i18n/format";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import type { BibleTranslation, ImportPreview } from "@/types/scripture";
import { FoundBooks, IssueGroup, LeftOut, Readiness, RecognitionSamples, SavedCorrections, Verification } from "./AddBibleView";
import { useScripture } from "./ScriptureContext";

/**
 * Looking for verses in a Bible kept as a document (v0.23): its kept files
 * are read again (nothing is added again), the person checks what was found,
 * exactly as when adding a Bible, and only "Use these verses" changes it.
 * Closing keeps it as a document.
 */
export function FindVerses({ bible, onClose }: { bible: BibleTranslation; onClose: () => void }) {
  const { t } = useTranslation();
  const api = useApi();
  const { reloadTranslations, setTranslation } = useScripture();
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [confirmEdition, setConfirmEdition] = useState(false);
  const editionHint = useId();
  const [progress, setProgress] = useState<{ done: number; total: number } | null>(null);
  const [done, setDone] = useState<"whole" | "partial" | null>(null);

  useEffect(() => {
    let active = true;
    const jobId = crypto.randomUUID();
    let stop: (() => void) | undefined;
    const recognise = async () => {
      try {
        stop = await api.onBibleImportProgress?.((p) => {
          if (active && jobId === p.id) setProgress({ done: p.done, total: p.total });
        });
        if (!active) return;
        const reviewed = await api.bibleDocumentRecognise(bible.id, jobId);
        if (active) setPreview(reviewed);
        else void api.bibleImportCancel(reviewed.previewId).catch(() => undefined);
      } catch (e) {
        if (active) setError(toApiError(e).message);
      } finally {
        stop?.();
      }
    };
    void recognise();
    return () => {
      active = false;
      void api.bibleImportCancel(jobId).catch(() => undefined);
    };
  }, [api, bible.id]);

  const close = () => {
    if (preview && !done) void api.bibleImportCancel(preview.previewId).catch(() => undefined);
    onClose();
  };

  // `partial` (v0.24): only the chapters found without doubt; the Bible stays
  // a document, and those passages can be found in it.
  const use = async (partial: boolean) => {
    if (!preview) return;
    setBusy(true);
    setError(null);
    try {
      await api.bibleDocumentAddVerses(preview.previewId, partial, confirmEdition);
      setDone(partial ? "partial" : "whole");
      await reloadTranslations();
      setTranslation(bible.id);
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setBusy(false);
    }
  };

  const doc = preview?.document ?? null;
  const errors = preview?.issues.filter((i) => i.severity === "error") ?? [];
  const warnings = preview?.issues.filter((i) => i.severity === "warning") ?? [];
  const notes = preview?.issues.filter((i) => i.severity === "note") ?? [];
  const partial = !!doc && !doc.versesUsable && doc.partialVerses > 0;

  return (
    <Dialog
      wide
      title={t("scripture.document.recognise.title", { name: bible.name })}
      onClose={close}
      actions={
        done ? (
          <Button variant="primary" onClick={onClose}>
            {t("common.close")}
          </Button>
        ) : (
          <>
            <Button onClick={close}>{t("scripture.document.recognise.keep")}</Button>
            {partial ? (
              <Button variant="primary" onClick={() => use(true)} disabled={busy}>
                {t("scripture.document.recognise.usePartial", { count: doc!.partialVerses, formatted: formatNumber(doc!.partialVerses) })}
              </Button>
            ) : (
              <Button variant="primary" onClick={() => use(false)} disabled={!doc?.versesUsable || busy}>
                {t("scripture.document.recognise.use")}
              </Button>
            )}
          </>
        )
      }
    >
      {error && <Notice tone="error">{error}</Notice>}
      {done ? (
        <p className="m-0">{t(done === "partial" ? "scripture.document.recognise.partialDone" : "scripture.document.recognise.done", { name: bible.name })}</p>
      ) : !preview ? (
        !error && (
          <p role="status" className="m-0 text-text-muted">
            {progress && progress.total > 0
              ? t("scripture.addBible.document.progress", { done: formatNumber(progress.done), total: formatNumber(progress.total) })
              : t("scripture.document.recognise.reading")}
          </p>
        )
      ) : (
        <div className="flex flex-col gap-4">
          {doc?.versesUsable ? (
            <p className="m-0">{t("scripture.addBible.document.usable")}</p>
          ) : partial ? (
            <Notice>{t("scripture.document.recognise.partial", { count: doc!.heldChapters, formatted: formatNumber(doc!.heldChapters) })}</Notice>
          ) : (
            <Notice>{doc?.books.some((b) => b.verses > 0) ? t("scripture.addBible.document.unusable") : t("scripture.addBible.document.noneFound")}</Notice>
          )}
          {preview.verseCount > 0 && (
            <div className="flex flex-col gap-0.5">
              <p className="m-0">{t("scripture.addBible.booksDetected", { count: preview.bookCount })}</p>
              <p className="m-0">{t("scripture.addBible.chaptersDetected", { count: preview.chapterCount, number: formatNumber(preview.chapterCount) })}</p>
              <p className="m-0">{t("scripture.addBible.versesDetected", { count: preview.verseCount, number: formatNumber(preview.verseCount) })}</p>
            </div>
          )}
          {preview.verseCount > 0 && <Readiness preview={preview} />}
          {doc && <Verification preview={doc} />}
          {doc && <RecognitionSamples preview={doc} />}
          {doc && !bible.editionConfirmed && (
            <div className="flex flex-col gap-1">
              <label className="flex items-start gap-2">
                <input type="checkbox" checked={confirmEdition} onChange={(e) => setConfirmEdition(e.target.checked)} aria-describedby={editionHint} />
                <span>{t("scripture.addBible.document.confirmEdition")}</span>
              </label>
              <p id={editionHint} className="m-0 text-[0.9375rem] text-text-muted">{t("scripture.addBible.editionOptional")}</p>
            </div>
          )}
          {doc && <SavedCorrections preview={doc} />}
          {doc && doc.books.length > 0 && (
            <FoundBooks
              preview={doc}
              onCorrect={async (books) => {
                setError(null);
                try {
                  setPreview(await api.bibleImportCorrect(preview.previewId, books));
                } catch (e) {
                  setError(toApiError(e).message);
                }
              }}
            />
          )}
          {errors.length > 0 && <IssueGroup title={t("scripture.addBible.problems", { count: errors.length })} issues={errors} />}
          {warnings.length > 0 && <IssueGroup title={t("scripture.addBible.warnings", { count: warnings.length })} issues={warnings} />}
          {notes.length > 0 && <IssueGroup title={t("scripture.addBible.notes", { count: notes.length })} issues={notes} />}
          {doc && doc.leftOut.length > 0 && <LeftOut preview={doc} />}
        </div>
      )}
    </Dialog>
  );
}
