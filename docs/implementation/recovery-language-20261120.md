# Recovery interface: six-language editorial decisions

This review covers the recovery controls and messages added or changed for the
recovery-and-preservation milestone. It is an AI editorial review of English
(British), German, Korean, French, Spanish and Brazilian Portuguese, followed
by project parity/corpus checks. It is not human native-speaker approval or
Windows layout acceptance. The existing `pt` resource serves the Portuguese
interface; the wording here follows the owner's Brazilian Portuguese request.

## Meaning and action decisions

| Source keys (`settings.data`) | Before | Decision and revised meaning |
| --- | --- | --- |
| `restoreWarning` | The safety copy meant “nothing is lost”; reload sounded automatic. | State the actual protections: a checked database safety copy, differing library files retained separately, and an explicit reload before continuing. Avoid an unconditional data-loss guarantee. |
| `safetyCopy` | “A copy of what was here before…” | Name the **database** safety copy so this cannot be mistaken for another full backup. Preserve `{{path}}` exactly. |
| `restoredContinue` | “Continue” | Name the action: **Reload Abiding Word**. All six buttons describe reloading, not merely dismissing a message. |
| `restoreReloadHint` | No dedicated explanation. | Editing remains paused until the restored work has been loaded. |
| `restoreInProgress`, `restoreInProgressHint` | Generic working message. | Explain that current work is being saved before restoration; keep the application open. |
| `restoreAlreadyRunning` | No dedicated explanation. | Wait for the existing restoration; do not start a concurrent replacement. |
| `restoreReloadRequired` | No dedicated explanation. | The database has already been restored; reload before continuing. |
| `restoreSaveFailed` | No dedicated explanation. | Restoration is blocked by a save failure. Keep the current writing and resolve that failure before retrying recovery. |
| `restoreNeedsAttention`, `restoreFailureReloadHint` | No separate locked failure state. | If recovery cannot fully put the previous work back, keep editing paused, retain the named safety copy and reload before trying to continue. Do not imply that every failure is harmless. |
| `dailyHint` | Daily copies excluded “books or images”. | Name imported source files, Bible files and images explicitly. Copies are created while the application runs; at most seven verified copies are retained. Point to the full-backup action. |
| `dailyWarning` | Missing imported files were described inconsistently across languages. | Current database writing/settings are replaced; source/Bible/image files stay unchanged, including files that are already missing. Older citations may therefore lack a source file. |
| `dailyRestored` | Combined completion, file status and “reopen” instruction. | Keep the factual completion sentence; the shared reload message and explicit button give the next action. |

## Individual language decisions

| Language | Before → after evidence | Review decision |
| --- | --- | --- |
| EN | “Continue” → “Reload Abiding Word”; “so nothing is lost” → “first saves and checks a safety copy of your current database” | Plain action verbs; British application style; precise preservation scope. |
| DE | “Weiter” → “Abiding Word neu laden”; “damit nichts verloren geht” → “legt zuerst eine Sicherheitskopie deiner aktuellen Datenbank an und prüft sie” | Retain informal **du/deine**. Use *Wiederherstellung*, *Datenbankkopie* and *Sicherheitskopie* consistently. |
| KO | “계속” → “Abiding Word 다시 불러오기”; “잃는 것은 없습니다” → “먼저 현재 데이터베이스의 복구용 사본을 만들고 검증합니다” | Retain polite instructions. *복원* is the action; *다시 불러오기* states the reload. Imported materials remain *자료 파일*, not software source code. |
| FR | “Continuer” → “Recharger Abiding Word”; “afin que rien ne soit perdu” → “crée d’abord une copie de sécurité … et la vérifie” | Retain **vous/votre**, typographic apostrophes and French quotation marks. Use *paramètres* consistently with the surrounding Settings UI. |
| ES | “Continuar” → “Recargar Abiding Word”; “para que no se pierda nada” → “primero guarda y verifica una copia…” | Retain **tú/tu**. The button names reloading; the explanation uses the familiar *vuelve a cargar*. |
| pt-BR | “Continuar” → “Recarregar o Abiding Word”; “para que nada se perca” → “primeiro salva e verifica uma cópia…” | Brazilian *arquivos*, *salvar*, *banco de dados* and **você/seu**; consistent product article *o Abiding Word*. |

Recovery-point metadata uses an ordinary database-version label in all six
languages, replacing abbreviated schema jargon in German, French and Spanish.
The unrecoverable-failure heading is adapted idiomatically (for example German
*Wiederherstellung fehlgeschlagen* and French *Récupération incomplète*) rather
than literally translating “needs attention”.

Existing available/unavailable labels, recovery-point selection, confirmation
labels, shared Cancel, full-backup privacy advice and native snapshot errors
were reviewed for consistency and retained where their meaning remained correct.
No theological terms, Bible text, teaching method, saved user writing or internal
identifier was changed.

## Source and generated material

- Edited source: six `src/i18n/locales/*/settings.json` files and the user guide
  `docs/guide/08-settings-updates-backups.md`.
- Generated by `node scripts/i18n-corpus.mjs`: the complete six-language review
  corpus and priority report under `docs/i18n/`. Neither report was hand-edited.
- The full final validation record belongs to the milestone's verification
  report. Corpus generation/parity does not establish native readability,
  screen-reader behaviour or Windows high-DPI acceptance.

No unresolved wording question blocks this change. Native Windows evidence
remains required for the long warnings, safety-copy paths and reload dialog at
narrow widths/high display scaling; source-language parity alone cannot prove it.

## Second consistency review

The parent review compared all six changed warning, database-copy, failure,
source-file-exclusion and reload messages against the actual recovery states.
The shared terminology and interpolation tokens are consistent; no protected
method or theological wording was changed. The real React browser matrix also
checks all six languages at narrow and reduced/scaled browser viewports, with
long invented Windows paths. The parent visually inspected English and Korean
completion captures. Exact final results are recorded in the milestone
verification JSON; none of these checks claims human or native Windows approval.
