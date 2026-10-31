import type { Ref } from "react";
import { Plus } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/design-system/Button";

/** Study is the usual first stage; a standalone Teaching is an explicit secondary path. */
export function StudyCreationActions({ onStudy, onTeaching, studyButtonRef }: { onStudy: () => void; onTeaching?: () => void; studyButtonRef?: Ref<HTMLButtonElement> }) {
  const { t } = useTranslation();
  return <div className="flex flex-wrap items-center gap-2">
    <Button ref={studyButtonRef} variant="primary" onClick={onStudy}><Plus className="size-4" aria-hidden="true" />{t("studies.newStudy.formLabel")}</Button>
    {onTeaching && <Button onClick={onTeaching}><Plus className="size-4" aria-hidden="true" />{t("studies.teaching.newTeaching")}</Button>}
  </div>;
}
