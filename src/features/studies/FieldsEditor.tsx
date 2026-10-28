import { useCallback, useEffect, useId, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ArrowDown, ArrowUp, Plus, Trash2 } from "lucide-react";
import { Button } from "@/design-system/Button";
import { useAutosave } from "@/hooks/useAutosave";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import type { EntryPart, StudyBlock, StudySection, TemplateField } from "@/types/domain";
import { ResearchBlock } from "./ResearchBlock";
import { structuredText } from "./builtinText";

const input =
  "block w-full rounded-aw border border-border bg-bg px-3 py-2 text-[1rem] text-text placeholder:text-text-muted focus:border-border-strong";

type Values = Record<string, string>;

function parseValues(data: string | null): Values {
  try {
    const parsed: unknown = data ? JSON.parse(data) : {};
    return parsed && typeof parsed === "object" ? (parsed as Values) : {};
  } catch {
    return {};
  }
}

/** The text a search and an export see for an entry: its parts' values, one per line. */
function joined(field: TemplateField, values: Values): string {
  return (field.parts ?? []).map((p) => values[p.key]?.trim() ?? "").filter(Boolean).join("\n");
}

interface FieldsEditorProps {
  section: StudySection;
  onChange: (section: StudySection) => void;
  onError: (message: string | null) => void;
}

/**
 * The answers of a structured part: one labelled box for each single-text
 * field and a list of entries for each repeatable one. Every answer is a
 * study block that says which field it answers; the field definitions are
 * the part's own snapshot. Writing without a field stays below.
 */
export function FieldsEditor({ section, onChange, onError }: FieldsEditorProps) {
  const fields = section.fields?.fields ?? [];
  return (
    <div className="flex flex-col gap-5">
      {fields.map((field) =>
        field.kind === "text" || field.kind === "writing" ? (
          <TextField key={field.key} section={section} field={field} onChange={onChange} onError={onError} />
        ) : field.kind === "entries" ? (
          <EntriesField key={field.key} section={section} field={field} onChange={onChange} onError={onError} />
        ) : <ResearchBlock key={field.key} section={section} field={field} onChange={onChange} onError={onError} />,
      )}
    </div>
  );
}

interface FieldProps {
  section: StudySection;
  field: TemplateField;
  onChange: (section: StudySection) => void;
  onError: (message: string | null) => void;
}

function labelOf(section: StudySection, field: TemplateField) {
  return field.label ?? structuredText(section, `fields.${field.key}.label`, field.key);
}

function TextField({ section, field, onChange, onError }: FieldProps) {
  const api = useApi();
  const block = section.blocks.find((b) => b.fieldKey === field.key);
  const id = useId();
  const helpId = `${id}-help`;
  const [text, setText] = useState(block?.content ?? "");
  const blockId = useRef<string | null>(block?.id ?? null);
  const creating = useRef<Promise<string> | null>(null);

  // A field without its block (an older copy) gets one when the first text is written.
  const ensure = useCallback(async () => {
    if (blockId.current) return blockId.current;
    creating.current ??= api
      .studyBlockCreate({ sectionId: section.id, blockType: "paragraph", fieldKey: field.key })
      .then((created) => {
        blockId.current = created.id;
        onChange({ ...section, blocks: [...section.blocks, created] });
        return created.id;
      });
    return creating.current;
  }, [api, field.key, onChange, section]);

  const save = useCallback(
    async (content: string) => {
      try {
        const saved = await api.studyBlockUpdate({ id: await ensure(), content });
        const current = (await api.studySectionList(section.studyId)).find(part => part.id === section.id);
        if (current) onChange(current);
        else onChange({ ...section, blocks: section.blocks.map(block => block.id === saved.id ? saved : block) });
      } catch (e) {
        onError(toApiError(e).message);
      }
    },
    [api, ensure, onError, onChange, section],
  );
  const { schedule, flush } = useAutosave(save);
  const area = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    const el = area.current;
    if (!el) return;
    el.style.height = "auto";
    el.style.height = `${Math.max(el.scrollHeight, field.kind === "writing" ? 288 : 72)}px`;
  }, [text, field.kind]);

  return (
    <div className="flex flex-col gap-1">
      <label htmlFor={id} className="text-[1rem] font-medium text-text-strong">
        {labelOf(section, field)}
      </label>
      <p id={helpId} className="m-0 text-[0.9375rem] text-text-muted">
        {field.config?.guidance ?? structuredText(section, `fields.${field.key}.help`, "")}
      </p>
      <textarea
        id={id}
        ref={area}
        rows={field.kind === "writing" ? 10 : 3}
        value={text}
        aria-describedby={helpId}
        onChange={(e) => {
          setText(e.target.value);
          schedule(e.target.value);
        }}
        onBlur={flush}
        className={`${input} resize-none overflow-hidden`}
      />
    </div>
  );
}

