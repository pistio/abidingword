# A.4.3 — Writing-first Bible Study **and** Bible Teaching (#25)

**Status: both editors implemented together and tested; Windows owner acceptance OPEN.**

## Before / after (German, 1100 px three-pane workspace)

| | A.3 (before) | A.4.3 (after) |
|---|---|---|
| Bible Study (POZETEK) | [before-a3-study-de-1100](images/before-a3-study-de-1100.png) | [a43-study-de-1100](images/a43-study-de-1100.png) |
| Bible Teaching (EVA) | [before-a3-teaching-de-1100](images/before-a3-teaching-de-1100.png) | [a43-teaching-de-1100](images/a43-teaching-de-1100.png) |

Also captured at 640/900/1440 (`a43-*-de-<width>.png`, `before-a3-*-de-<width>.png`). The "before" images come from the
unchanged A.3 code (`3b59c28`), fed the same fixture by the same harness. These are browser captures, not Windows screenshots.

**Study, before:** Bibelstudium/Bibellehre + Angaben, In der Bibel öffnen, Schritt für Schritt / Fortlaufend schreiben,
Text · Verstehen · Anwendung, a TEXT phase header, then per section Hinweise (optional), Deine Antwort, an instruction and
+ Hinzufügen.
**Study, after:** **Römer 8** (the Study's own title, not the project) · `⋯` · **Bibelstudium | Bibellehre** · **Personen** ·
a spacious field · **Ort** …

**Teaching, before:** session title + Unterlagen, a Grundlage/Bearbeiten box, Angaben, Neue Bibellehre, Fortlaufend/Gliederung, then
per block four tool icons, "Den Teilnehmenden zeigen" (repeated for section and block), Hilfen and more tool icons.
**Teaching, after:** session title · quiet Materials icon · `⋯` · **Erkennen** with a quiet **Nur für die Leitung** mark and one `⋯` ·
a spacious field · **Verstehen** …

## What changed

Study (`StudyEditor`, `SectionEditor`, `FieldsEditor`):
- Header: back, title, one **Study options** toggle (closed by default; Escape returns focus). It reveals the unchanged
  Angaben (information dialog), passages, note filter and Continuous / Step-by-step choice. **Continuous writing stays the default.**
  An active note filter is still announced under the title when the options are closed.
- Removed from the default surface: the phase outline (Text · Verstehen · Anwendung), phase headers, the ten-step attribution at the
  top (moved to the end of the study, kept), per-field "Deine Antwort" labels and explanations, section status, the per-section Add bar.
- A part whose only field is a writing field has **no second label**. Its accessible name is "Personen — Deine Antwort".
- Each part has **one** `⋯`: Guidance (hints, goal, field help and the example), Help (AI, only when the assistant is on), Add
  (writing or note), Rename, Move up/down, Delete (named confirmation, as before), phase and (structured parts) status. Guidance is
  **kept, never lost**: on request, also after writing. A free-form part with nothing to write in yet still shows its Add.
- Save state: nothing when idle, a muted "Gespeichert" briefly, a visible "Speichert…" and a bordered red message on failure.
  **Fixed:** a failed structured-field save used to be reported as saved, because `FieldsEditor` swallowed the error; it now reaches the indicator.

Teaching (`TeachingView`, `BlockCanvas`):
- Header: session title, a quiet **Materials** icon (same name and menu as before), one **Teaching options** toggle with the
  Study link, information, the session switcher and **New Teaching**, passages and the Continuous/Outline view. The toggle's state
  survives switching sessions, so the keyboard stays on the picker.
- Each block, or each section holding one block of the same name, is **one unit**: heading, a persistent **Leader only** mark
  when its writing (or section) is private, and one `⋯` with Guidance, move, duplicate, add, **who it is for** and heading.
  **Delete now asks by name** (it used to delete at once). Visibility changes only by the person's explicit choice.
- Multiple Teaching sessions per Study, Study-first creation, independent Study/Teaching data and the legacy standalone route are unchanged.

No migration, no stored field keys or order changed, templates and attribution untouched. All six languages carry the new strings.

## Tests

`tests/ui/a4-writing-first.test.tsx` (8): German POZETEK surface (title ≠ project, two tabs, Personen first with a single
spacious field, no Angaben / Schritt für Schritt / Deine Antwort / Hinweise / Hinzufügen on screen), seven sections with continuous
default and kept hints, Study options + Escape focus, save error visible, Teaching surface, leader-only marks and explicit visibility, named
delete confirmation, Teaching options. Existing regressions now reach controls through their new place (`tests/support/studyActions.ts`);
the tests that described the removed outline/phase header or always-visible guidance were rewritten to the new rule, and none were deleted.

## Still for the owner (Windows)

Writing comfort at 100–200 % scaling, Narrator/NVDA reading of headings, fields and the `⋯` groups, German/Korean label lengths in real
use, and the judgement that hints and options are discoverable enough.
