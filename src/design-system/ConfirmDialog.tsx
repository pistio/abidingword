import { useId, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "./Button";
import { Dialog } from "./Dialog";

interface ConfirmDialogProps {
  /** What is asked, e.g. "Move “Romans” to Trash?". */
  title: string;
  /** What will happen, in plain words: what goes, what stays, whether it can be undone. */
  children: ReactNode;
  /** The one button that does it, named for what it does. */
  confirmLabel: string;
  /**
   * For deleting many things for good: the person types this (the number of
   * items) before the button can be used.
   */
  typeToConfirm?: string;
  onConfirm: () => Promise<void> | void;
  onCancel: () => void;
  /** Where focus goes afterwards when the button that opened it is gone. */
  returnFocus?: () => HTMLElement | null;
}

/**
 * A confirmation of something that removes or loses things (v0.24),
 * shared by Bibles and sources; its one action is the danger button. It opens centred
 * over a dimmed page with focus on Cancel. Escape, the backdrop and Cancel
 * all cancel; Enter on Cancel cancels, Enter in the typing field does
 * nothing; only the named button confirms. A failure is shown inside it.
 */
export function ConfirmDialog({ title, children, confirmLabel, typeToConfirm, onConfirm, onCancel, returnFocus }: ConfirmDialogProps) {
  const { t } = useTranslation();
  const bodyId = useId();
  const fieldId = useId();
  const [typed, setTyped] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const ready = !typeToConfirm || typed.trim() === typeToConfirm;

  const confirm = async () => {
    if (!ready || busy) return;
    setBusy(true);
    setError(null);
    try {
      await onConfirm();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setBusy(false);
    }
  };

  return (
    <Dialog
      role="alertdialog"
      title={title}
      describedBy={bodyId}
      onClose={() => !busy && onCancel()}
      returnFocus={returnFocus}
      actions={
        <>
          <Button onClick={onCancel} disabled={busy}>
            {t("common.cancel")}
          </Button>
          <Button variant="danger" onClick={confirm} disabled={!ready || busy}>
            {confirmLabel}
          </Button>
        </>
      }
    >
      <div id={bodyId} className="flex flex-col gap-2">
        {children}
      </div>
      {typeToConfirm && (
        <label htmlFor={fieldId} className="flex flex-col gap-1">
          <span>{t("common.confirm.typeCount", { count: Number(typeToConfirm) || 0, value: typeToConfirm })}</span>
          <input
            id={fieldId}
            value={typed}
            inputMode="numeric"
            autoComplete="off"
            onChange={(e) => setTyped(e.target.value)}
            onKeyDown={(e) => {
              // Enter never confirms from here: only the button does.
              if (e.key === "Enter") e.preventDefault();
            }}
            className="h-10 aw-input px-3 text-base"
          />
        </label>
      )}
      {error && (
        <p role="alert" className="m-0 text-danger">
          {error}
        </p>
      )}
    </Dialog>
  );
}