function EntriesField({ section, field, onChange, onError }: FieldProps) {
  const { t } = useTranslation();
  const api = useApi();
  const entries = section.blocks.filter((b) => b.fieldKey === field.key);
  const label = labelOf(section, field);
  const [newId, setNewId] = useState<string | null>(null);
  const headingId = useId();

  const add = async () => {
    onError(null);
    try {
      const created = await api.studyBlockCreate({ sectionId: section.id, blockType: "paragraph", fieldKey: field.key });
      setNewId(created.id);
      onChange({ ...section, blocks: [...section.blocks, created] });
    } catch (e) {
      onError(toApiError(e).message || t("studies.fields.addFailed"));
    }
  };

  const remove = async (block: StudyBlock) => {
    onError(null);
    try {
      await api.studyBlockDelete(block.id);
      onChange({ ...section, blocks: section.blocks.filter((b) => b.id !== block.id) });
    } catch {
      onError(t("studies.fields.removeFailed"));
    }
  };

  // Entries swap places with their neighbour; all other blocks keep their places.
  const move = async (block: StudyBlock, delta: -1 | 1) => {
    const ids = section.blocks.map((b) => b.id);
    const mine = entries.map((b) => b.id);
    const from = mine.indexOf(block.id);
    const to = from + delta;
    if (to < 0 || to >= mine.length) return;
    const a = ids.indexOf(mine[from] as string);
    const b = ids.indexOf(mine[to] as string);
    [ids[a], ids[b]] = [ids[b] as string, ids[a] as string];
    onError(null);
    try {
      onChange({ ...section, blocks: await api.studyBlockReorder({ parentId: section.id, ids }) });
    } catch (e) {
      onError(toApiError(e).message);
    }
  };

  return (
    <div role="group" aria-labelledby={headingId} className="flex flex-col gap-2">
      <div>
        <h3 id={headingId} className="m-0 text-[1rem] font-medium text-text-strong">{label}</h3>
        <p className="m-0 text-[0.9375rem] text-text-muted">{field.config?.guidance ?? structuredText(section, `fields.${field.key}.help`, "")}</p>
      </div>
      {entries.length === 0 && <p className="m-0 text-[0.9375rem] text-text-muted">{t("studies.fields.noEntries")}</p>}
      {entries.map((block, index) => (
        <Entry
          key={block.id}
          section={section}
          field={field}
          block={block}
          number={index + 1}
          isFirst={index === 0}
          isLast={index === entries.length - 1}
          autoFocus={block.id === newId}
          onRemove={() => remove(block)}
          onMove={(delta) => move(block, delta)}
          onError={onError}
        />
      ))}
      <div>
        <Button onClick={() => void add()} aria-label={`${structuredText(section, `fields.${field.key}.add`, label)} — ${label}`}>
          <Plus className="size-4" aria-hidden="true" />
          {structuredText(section, `fields.${field.key}.add`, label)}
        </Button>
      </div>
    </div>
  );
}

interface EntryProps {
  section: StudySection;
  field: TemplateField;
  block: StudyBlock;
  number: number;
  isFirst: boolean;
  isLast: boolean;
  autoFocus: boolean;
  onRemove: () => void;
  onMove: (delta: -1 | 1) => void;
  onError: (message: string | null) => void;
}

