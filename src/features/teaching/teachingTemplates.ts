import i18n, { type TFunction } from "i18next";
import { LANGUAGES } from "@/i18n/languages";
import builtin from "../../../resources/teaching-templates.json";
import type { StepKind, TeachingBlock, TeachingKind, TeachingPreparation, TeachingTemplate, TeachingTemplateBlock, TeachingTemplatePoint } from "@/types/teaching";
import { canvasShape, shapeText, shapeWords, startingStepTitle } from "./labels";

/**
 * Bible Teaching templates (v0.18.7): what a new teaching starts with, its
 * format and its points, questions or steps. The built-in ones (one per
 * format, from resources/teaching-templates.json) are application text in
 * the person's language and cannot be changed; the person's own are kept by
 * the core. A teaching copies its template when it is made, and is never
 * linked to it afterwards.
 */
export interface BuiltinTeachingTemplate {
  key: string;
  kind: TeachingKind;
  points: { key: string; kind: StepKind }[];
}

interface RawTeachingTemplate {
  key: string;
  kind?: TeachingKind;
  version?: number;
  legacyKind?: TeachingKind;
  points?: { key: string; kind: StepKind }[];
  blocks?: { key: string; type: string; parentKey?: string; role?: string; participantFacing?: boolean; guidanceKey?: string }[];
}

const RAW_TEMPLATES = builtin.templates as RawTeachingTemplate[];

export const BUILTIN_TEACHING_TEMPLATES: readonly BuiltinTeachingTemplate[] = RAW_TEMPLATES.filter((t) => t.version !== 2).map((t) => ({
  key: t.key,
  kind: t.kind!,
  points: t.points ?? [],
}));

/** The three built-in block templates. The earlier formats stay in the file for teachings already made. */
export interface BuiltinBlockTemplate {
  key: string;
  legacyKind: TeachingKind;
  blocks: NonNullable<RawTeachingTemplate["blocks"]>;
}

export const BUILTIN_BLOCK_TEMPLATES: readonly BuiltinBlockTemplate[] = RAW_TEMPLATES.filter((t) => t.version === 2).map((t) => ({
  key: t.key,
  legacyKind: t.legacyKind!,
  blocks: t.blocks ?? [],
}));

/** The i18n group under studies.teaching.builtin for a built-in block template. */
export function blockTemplateGroup(key: string): "eva" | "sermonTalk" | "bibleLesson" | "blank" {
  if (key === "sermon-talk") return "sermonTalk";
  if (key === "bible-lesson") return "bibleLesson";
  if (key === "blank") return "blank";
  return "eva";
}

/** A built-in block's heading in the person's language. */
export function builtinBlockTitle(t: TFunction, templateKey: string, blockKey: string): string {
  const group = blockTemplateGroup(templateKey);
  return (t as unknown as (key: string) => string)(`studies.teaching.builtin.${group}.blocks.${blockKey}.title`);
}

/** The blocks a new teaching copies from a built-in template, titled in the person's language. */
export function blocksFromBuiltin(t: TFunction, template: BuiltinBlockTemplate): TeachingTemplateBlock[] {
  const sectionTitle = new Map(template.blocks.filter((b) => b.type === "section").map((b) => [b.key, builtinBlockTitle(t, template.key, b.key)]));
  return template.blocks.map((block) => ({
    key: block.key,
    parentKey: block.parentKey ?? null,
    blockType: block.type,
    title: block.type === "section" ? (sectionTitle.get(block.key) ?? "") : block.parentKey ? (sectionTitle.get(block.parentKey) ?? "") : "",
    ...(block.participantFacing === undefined ? {} : { participantFacing: block.participantFacing }),
    ...(block.role ? { role: block.role } : {}),
    ...(block.guidanceKey ? { guidanceKey: block.guidanceKey } : {}),
  }));
}

