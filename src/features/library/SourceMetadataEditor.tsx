import { useCallback, useId, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Pencil } from "lucide-react";
import { Button } from "@/design-system/Button";
import { Dialog } from "@/design-system/Dialog";
import { Notice } from "@/design-system/Notice";
import { useLoad } from "@/hooks/useLoad";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import { LICENCE_STATUSES, SOURCE_TYPES, type Source, type SourceCoverageMode, type SourceType, type UpdateSource } from "@/types/library";
import { coverageLabel, sourceTypeLabel } from "./labels";

/** Raw references belong to the disposable draft until the native core validates Save. */
export type SourceMetadataDraft = Source & { coverageEdit?: { mode: SourceCoverageMode; passagesText: string } };

/** The same bibliographic fields in Settings, the Library and the reader. */
export function SourceMetadataFields({ draft, onChange, children }: { draft: SourceMetadataDraft; onChange: (draft: SourceMetadataDraft) => void; children?: ReactNode }) {
  const { t } = useTranslation();
  const id = useId();
  const set = <K extends keyof Source>(key: K, value: Source[K]) => onChange({ ...draft, [key]: value });
  const coverageMode = draft.coverageEdit?.mode ?? draft.coverageMode ?? "automatic";
  const passagesText = draft.coverageEdit?.passagesText ?? (draft.coverage ?? []).map(coverageLabel).join("\n");
  const setCoverage = (mode: SourceCoverageMode, text = passagesText) => onChange({ ...draft, coverageEdit: { mode, passagesText: text } });
  const field = (key: "title" | "subtitle" | "author" | "publisher" | "edition" | "isbn" | "doi" | "language" | "licence", label: string) => (
    <div className="flex flex-col gap-1">
      <label htmlFor={`${id}-${key}`} className="text-[0.9375rem] font-medium text-text-strong">{label}</label>
      <input id={`${id}-${key}`} value={draft[key] ?? ""} onChange={(event) => set(key, event.target.value || (key === "title" ? "" : null))} className="h-10 aw-input px-3" />
    </div>
  );
  return <>
    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
      {field("title", t("library.manage.field.title"))}
      {field("author", t("library.manage.field.author"))}
      <div className="flex flex-col gap-1">
        <label htmlFor={`${id}-type`} className="text-[0.9375rem] font-medium text-text-strong">{t("library.manage.field.category")}</label>
        <select id={`${id}-type`} value={draft.sourceType} onChange={(event) => set("sourceType", event.target.value as SourceType)} className="h-10 aw-input px-2">
          {SOURCE_TYPES.map((type) => <option key={type} value={type}>{sourceTypeLabel(t, type)}</option>)}
        </select>
      </div>
      {field("language", t("library.manage.field.language"))}
    </div>
    <details>
      <summary className="cursor-pointer text-[0.9375rem] font-medium text-text-strong">{t("library.manage.moreDetails")}</summary>
      <div className="mt-3 grid grid-cols-1 gap-3 sm:grid-cols-2">
        {field("subtitle", t("library.manage.field.subtitle"))}
        {field("publisher", t("library.manage.field.publisher"))}
        {field("edition", t("library.manage.field.edition"))}
        <div className="flex flex-col gap-1">
          <label htmlFor={`${id}-year`} className="text-[0.9375rem] font-medium text-text-strong">{t("library.manage.field.year")}</label>
          <input id={`${id}-year`} inputMode="numeric" value={draft.publicationYear ?? ""} onChange={(event) => set("publicationYear", event.target.value ? Number(event.target.value.replace(/\D/g, "")) : null)} className="h-10 aw-input px-3" />
        </div>
        {field("isbn", t("library.manage.field.isbn"))}
        {field("doi", t("library.manage.field.doi"))}
        <div className="flex flex-col gap-1">
          <label htmlFor={`${id}-licence-status`} className="text-[0.9375rem] font-medium text-text-strong">{t("library.manage.field.licenceStatus")}</label>
          <select id={`${id}-licence-status`} value={draft.licenceStatus} onChange={(event) => set("licenceStatus", event.target.value as Source["licenceStatus"])} className="h-10 aw-input px-2">
            {LICENCE_STATUSES.map((status) => <option key={status} value={status}>{t(`library.licence.${status}`)}</option>)}
          </select>
        </div>
        {field("licence", t("library.manage.field.licence"))}
        <div className="flex flex-col gap-1 sm:col-span-2">
          <label htmlFor={`${id}-notes`} className="text-[0.9375rem] font-medium text-text-strong">{t("library.manage.field.notes")}</label>
          <textarea id={`${id}-notes`} value={draft.notes ?? ""} onChange={(event) => set("notes", event.target.value || null)} rows={3} className="aw-input px-3 py-2" />
        </div>
        <fieldset className="m-0 flex flex-col gap-2 border-0 p-0 sm:col-span-2">
          <legend className="mb-1 text-[0.9375rem] font-medium text-text-strong">{t("library.coverage.title")}</legend>
          <p id={`${id}-coverage-help`} className="m-0 type-meta">{t("library.coverage.help")}</p>
          <label htmlFor={`${id}-coverage-mode`} className="sr-only">{t("library.coverage.mode")}</label>
          <select id={`${id}-coverage-mode`} value={coverageMode} aria-describedby={`${id}-coverage-help`} onChange={(event) => setCoverage(event.target.value as SourceCoverageMode)} className="h-10 aw-input px-2">
            <option value="automatic">{t("library.coverage.automatic")}</option>
            <option value="manual">{t("library.coverage.manual")}</option>
            <option value="none">{t("library.coverage.none")}</option>
          </select>
          {coverageMode === "manual" ? <>
            <label htmlFor={`${id}-coverage-passages`} className="text-[0.9375rem] font-medium text-text-strong">{t("library.coverage.passages")}</label>
            <textarea id={`${id}-coverage-passages`} value={passagesText} onChange={(event) => setCoverage("manual", event.target.value)} rows={3} aria-describedby={`${id}-coverage-format`} className="aw-input px-3 py-2" />
            <p id={`${id}-coverage-format`} className="m-0 type-meta">{t("library.coverage.passagesHelp", { max: 8 })}</p>
          </> : coverageMode === "automatic" && <p className="m-0 type-meta">{(draft.coverage ?? []).length > 0 && draft.coverageMode !== "manual" ? t("library.coverage.detected", { passages: (draft.coverage ?? []).map(coverageLabel).join(t("common.listSeparator")) }) : t("library.coverage.automaticHelp")}</p>}
        </fieldset>
      </div>
      {children}
    </details>
  </>;
}

