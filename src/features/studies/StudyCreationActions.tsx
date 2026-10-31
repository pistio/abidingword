import type { Ref } from "react";
import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";

/** Teaching begins within a Study; existing standalone records have their own compatibility route. */
export function StudyCreationActions({ onStudy, studyButtonRef }: { onStudy: () => void; studyButtonRef?: Ref<HTMLButtonElement> }) {
  const { t } = useTranslation();
  return <div className="flex flex-wrap items-center gap-2">
    <Button ref={studyButtonRef} variant="primary" onClick={onStudy}><Plus className="size-4" aria-hidden="true" />{t("studies.newStudy.formLabel")}</Button>
  </div>;
}
