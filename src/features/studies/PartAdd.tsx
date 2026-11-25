import { useTranslation } from "react-i18next";
import { Plus } from "lucide-react";
import { Button } from "@/design-system/Button";
import { FINDING_TYPES, type FindingType } from "@/types/studies";
import { ActionMenu } from "@/design-system/ActionMenu";
import { findingLabel } from "./labels";

interface PartAddProps {
  part: string;
  onWriting: () => void;
  onNote: (kind: FindingType) => void;
  onQa?: () => void;
}

/** Ordinary writing is one action; Q&A remains a distinct optional action. */
export function PartAdd({ part, onWriting, onNote, onQa }: PartAddProps) {
  const { t } = useTranslation();
  return <div className="flex flex-wrap items-center gap-1">
    <Button variant="quiet" onClick={onWriting} aria-label={t("studies.add.textTo", { part })}>
      <Plus className="size-4" aria-hidden="true" />{t("studies.add.textField")}
    </Button>
    {onQa && <Button variant="quiet" onClick={onQa}>{t("studies.qa.add")}</Button>}
    <span data-part-note><ActionMenu portal label={`${t("studies.add.note")} — ${part}`} actions={FINDING_TYPES.map(kind => ({
      label: findingLabel(t, kind), onSelect: () => onNote(kind),
    }))} /></span>
  </div>;
}
