# Study and Teaching editor corrections — stage #33

## Method and data preservation

Git history identifies `b2345e0` (`feat(studies): preserve legacy answers and
introduce continuous writing templates`) as the introduction of the incorrect
writing-template keys. The older structured POZETEK template had the correct
method. New writing Studies now use exactly Personen, Ort, Zeit, Ereignisse,
Textzusammenhang, Erklärungen and Kern (`persons/place/time/events/context/
explanations/core`), with one primary free-text answer each. Personal application
is a distinct optional part. The German name remains POZETEK; the other languages keep their descriptive
names, without inventing foreign acronyms. Guidance combines the existing original-method questions with
the owner's additional contextual distinctions. Ten-Step guidance adds practical
questions/output and retains the independently written Matthew 8:18–27 example.
It does not insert example answers into anyone's writing.

Installing the corrected builtin never rewrites existing Study snapshots. Old
Theme/Understanding/Consequences keep their keys, names and answers. Historical
wording remains translatable through `formerSections`; customized wording remains
verbatim. Existing templates, structured legacy fields, citations and Teaching
links remain supported.

**Review POZETEK parts** is an explicit preview and consent path for builtin
writing Studies. It shows which canonical parts exist or will be added empty and
reports duplicate keys. It moves the first existing occurrence of each canonical
key into the seven-part sequence, then keeps all other parts in their prior
relative order. It never infers that Theme writing means Textzusammenhang, renames
an existing part, merges duplicate Zeit parts or discards content. The current
section-create API already refuses duplicate keys; the owner's historical Study
is not present, so its exact duplicate origin is still an owner-data investigation.

Apply and receipt persistence are one SQLite transaction; the receipt uses the
existing settings table. Undo survives restart and restores the prior order,
removing only empty and completely unchanged parts added by that repair. It
refuses if added writing/metadata/notes changed or the section list/order changed.
Existing writing can continue to change without being discarded. Revisions advance
on reorder and Undo; no revision is rewound. A failed receipt write rolls back all
parts, blocks, ordering and revisions. No schema migration is added.

## Editor behavior

- Five initial writing lines, native vertical resize/scrolling, full autosave.
  JavaScript no longer overwrites an explicit drag size while typing.
- Meaningful hints start visible for empty parts; the visible Hide hints action
  stores a per-section preference. Hints in part options can reopen them. The
  generic repeated "Write in your own words" field help is suppressed for a
  part's sole writing field. Populated parts naturally open without default hints.
- The ordinary phase selector is removed; saved phases and advanced template
  editing remain. Rename, movement, Hints, optional AI Help and Delete remain.
- Every section deletion, empty or populated, opens a named modal. Keep/Escape,
  focus trap/return, pending-save check, failures and empty-part recovery remain.
- Text field and Q&A are direct additions. Distinct typed notes remain in a
  single secondary menu, preserving per-part drafts and focus without a nested
  "Add → More types" bar.
- Teaching uses **Textfeld hinzufügen** to create ordinary private writing in its
  actual parent. The secondary menu retains genuinely distinct Q&A, Scripture
  and structural section/outline-point controls. Equivalent Explanation,
  Illustration, Application, Time and other prose-block choices are removed from
  this menu. Their existing saved blocks/templates still render and export.
- Mein Denken prüfen retains exactly the three concise existing questions. No
  explanatory prose is added; preview/consent remains before optional AI. The
  redundant further-study-questions action is removed from Study Help only;
  stored inquiry results remain readable.

## Evidence and Windows owner procedure

Native invented schema-27 regressions exercise an old incorrect-key Study with
writing and duplicate Zeit, persistent repair/Undo after reopening the database,
changed-writing refusal, stale preview and injected receipt SQL failure. Existing
restore-missing, template catalogue, permission enforcement and #31 transfer
regressions remain in the full suite. UI regressions cover six-language method
identity, hints, full text, modal behavior, explicit preview/cancel/repair/Undo,
notes and Teaching privacy. See `images/33-*` for before/after, repair preview,
empty-section modal and numeric browser checks.

The Linux Chromium check really drags a textarea from about 164px to 248px, types
and blurs without resetting it, at 900 and 1440px; it also tests the empty-section
modal/Escape/focus and repair preservation. Its invented memory adapter is not
native SQLite or Windows acceptance; those are separate test layers.

On Windows, first back up and use a copied profile. Compare the original guide
and all seven parts in a new Study; review an old incorrect-key Study before
applying repair. Cancel must change nothing. Inspect every old answer, finding,
citation and Teaching link after Apply. Undo before writing new parts, then apply
again, add new writing, and confirm Undo refuses safely. Investigate duplicate
Zeit using actual section identities, never by deleting either part automatically.
Test five-line size, manual drag, long writing, restart, hints dismissal/reopen,
keyboard-only empty/populated deletion and recovery. Review Ten-Step Matthew
examples and the three unchanged thinking questions. Test Teaching text addition,
Q&A/Scripture and all private/public export variants. Real DeepSeek Korean and
all original A.4 gates remain owner acceptance, not automated acceptance.
