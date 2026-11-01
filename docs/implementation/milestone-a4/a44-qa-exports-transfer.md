# A.4.4b — Q&A blocks, safe exports and "Add from Bible Study" (#27)

**Status: implemented and tested; Windows owner acceptance OPEN. No migration: schema stays 27; 0028 stays free for the
provisional Bible-import lineage.**

## Data (additive, no schema change)

- **Study Q&A**: a Study `question` block whose JSON `data` is `{"kind":"qa","question":…,"answer":…}`. The native core accepts such data
  only on a free question block and derives its `content` (question, blank line, answer) for search, export and **older builds**, which keep
  reading it as an ordinary question block (no new enum value an A.3 build could not load).
- **Teaching Q&A**: block type `qa` (block types are open text; an unknown type opens as writing in older builds). The question is the
  block title, validated with the body limit (no 200-character heading limit); the answer is the body. **For participants by default**
  (like discussion questions); the person can mark it leader-only.
- **Provenance** of copies lives in the existing `teaching_blocks.config` JSON as `origin {studyId, kind, itemId, label, copiedAt,
  citation?}`. Editing, moving and duplicating keep it; A.3 configs without it still load.

## Q&A in the editors

Optional and never forced. In a Study part: `⋯` → Add → **Question and answer**; several per part, numbered ("Question and answer 2"),
each with its own Question and Answer fields (autosaved; no short limit), moved or deleted from its `⋯` (an answered one asks by name). In a
Teaching: Add a block → **Question and answer**; question and answer fields both have AI help tied to the block.

## Exports (privacy)

Teaching materials (`derive.ts`):
- **Question sheet**: *questions only* by default; **With the answers (shared questions only)** is an explicit option (hidden on the leader's
  version, which always has the answers). Leader-only Q&A never appears on a participant sheet.
- **Worksheet**: shared questions with room to write.
- **Full lesson** (manuscript/leader's guide): Q&A in place; leader-only content only with "my notes".

Study: Study options → **Export questions…** → *Questions only / Questions and answers / Whole study*, previewed, then saved as Markdown
through the existing safe save path. Only the writing in the study's parts; separate notes are never exported.
Existing passage options keep their Bible edition behaviour; Q&A exports add no Scripture text.

## Add from Bible Study

Teaching options → **Add from Bible Study…** (block teachings with a linked study). Grouped checkboxes: writing, questions and answers,
notes (findings), quotations (with citation), passages. **Add selected** creates new blocks at the end:
- text read by the native core from **this teaching's own study** (never from the interface); another study's item, or one deleted
  meanwhile, refuses the whole request (all or nothing);
- every copy **starts for the leader only**, with a visible "From the Bible Study: …" (or "Quotation from the Bible Study · title,
  author, place") and the existing Leader only mark;
- snapshots: later edits on either side never synchronise; nothing existing is overwritten; passages become a Scripture block with the
  passage attached; an independent teaching is refused (link a study first).

## Tests

Rust `tests/a4_qa_transfer.rs` (6): Q&A data/derived text/long answers/refusals; copies with provenance, leader-only, citation line,
passage attached; snapshots never synchronised, provenance kept through edit and duplicate; foreign/deleted/unknown items refused with
nothing added and no leak; independent teaching refused; `qa` participant default and A.3 config compatibility.
UI `tests/ui/a4-qa-transfer.test.tsx` (7): question sheet privacy (questions only / with answers / leader / manuscript with and without
notes / worksheet), Study Markdown modes, Study Q&A add/save/numbering/delete confirmation, Study export dialog, Add from Bible Study end to end.
