import { useId, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, Copy, Info, Lock, MoreHorizontal, Plus, Trash2 } from "lucide-react";
import { Button } from "@/design-system/Button";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { TEACHING_BLOCK_TYPES, type TeachingBlock, type TeachingPart, type TeachingPreparation } from "@/types/teaching";
import { TeachingField, type AssistantHooks } from "./TeachingField";

function blockTypeLabel(t: ReturnType<typeof useTranslation>["t"], blockType: string): string {
  return (t as unknown as (key: string) => string)(`studies.teaching.blocks.types.${blockType}`);
}

function assistPart(role: string | null): TeachingPart {
  if (role === "introduction" || role === "conclusion" || role === "notes") return role;
  if (role === "central_message") return "centralMessage";
  if (role === "application") return "pointApplication";
  if (role === "illustration") return "pointIllustration";
  return "pointExplanation";
}
import { TeachingLinks } from "./TeachingLinks";
import type { Finding } from "@/types/studies";

export type CanvasView = "continuous" | "outline";

function writingRows(blockType: string, view: CanvasView): number {
  const spacious = blockType === "teaching_notes" || blockType === "discussion_questions" ? 4 : 8;
  if (view === "outline") return blockType === "outline_point" ? 6 : 3;
  return blockType === "outline_point" ? 3 : spacious;
}

/**
 * The block editor for a teaching started from a block template. Teachings
 * with no blocks keep the earlier canvas. Private leader notes are marked
 * as such; a handout includes a block only when it is for participants.
 * Continuous and outline views show the same stored blocks.
 */
