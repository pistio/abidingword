import { useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { LocateFixed } from "lucide-react";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { useScripture } from "@/features/scripture/ScriptureContext";
import { bookLabel, referenceLabel } from "@/i18n/scripture";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import type { PassageLocation, SegmentWindow } from "@/types/library";
import type { ScriptureRef } from "@/types/scripture";
import { locatorLabel, passageReason } from "./labels";

interface Outcome {
  key: string;
  busy: boolean;
  choices?: PassageLocation[];
  empty?: boolean;
  error?: string;
}

interface NavigationRequest {
  isCurrent: () => boolean;
  complete: () => void;
}

/**
 * Scripture changes only the label here. A deliberate request may navigate this
 * source to a grounded section, or offer literal mentions for the reader to choose.
 * Both matching and target loading belong to that request: stale responses never move the reader.
 */
export function SourcePassageNavigation({ sourceId, enabled, prepareEpubContents, beginNavigation, onNavigate }: {
  sourceId: string;
  enabled: boolean;
  prepareEpubContents: boolean;
  /** Shared with the reader's Contents, history, page and view navigation. */
  beginNavigation: (onCancel?: () => void) => NavigationRequest;
  onNavigate: (location: PassageLocation, window: SegmentWindow) => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const { currentPassage, current, books, translation, documentBible, documentPassage } = useScripture();
  const requested = documentBible ? documentPassage?.reference ?? null : translation ? currentPassage ?? current : null;
  const key = JSON.stringify([sourceId, requested?.canonical ?? null]);
  const label = (reference: ScriptureRef) => referenceLabel(reference, books.find((book) => book.code === reference.book)?.chapterCount);
  const passage = requested ? label(requested) : null;
  const matchLabel = (location: PassageLocation) => location.origin === "section" && location.structureScope === "book"
    ? t("library.passage.bookSection", { passage: bookLabel(location.reference.book, false) })
    : location.origin === "section" && location.structureScope === "chapter"
      ? t("library.passage.chapterSection", { passage: label(location.reference) })
      : label(location.reference);
  const [outcome, setOutcome] = useState<Outcome | null>(null);
  const shown = outcome?.key === key ? outcome : null;
  const [opened, setOpened] = useState<{ sourceId: string; location: PassageLocation } | null>(null);
  const operation = useRef(0);
  const navigate = useRef(onNavigate);
  useLayoutEffect(() => { navigate.current = onNavigate; });
  useLayoutEffect(() => {
    operation.current += 1;
    return () => { operation.current += 1; };
  }, [key]);

  const startRequest = () => {
    const request = beginNavigation(() => {
      operation.current += 1;
      setOutcome(null);
    });
    return { request, ticket: ++operation.current };
  };
  const openLocation = async (location: PassageLocation, ticket: number, request: NavigationRequest, choices?: PassageLocation[]) => {
    setOutcome({ key, busy: true, choices });
    try {
      const loaded = await api.libraryOpenSegment(location.segmentId);
      if (ticket !== operation.current || !request.isCurrent()) return;
      if (loaded.sourceId !== sourceId || loaded.targetSegmentId !== location.segmentId || !loaded.segments.some((segment) => segment.id === location.segmentId)) {
        throw new Error(t("library.reader.targetUnavailable"));
      }
      // The reader applies the loaded target synchronously. It starts its own
      // arrival generation, so release this request's cancellation callback first.
      request.complete();
      navigate.current(location, loaded);
      setOpened({ sourceId, location });
      setOutcome({ key, busy: false });
    } catch (error) {
      if (ticket === operation.current) setOutcome({ key, busy: false, choices, error: toApiError(error).message });
    } finally {
      request.complete();
    }
  };
  const findPassage = async () => {
    if (!requested || !enabled) return;
    const { request, ticket } = startRequest();
    setOutcome({ key, busy: true });
    try {
      // Opening an older EPUB refreshes its derived TOC links. Finish that bounded
      // read first so a fast click cannot search the previous section index.
      if (prepareEpubContents) {
        await api.librarySourceContents(sourceId);
        if (ticket !== operation.current || !request.isCurrent()) return;
      }
      const locations = await api.libraryPassageSourceLocations(sourceId, requested.canonical);
      if (ticket !== operation.current || !request.isCurrent()) return;
      // The native resolver ranks grounded destinations first, deepest reliable ancestor first.
      // Inherited context is not a destination; incidental mentions always need a separate choice.
      const destination = locations.find((location) => location.origin === "section" || location.origin === "manual");
      if (destination) await openLocation(destination, ticket, request);
      else {
        const choices = locations.filter((location) => location.origin === "explicit");
        setOutcome({ key, busy: false, ...(choices.length > 0 ? { choices } : { empty: true }) });
      }
    } catch (error) {
      if (ticket === operation.current) setOutcome({ key, busy: false, error: toApiError(error).message });
    } finally {
      request.complete();
    }
  };
  const closeChoices = () => {
    operation.current += 1;
    setOutcome({ key, busy: false });
  };
  const choices = shown?.choices;
  return <>
    <div className="shrink-0 border-b border-border-subtle px-3 py-1 text-[0.875rem]" aria-busy={shown?.busy ?? false}>
      <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
        <Button variant="quiet" onClick={() => void findPassage()} disabled={!enabled || !requested || shown?.busy} title={t("library.sync.help")}>
          <LocateFixed className="size-4 shrink-0" aria-hidden="true" />{t("library.sync.go")}
        </Button>
        <span className="text-text-muted">{passage ? t("library.sync.scripture", { passage }) : t("library.sync.noPassage")}</span>
      </div>
      <p role="status" className="m-0 pb-1 text-text-muted">
        {shown?.busy ? t("library.sync.loading") : shown?.empty ? t("library.sync.noMatch", { passage }) : opened?.sourceId === sourceId ? t("library.sync.opened", { passage: matchLabel(opened.location) }) : t("library.sync.help")}
      </p>
      {shown?.error && !choices && <Notice tone="error">{shown.error}</Notice>}
    </div>
    {choices && <Dialog wide title={t("library.sync.mentionsTitle", { passage })} onClose={closeChoices} actions={<Button onClick={closeChoices}>{t("common.cancel")}</Button>}>
      <p className="m-0 text-text-muted">{t("library.sync.mentionsHelp")}</p>
      {shown.error && <Notice tone="error">{shown.error}</Notice>}
      <ul aria-label={t("library.sync.mentionsTitle", { passage })} className="m-0 flex list-none flex-col gap-1 p-0">
        {choices.map((location, index) => <li key={`${location.segmentId}:${index}`}>
          <button type="button" disabled={shown.busy} onClick={() => { const { request, ticket } = startRequest(); void openLocation(location, ticket, request, choices); }} className="flex min-h-11 w-full flex-col items-start gap-1 rounded-aw px-3 py-2 text-left hover:bg-hover disabled:opacity-50">
            <span className="font-medium">{locatorLabel(t, location.locator)}</span>
            <span className="text-[0.875rem] text-text-muted">{passageReason(t, location.origin, label(location.reference))}</span>
            {location.sectionHeading && <span className="text-[0.875rem] text-text-muted">{location.sectionHeading}</span>}
          </button>
        </li>)}
      </ul>
    </Dialog>}
  </>;
}
