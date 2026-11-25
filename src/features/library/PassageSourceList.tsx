import { useId, useRef } from "react";
import { useTranslation } from "react-i18next";
import { bookLabel } from "@/i18n/scripture";
import type { PassageLocation, PassageSourceGroup } from "@/types/library";
import type { ScriptureRef } from "@/types/scripture";
import { coverageLabel, locatorLabel, passageReason } from "./labels";
import type { ExpansionState } from "./usePassageSourceExpansion";
import { SourceEditButton } from "./SourceMetadataEditor";

interface PassageSourceListProps {
  groups: PassageSourceGroup[];
  /** Accessible name of the list. */
  label: string;
  /** The passage a link points at, written in the interface language. */
  passageLabel: (reference: ScriptureRef) => string;
  onOpen: (sourceId: string, segmentId?: string, structureNodeId?: string) => void;
  onEdit?: (sourceId: string) => void;
  /** Which sources show their further places. */
  expansion?: Record<string, ExpansionState>;
  onToggleMore?: (sourceId: string) => void;
}

/**
 * Sources that fit the passage being read: one entry per source (not one per
 * paragraph), its best place first and why it was chosen, a few more places
 * beneath, and a count of the rest.
 */
export function PassageSourceList({
  groups,
  label,
  passageLabel,
  onOpen,
  onEdit,
  expansion = {},
  onToggleMore,
}: PassageSourceListProps) {
  const { t } = useTranslation();
  const prefix = useId();
  const controls = useRef<Record<string, HTMLButtonElement | null>>({});
  const reason = (place: PassageLocation) => {
    // A.4.1: say how broad the grounded heading is (#22): a book section is
    // broader than the passage, a chapter heading names the chapter.
    const reason = place.origin === "section" && place.structureScope === "book"
      ? t("library.passage.bookSection", { passage: bookLabel(place.reference.book, false) })
      : place.origin === "section" && place.structureScope === "chapter"
        ? t("library.passage.chapterSection", { passage: passageLabel(place.reference) })
        : passageReason(t, place.origin, passageLabel(place.reference));
    const basis = place.structureBasis;
    if(basis === "outline" || basis === "printed" || basis === "tagged" || basis === "layout" || basis === "user") return `${reason} · ${t(`library.structure.basis.${basis}`)}`;
    return reason;
  };
  return (
    <ul aria-label={label} className="m-0 -mx-2 flex list-none flex-col gap-1 p-0">
      {groups.map((group, index) => {
        const coverage = group.coverage ?? [];
        const precise = group.locations.find((place) => place.origin === "section" || place.origin === "manual");
        const best = precise ?? (coverage.length === 0 ? group.locations[0] : undefined);
        if (!best && coverage.length === 0) return null;
        // Whole-source relevance opens without a target. Every literal mention
        // stays a separate choice, including the first one in the native result.
        const others = best ? group.locations.filter((place) => place !== best) : group.locations;
        const titleId = `${prefix}-${index}`;
        const moreId = `${prefix}-${index}-more`;
        const more = expansion[group.sourceId];
        const extra = more?.open ? [...others, ...(more.places ?? []).slice(group.locations.length)] : [];
        return (
          <li key={group.sourceId} className="border-b border-border-subtle pb-1">
            <div role="group" aria-labelledby={titleId}>
              <div className="flex items-start">
              <button
                type="button"
                data-source-result={group.sourceId}
                data-source-segment={best?.segmentId}
                onClick={() => best ? onOpen(group.sourceId, best.segmentId, best.structureNodeId ?? undefined) : onOpen(group.sourceId)}
                className="flex min-w-0 flex-1 flex-col items-start gap-0.5 rounded-aw px-2 py-2 text-left hover:bg-hover"
              >
                {/* A source's name is readable, not loud: many of them stand one under another. */}
                <span id={titleId} className="font-medium text-text">
                  <cite className="not-italic">{group.sourceTitle}</cite>
                </span>
                {(group.author || best) && <span className="type-meta">
                  {[group.author, best && locatorLabel(t, best.locator)].filter(Boolean).join(t("common.listSeparator"))}
                </span>}
                {coverage.length > 0 && <span className="type-meta">{t("library.coverage.covers", { passage: coverage.map(coverageLabel).join(t("common.listSeparator")) })}</span>}
                <span className="type-meta">{best ? reason(best) : t("library.coverage.openSource")}</span>
                {best?.structureBasis && best.sectionHeading && <span className="type-meta break-words">{best.sectionHeading}</span>}
              </button>
              {onEdit && <SourceEditButton title={group.sourceTitle} onClick={() => onEdit(group.sourceId)} />}
              </div>
              {others.length + group.moreLocations > 0 && onToggleMore && (
                <div
                  className="pb-2 pl-3"
                  onKeyDown={(event) => {
                    if (event.key === "Escape" && more?.open) {
                      event.stopPropagation();
                      onToggleMore(group.sourceId);
                      controls.current[group.sourceId]?.focus();
                    }
                  }}
                >
                  {extra.length > 0 && (
                    <ul id={moreId} className="m-0 mb-1 flex list-none flex-col border-l border-border-subtle p-0">
                      {extra.map((place) => (
                        <li key={place.segmentId}>
                          <button
                            type="button"
                            data-source-result={group.sourceId}
                            data-source-segment={place.segmentId}
                            onClick={() => onOpen(group.sourceId, place.segmentId, place.structureNodeId ?? undefined)}
                            className="flex w-full flex-wrap items-baseline gap-x-2 rounded-aw py-1 pl-3 text-left text-[0.875rem] hover:bg-hover"
                          >
                            <span className="text-text">{locatorLabel(t, place.locator)}</span>
                            <span className="text-text-muted">{reason(place)}</span>
                          </button>
                        </li>
                      ))}
                    </ul>
                  )}
                  <button
                    type="button"
                    ref={(node) => {
                      controls.current[group.sourceId] = node;
                    }}
                    aria-expanded={Boolean(more?.open)}
                    aria-controls={extra.length > 0 ? moreId : undefined}
                    onClick={() => onToggleMore(group.sourceId)}
                    className="rounded-sm text-left type-meta underline-offset-2 hover:text-text-strong hover:underline"
                  >
                    {more?.open
                      ? t("library.pane.showFewerLocations")
                      : t(best ? "library.pane.moreLocations" : "library.coverage.mentions", { count: others.length + group.moreLocations })}
                  </button>
                  {more?.status === "loading" && (
                    <p role="status" className="m-0 pt-1 type-meta">
                      {t("library.pane.loadingLocations")}
                    </p>
                  )}
                  {more?.status === "error" && (
                    <p role="alert" className="m-0 pt-1 text-[0.875rem] text-danger">
                      {t("library.pane.locationsFailed")}
                    </p>
                  )}
                </div>
              )}
            </div>
          </li>
        );
      })}
    </ul>
  );
}
