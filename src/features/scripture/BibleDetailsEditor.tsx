import { useId, useState, type FormEvent } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { LanguageChoice, type LanguageValue } from "./LanguageChoice";
import { toApiError } from "@/lib/api";
import { useApi } from "@/lib/ApiContext";
import type { BibleTranslation } from "@/types/scripture";

/** Editable display details only. Scripture, source files, licence, and evidence stay untouched. */
export function BibleDetailsEditor({ bible, onSaved, onCancel }: {
  bible: BibleTranslation;
  onSaved: (updated: BibleTranslation) => Promise<void> | void;
  onCancel: () => void;
}) {
  const { t } = useTranslation();
  const api = useApi();
  const id = useId();
  const [name, setName] = useState(bible.name);
  const [shortName, setShortName] = useState(bible.shortName);
  const [language, setLanguage] = useState<LanguageValue | null>({
    code: bible.languageCode, name: bible.language,
  });
  const [publisher, setPublisher] = useState(bible.publisher ?? "");
  const [notice, setNotice] = useState(bible.copyrightNotice ?? "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const canEditAttribution = bible.sourceOrigin !== "bundled";
  const complete = !!name.trim() && !!shortName.trim() && !!language?.name.trim();
  const save = async (event: FormEvent) => {
    event.preventDefault();
    if (!complete || saving) return;
    setSaving(true);
    setError(null);
    try {
      const updated = await api.bibleUpdateDetails({
        id: bible.id, name: name.trim(), shortName: shortName.trim(),
        language: language!.name.trim(), languageCode: language!.code,
        publisher: canEditAttribution ? publisher : bible.publisher,
        copyrightNotice: canEditAttribution ? notice : bible.copyrightNotice,
      });
      await onSaved(updated);
    } catch (e) {
      setError(toApiError(e).message);
    } finally {
      setSaving(false);
    }
  };
  return (
    <form onSubmit={(e) => void save(e)} aria-label={t("scripture.bibles.editDetails")} className="flex max-w-xl flex-col gap-3 rounded-aw border border-border p-4">
      <p className="m-0 text-[0.9375rem] text-text-muted">{t("scripture.bibles.detailsOnly")}</p>
      <label className="flex flex-col gap-1" htmlFor={`${id}-name`}>
        <span className="font-medium">{t("scripture.addBible.field.name")}</span>
        <input id={`${id}-name`} className="h-10 aw-input px-3" value={name} onChange={(e) => setName(e.target.value)} required maxLength={200} />
      </label>
      <label className="flex flex-col gap-1" htmlFor={`${id}-short`}>
        <span className="font-medium">{t("scripture.addBible.field.shortName")}</span>
        <input id={`${id}-short`} className="h-10 aw-input px-3" value={shortName} onChange={(e) => setShortName(e.target.value)} required maxLength={20} />
      </label>
      <LanguageChoice id={`${id}-language`} label={t("scripture.addBible.field.language")} value={language} onChange={setLanguage} />
      {canEditAttribution ? (
        <>
          <label htmlFor={`${id}-publisher`} className="flex flex-col gap-1">
            <span className="font-medium">{t("scripture.bibles.field.publisher")}</span>
            <input id={`${id}-publisher`} className="h-10 aw-input px-3" value={publisher} onChange={(e) => setPublisher(e.target.value)} maxLength={200} />
          </label>
          <label htmlFor={`${id}-notice`} className="flex flex-col gap-1">
            <span className="font-medium">{t("scripture.addBible.document.licenceField")}</span>
            <textarea id={`${id}-notice`} className="aw-input min-h-24 px-3 py-2" value={notice} onChange={(e) => setNotice(e.target.value)} maxLength={2000} />
          </label>
        </>
      ) : (
        <p className="m-0 text-[0.9375rem] text-text-muted">{t("scripture.bibles.attributionFixed")}</p>
      )}
      {error && <p role="alert" className="m-0 text-danger">{error}</p>}
      {!complete && <p role="status" className="m-0 text-[0.9375rem] text-text-muted">{t("scripture.bibles.requiredDetails")}</p>}
      <div className="flex flex-wrap gap-2">
        <Button type="submit" variant="primary" disabled={!complete || saving}>{t("scripture.bibles.saveDetails")}</Button>
        <Button onClick={onCancel}>{t("common.cancel")}</Button>
      </div>
    </form>
  );
}
