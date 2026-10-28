import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, Copy, Plus, Trash2 } from "lucide-react";
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

/**
 * The block editor for a teaching started from a block template. Teachings
 * with no blocks keep the earlier canvas. Private leader notes are marked
 * as such; a handout includes a block only when it is for participants.
 */
export function BlockCanvas({
  prep,
  findings,
  assistant,
  onChanged,
  onRevealScripture,
}: {
  prep: TeachingPreparation;
  findings: Finding[];
  assistant: AssistantHooks | null;
  onChanged: (prep: TeachingPreparation) => void;
  onRevealScripture: () => void;
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

  const renderBlock = (block: TeachingBlock, sectionTitle: string | null) => {
    const name = block.title.trim() || blockTypeLabel(t, block.blockType);
    const guidance = block.guidanceKey ? t(block.guidanceKey as never) : "";
    const sameAsSection = sectionTitle !== null && block.title.trim() === sectionTitle;
    if (block.blockType === "section") {
      const nested = children(block.id);
      return (
        <section key={block.id} aria-labelledby={`${id}-${block.id}`} className="flex flex-col gap-3">
          <div className="flex flex-wrap items-center gap-1">
            <h3 id={`${id}-${block.id}`} className="type-section m-0 min-w-0 flex-1">
              {name}
            </h3>
            <BlockTools id={id} block={block} name={name} onMove={move} onDuplicate={() => void act(() => api.teachingBlockDuplicate(block.id))} onDelete={() => void act(() => api.teachingBlockDelete(block.id))} />
          </div>
          {guidance && (
            <details className="text-[0.9375rem] text-text-muted">
              <summary className="cursor-pointer">{t("studies.teaching.blocks.guidance")}</summary>
              <p className="m-0 mt-1 max-w-prose">{guidance}</p>
            </details>
          )}
          {nested.map((child) => renderBlock(child, name))}
          <AddBlock label={t("studies.teaching.blocks.addIn", { section: name })} onAdd={(type) => add(type, block.id)} />
        </section>
      );
    }
    return (
      <div key={block.id} className="flex flex-col gap-2">
        <div className="flex flex-wrap items-start gap-1">
          <div className="min-w-0 flex-1">
            {block.blockType === "scripture" ? (
              <div className="flex flex-col gap-1">
                <p className="type-subhead m-0">{name}</p>
                <TeachingLinks preparation={prep} pointId={null} blockId={block.id} scope={name} studyFindings={findings} onChanged={onChanged} onRevealScripture={onRevealScripture} />
              </div>
            ) : (
              <TeachingField
                label={name}
                labelHidden={sameAsSection}
                value={block.body}
                save={(value) => api.teachingBlockUpdate({ id: block.id, body: value }).then((next) => (onChanged(next), next))}
                part={assistPart(block.role)}
                rows={block.blockType === "teaching_notes" || block.blockType === "discussion_questions" ? 4 : 6}
                placeholder={t("studies.teaching.blocks.placeholder")}
                assistant={assistant}
                onRevealScripture={onRevealScripture}
              />
            )}
          </div>
          <BlockTools id={id} block={block} name={name} onMove={move} onDuplicate={() => void act(() => api.teachingBlockDuplicate(block.id))} onDelete={() => void act(() => api.teachingBlockDelete(block.id))} />
        </div>
        {!block.participantFacing && <p className="m-0 text-[0.875rem] text-text-muted">{t("studies.teaching.blocks.leaderOnly")}</p>}
        <details className="text-[0.9375rem]">
          <summary className="cursor-pointer text-text-muted">{t("studies.teaching.blocks.settings")}</summary>
          <label className="mt-2 flex min-h-11 items-center gap-2">
            <input
              type="checkbox"
              checked={block.participantFacing}
              onChange={(e) => void act(() => api.teachingBlockUpdate({ id: block.id, participantFacing: e.target.checked }))}
            />
            {t("studies.teaching.blocks.participant")}
          </label>
        </details>
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

function AddBlock({ label, onAdd }: { label: string; onAdd: (type: string) => void }) {
  const { t } = useTranslation();
  const id = useId();
  const [open, setOpen] = useState(false);
  return (
    <div className="relative">
      <Button type="button" variant="quiet" aria-expanded={open} aria-controls={`${id}-menu`} onClick={() => setOpen((v) => !v)}>
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
