// In-memory Teaching Preparation and assistant (v0.16) for interface tests.
// Mirrors the core's rules (db/teaching.rs, assistant/): findings are linked
// live and never copied; Scripture is named without touching the study;
// kept material has an honest standing; the assistant never changes text,
// refuses to write without the person's material, and is scripted by the test.
import { ApiError, type Api } from "@/lib/api";
import { isLocalAddress } from "@/lib/localAddress";
import type { ScriptureRef, StudyPassage } from "@/types/scripture";
import type { Finding } from "@/types/studies";
import type {
  AssistantConfig,
  AssistantSettings,
  AssistInput,
  AssistReply,
  FoundImage,
  KeepTeachingMaterial,
  SharedItem,
  TeachingBlock,
  TeachingMaterial,
  TeachingPreparation,
  TeachingTemplate,
  TeachingTemplateBlock,
  TeachingTemplateInput,
} from "@/types/teaching";
import { STEP_KINDS, TEACHING_KINDS } from "@/types/teaching";

interface StoredPrep extends Omit<TeachingPreparation, "findings" | "passages" | "materials"> {
  links: { id: string; pointId: string | null; findingId: string }[];
  passages: { id: string; pointId: string | null; blockId?: string | null; reference: ScriptureRef }[];
  materials: TeachingMaterial[];
}

export interface TeachingState {
  preparations: StoredPrep[];
  /** The person's own Bible Teaching templates (v0.18.7). */
  templates: TeachingTemplate[];
  assistant: {
    config: AssistantConfig;
    /** provider kind, or "search:<kind>" → key (never returned). */
    keys: Map<string, string>;
    /** What the assistant answers, in order (a test sets these). */
    replies: (AssistReply | ApiError)[];
    /** Every request the interface made, for tests of what was asked. */
    asked: AssistInput[];
    previews: AssistInput[];
    foundImages: FoundImage[];
    findCalls: string[];
    generated: string[];
    tested: number;
    searchTested: number;
    /** When set, asking, finding and making images wait for it: a test can see the waiting state. */
    hold: Promise<void> | null;
  };
}

export function defaultAssistantConfig(): AssistantConfig {
  return {
    enabled: false,
    provider: "openai",
    baseUrl: "",
    model: "",
    search: "none",
    searchUrl: "",
    findImages: true,
    generateImages: false,
    imageModel: "",
    imageService: "same",
    perspective: "",
  };
}

export function newTeachingState(): TeachingState {
  return {
    preparations: [],
    templates: [],
    assistant: {
      config: defaultAssistantConfig(),
      keys: new Map(),
      replies: [],
      asked: [],
      previews: [],
      foundImages: [],
      findCalls: [],
      generated: [],
      tested: 0,
      searchTested: 0,
      hold: null,
    },
  };
}

interface Helpers {
  id: () => string;
  now: () => string;
  findings: () => Finding[];
  studyExists: (studyId: string) => void;
  studyPassages: () => StudyPassage[];
  ref: (canonical: string) => ScriptureRef | null;
}

type TeachingApi = Pick<
  Api,
  | "teachingTemplateList"
  | "teachingTemplateCreate"
  | "teachingTemplateUpdate"
  | "teachingTemplateDelete"
  | "teachingList"
  | "teachingGet"
  | "teachingCreate"
  | "teachingUpdate"
  | "teachingDelete"
  | "teachingPointCreate"
  | "teachingPointUpdate"
  | "teachingPointDelete"
  | "teachingPointReorder"
  | "teachingBlockCreate"
  | "teachingBlockUpdate"
  | "teachingBlockDelete"
  | "teachingBlockReorder"
  | "teachingBlockDuplicate"
  | "teachingFindingLink"
  | "teachingFindingUnlink"
  | "teachingPassageAdd"
  | "teachingPassageRemove"
  | "teachingMaterialKeep"
  | "teachingMaterialRemove"
  | "teachingImage"
  | "assistantSettingsGet"
  | "assistantSettingsSet"
  | "assistantKeySet"
  | "assistantTest"
  | "assistantSearchTest"
  | "assistantPreview"
  | "assistantAsk"
  | "assistantFindImages"
  | "assistantImagePreview"
  | "assistantGenerateImage"