export function BlockCanvas({
  prep,
  findings,
  assistant,
  onChanged,
  onRevealScripture,
  view = "continuous",
}: {
  prep: TeachingPreparation;
  findings: Finding[];
  assistant: AssistantHooks | null;
  onChanged: (prep: TeachingPreparation) => void;
  onRevealScripture: () => void;
  /** A.4.3: chosen in the teaching's options (writing continuously is the default). */
  view?: CanvasView;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const [error, setError] = useState<string | null>(null);
  const blocks = prep.blocks ?? [];
  const act = async (run: () => Promise<TeachingPreparation>) => {
    setError(null);
    try {
      onChanged(await run());
    } catch (e) {
      setError(toApiError(e).message);
    }
  };
  const children = (parentId: string | null) => blocks.filter((b) => (b.parentId ?? null) === parentId).sort((a, b) => a.position - b.position);
  const move = (block: TeachingBlock, delta: -1 | 1) => {
    const siblings = children(block.parentId);
    const index = siblings.findIndex((b) => b.id === block.id);
    const target = index + delta;
    if (index < 0 || target < 0 || target >= siblings.length) return;
    const ids = siblings.map((b) => b.id);
    [ids[index], ids[target]] = [ids[target]!, ids[index]!];
    void act(() => api.teachingBlockReorder({ preparationId: prep.id, parentId: block.parentId, ids }));
  };
  const add = (blockType: string, parentId: string | null) => {
    const siblings = children(parentId);
    void act(() =>
      api.teachingBlockCreate({
        preparationId: prep.id,
        parentId,
        afterBlockId: siblings.at(-1)?.id ?? null,
        blockType,
        title: "",
      }),
    );
  };

  const renderBlock = (block: TeachingBlock, sectionTitle: string | null): ReactNode => {
    // A question-and-answer block is named by its kind; its question is the first field.
    const name = block.blockType === "qa" ? blockTypeLabel(t, "qa") : block.title.trim() || blockTypeLabel(t, block.blockType);
    const guidance = block.guidanceKey ? t(block.guidanceKey as never) : "";
    const sameAsSection = sectionTitle !== null && block.title.trim() === sectionTitle;
    const frame = {
      id,
      block,
      name,
      guidance,
      onMove: move,
      onDuplicate: () => void act(() => api.teachingBlockDuplicate(block.id)),
      onDelete: () => void act(() => api.teachingBlockDelete(block.id)),
      onVisibility: (participantFacing: boolean) => void act(() => api.teachingBlockUpdate({ id: block.id, participantFacing })),
      onTitle: (title: string) => void act(() => api.teachingBlockUpdate({ id: block.id, title })),
    };
    if (block.blockType === "section") {
      const nested = children(block.id);
      // A section holding one block of the same name is one unit: one heading, one mark, one "…".
      const sole = nested.length === 1 && nested[0]!.blockType !== "section" && nested[0]!.blockType !== "scripture" && nested[0]!.title.trim() === name ? nested[0]! : null;
      if (sole) {
        return (
          <section key={block.id} aria-labelledby={`${id}-${block.id}`} className="flex flex-col gap-2">
            <BlockFrame
              {...frame}
              content={sole}
              guidance={frame.guidance || (sole.guidanceKey ? t(sole.guidanceKey as never) : "")}
              onVisibility={(participantFacing) => void act(() => api.teachingBlockUpdate({ id: sole.id, participantFacing }))}
              onTitle={(title) => void act(() => api.teachingBlockUpdate({ id: sole.id, title }))}
              heading={<h3 id={`${id}-${block.id}`} className="type-section m-0 min-w-0">{name}</h3>}
              onAdd={(type) => add(type, block.id)}
            >
              <div role="group" aria-label={name}>
                <TeachingField
                  label={name}
                  labelHidden
                  blockId={sole.id}
                  value={sole.body}
                  save={(value) => api.teachingBlockUpdate({ id: sole.id, body: value }).then((next) => (onChanged(next), next))}
                  part={assistPart(sole.role)}
                  rows={writingRows(sole.blockType, view)}
                  placeholder={t("studies.teaching.blocks.placeholder")}
                  assistant={assistant}
                  onRevealScripture={onRevealScripture}
                />
              </div>
            </BlockFrame>
          </section>
        );
      }
      return (
        <section key={block.id} aria-labelledby={`${id}-${block.id}`} className="flex flex-col gap-3">
          <BlockFrame {...frame} heading={<h3 id={`${id}-${block.id}`} className="type-section m-0 min-w-0">{name}</h3>} onAdd={(type) => add(type, block.id)}>
            {nested.map((child) => renderBlock(child, name))}
          </BlockFrame>
        </section>
      );
    }
    const field = block.blockType === "qa" ? (
      <div className="flex flex-col gap-2" data-teaching-qa={block.id}>
        <TeachingField
          label={t("studies.qa.question")}
          level="field"
          value={block.title}
          save={(value) => api.teachingBlockUpdate({ id: block.id, title: value }).then((next) => (onChanged(next), next))}
          part={assistPart(block.role)}
          blockId={block.id}
          rows={2}
          assistant={assistant}
          onRevealScripture={onRevealScripture}
        />
        <TeachingField
          label={t("studies.qa.answer")}
          level="field"
          value={block.body}
          save={(value) => api.teachingBlockUpdate({ id: block.id, body: value }).then((next) => (onChanged(next), next))}
          part={assistPart(block.role)}
          blockId={block.id}
          rows={5}
          assistant={assistant}
          onRevealScripture={onRevealScripture}
        />
      </div>
    ) : block.blockType === "scripture" ? (
      <TeachingLinks preparation={prep} pointId={null} blockId={block.id} scope={name} studyFindings={findings} onChanged={onChanged} onRevealScripture={onRevealScripture} />
    ) : (
      <>
        {block.blockType === "outline_point" && (
          <TeachingField
            label={t("studies.teaching.blocks.headingLabel")}
            value={block.title}
            save={(value) => api.teachingBlockUpdate({ id: block.id, title: value }).then((next) => (onChanged(next), next))}
            part={assistPart(block.role)}
            multiline={false}
            assistant={null}
            onRevealScripture={onRevealScripture}
          />
        )}
        <TeachingField
          label={name}
          labelHidden
          blockId={block.id}
          value={block.body}
          save={(value) => api.teachingBlockUpdate({ id: block.id, body: value }).then((next) => (onChanged(next), next))}
          part={assistPart(block.role)}
          rows={writingRows(block.blockType, view)}
          placeholder={t("studies.teaching.blocks.placeholder")}
          assistant={assistant}
          onRevealScripture={onRevealScripture}
        />
      </>
    );
    return (
      <div key={block.id} role="group" aria-label={name} className="flex flex-col gap-1">
        {/* A block named like its section needs no second heading; its label is still read out. */}
        <BlockFrame {...frame} heading={sameAsSection ? <span className="sr-only">{name}</span> : <p className="type-subhead m-0 min-w-0">{name}</p>}>
          {field}
        </BlockFrame>
      </div>
    );
  };

  const top = children(null);
  return (
    <div className="flex flex-col gap-6">
      {top.length === 0 && <p className="m-0 text-text-muted">{t("studies.teaching.blocks.empty")}</p>}
      {top.map((block) => renderBlock(block, null))}
      <AddBlock label={t("studies.teaching.blocks.add")} onAdd={(type) => add(type, null)} />
      {error && <p role="alert" className="m-0 text-danger">{error}</p>}
    </div>
  );
}

/** The view choice for a block teaching, shown in the teaching's options (A.4.3). */
export function CanvasViewChoice({ view, onView }: { view: CanvasView; onView: (view: CanvasView) => void }) {
  const { t } = useTranslation();
  const id = useId();
  return (
    <div role="radiogroup" aria-label={t("studies.teaching.blocks.viewLabel")} className="flex flex-wrap gap-1">
      {(["continuous", "outline"] as const).map((mode) => (
        <label key={mode} className="aw-choice aw-choice-card min-h-11 cursor-pointer gap-2 px-3">
          <input type="radio" name={`${id}-view`} checked={view === mode} onChange={() => onView(mode)} className="sr-only" />
          {t(mode === "continuous" ? "studies.teaching.blocks.viewContinuous" : "studies.teaching.blocks.viewOutline")}
        </label>
      ))}
    </div>
  );
}

/**
 * A.4.3 (#25): one block of a teaching, writing first. Its heading, a quiet
 * but persistent "Leader only" mark on private content, and one "…" that
 * holds everything else: guidance, who it is for, its heading, moving,
 * duplicating, adding inside a section and deleting (with a named
 * confirmation). Nothing changes visibility silently.
 */
function BlockFrame({
  id,
  block,
  content,
  name,
  guidance,
  heading,
  children,
  onMove,
  onDuplicate,
  onDelete,
  onVisibility,
  onTitle,
  onAdd,
}: {
  id: string;
  block: TeachingBlock;
  /** The writing block shown inside this frame when it is one unit with its section (A.4.3). */
  content?: TeachingBlock;
  name: string;
  guidance: string;
  heading: ReactNode;
  children: ReactNode;
  onMove: (block: TeachingBlock, delta: -1 | 1) => void;
  onDuplicate: () => void;
  onDelete: () => void;
  onVisibility: (participantFacing: boolean) => void;
  onTitle: (title: string) => void;
  onAdd?: (type: string) => void;
}) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [hints, setHints] = useState(false);
  const [confirming, setConfirming] = useState(false);
  const toggle = useRef<HTMLButtonElement>(null);
  const panelId = `${id}-${block.id}-options`;
  const writing = content ?? block;
  // Private if the writing is for the leader (or its whole section is).
  const leaderOnly = !writing.participantFacing || !block.participantFacing;
  return (
    <>
      <div className="flex min-h-11 items-center gap-2">
        <div className="min-w-0 flex-1">{heading}</div>
        {leaderOnly && (
          <span data-leader-only className="inline-flex shrink-0 items-center gap-1 rounded-full border border-border-subtle px-2 py-0.5 text-[0.8125rem] text-text-muted" title={t("studies.teaching.blocks.leaderOnly")}>
            <Lock className="size-3" aria-hidden="true" />
            {t("studies.teaching.blocks.leaderBadge")}
          </span>
        )}
        <button
          ref={toggle}
          type="button"
          aria-expanded={open}
          aria-controls={open ? panelId : undefined}
          aria-label={t("studies.teaching.blocks.options", { name })}
          title={t("studies.teaching.blocks.options", { name })}
          onClick={() => setOpen((v) => !v)}
          className="inline-flex size-11 shrink-0 items-center justify-center rounded-aw text-text-muted hover:bg-hover hover:text-text-strong aria-expanded:bg-selected"
        >
          <MoreHorizontal className="size-4" aria-hidden="true" />
        </button>
      </div>
      {open && (
        <div
          id={panelId}
          role="group"
          aria-label={t("studies.teaching.blocks.options", { name })}
          onKeyDown={(e) => { if (e.key === "Escape") { e.stopPropagation(); setOpen(false); toggle.current?.focus(); } }}
          className="flex flex-col gap-2 rounded-aw bg-bg-secondary px-2 py-2 text-[0.9375rem]"
        >
          <div className="flex flex-wrap items-center gap-1">
            {guidance && (
              <button type="button" aria-pressed={hints} onClick={() => setHints(!hints)} className="inline-flex min-h-11 items-center gap-1 rounded-aw px-2 text-text-muted hover:bg-hover hover:text-text-strong">
                <Info className="size-4" aria-hidden="true" />
                {t("studies.teaching.blocks.guidance")}
              </button>
            )}
            <BlockTools id={id} block={block} name={name} onMove={onMove} onDuplicate={onDuplicate} onDelete={() => setConfirming(true)} />
            {onAdd && <AddBlock label={t("studies.teaching.blocks.addIn", { section: name })} onAdd={onAdd} />}
          </div>
          <label className="flex min-h-11 items-center gap-2">
            <input type="checkbox" checked={writing.participantFacing} onChange={(e) => onVisibility(e.target.checked)} />
            {t("studies.teaching.blocks.participant")}
          </label>
          {writing.blockType !== "section" && writing.blockType !== "qa" && (
            <label className="flex flex-col gap-1">
              <span>{t("studies.teaching.blocks.headingLabel")}</span>
              <input
                defaultValue={writing.title}
                aria-label={t("studies.teaching.blocks.titleOf", { name })}
                onBlur={(e) => { if (e.target.value !== writing.title) onTitle(e.target.value); }}
                className="aw-input h-10 px-3 text-base"
              />
            </label>
          )}
        </div>
      )}
      {confirming && (
        <div role="alertdialog" aria-label={t("studies.teaching.blocks.confirmDelete", { name })} className="flex flex-wrap items-center gap-2 border-l-[3px] border-danger bg-hover px-3 py-2">
          <span>{t("studies.teaching.blocks.confirmDelete", { name })}</span>
          <Button variant="danger" onClick={() => { setConfirming(false); onDelete(); }}>{t("common.delete")}</Button>
          <Button autoFocus onClick={() => setConfirming(false)}>{t("common.keep")}</Button>
        </div>
      )}
      {writing.origin && (
        <p data-origin={writing.origin.kind} className="type-meta m-0">
          {writing.origin.kind === "citation" && writing.origin.citation
            ? t("studies.teaching.fromStudy.quotation", { citation: writing.origin.citation })
            : t("studies.teaching.fromStudy.provenance", { label: writing.origin.label || t("studies.teaching.fromStudy.item") })}
        </p>
      )}
      {hints && guidance && <p data-block-hints className="m-0 max-w-prose border-l-[3px] border-border bg-bg-secondary px-3 py-2 text-[0.9375rem] text-text-muted">{guidance}</p>}
      {children}
    </>
  );
}

