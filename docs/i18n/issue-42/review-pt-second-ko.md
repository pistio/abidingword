# Korean — independent second consistency review

Reviewer: the separate Brazilian Portuguese editorial agent, acting as an AI Korean consistency reviewer. No human-native approval is claimed.

Scope: independently read **all 46 edited keys**, **all 30 new display labels** and the **48 selected high-risk contexts listed below** after the first Korean agent's complete corpus read. This is a second consistency/challenge pass, not a claim to have independently reread all 4,046 final Korean leaves. The primary per-key decisions are in [review-ko-decisions.json](review-ko-decisions.json).

## Findings and resolution

| Context | Finding | Adjudication |
| --- | --- | --- |
| `settings.assistant.imagesLeave` | The retained “글쓰기 서비스와 달리” made the local-writing contrast implicit. This consent warning should explicitly identify remote image generation even when writing is local. | Suggested **글쓰기 도우미가 이 컴퓨터에서 실행되더라도 이미지 설명은 인터넷으로 OpenAI에 전송됩니다.** The Korean agent accepted and applied it; wording, provider and outgoing-data meaning checked again. Resolved. |
| `library.structure.processing.background` | Challenge whether replacing a “safe processing boundary” with a natural stopping point weakens deferred cancellation or implies immediate cancellation. | Accepted: **취소 요청은 … 시점에 적용** still says the request takes effect at a safe point; **최종 저장 중에는 취소할 수 없습니다** explicitly retains the commit restriction. |
| `studies.builtin.templates.passage-study.sections.related-scripture.guidance` | Challenge whether **되울리다 → 그 내용을 떠올리게 하다** changes the meaning of literary echo. | Accepted as a natural nontechnical paraphrase of a passage recalling another passage's content. The distinct explain/echo/fulfil alternatives remain; it asserts no new correspondence or theology. |
| Bible lesson hint/explanation/questions guidance | Challenge whether naturalising the age/communication and leader-question wording changes the method or privacy defaults. | Accepted: delivery can vary by age while the passage's meaning cannot; explanation does not replace meaning; leader notes remain visible only to the leader. No new instructional content is added. |
| Word information terminology | Check 원형/사전 번호 against existing controls and distinguish metadata from invented Scripture or dictionary definitions. | Accepted: guide/download labels now match live word detail/search controls. No new lexical data, definitions or source text is supplied. |
| Study findings vs annotations | Check 기록/메모 distinction in Q&A export, repair, Study import and settings unlink warning. | Accepted: Study findings use 기록; private leader and verse/source notes retain 메모. No additional notes are shared or exported. |
| New additional-book and format labels | Read all 19 enum names and two format labels against `src-tauri/src/scripture/canons.rs`, including 1/2 Esdras, Greek Esther/Daniel, Psalm 151 and Maccabees numbering. | Accepted: Greek/Latin qualifiers and source identities retained; conventional Korean titles identify omitted import content. Canon registry, references, ordinals and source titles remain untouched. The `{{format}} 파일` wrapper works for 웹 페이지 and OSIS 및 USX. |
| Three Word custom paragraph style display names | Read **항목 제목 / 답변 줄 / 마지막 답변 줄** for the small field heading, ruled response line and final response line. | Accepted: natural Korean role labels with an explicit final-line distinction; custom display names only, no stable style ID, built-in Word name or user text changed. |
| Three PowerPoint selection/accessibility names | Read **색 띠 / 빈 화면 / 슬라이드 이미지** for the colour rectangle, blank layout and notes-page thumbnail. | Accepted: concise Korean descriptors, colour-band term consistent with the template importer, familiar PowerPoint blank-slide wording. Names only; no schema identifiers changed. |
| Three source locator labels | Read page ranges and singular/range paragraphs in consent/evidence metadata. | Accepted: Korean number-before-unit syntax, exact `first`, `last`, `number` placeholders; no quoted source or citation content changed. |

## Preservation and high-risk checks

The 46 changes and 30 additions preserve respectful Korean UI register and the established **성경공부 / 성경 가르침**, **자료실 / 자료 / 가르침 자료**, **공부·가르침 틀 / 교회 서식**, **기록 / 메모** distinctions. Proper publisher/provider names and source quotations are intentional literals.

All 456 protected-retain entries were also checked mechanically against their recorded before strings: unchanged. In particular, accepted Ten-Step wording is **not** normalised to POZETEK terminology. Its pre-existing description of subject/statement/theme remains owner-protected, while the POZETEK Kern triplet retains **대상 / 진술 / 주제**. Imported Scripture and the sample verse retain their source wording rather than being modernised to the UI register.

Selected warnings continue to disclose unsaved last changes, database safety copies, unchanged metadata after timeout/cancel/failure, rollback failure and recovery path, unsupported-code fallback, original-file preservation, no invented word metadata, unverified Scripture exclusion, backup replacement, key exclusion, local/remote AI boundary, leader-only materials, no automatic relinking and repair Undo gates. The presenter still says only slides reach the audience, never notes/manuscript/sources.

All edited/retained ledger placeholders were mechanically compared, including new locator placeholders; the second linguistic assessment is the read described above, not that comparison. Full runtime/browser/native verification belongs to the coordinating evidence log. No real Windows or human-native approval is inferred.

No unresolved Korean wording finding remains from this consistency pass. Owner evaluation of protected teaching materials, real Windows acceptance and any later qualified human proofreading remain separate gates.

## Selected high-risk contexts read

- `errors.kind.database`
- `errors.kind.backup`
- `errors.kind.migration`
- `errors.library.structure.timeout`
- `errors.library.structure.cancelled`
- `errors.library.structure.failed`
- `errors.messageUnavailable`
- `errors.assistant.keyStore`
- `errors.assistant.staleTarget`
- `errors.portability.restoreFailed`
- `errors.portability.rollbackFailed`
- `library.structure.timeout`
- `library.structure.processing.background`
- `library.structure.processing.previous`
- `library.pane.matchEvidence`
- `library.pane.mentionsOnly`
- `library.delete.text`
- `library.delete.kept`
- `scripture.addBible.document.noWords`
- `scripture.addBible.document.unverified_other`
- `scripture.bibles.lifecycle.keptText`
- `scripture.bibles.words.replaces`
- `scripture.addBible.guide.intro`
- `settings.assistant.remote`
- `settings.assistant.local`
- `settings.assistant.imagesLeave`
- `settings.data.backupHint`
- `settings.data.restoreWarning`
- `settings.trash.studyHint`
- `studies.teaching.notesPrivate`
- `studies.teaching.blocks.leaderOnly`
- `studies.teaching.fromStudy.help`
- `studies.teaching.fromStudy.added`
- `studies.teaching.workspace.keepNotes`
- `studies.qaExport.help`
- `studies.repair.explain`
- `studies.repair.safety`
- `studies.repair.finishDrafts`
- `studies.builtin.templates.pozetek-writing-study.sections.core.guidance`
- `studies.builtin.templates.ten-step-writing-study.sections.step-09-message.guidance`
- `studies.builtin.templates.passage-study.sections.related-scripture.guidance`
- `studies.teaching.builtin.bibleLesson.hint`
- `studies.teaching.builtin.bibleLesson.blocks.explain.guidance`
- `studies.teaching.builtin.bibleLesson.blocks.questions.guidance`
- `presentation.presenter.audienceHelp`
- `presentation.export.includeNotes`
- `presentation.export.private`
- `presentation.templates.sample.verse`