>;

const SEARCH_NEEDS_KEY = new Set(["brave", "tavily", "exa", "serpapi"]);
const NEEDS_KEY = new Set(["openai", "anthropic", "google", "xai", "perplexity", "deepseek", "qwen", "mistral", "cohere", "moonshot", "zai", "minimax"]);
const LABELS: Record<string, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  google: "Google Gemini",
  xai: "xAI (Grok)",
  perplexity: "Perplexity",
  deepseek: "DeepSeek",
  qwen: "Qwen",
  mistral: "Mistral",
  cohere: "Cohere",
  moonshot: "Moonshot AI (Kimi)",
  zai: "Z.ai (GLM)",
  minimax: "MiniMax",
  ollama: "Ollama",
  lmstudio: "LM Studio",
  openai_compatible: "Other compatible service",
};

const imageProvider = (c: AssistantConfig) => (c.imageService === "openai" ? "openai" : c.provider);
/** As the core decides it: from the service, never from a model's name. */
const imageCapability = (c: AssistantConfig) => {
  const p = imageProvider(c);
  return p === "openai" ? "supported" : p === "openai_compatible" ? "unknown" : "unsupported";
};

function remoteOf(c: AssistantConfig): boolean {
  const base = c.baseUrl || { openai: "https://api.openai.com/v1", anthropic: "https://api.anthropic.com/v1", google: "https://g", xai: "https://api.x.ai/v1", perplexity: "https://api.perplexity.ai", deepseek: "https://api.deepseek.com/v1", qwen: "https://q", mistral: "https://api.mistral.ai/v1", cohere: "https://api.cohere.ai/compatibility/v1", moonshot: "https://api.moonshot.ai/v1", zai: "https://api.z.ai/api/paas/v4", minimax: "https://api.minimax.io/v1", ollama: "http://localhost:11434/v1", lmstudio: "http://localhost:1234/v1", openai_compatible: "" }[c.provider];
  return !base || !isLocalAddress(base);
}

/** Who would answer, as the core names it, whether text leaves the computer, and whether its key is kept. */
export function assistantFacts(state: TeachingState) {
  const c = state.assistant.config;
  return {
    label: c.model ? `${LABELS[c.provider]} · ${c.model}` : LABELS[c.provider]!,
    remote: remoteOf(c),
    hasKey: !NEEDS_KEY.has(c.provider) || state.assistant.keys.has(c.provider),
  };
}

const PRIVATE = new Set(["teaching_notes", "presentation_notes", "writing", "explanation", "illustration", "application", "activity", "objectives", "time"]);

function makeBlock(h: { id: () => string }, block: TeachingTemplateBlock & { body?: string }): TeachingBlock {
  const facing = block.participantFacing ?? !PRIVATE.has(block.blockType);
  return {
    id: h.id(),
    parentId: null,
    position: 0,
    blockType: block.blockType,
    blockKey: block.key ?? null,
    title: block.title ?? "",
    body: block.body ?? "",
    participantFacing: facing,
    role: block.role ?? null,
    guidanceKey: block.guidanceKey ?? null,
    pointId: null,
  };
}

function materialise(h: { id: () => string }, blocks: TeachingTemplateBlock[]) {
  const made = blocks.map((block) => makeBlock(h, block));
  const byKey = new Map(blocks.map((block, index) => [block.key, made[index]!]));
  const points: StoredPoint[] = [];
  blocks.forEach((block, index) => {
    const row = made[index]!;
    row.parentId = block.parentKey ? (byKey.get(block.parentKey)?.id ?? null) : null;
    if (block.blockType === "outline_point") {
      const point = { id: h.id(), position: points.length, kind: null, title: row.title, explanation: row.body, application: "", illustration: "" };
      points.push(point);
      row.pointId = point.id;
    }
  });
  return { blocks: made, points };
}