function BlockTools({
  id,
  block,
  name,
  onMove,
  onDuplicate,
  onDelete,
}: {
  id: string;
  block: TeachingBlock;
  name: string;
  onMove: (block: TeachingBlock, delta: -1 | 1) => void;
  onDuplicate: () => void;
  onDelete: () => void;
}) {
  const { t } = useTranslation();
  const tool = "inline-flex size-11 items-center justify-center rounded-aw text-text-muted hover:bg-hover";
  return (
    <span className="flex shrink-0">
      <button type="button" id={`${id}-${block.id}-up`} className={tool} aria-label={t("studies.teaching.blocks.moveUp", { name })} onClick={() => onMove(block, -1)}>
        <ArrowUp className="size-4" aria-hidden="true" />
      </button>
      <button type="button" id={`${id}-${block.id}-down`} className={tool} aria-label={t("studies.teaching.blocks.moveDown", { name })} onClick={() => onMove(block, 1)}>
        <ArrowDown className="size-4" aria-hidden="true" />
      </button>
      <button type="button" className={tool} aria-label={t("studies.teaching.blocks.duplicate", { name })} onClick={onDuplicate}>
        <Copy className="size-4" aria-hidden="true" />
      </button>
      <button type="button" className={tool} aria-label={t("studies.teaching.blocks.remove", { name })} onClick={onDelete}>
        <Trash2 className="size-4" aria-hidden="true" />
      </button>
    </span>
  );
}

export function AddBlock({ label, onAdd, id: buttonId }: { label: string; onAdd: (type: string) => void; id?: string }) {
  const { t } = useTranslation();
  const id = useId();
  const [open, setOpen] = useState(false);
  return (
    <div className="relative">
      <Button type="button" id={buttonId} variant="quiet" aria-expanded={open} aria-haspopup="menu" aria-controls={`${id}-menu`} onClick={() => setOpen((v) => !v)}>
        <Plus className="size-4" aria-hidden="true" />
        {label}
      </Button>
      {open && (
        <ul id={`${id}-menu`} role="menu" aria-label={label} className="absolute z-10 m-0 mt-1 max-h-80 list-none overflow-y-auto rounded-aw border border-border bg-bg p-1 shadow-none">
          {TEACHING_BLOCK_TYPES.map((type) => (
            <li key={type} role="none">
              <button
                type="button"
                role="menuitem"
                className="block w-full rounded-aw px-3 py-2 text-left text-[0.9375rem] hover:bg-hover"
                onClick={() => {
                  setOpen(false);
                  onAdd(type);
                }}
              >
                {t(`studies.teaching.blocks.types.${type}`)}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