function Entry({ section, field, block, number, isFirst, isLast, autoFocus, onRemove, onMove, onError }: EntryProps) {
  const { t } = useTranslation();
  const api = useApi();
  const label = labelOf(section, field);
  const [values, setValues] = useState<Values>(() => parseValues(block.data));
  const [confirming, setConfirming] = useState(false);
  const first = useRef<HTMLInputElement & HTMLTextAreaElement & HTMLSelectElement>(null);
  const latest = useRef(values);

  const save = useCallback(
    async (json: string) => {
      const parsed = parseValues(json);
      try {
        await api.studyBlockUpdate({ id: block.id, content: joined(field, parsed), data: json });
      } catch (e) {
        onError(toApiError(e).message);
      }
    },
    [api, block.id, field, onError],
  );
  const { schedule, flush } = useAutosave(save);

  useEffect(() => {
    if (autoFocus) first.current?.focus();
  }, [autoFocus]);

  const set = (key: string, value: string) => {
    const next = { ...latest.current, [key]: value };
    latest.current = next;
    setValues(next);
    schedule(JSON.stringify(next));
  };
  const empty = (field.parts ?? []).every((p) => !(values[p.key] ?? "").trim());
  const tool =
    "inline-flex min-h-11 items-center gap-1 rounded-aw px-2 text-[0.9375rem] text-text-muted hover:bg-hover hover:text-text-strong disabled:opacity-40";
  const params = { field: label, number };

  return (
    <div className="flex flex-col gap-2 rounded-aw border border-border-subtle p-3" role="group" aria-label={t("studies.fields.entryLabel", params)}>
      <div className="flex flex-wrap items-center gap-1">
        <h4 className="m-0 mr-auto text-[0.9375rem] font-medium text-text-muted">{t("studies.fields.entryHeading", { number })}</h4>
        <button type="button" className={tool} disabled={isFirst} onClick={() => onMove(-1)} aria-label={t("studies.fields.moveEntryUp", params)}>
          <ArrowUp className="size-4" aria-hidden="true" />
        </button>
        <button type="button" className={tool} disabled={isLast} onClick={() => onMove(1)} aria-label={t("studies.fields.moveEntryDown", params)}>
          <ArrowDown className="size-4" aria-hidden="true" />
        </button>
        <button
          type="button"
          className={`${tool} hover:text-danger`}
          aria-label={t("studies.fields.removeEntry", params)}
          onClick={() => (empty ? onRemove() : setConfirming(true))}
        >
          <Trash2 className="size-4" aria-hidden="true" />
          {t("common.delete")}
        </button>
      </div>
      {confirming && (
        <div role="alertdialog" aria-label={t("studies.fields.confirmRemove")} className="flex flex-wrap items-center gap-2 border-l-[3px] border-danger bg-bg-secondary px-3 py-2">
          <span>{t("studies.fields.confirmRemove")}</span>
          <Button variant="danger" onClick={onRemove}>{t("common.delete")}</Button>
          <Button autoFocus onClick={() => setConfirming(false)}>{t("common.keep")}</Button>
        </div>
      )}
      <div className="@container"><div className="grid grid-cols-1 gap-3 @md:grid-cols-2">
        {(field.parts ?? []).map((part, index) => (
          <EntryPartInput
            key={part.key}
            section={section}
            field={field}
            part={part}
            number={number}
            value={values[part.key] ?? ""}
            onChange={(v) => set(part.key, v)}
            onBlur={flush}
            inputRef={index === 0 ? first : undefined}
          />
        ))}
      </div>
      </div>
    </div>
  );
}

interface PartInputProps {
  section: StudySection;
  field: TemplateField;
  part: EntryPart;
  number: number;
  value: string;
  onChange: (value: string) => void;
  onBlur: () => void;
  inputRef?: React.Ref<HTMLInputElement & HTMLTextAreaElement & HTMLSelectElement>;
}

function EntryPartInput({ section, field, part, number, value, onChange, onBlur, inputRef }: PartInputProps) {
  const { t } = useTranslation();
  const id = useId();
  const partLabel = structuredText(section, `fields.${field.key}.parts.${part.key}`, part.key);
  const full = t("studies.fields.entryPart", { part: partLabel, field: labelOf(section, field), number });
  const wide = part.kind === "text";
  return (
    <div className={`flex flex-col gap-1 ${wide ? "@md:col-span-2" : ""}`}>
      <label htmlFor={id} className="text-[0.9375rem] font-medium text-text-strong">
        {partLabel}
      </label>
      {part.kind === "line" && (
        <input id={id} ref={inputRef} type="text" value={value} aria-label={full} onChange={(e) => onChange(e.target.value)} onBlur={onBlur} className={`${input} min-h-11`} />
      )}
      {part.kind === "text" && (
        <textarea id={id} ref={inputRef} rows={2} value={value} aria-label={full} onChange={(e) => onChange(e.target.value)} onBlur={onBlur} className={`${input} resize-y`} />
      )}
      {part.kind === "choice" && (
        <select
          id={id}
          ref={inputRef}
          value={value || part.options?.[0] || ""}
          aria-label={full}
          onChange={(e) => onChange(e.target.value)}
          onBlur={onBlur}
          className={`${input} min-h-11`}
        >
          {(part.options ?? []).map((option) => (
            <option key={option} value={option}>
              {structuredText(section, `fields.${field.key}.options.${option}`, option)}
            </option>
          ))}
        </select>
      )}
    </div>
  );
}