function applyRoles(p: { introduction: string; conclusion: string; notes: string; centralMessage: string | null; blocks?: TeachingBlock[] }) {
  if (!(p.blocks?.length)) return;
  p.introduction = ""; p.conclusion = ""; p.notes = ""; p.centralMessage = null;
  const claimed = new Set<string>();
  for (const block of p.blocks ?? []) {
    if (block.role && ["introduction", "conclusion", "central_message", "notes"].includes(block.role)) {
      if (claimed.has(block.role)) block.role = null;
      else claimed.add(block.role);
    }
    if (block.role === "introduction") p.introduction = block.body;
    if (block.role === "conclusion") p.conclusion = block.body;
    if (block.role === "notes") p.notes = block.body;
    if (block.role === "central_message") p.centralMessage = block.body || null;
  }
}

type StoredPoint = { id: string; position: number; kind: null; title: string; explanation: string; application: string; illustration: string };

export function teachingApi(state: TeachingState, h: Helpers): TeachingApi {
  const prep = (prepId: string): StoredPrep => {
    const p = state.preparations.find((x) => x.id === prepId);
    if (!p) throw new ApiError("notFound", "This teaching preparation could not be found. It may have been deleted.");
    return p;
  };
  const owner = (pointId: string): StoredPrep => {
    const p = state.preparations.find((x) => x.points.some((pt) => pt.id === pointId));
    if (!p) throw new ApiError("notFound", "This teaching point could not be found. It may have been deleted.");
    return p;
  };
  const touch = (p: StoredPrep) => {
    p.updatedAt = h.now();
    p.revision += 1;
  };
  const view = (p: StoredPrep): TeachingPreparation => {
    const live = new Map(h.findings().map((f) => [f.id, f]));
    const points = p.points.map((x, i) => ({ ...x, position: i }));
    return {
      ...p,
      points,
      findings: p.links.filter((l) => live.has(l.findingId)).map((l) => ({ id: l.id, pointId: l.pointId, finding: { ...live.get(l.findingId)! } })),
      passages: p.passages.map((x) => ({ ...x })),
      materials: p.materials.map((x) => ({ ...x })),
    };
  };
  const remote = remoteOf;
  const settings = (): AssistantSettings => {
    const c = state.assistant.config;
    return {
      config: { ...c },
      hasKey: state.assistant.keys.has(c.provider),
      hasSearchKey: SEARCH_NEEDS_KEY.has(c.search) && state.assistant.keys.has(`search:${c.search}`),
      remote: remote(c),
      // As the core: the image service's key is needed too, when that service needs one.
      canGenerateImages:
        c.enabled && c.generateImages && imageCapability(c) !== "unsupported" && (state.assistant.keys.has(imageProvider(c)) || !NEEDS_KEY.has(imageProvider(c))),
      imageCapability: imageCapability(c),
      hasImageKey: state.assistant.keys.has(imageProvider(c)),
    };
  };
  const label = () => {
    const c = state.assistant.config;
    return c.model ? `${LABELS[c.provider]} · ${c.model}` : LABELS[c.provider]!;
  };
  const ready = (input: AssistInput) => {
    const c = state.assistant.config;
    if (!c.enabled) throw new ApiError("assistant", "The assistant is turned off. You can turn it on in Settings.");
    if (NEEDS_KEY.has(c.provider) && !state.assistant.keys.has(c.provider))
      throw new ApiError("assistant", "The assistant needs its key. You can add it in Settings.");
    const p = prep(input.preparationId);
    const text = input.text.trim();
    if (input.action === "structure" || input.action === "restructure") {
      if (p.points.filter((x) => x.title.trim() || x.explanation.trim()).length < 2)
        throw new ApiError("assistant", "Write at least two points first. The assistant comments on the structure you have.");
    } else if (input.action === "question" || input.action === "story" || input.action === "free") {
      if (!input.request?.trim()) throw new ApiError("assistant", "Say in a few words what you are looking for.");
    } else if (input.action === "visual") {
      if (!text && !input.request?.trim()) throw new ApiError("assistant", "Write something here first. The assistant helps with what you have written.");
    } else if (!text) {
      throw new ApiError("assistant", "Write something here first. The assistant helps with what you have written.");
    }
    if (input.action === "story" && c.search === "none")
      throw new ApiError("assistant", "Real stories can only be found and checked with a search service, which is not set up. You can add one in Settings, or ask for an invented illustration instead.");
    return p;
  };

  // As the core (db/teaching_templates.rs): a format, at most 20 points, known step kinds; structure only.
  const checkedTemplate = (input: TeachingTemplateInput) => {
    const name = input.name.trim();
    if (!name) throw new ApiError("invalidInput", "Please give the template a name.");
    if (!TEACHING_KINDS.includes(input.kind)) throw new ApiError("invalidInput", "Choose what the template is for.");
    if (input.points.length > 20) throw new ApiError("invalidInput", "A template can start with at most 20 points or steps.");
    for (const point of input.points)
      if (point.kind && point.kind !== "audio" && !(STEP_KINDS as readonly string[]).includes(point.kind)) throw new ApiError("invalidInput", `“${point.kind}” is not a valid kind of step.`);
    const version = input.version ?? ((input.blocks?.length ?? 0) > 0 ? 2 : 1);
    return {
      name,
      description: input.description?.trim() || null,
      kind: input.kind,
      points: version >= 2 ? [] : input.points.map((point) => ({ ...(point.key ? { key: point.key } : {}), title: point.title.trim(), ...(point.kind ? { kind: point.kind } : {}) })),
      ...(version >= 2 ? { version: 2, blocks: input.blocks ?? [] } : {}),
    };
  };
  const template = (id: string) => {
    const found = state.templates.find((x) => x.id === id);
    if (!found) throw new ApiError("notFound", "This template no longer exists.");
    return found;
  };

  return {
    teachingTemplateList: async () => [...state.templates].sort((a, b) => a.name.localeCompare(b.name)).map((x) => structuredClone(x)),
    teachingTemplateCreate: async (input) => {
      const t = h.now();
      const created: TeachingTemplate = { id: h.id(), ...checkedTemplate(input), createdAt: t, updatedAt: t, revision: 1 };
      state.templates.push(created);
      return structuredClone(created);
    },
    teachingTemplateUpdate: async (input) => {
      const found = template(input.id);
      Object.assign(found, checkedTemplate(input), { updatedAt: h.now(), revision: found.revision + 1 });
      return structuredClone(found);
    },
    teachingTemplateDelete: async (id) => {
      template(id);
      state.templates = state.templates.filter((x) => x.id !== id);
    },
    teachingList: async (studyId) => {
      h.studyExists(studyId);
      return state.preparations.filter((p) => p.studyId === studyId).map((p) => ({ id: p.id, studyId, title: p.title, kind: p.kind, updatedAt: p.updatedAt }));
    },
    teachingGet: async (prepId) => view(prep(prepId)),
    teachingCreate: async ({ studyId, title, kind, points = [], templateKey, templateVersion, blocks = [] }) => {
      h.studyExists(studyId);
      const name = title.trim();
      if (!name) throw new ApiError("invalidInput", "Please give the teaching a title.");
      const t = h.now();
      const made = materialise(h, blocks);
      const p: StoredPrep = {
        id: h.id(), studyId, title: name, kind: kind ?? null, audience: null, purpose: null, centralMessage: null,
        introduction: "", conclusion: "", notes: "",
        points: [
          ...points.map((x) => ({ id: h.id(), position: 0, kind: x.kind ?? null, title: x.title.trim(), explanation: "", application: "", illustration: "" })),
          ...made.points,
        ],
        links: [], passages: [], materials: [],
        templateKey: templateKey ?? null,
        templateVersion: templateVersion ?? (blocks.length ? 2 : null),
        blocks: made.blocks,
        createdAt: t, updatedAt: t, revision: 1,
      };
      applyRoles(p);
      state.preparations.push(p);
      return view(p);
    },
    teachingUpdate: async (input) => {
      const p = prep(input.id);
      if (input.title !== undefined) {
        if (!input.title.trim()) throw new ApiError("invalidInput", "Please give the teaching a title.");
        p.title = input.title.trim();
      }
      if (input.kind !== undefined) p.kind = input.kind || null;
      for (const key of ["audience", "purpose", "centralMessage"] as const) {
        const value = input[key];
        if (value !== undefined) p[key] = value.trim() || null;
      }
      for (const key of ["introduction", "conclusion", "notes"] as const) {
        const value = input[key];
        if (value !== undefined) p[key] = value;
      }
      const roleOf: Record<string, string> = { introduction: "introduction", conclusion: "conclusion", notes: "notes", centralMessage: "central_message" };
      for (const [field, role] of Object.entries(roleOf)) {
        const value = input[field as keyof typeof input];
        if (typeof value === "string") {
          for (const block of p.blocks ?? []) if (block.role === role) block.body = value;
        }
      }
      touch(p);
      return view(p);
    },
    teachingDelete: async (prepId) => {
      prep(prepId);
      state.preparations = state.preparations.filter((p) => p.id !== prepId);
    },
    teachingPointCreate: async ({ preparationId, title, kind, afterPointId }) => {
      const p = prep(preparationId);
      const at = afterPointId ? p.points.findIndex((x) => x.id === afterPointId) + 1 : p.points.length;
      p.points.splice(at <= 0 ? p.points.length : at, 0, { id: h.id(), position: 0, kind: kind ?? null, title: (title ?? "").trim(), explanation: "", application: "", illustration: "" });
      touch(p);
      return view(p);
    },
    teachingPointUpdate: async (input) => {
      const p = owner(input.id);
      const point = p.points.find((x) => x.id === input.id)!;
      if (input.title !== undefined) point.title = input.title.trim();
      if (input.kind !== undefined) point.kind = input.kind || null;
      for (const key of ["explanation", "application", "illustration"] as const) {
        const value = input[key];
        if (value !== undefined) point[key] = value;
      }
      touch(p);
      return view(p);
    },
    teachingPointDelete: async (pointId) => {
      const p = owner(pointId);
      p.points = p.points.filter((x) => x.id !== pointId);
      p.links = p.links.filter((l) => l.pointId !== pointId);
      p.passages = p.passages.filter((l) => l.pointId !== pointId);
      p.materials = p.materials.filter((l) => l.pointId !== pointId);
      touch(p);
      return view(p);
    },
    teachingBlockCreate: async ({ preparationId, parentId, afterBlockId, blockType, title, body, participantFacing, role, guidanceKey }) => {
      const p = prep(preparationId);
      if ((p.templateVersion ?? 0) < 2 && !p.blocks?.length) throw new ApiError("invalidInput", "That is not a kind of block this teaching can hold.");
      if (role && ["introduction", "conclusion", "central_message", "notes"].includes(role) && p.blocks?.some((b) => b.role === role)) throw new ApiError("invalidInput", "That block cannot be tied to that part of the teaching.");
      const block = makeBlock(h, { blockType, title: title ?? "", body: body ?? "", parentKey: null, participantFacing, role, guidanceKey });
      block.parentId = parentId ?? null;
      const siblings = (p.blocks ?? []).filter((b) => (b.parentId ?? null) === (parentId ?? null));
      const at = afterBlockId ? siblings.findIndex((b) => b.id === afterBlockId) + 1 : siblings.length;
      const next = [...(p.blocks ?? [])];
      const index = next.findIndex((b) => b.id === (siblings[Math.max(0, at - 1)]?.id ?? ""));
      next.splice(index >= 0 ? index + 1 : next.length, 0, block);
      if (blockType === "outline_point") {
        const point = { id: h.id(), position: p.points.length, kind: null, title: block.title, explanation: block.body, application: "", illustration: "" };
        p.points.push(point);
        block.pointId = point.id;
      }
      p.blocks = next;
      p.templateVersion = p.templateVersion ?? 2;
      applyRoles(p);
      touch(p);
      return view(p);
    },
    teachingBlockUpdate: async (input) => {
      const p = state.preparations.find((x) => (x.blocks ?? []).some((b) => b.id === input.id));
      if (!p) throw new ApiError("notFound", "This block could not be found.");
      const block = p.blocks!.find((b) => b.id === input.id)!;
      if (input.role && ["introduction", "conclusion", "central_message", "notes"].includes(input.role) && p.blocks?.some((b) => b.id !== block.id && b.role === input.role)) throw new ApiError("invalidInput", "That block cannot be tied to that part of the teaching.");
      if (input.title !== undefined) block.title = input.title;
      if (input.body !== undefined) block.body = input.body;
      if (input.blockType !== undefined) block.blockType = input.blockType;
      if (input.participantFacing !== undefined) block.participantFacing = input.participantFacing;
      if (input.role !== undefined) block.role = input.role || null;
      if (input.guidanceKey !== undefined) block.guidanceKey = input.guidanceKey || null;
      if (input.parentId !== undefined) block.parentId = input.parentId || null;
      if (block.pointId) {
        const point = p.points.find((pt) => pt.id === block.pointId);
        if (point) {
          point.title = block.title;
          point.explanation = block.body;
        }
      }
      applyRoles(p);
      touch(p);
      return view(p);
    },
    teachingBlockDelete: async (id) => {
      const p = state.preparations.find((x) => (x.blocks ?? []).some((b) => b.id === id));
      if (!p) throw new ApiError("notFound", "This block could not be found.");
      const block = p.blocks!.find((b) => b.id === id)!;
      p.blocks = p.blocks!.filter((b) => b.id !== id).map((b) => (b.parentId === id ? { ...b, parentId: block.parentId, participantFacing: b.participantFacing && block.participantFacing } : b));
      if (block.pointId) p.points = p.points.filter((pt) => pt.id !== block.pointId);
      p.passages = p.passages.filter((passage) => passage.blockId !== id);
      if (block.role === "introduction") p.introduction = "";
      if (block.role === "conclusion") p.conclusion = "";
      if (block.role === "central_message") p.centralMessage = null;
      if (block.role === "notes") p.notes = "";
      applyRoles(p);
      touch(p);
      return view(p);
    },
    teachingBlockReorder: async ({ preparationId, parentId, ids }) => {
      const p = prep(preparationId);
      const parent = parentId ?? null;
      const siblings = (p.blocks ?? []).filter((b) => (b.parentId ?? null) === parent).map((b) => b.id).sort();
      if (JSON.stringify([...ids].sort()) !== JSON.stringify(siblings)) throw new ApiError("invalidInput", "The blocks have changed since this page was shown. Please try again.");
      const rank = new Map(ids.map((bid, index) => [bid, index]));
      p.blocks = [...(p.blocks ?? [])].sort((a, b) => (rank.get(a.id) ?? 0) - (rank.get(b.id) ?? 0));
      touch(p);
      return view(p);
    },
    teachingBlockDuplicate: async (id) => {
      const p = state.preparations.find((x) => (x.blocks ?? []).some((b) => b.id === id));
      if (!p) throw new ApiError("notFound", "This block could not be found.");
      const block = p.blocks!.find((b) => b.id === id)!;
      const copy = { ...block, id: h.id(), role: null, pointId: null as string | null };
      if (block.blockType === "outline_point") {
        const point = { id: h.id(), position: p.points.length, kind: null, title: copy.title, explanation: copy.body, application: "", illustration: "" };
        p.points.push(point);
        copy.pointId = point.id;
      }
      const at = p.blocks!.findIndex((b) => b.id === id);
      p.blocks = [...p.blocks!.slice(0, at + 1), copy, ...p.blocks!.slice(at + 1)];
      touch(p);
      return view(p);
    },
    teachingPointReorder: async ({ parentId, ids }) => {
      const p = prep(parentId);
      const current = p.points.map((x) => x.id).sort();
      if (JSON.stringify([...ids].sort()) !== JSON.stringify(current))
        throw new ApiError("invalidInput", "The points have changed since this page was shown. Please try again.");
      p.points = ids.map((pid) => p.points.find((x) => x.id === pid)!);
      touch(p);
      return view(p);
    },
    teachingFindingLink: async ({ preparationId, pointId, findingId }) => {
      const p = prep(preparationId);
      const finding = h.findings().find((f) => f.id === findingId);
      if (!finding) throw new ApiError("notFound", "This finding could not be found. It may have been deleted.");
      if (finding.studyId !== p.studyId) throw new ApiError("invalidInput", "That finding belongs to another study.");
      const where = pointId ?? null;
      if (!p.links.some((l) => l.findingId === findingId && l.pointId === where)) p.links.push({ id: h.id(), pointId: where, findingId });
      touch(p);
      return view(p);
    },
    teachingFindingUnlink: async (linkId) => {
      const p = state.preparations.find((x) => x.links.some((l) => l.id === linkId));
      if (!p) throw new ApiError("notFound", "This link could not be found.");
      p.links = p.links.filter((l) => l.id !== linkId);
      touch(p);
      return view(p);
    },
    teachingPassageAdd: async ({ preparationId, pointId, blockId, reference }) => {
      const p = prep(preparationId);
      if (blockId) {
        const block = p.blocks?.find((b) => b.id === blockId);
        if (pointId || !block || block.blockType !== "scripture") throw new ApiError("invalidInput", "That block cannot be used here.");
      }
      const ref = h.ref(reference);
      if (!ref) throw new ApiError("invalidReference", "Invalid reference.");
      const where = pointId ?? null;
      if (!p.passages.some((x) => x.pointId === where && (x.blockId ?? null) === (blockId ?? null) && x.reference.canonical === ref.canonical)) p.passages.push({ id: h.id(), pointId: where, blockId: blockId ?? null, reference: ref });
      touch(p);
      return view(p);
    },
    teachingPassageRemove: async (passageId) => {
      const p = state.preparations.find((x) => x.passages.some((l) => l.id === passageId));
      if (!p) throw new ApiError("notFound", "This passage could not be found.");
      p.passages = p.passages.filter((l) => l.id !== passageId);
      touch(p);
      return view(p);
    },
    teachingMaterialKeep: async (input: KeepTeachingMaterial) => {
      const p = prep(input.preparationId);
      const allowed: Record<string, string[]> = { story: ["verified", "unverified"], illustration: ["hypothetical"], image: ["found", "generated"], visual: ["suggestion"] };
      if (!allowed[input.kind]?.includes(input.standing)) throw new ApiError("invalidInput", "This item cannot be kept.");
      if (input.standing === "verified" && new Set((input.sources ?? []).map((s) => s.site)).size < 2)
        throw new ApiError("invalidInput", "A story is only kept as checked with two independent sources.");
      p.materials.push({
        id: h.id(), pointId: input.pointId ?? null, kind: input.kind, standing: input.standing, title: input.title.trim(), content: input.content.trim(),
        sources: input.sources ?? [], image: input.image ?? null, assistant: input.assistant ?? null, createdAt: h.now(),
      });
      touch(p);
      return view(p);
    },
    teachingMaterialRemove: async (materialId) => {
      const p = state.preparations.find((x) => x.materials.some((m) => m.id === materialId));
      if (!p) throw new ApiError("notFound", "This item could not be found.");
      p.materials = p.materials.filter((m) => m.id !== materialId);
      touch(p);
      return view(p);
    },
    teachingImage: async (file) => `data:image/png;base64,${btoa(file)}`,

    assistantSettingsGet: async () => settings(),
    assistantSettingsSet: async (config) => {
      if (config.baseUrl && !/^https?:\/\//i.test(config.baseUrl.trim()))
        throw new ApiError("invalidInput", "Please enter a web address beginning with http:// or https://.");
      state.assistant.config = { ...config, baseUrl: config.baseUrl.trim(), model: config.model.trim(), perspective: config.perspective.trim() };
      return settings();
    },
    assistantKeySet: async (which, key) => {
      const name = which === "search" ? `search:${state.assistant.config.search}` : which === "image" ? imageProvider(state.assistant.config) : state.assistant.config.provider;
      if (key.trim()) state.assistant.keys.set(name, key.trim());
      else state.assistant.keys.delete(name);
      return settings();
    },
    assistantSearchTest: async () => {
      state.assistant.searchTested += 1;
      const c = state.assistant.config;
      if (c.search === "none") throw new ApiError("assistant", "Real stories can only be found and checked with a search service, which is not set up. You can add one in Settings, or ask for an invented illustration instead.");
      if (SEARCH_NEEDS_KEY.has(c.search) && !state.assistant.keys.has(`search:${c.search}`))
        throw new ApiError("assistant", "The search service needs its key. You can add it in Settings.");
      return 3;
    },
    assistantTest: async () => {
      state.assistant.tested += 1;
      const c = state.assistant.config;
      if (NEEDS_KEY.has(c.provider) && !state.assistant.keys.has(c.provider))
        throw new ApiError("assistant", "The assistant needs its key. You can add it in Settings.");
    },
    assistantPreview: async (input) => {
      const p = ready(input);
      state.assistant.previews.push(input);
      const shared: SharedItem[] = [];
      if (input.text.trim()) shared.push({ kind: "text", label: input.part, reference: null });
      if (input.request?.trim()) shared.push({ kind: "request", label: input.request.trim(), reference: null });
      if (p.centralMessage || p.audience || p.purpose) shared.push({ kind: "teaching", label: p.title, reference: null });
      const primary = h.studyPassages().find((x) => x.studyId === p.studyId && x.role === "primary");
      if (primary) shared.push({ kind: "passage", label: primary.reference.display, reference: primary.reference });
      const linked = p.links.filter((l) => l.pointId === null || l.pointId === input.pointId).length;
      if (linked) shared.push({ kind: "finding", label: String(linked), reference: null });
      return { remote: remote(state.assistant.config), assistant: label(), shared, searches: input.action === "story" };
    },
    assistantAsk: async (input) => {
      ready(input);
      state.assistant.asked.push(input);
      if (state.assistant.hold) await state.assistant.hold;
      const next = state.assistant.replies.shift();
      if (!next) throw new ApiError("assistant", "The assistant service could not answer this time. Please try again.");
      if (next instanceof ApiError) throw next;
      return next;
    },
    assistantFindImages: async (terms) => {
      const c = state.assistant.config;
      if (!c.enabled || !c.findImages) throw new ApiError("assistant", "Finding images is turned off. You can turn it on in Settings.");
      state.assistant.findCalls.push(terms);
      if (state.assistant.hold) await state.assistant.hold;
      return state.assistant.foundImages;
    },
    assistantImagePreview: async (url) => (url.startsWith("https://upload.wikimedia.org/") ? "data:image/jpeg;base64,/9j/" : null),
    assistantGenerateImage: async (description) => {
      if (!settings().canGenerateImages)
        throw new ApiError("assistant", "Making images is not set up. You can turn it on in Settings (OpenAI image models only).");
      state.assistant.generated.push(description);
      if (state.assistant.hold) await state.assistant.hold;
      const file = `${h.id().replace(/[^0-9a-f-]/g, "")}.png`;
      return { file, dataUrl: "data:image/png;base64,iVBORw0KGgo=", assistant: `${LABELS[state.assistant.config.provider]} · gpt-image-2.5-flare` };
    },
  };
}
