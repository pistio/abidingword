import { useCallback, useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { MapPin, X } from "lucide-react";
import { Button } from "@/design-system/Button";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { formatNumber } from "@/i18n/format";
import { bookName, referenceLabel } from "@/i18n/scripture";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { refFromCanonical } from "@/lib/scripture";
import type { BibleDocument, BibleTranslation, DocumentPassage, MarkedPlace, ScriptureRef } from "@/types/scripture";
import { useScripture } from "./ScriptureContext";

/** Where the reader is in a document: what a mark is made from. */
export interface DocumentHere {
  documentId: string;
  page: number | null;
  segment: number | null;
  item: string | null;
}

/** Where to go in a document: a page, or a paragraph. */
export interface DocumentTarget {
  documentId: string;
  page: number | null;
  position: number | null;
}

/**
 * How much of a document Bible can be found by passage (v0.24), in one line:
 * verified verses by book, and the places marked by hand.
 */
export function CoverageLine({ bible, version }: { bible: BibleTranslation; version: number }) {
  const { t } = useTranslation();
  const api = useApi();
  const coverage = useLoad(
    // eslint-disable-next-line react-hooks/exhaustive-deps -- version and the verse count ask for a fresh count after a change.
    useCallback(() => api.bibleDocumentCoverage(bible.id), [api, bible.id, bible.verseCount, version]),
  );
  const data = coverage.data;
  if (!data) return null;
  if (data.verses === 0 && data.marked === 0) return <p className="m-0 type-meta">{t("scripture.document.passages.none")}</p>;
  const books = data.books.map((b) => (b.chapters >= b.of ? bookName(b.book) : t("scripture.document.passages.partOf", { book: bookName(b.book), chapters: formatNumber(b.chapters), of: formatNumber(b.of) })));
  return (
    <p className="m-0 type-meta">
      {data.verses > 0 && t("scripture.document.passages.verified", { count: data.verses, formatted: formatNumber(data.verses), books: books.join(t("common.listSeparator")) })}
      {data.verses > 0 && data.marked > 0 && " · "}
      {data.marked > 0 && t("scripture.document.passages.marked", { count: data.marked, formatted: formatNumber(data.marked) })}
    </p>
  );
}

/** A passage typed in the document reader, looked for in this Bible. */
export function GoToPassage({ bible, onGo }: { bible: BibleTranslation; onGo: (ref: ScriptureRef) => void }) {
  const { t } = useTranslation();
  const api = useApi();
  const { bookByCode } = useScripture();
  const id = useId();
  const [text, setText] = useState("");
  const [error, setError] = useState<string | null>(null);
  const submit = async (e: FormEvent) => {
    e.preventDefault();
    if (!text.trim()) return;
    setError(null);
    try {
      const parsed = await api.referenceParse(text, bible.id);
      const ref = refFromCanonical(parsed.canonical, bookByCode) ?? parsed;
      onGo(ref);
    } catch (err) {
      setError(toApiError(err).message);
    }
  };
  return (
    <form onSubmit={submit} className="flex flex-col gap-1">
      <div className="flex gap-2">
        <label htmlFor={id} className="sr-only">
          {t("scripture.document.passages.goLabel")}
        </label>
        <input id={id} value={text} onChange={(e) => setText(e.target.value)} placeholder={t("scripture.document.passages.goPlaceholder")} className="h-9 min-w-0 flex-1 aw-input px-3" />
        <Button type="submit">{t("scripture.document.passages.go")}</Button>
      </div>
      {error && (
        <p role="alert" className="m-0 text-[0.9375rem] text-danger">
          {error}
        </p>
      )}
    </form>
  );
}

/**
 * A passage in a document Bible (v0.24): its verses where they were found
 * and confirmed, each traceable to the document; the places marked by hand;
 * or a plain statement that it has not been identified yet. Another Bible's
 * text is offered, never shown in its place.
 */
export function PassagePanel({
  bible,
  reference,
  onShow,
  onClose,
  onMark,
  version,
  onChanged,
}: {
  bible: BibleTranslation;
  reference: ScriptureRef;
  onShow: (target: DocumentTarget) => void;
  onClose: () => void;
  onMark: () => void;
  version: number;
  onChanged: () => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const { translation, readInBible } = useScripture();
  const found = useLoad(
    // eslint-disable-next-line react-hooks/exhaustive-deps -- version asks again after a mark is made or removed.
    useCallback(() => api.bibleDocumentLocate(bible.id, reference.canonical), [api, bible.id, reference.canonical, version]),
  );
  const label = referenceLabel(reference);
  const data: DocumentPassage | null = found.data ?? null;
  return (
    <section aria-label={t("scripture.document.passages.panel", { passage: label })} className="flex max-h-[45%] shrink-0 flex-col gap-2 overflow-y-auto border-b border-border-subtle bg-bg px-5 py-3">
      <div className="flex items-start justify-between gap-2">
        <h3 className="type-subhead m-0">{label}</h3>
        <button type="button" onClick={onClose} aria-label={t("scripture.document.passages.close")} className="aw-icon-button size-9">
          <X className="size-4" aria-hidden="true" />
        </button>
      </div>
      {found.error && <Notice tone="error">{found.error.message}</Notice>}
      {data && data.verses.length > 0 && (
        <>
          <ol className="reading m-0 flex list-none flex-col gap-1 p-0">
            {data.verses.map((v) => (
              <li key={v.canonical}>
                <sup className="mr-1 text-text-muted">{v.verse}</sup>
                {v.text}
              </li>
            ))}
          </ol>
          {!data.complete && <Notice>{t("scripture.document.passages.restUnknown")}</Notice>}
          {data.origin && (
            <div>
              <Button variant="quiet" onClick={() => onShow({ documentId: data.origin!.documentId, page: data.origin!.firstPage, position: data.origin!.firstSegment })}>
                {data.origin.firstPage !== null ? t("scripture.document.passages.showPage", { page: formatNumber(data.origin.firstPage) }) : t("scripture.document.passages.show")}
              </Button>
            </div>
          )}
        </>
      )}
      {data && data.locations.length > 0 && (
        <ul aria-label={t("scripture.document.passages.markedTitle")} className="m-0 flex list-none flex-col gap-1 p-0">
          {data.locations.map((place) => (
            <MarkedRow key={place.id} place={place} onShow={onShow} onRemoved={onChanged} />
          ))}
        </ul>
      )}
      {data && data.verses.length === 0 && data.locations.length === 0 && <p className="m-0">{t("scripture.document.passages.unknown", { passage: label })}</p>}
      {data && (
        <div className="flex flex-wrap gap-2">
          <Button variant="quiet" onClick={onMark}>
            <MapPin className="size-4" aria-hidden="true" />
            {t("scripture.document.passages.markHere")}
          </Button>
          {translation && !data.complete && (
            <Button variant="quiet" onClick={() => readInBible(reference)}>
              {t("scripture.document.passages.readIn", { name: translation.shortName })}
            </Button>
          )}
        </div>
      )}
    </section>
  );
}

function MarkedRow({ place, onShow, onRemoved }: { place: MarkedPlace; onShow: (target: DocumentTarget) => void; onRemoved: () => void }) {
  const { t } = useTranslation();
  const api = useApi();
  const [busy, setBusy] = useState(false);
  const [failed, setFailed] = useState<string | null>(null);
  const { bookByCode } = useScripture();
  const ref = refFromCanonical(place.reference, bookByCode);
  const where = place.page !== null ? t("scripture.document.passages.onPage", { page: formatNumber(place.page) }) : t("scripture.document.passages.inText");
  return (
    <li className="flex flex-wrap items-center gap-2">
      <Button variant="quiet" onClick={() => onShow({ documentId: place.documentId, page: place.page, position: place.segment })}>
        {t("scripture.document.passages.markedPlace", { passage: ref ? referenceLabel(ref) : place.reference, where })}
      </Button>
      {place.quote && <span className="type-meta">“{place.quote}”</span>}
      <Button
        variant="quiet"
        disabled={busy}
        aria-label={t("scripture.document.passages.unmark", { passage: ref ? referenceLabel(ref) : place.reference })}
        onClick={async () => {
          setBusy(true);
          setFailed(null);
          try {
            await api.bibleLocationRemove(place.id);
            onRemoved();
          } catch (e) {
            setFailed(toApiError(e).message);
            setBusy(false);
          }
        }}
      >
        {t("scripture.document.passages.unmarkShort")}
      </Button>
      {failed && (
        <span role="alert" className="text-danger">
          {failed}
        </span>
      )}
    </li>
  );
}

/**
 * Marks where a passage is (v0.24): the place the reader is at now (the PDF
 * page, or the paragraph or part shown), and the passage it holds. The
 * words selected there, if any, are kept with it as a check.
 */
export function MarkPassage({
  bible,
  documents,
  here,
  initial,
  onDone,
  onCancel,
}: {
  bible: BibleTranslation;
  documents: BibleDocument[];
  here: () => DocumentHere | null;
  initial: ScriptureRef | null;
  onDone: () => void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const { bookByCode } = useScripture();
  const id = useId();
  const [place] = useState(here);
  const [text, setText] = useState(initial ? referenceLabel(initial) : "");
  const [quote] = useState(() => (typeof window === "undefined" ? "" : (window.getSelection()?.toString() ?? "").trim().slice(0, 500)));
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const document = documents.find((d) => d.id === place?.documentId);
  const where = !place ? null : place.page !== null ? t("scripture.document.passages.onPage", { page: formatNumber(place.page) }) : t("scripture.document.passages.inText");
  const save = async (e: FormEvent) => {
    e.preventDefault();
    if (!place || saving) return;
    setSaving(true);
    try {
      const parsed = await api.referenceParse(text, bible.id);
      const ref = refFromCanonical(parsed.canonical, bookByCode) ?? parsed;
      await api.bibleLocationAdd({ translationId: bible.id, reference: ref.canonical, documentId: place.documentId, page: place.page, segment: place.segment, item: place.item, quote: quote || null });
      onDone();
    } catch (err) {
      setError(toApiError(err).message);
      setSaving(false);
    }
  };
  return (
    <form onSubmit={save} aria-label={t("scripture.document.passages.markTitle")} className="flex shrink-0 flex-col gap-2 border-b border-border-subtle bg-hover px-5 py-3">
      <p className="m-0 font-medium">{t("scripture.document.passages.markTitle")}</p>
      {place ? (
        <p className="m-0 text-[0.9375rem]">{t("scripture.document.passages.markWhere", { where, name: document?.name ?? "" })}</p>
      ) : (
        <Notice>{t("scripture.document.passages.markNoPlace")}</Notice>
      )}
      {quote && <p className="m-0 type-meta">{t("scripture.document.passages.markQuote", { quote })}</p>}
      <label htmlFor={id} className="flex flex-col gap-1">
        <span className="text-[0.9375rem] font-medium">{t("scripture.document.passages.markPassage")}</span>
        <input
          id={id}
          value={text}
          onChange={(e) => {
            setText(e.target.value);
            setError(null);
          }}
          placeholder={t("scripture.document.passages.goPlaceholder")}
          className="h-9 aw-input px-3"
        />
      </label>
      <p className="m-0 type-meta">{t("scripture.document.passages.markHint")}</p>
      {error && (
        <p role="alert" className="m-0 text-danger">
          {error}
        </p>
      )}
      <div className="flex gap-2">
        <Button type="submit" variant="primary" disabled={!place || !text.trim() || saving}>
          {t("scripture.document.passages.markSave")}
        </Button>
        <Button onClick={onCancel}>{t("common.cancel")}</Button>
      </div>
    </form>
  );
}
