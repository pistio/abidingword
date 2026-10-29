import { useTranslation } from "react-i18next";
import { formatNumber } from "@/i18n/format";
import type { Source, SourceFormat } from "@/types/library";
import { formatLabel, licenceLabel, sourceTypeLabel } from "./labels";

/**
 * Bibliographic and provenance details for one source. Shown inside a
 * disclosure so the reading view stays a page of text.
 */
export function SourceAbout({ source, citation }: { source: Source; citation: string }) {
  const { t } = useTranslation();
  const sep = t("common.listSeparator");
  const format = source.format ? formatLabel(t, source.format as SourceFormat) : null;
  const pages = source.pageCount
    ? t("library.size.pages", { count: source.pageCount, number: formatNumber(source.pageCount) })
    : null;
  return (
    <>
    <dl className="m-0 mt-3 flex flex-col gap-3 text-[0.9375rem]">
      <Row term={t("library.manage.field.author")}>{source.author}</Row>
      <Row term={t("library.manage.field.year")}>{source.publicationYear}</Row>
      <Row term={t("library.manage.field.publisher")}>{source.publisher}</Row>
      <Row term={t("library.manage.field.edition")}>{source.edition}</Row>
      <Row term={t("library.manage.field.subtitle")}>{source.subtitle}</Row>
      <Row term={t("library.manage.field.isbn")}>{source.isbn}</Row>
      <Row term={t("library.manage.field.doi")}>{source.doi}</Row>
      <Row term={t("library.metadata.language")}>{source.language}</Row>
      <Row term={t("library.manage.field.category")}>{sourceTypeLabel(t, source.sourceType)}</Row>
      <Row term={t("library.manage.filter.format")}>{format}</Row>
      <Row term={t("library.manage.file.pages")}>{pages}</Row>
      <Row term={t("library.manage.field.collections")}>{source.collectionNames.length > 0 ? source.collectionNames.join(sep) : null}</Row>
      <Row term={t("library.manage.field.tags")}>{source.tagNames.length > 0 ? source.tagNames.join(sep) : null}</Row>
      <Row term={t("library.manage.field.licenceStatus")}>{licenceLabel(t, source.licenceStatus)}</Row>
      <Row term={t("library.manage.field.licence")}>{source.licence}</Row>
      <Row term={t("library.manage.file.name")}>{source.originalFilename}</Row>
      <Row term={t("library.filter.archive")}>
        {source.state === "archived" ? t("common.lifecycle.state.archived") : t("library.filter.inUse")}
      </Row>
      {citation && <Row term={t("library.about.citation")}>{citation}</Row>}
    </dl>
    <p className="mb-0 mt-3 text-[0.9375rem] text-text-muted">{source.filePresent ? t("library.about.fileHere") : t("library.about.fileMissing")}</p>
    </>
  );
}

function Row({ term, children }: { term: string; children: string | number | null | undefined }) {
  if (children === null || children === undefined || children === "") return null;
  return (
    <div>
      <dt className="text-text-muted">{term}</dt>
      <dd className="m-0 text-text-strong">{children}</dd>
    </div>
  );
}
