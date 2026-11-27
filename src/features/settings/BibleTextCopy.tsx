import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { ReviewStep } from "@/features/scripture/AddBibleView";
import { formatNumber } from "@/i18n/format";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import type { BibleTextCopyPreview, BibleTranslation } from "@/types/scripture";

/** Compare kept USFM files, then use the ordinary import review to add a separate Bible. */
export function BibleTextCopy({ bible, onClose, onUpdated }: { bible: BibleTranslation; onClose: () => void; onUpdated: () => Promise<void> }) {
  const { t } = useTranslation();
  const api = useApi();
  const [review, setReview] = useState<BibleTextCopyPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<BibleTranslation | null>(null);
  const [busy, setBusy] = useState(false);
  const committed = useRef(false);
  const committing = useRef(false);

  useEffect(() => {
    let active = true;
    let previewId: string | null = null;
    const jobId = crypto.randomUUID();
    void api.bibleTextCopyPreview(bible.id, jobId).then((result) => {
      previewId = result.preview.previewId;
      if (active) setReview(result);
      else void api.bibleImportCancel(previewId).catch(() => undefined);
    }, (e: unknown) => {
      if (active) setError(toApiError(e).message);
    });
    return () => {
      active = false;
      void api.bibleImportCancel(jobId).catch(() => undefined);
      if (previewId && !committed.current) void api.bibleImportCancel(previewId).catch(() => undefined);
    };
  }, [api, bible.id]);

  const close = () => {
    if (!committing.current) onClose();
  };
  const unchanged = review?.preview.canImport && review.changedVerses === 0 && review.addedVerses === 0 && review.removedVerses === 0 && review.changedSuperscriptions === 0;

  return (
    <Dialog
      wide
      title={t("scripture.bibles.textCopy.title", { name: bible.name })}
      onClose={close}
      actions={<Button onClick={close} disabled={busy}>{t(saved || unchanged ? "common.close" : "common.cancel")}</Button>}
    >
      {error && <Notice tone="error">{error}</Notice>}
      {saved ? (
        <p role="status" className="m-0">{t("scripture.bibles.textCopy.added", { name: saved.name, shortName: saved.shortName })}</p>
      ) : !review ? (
        !error && <p role="status" className="m-0 text-text-muted">{t("scripture.addBible.checking")}</p>
      ) : unchanged ? (
        <p role="status" className="m-0">{t("scripture.bibles.textCopy.unchanged")}</p>
      ) : (
        <>
          <p className="m-0">{t("scripture.bibles.textCopy.explanation")}</p>
          <dl aria-label={t("scripture.bibles.textCopy.differences")} className="m-0 grid grid-cols-[1fr_auto] gap-x-4 gap-y-1 text-[0.9375rem]">
            <dt>{t("scripture.bibles.textCopy.changedVerses")}</dt><dd className="m-0">{formatNumber(review.changedVerses)}</dd>
            <dt>{t("scripture.bibles.textCopy.addedVerses")}</dt><dd className="m-0">{formatNumber(review.addedVerses)}</dd>
            <dt>{t("scripture.bibles.textCopy.removedVerses")}</dt><dd className="m-0">{formatNumber(review.removedVerses)}</dd>
            <dt>{t("scripture.bibles.textCopy.changedSuperscriptions")}</dt><dd className="m-0">{formatNumber(review.changedSuperscriptions)}</dd>
          </dl>
          <Notice>{t("scripture.bibles.textCopy.kept", { name: review.originalName })}</Notice>
          <ReviewStep
            preview={review.preview}
            addLabel={t("scripture.bibles.textCopy.add")}
            hideCancel
            onCancel={close}
            onCorrect={async (books) => {
              try {
                setReview({ ...review, preview: await api.bibleImportCorrect(review.preview.previewId, books) });
              } catch (e) {
                setError(toApiError(e).message);
              }
            }}
            onAdd={async (details) => {
              setError(null);
              setBusy(true);
              committing.current = true;
              try {
                const copy = await api.bibleImportCommit({ previewId: review.preview.previewId, ...details });
                committed.current = true;
                setSaved(copy);
                await onUpdated();
              } catch (e) {
                setError(toApiError(e).message);
              } finally {
                committing.current = false;
                setBusy(false);
              }
            }}
          />
        </>
      )}
    </Dialog>
  );
}