/** True when this teaching is edited as blocks. A teaching with neither a version nor blocks keeps the earlier editor. */
export function usesBlocks(prep: Pick<TeachingPreparation, "templateVersion" | "blocks">): boolean {
  return (prep.templateVersion ?? 0) >= 2 || (prep.blocks?.length ?? 0) > 0;
}

/** A template to start from: a built-in one, or one of the person's own. */
export type TeachingStart = { builtin: BuiltinTeachingTemplate } | { mine: TeachingTemplate };

export const startKind = (start: TeachingStart): TeachingKind => ("builtin" in start ? start.builtin.kind : start.mine.kind);

/**
 * The three formats a new teaching is offered in (v0.19), each a way of
 * teaching rather than an audience: a message (a sermon or a talk), a group
 * Bible study led by discussion, and an interactive lesson in steps for
 * children, youth or adults. Age and audience belong to the teaching's
 * information. The kinds stay those stored since v0.16: every one belongs to
 * exactly one format, so older teachings and templates keep their place.
 */
export const TEACHING_FORMATS = ["message", "group", "lesson"] as const;
export type TeachingFormat = (typeof TEACHING_FORMATS)[number];

export const FORMAT_KINDS: Record<TeachingFormat, readonly TeachingKind[]> = {
  message: ["sermon", "talk", "youth", "other"],
  group: ["bible_study"],
  lesson: ["lesson", "children"],
};

export const formatOf = (kind: TeachingKind): TeachingFormat => (TEACHING_FORMATS.find((f) => FORMAT_KINDS[f].includes(kind)) ?? "message");

/**
 * Where a new teaching of a format can start: the format's built-in starting
 * points first (a message: the sermon's or the talk's, as chosen; a lesson:
 * the lesson's and the children's), then the person's own templates of any
 * kind in that format. The kind of the teaching is the start's own.
 */
export function formatStarts(format: TeachingFormat, messageKind: "sermon" | "talk", mine: readonly TeachingTemplate[]): { id: string; start: TeachingStart }[] {
  const builtins = format === "message" ? [messageKind] : format === "group" ? (["bible_study"] as const) : (["lesson", "children"] as const);
  return [
    ...builtins.map((kind) => ({ id: `builtin:${kind}`, start: { builtin: BUILTIN_TEACHING_TEMPLATES.find((b) => b.kind === kind)! } as TeachingStart })),
    ...mine.filter((m) => FORMAT_KINDS[format].includes(m.kind)).map((m) => ({ id: m.id, start: { mine: m } as TeachingStart })),
  ];
}

/** The built-in starting steps a kind of teaching knows, by key. */
const builtinStep = (kind: TeachingKind, key: string | null | undefined) =>
  key ? BUILTIN_TEACHING_TEMPLATES.find((b) => b.kind === kind)?.points.find((p) => p.key === key) : undefined;

/**
 * A template point's title: a built-in step's in the person's language while
 * it is unchanged (as saved in any language, or left empty), else as written.
 */
export function templatePointTitle(t: TFunction, kind: TeachingKind, point: TeachingTemplatePoint): string {
  if (point.key && builtinStep(kind, point.key)) {
    const key = `studies.teaching.starts.${kind}.${point.key}`;
    const unchanged = !point.title.trim() || LANGUAGES.some(({ code }) => i18n.t(key as never, { lng: code }) === point.title);
    if (unchanged) return startingStepTitle(t, kind, point.key);
  }
  return point.title;
}

/** The points a new teaching starts with: titled in the person's language, with their kinds. */
export function startingPoints(t: TFunction, start: TeachingStart): { title: string; kind: StepKind | null }[] {
  if ("builtin" in start) return start.builtin.points.map((p) => ({ title: startingStepTitle(t, start.builtin.kind, p.key), kind: p.kind }));
  return start.mine.points.map((p) => ({ title: templatePointTitle(t, start.mine.kind, p), kind: p.kind ?? null }));
}

/** A built-in template's points as the person's own copy, in their language ("Save as my template"). */
export function builtinPoints(t: TFunction, template: BuiltinTeachingTemplate): TeachingTemplatePoint[] {
  return template.points.map((p) => ({ key: p.key, title: startingStepTitle(t, template.kind, p.key), kind: p.kind }));
}

