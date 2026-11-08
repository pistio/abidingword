# Owner follow-up: Issues 35 and 33, 8 November 2026

Base: reviewed Windows branch `2b39fe838158072dfce9f30d6bd942916a408d43`.
No merge, release, paid Actions, installer, schema or Scripture-recognition change.
The separate Issue 38 packaging branch is not included.

## Issue 35: measured indexing correction

Latest owner Windows evidence: index 304,681 ms, 448/1,689 blocks at timeout;
other analysis work about four seconds. That attempt rolled back. Legacy zero
headings do not describe the latest extractor's result. The actual PDF is not
available here; Windows real-file acceptance remains **failed / open**.

A minimal SQLite reproduction with the established indexes and no ANALYZE shows
the old per-block query selecting `source_passages_origin_range (origin=?)`,
scanning all explicit links for each block despite the existing segment index.
With 4,112 links, 2,719 segment identities and 1,689 lookups:

| Lookup strategy | Approximate VM steps | Local wall time |
| --- | ---: | ---: |
| Old per-block query | 34,751,600 | 570.1 ms |
| One source-indexed fetch | 45,200 | 2.9 ms |

This isolates a query-plan hazard; it is not a claim to reproduce the owner's
305-second Windows wall time. No missing index is assumed. Existing migrations
define no source_passages triggers; source-segment FTS triggers do not apply to
role-only updates. SQLite's primary-key index handles role writes.

The new path fetches literals once through the existing source index, groups
them by segment, updates only changed roles, clears obsolete roles and retains
the original transaction/300-second deadline. It does not parse old text again
or change recognition/ranking. IDs, timestamps and literal provenance remain.
Separate read/role-write stages and aggregate timing/count logs contain no
source text, paths or IDs. Cancel/SQL errors roll back structure and all links.

The UI labels legacy metadata after a failed/cancelled run, suppresses the
misleading saved-empty warning for that run, explicitly explains blank-query
diagnostics and localises native analysis error codes in all six languages.

Tests cover the existing 400-page/2,719-segment/4,112-link fixture, cancellation
at the new stages, injected derived-link and role-update failures, literal
identity/timestamp preservation, fresh-session reuse, redundant-role-write
refusal, old-metadata UI and native coded-error localisation.

Owner gate: rebuild the real MacArthur PDF on a disposable Windows profile,
retain stage timings and metadata-only report, then **enter** `Apg 6` and save
another diagnostic. Check persisted analysis, heading provenance and navigation
after restarting. A blank field is general status, not an Acts diagnosis.

## Issue 33: compact source-faithful guidance

Source authority is the owner's latest Issue 33 comments, including quoted
private-source passages. The original unpublished guide/workshop files are not
available in this checkout and are not represented as directly inspected.

| Area | Source and treatment |
| --- | --- |
| P/O/Z/E/T/E | Owner's six concise German questions, attributed in Issue 33 to *Die Bibel verstehen und auslegen*, Part III and the 27-Oct template brief; translated individually. |
| K — Kern | Exact approved German triplet from latest comment: Textgegenstand → Textaussage → Textthema; three sequential paragraphs, one free-writing field. No examples or extra methodology. |
| Sermon/Talk introduction | YCC workshop, *Vorbereitung einer Bibelarbeit*: “Sie dient dazu, die Leute in das Thema hineinzuführen.” → “Führe die Teilnehmer in das Thema ein.” |
| Sermon/Talk main | Same section: “... zuerst die allgemeine Bedeutung des Textes herauszufinden und dann die persönliche Beziehung zu meinem Leben herzustellen.” → “Erarbeitet die Bedeutung des Textes und seinen Bezug zum Leben.” |
| Sermon/Talk conclusion | Same section: “Zum Schluss sollte das Gelernte und die wichtigsten Punkte zusammengefasst werden.” Owner-approved condensation includes transition to practical application. |
| EVA / Bible Lesson | No complete primary section definitions provided. Existing guidance retained; owner source confirmation remains open. No new teaching method invented. |

All six languages receive the Kern triplet and three source-backed Teaching
hints. Specialist terms distinguish subject, statement about that subject and
one-sentence synthesis; human linguistic/method confirmation remains an owner
gate. German POZETEK name, seven keys/order, optional personal application,
saved writing, repair/Undo and the accepted Ten-Step content remain unchanged.
Former English guidance is retained as recognised baseline metadata so existing
unchanged built-in sections display the compact guidance without rewriting data.
Custom guidance remains as authored. Hint dismissal and resizing are preserved.

## Verification / acceptance

Technical results and browser evidence are recorded with the follow-up commits.
Linux Chromium with invented data cannot establish Windows WebView2 or real-file
acceptance. All existing A.4 owner gates remain open. AI linguistic review is
not qualified human native-speaker approval.
