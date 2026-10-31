import { useId, useState } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";
import { formatNumber } from "@/i18n/format";

/** Physical PDF pages, shared by Reading and Original. Printed labels are evidence, never offsets. */
export function PdfPageNavigation({ page, total, printedLabel, onGo }: {
  page: number; total: number; printedLabel?: string | null; onGo: (page: number) => void;
}) {
  const { t } = useTranslation();
  const id = useId();
  const [draft, setDraft] = useState({ page, text: String(page), dirty: false });
  if (draft.page !== page) setDraft({ page, text: draft.dirty ? draft.text : String(page), dirty: draft.dirty });
  const wanted = Number(draft.text);
  const valid = Number.isInteger(wanted) && wanted >= 1 && wanted <= total;
  const navigate = (target: number) => {
    setDraft({ page, text: String(target), dirty: false });
    onGo(target);
  };
  return <nav aria-label={t("library.pageNav.label")} className="shrink-0 border-b border-border-subtle px-4 py-2">
    <form className="flex flex-wrap items-center gap-2" onSubmit={(event) => { event.preventDefault(); if (valid) navigate(wanted); }}>
      <Button disabled={page <= 1} onClick={() => navigate(page - 1)}>{t("library.pageNav.previous")}</Button>
      <label htmlFor={id} className="text-[0.9375rem]">{t("library.pageNav.page")}</label>
      <input id={id} type="number" min={1} max={total} step={1} value={draft.text} onChange={(event) => setDraft({ page, text: event.target.value, dirty: true })} className="aw-input h-10 w-20 px-2" aria-describedby={`${id}-hint`} />
      <span className="type-meta">{t("library.pageNav.total", { total: formatNumber(total) })}</span>
      <Button type="submit" disabled={!valid}>{t("library.pageNav.go")}</Button>
      <Button disabled={page >= total} onClick={() => navigate(page + 1)}>{t("library.pageNav.next")}</Button>
    </form>
    <p id={`${id}-hint`} className="type-meta m-0 mt-1">{t("library.pageNav.physical")}{printedLabel ? ` ${t("library.pageNav.printed", { label: printedLabel })}` : ""}</p>
  </nav>;
}