/**
 * A teaching's structure as a template of the person's own ("Save as my
 * template"): its format and as many points, questions or steps as it has,
 * each step with its kind. The steps of a lesson keep their titles ("Welcome",
 * "Game"): they name what happens. The points of a sermon and the questions
 * of a group study are the teaching itself, so they start untitled. Nothing
 * written in the teaching comes with it: no audience, purpose, message,
 * introduction, conclusion, notes, explanation, application, illustration,
 * passages, findings or kept material.
 */
export function templateFromTeaching(
  prep: Pick<TeachingPreparation, "kind" | "points" | "templateVersion" | "blocks">,
): { kind: TeachingKind; points: TeachingTemplatePoint[]; version?: number; blocks?: TeachingTemplateBlock[] } {
  if (usesBlocks(prep)) {
    return {
      kind: prep.kind ?? "other",
      points: [],
      version: 2,
      blocks: templateBlocksFromTeaching(prep.blocks ?? []),
    };
  }
  const kind = prep.kind ?? "other";
  const steps = canvasShape(kind) === "steps";
  return {
    kind,
    points: prep.points.map((p) => ({
      title: steps ? p.title.trim() : "",
      ...(p.kind ? { kind: p.kind } : {}),
    })),
  };
}

/** A teaching's blocks as a template: headings, kinds and privacy, never the writing. */
export function templateBlocksFromTeaching(blocks: TeachingBlock[]): TeachingTemplateBlock[] {
  const keyOf = (block: TeachingBlock) => block.blockKey || block.id;
  const byId = new Map(blocks.map((block) => [block.id, block]));
  return [...blocks]
    .sort((a, b) => a.position - b.position)
    .map((block) => {
      const parent = block.parentId ? byId.get(block.parentId) : undefined;
      return {
        key: keyOf(block),
        parentKey: parent ? keyOf(parent) : null,
        blockType: block.blockType,
        title: block.title,
        ...(block.participantFacing === undefined ? {} : { participantFacing: block.participantFacing }),
        ...(block.role ? { role: block.role } : {}),
        ...(block.guidanceKey ? { guidanceKey: block.guidanceKey } : {}),
      };
    });
}

/** What a built-in block template starts with, in the person's language. */
export function builtinBlockStartSummary(t: TFunction, template: BuiltinBlockTemplate): string {
  const format = t(`studies.teaching.format.${shapeWords(template.legacyKind)}`);
  const names = template.blocks.filter((b) => b.type === "section").map((b) => builtinBlockTitle(t, template.key, b.key));
  if (names.length === 0) return format;
  return `${format} ${t("studies.teaching.startsWith", { steps: names.join(t("common.listSeparator")) })}`;
}

/** What a template gives to work in: its format, and what it starts with ("Starts with: Welcome · Game · Step 3."). */
export function startSummary(t: TFunction, start: TeachingStart): string {
  if ("mine" in start && (start.mine.version ?? 1) >= 2) {
    const format = t(`studies.teaching.format.${shapeWords(start.mine.kind)}`);
    const names = (start.mine.blocks ?? [])
      .filter((b) => b.blockType === "section" || !b.parentKey)
      .map((b) => b.title.trim() || (t as unknown as (key: string) => string)(`studies.teaching.blocks.types.${b.blockType}`));
    if (names.length === 0) return format;
    return `${format} ${t("studies.teaching.startsWith", { steps: names.join(t("common.listSeparator")) })}`;
  }
  const kind = startKind(start);
  const format = t(`studies.teaching.format.${shapeWords(kind)}`);
  const points = startingPoints(t, start);
  if (points.length === 0) return format;
  const names = points.map((p, i) => p.title || shapeText(t, kind, "untitled", { number: i + 1 }));
  return `${format} ${t("studies.teaching.startsWith", { steps: names.join(t("common.listSeparator")) })}`;
}