/** Send details and any deliberate coverage edit; file identity, text and annotations stay with the source. */
export function sourceMetadataInput(source: SourceMetadataDraft): UpdateSource {
  const { id, title, subtitle, author, publisher, edition, publicationYear, isbn, doi, language, sourceType, licenceStatus, licence, notes } = source;
  const input: UpdateSource = { id, title, subtitle, author, publisher, edition, publicationYear, isbn, doi, language, sourceType, licenceStatus, licence, notes };
  if (source.coverageEdit) {
    const { mode, passagesText } = source.coverageEdit;
    input.coverage = { mode, ...(mode === "manual" ? { passages: passagesText.split("\n").map((passage) => passage.trim()).filter(Boolean) } : {}) };
  }
  return input;
}

export function SourceEditButton({ title, onClick }: { title: string; onClick: () => void }) {
  const { t } = useTranslation();
  return <button type="button" onClick={onClick} aria-label={t("library.metadata.editNamed", { title })} title={t("library.metadata.editNamed", { title })} className="aw-icon-button size-11 shrink-0">
    <Pencil className="size-4" aria-hidden="true" />
  </button>;
}

/** A disposable draft: closing never writes, and Save keeps the same source and original file. */
export function SourceMetadataEditor({ sourceId, onClose, onSaved, returnFocus }: { sourceId: string; onClose: () => void; onSaved: (source: Source) => void; returnFocus?: () => HTMLElement | null }) {
  const { t } = useTranslation();
  const api = useApi();
  const loaded = useLoad(useCallback(() => api.libraryGetSource(sourceId), [api, sourceId]));
  const [edited, setEdited] = useState<SourceMetadataDraft | null>(null);
  const draft = edited ?? loaded.data;
  const [busy, setBusy] = useState(false);
  const saving = useRef(false);
  const [error, setError] = useState<string | null>(null);
  const close = () => { if (!saving.current) onClose(); };
  const save = async () => {
    if (!draft || !draft.title.trim() || saving.current) return;
    saving.current = true;
    setBusy(true);
    setError(null);
    try {
      const saved = await api.libraryUpdateSource(sourceMetadataInput(draft));
      onSaved(saved);
    } catch (error) {
      setError(toApiError(error).message);
    } finally {
      saving.current = false;
      setBusy(false);
    }
  };
  return <Dialog wide title={t("library.metadata.editTitle")} onClose={close} returnFocus={returnFocus} actions={<>
    <Button onClick={close} disabled={busy}>{t("common.cancel")}</Button>
    <Button variant="primary" onClick={() => void save()} disabled={busy || !draft?.title.trim()}>{t("library.manage.save")}</Button>
  </>}>
    {loaded.error && <Notice tone="error">{loaded.error.message}</Notice>}
    {error && <Notice tone="error">{error}</Notice>}
    {draft && <fieldset disabled={busy} className="m-0 flex flex-col gap-3 border-0 p-0">
      <SourceMetadataFields draft={draft} onChange={setEdited}>
        <dl className="mt-3 grid grid-cols-1 gap-x-4 gap-y-1 text-[0.9375rem] sm:grid-cols-[10rem_1fr]">
          <dt className="font-medium text-text-strong">{t("library.manage.file.name")}</dt>
          <dd className="m-0 break-all">{draft.originalFilename ?? "—"}</dd>
        </dl>
      </SourceMetadataFields>
    </fieldset>}
  </Dialog>;
}
