# Korean — complete individual AI editorial pass

Baseline: `64dadc0d1bb263362aa384951612e9395d705725`. All **4,016 baseline Korean string leaves** were read in the nine namespace files, including unchanged/legacy keys and every Korean `_other` branch. Three subsequently introduced source-location labels were individually reviewed as well (**4,046 final leaves**, including the 21 supplemental native enum labels below). This is an **AI linguistic review, not human native-speaker approval**.

[The decision ledger](review-ko-decisions.json) records the key, file, visible context, before/after text, decision and reason for every leaf. It was serialised after actual reading of the namespace batches; it is not a fingerprint inventory presented as editorial work. Shared retention reasons describe the criteria applied to the indicated feature context. [The focused edit log](review-ko-edits.json) records the 46 existing-key corrections separately.

## Register and terminology

- Respectful Korean: concise noun/action labels, polite status messages and instructions. Existing self-questions in source-backed study guidance retain their reflective register; they are not impolite application commands.
- **성경공부** is the Study workspace; **성경 가르침** is Teaching. Correct the new separated `성경 공부` labels to match the accepted product term.
- **기록** is a separate Study finding; **메모** is a verse/source annotation or private Teaching/presenter note. Filtered findings, imported findings, retained snapshots, Q&A-export exclusions and repair drafts now follow that distinction.
- **성경공부 틀 / 성경 가르침 틀** are reusable writing structures. **교회 서식** is a slide design. No new `템플릿`, `섹션`, `블록` or `프로젝트` loanwords were introduced.
- **자료실** is the Library, **자료** is a Source, **출처** is citation/provenance, **가르침 자료** is Teaching Materials. **전체 원고 / 인도자용 안내서 / 교사용 지도안 / 질문지 / 활동지 / 말씀 노트** retain the accepted distinct meanings.
- **용례 찾기** is the live concordance tool. Two new block labels incorrectly called it a dictionary (`성구 사전`); they now match the existing tool. Word-information controls consistently say **원형 / 문법 / 사전 번호**; specialist template prose may still use 표제어 where discussing a lexical entry rather than a control name.
- Bible names, 장/절 reference distinctions, source edition names and technical morphological categories retain their established identity. Dynamic Korean particles such as `은(는)` remain where the ending of user/source text cannot be known statically; no unsafe particle inference was introduced.

## Coverage and adjudication

| Namespace | Baseline leaves | Edits | Protected retain | Retain | Added |
| --- | ---: | ---: | ---: | ---: | ---: |
| app | 43 | 1 | 0 | 42 | 0 |
| books | 68 | 0 | 68 | 0 | 0 |
| common | 48 | 0 | 0 | 48 | 0 |
| errors | 280 | 2 | 0 | 278 | 2 |
| library | 550 | 6 | 0 | 544 | 0 |
| presentation | 405 | 1 | 1 | 403 | 3 |
| scripture | 580 | 5 | 0 | 575 | 19 |
| settings | 256 | 3 | 0 | 253 | 0 |
| studies | 1786 | 28 | 387 | 1371 | 6 |

The reading covered navigation/accessibility and keyboard labels; native error/dialog captions; Source import, PDF analysis, original and formatted readers, citations and destructive actions; Bible import/editions/recognition warnings and morphology; all Study and Teaching templates, findings, Q&A and repair; materials, slides, presenter and exports; AI consent/provider setup/search/images; and settings, backup/restore and lifecycle states.

The entire Ten-Step exegesis and writing variants, attribution, Bible-book labels, POZETEK text/Kern triplet and existing quoted sample Scripture were read and explicitly retained. No substantive method rewrite was made. Three Bible Lesson sentences receive only idiomatic Korean repairs: one lesson is a 성경 수업, explanation uses words that can be understood, and question/leader-answer notes are described in ordinary Korean. The distinction between changing delivery for age and preserving the passage’s meaning is unchanged.

Source usage was consulted for the actual 구조 분석 button, concordance block kind, finding filters/import groups, Study Q&A export exclusions, repair capture drafts, native caption roles and source-location metadata. Runtime screenshot/platform checks are tracked by the integration reviewer; this language pass does not claim every key was rendered in an actual Windows dialog.

## Before/after evidence

| Key/context | Before | After | Reason |
| --- | --- | --- | --- |
| `app.updates.help` | Abiding Word는 가끔, 또는 요청할 때만 확인합니다. 작업에 인터넷 연결이 필요 없고, 성경공부에는 영향이 없습니다. | Abiding Word는 가끔 또는 직접 요청할 때만 업데이트를 확인합니다. 인터넷 연결 없이도 작업할 수 있으며, 성경공부에는 영향이 없습니다. | Remove the dangling 가끔/or coordination; keep occasional/on-request checks, offline work and unaffected studies. |
| `errors.kind.integrity` | 성경공부 데이터가 손상된 것으로 보여, Abiding Word가 변경하기 전에 작업을 멈췄습니다. 파일은 수정되지 않았습니다. 자세한 내용은 로그 파일에 있습니다. | 성경공부 데이터가 손상된 것으로 보여 Abiding Word가 데이터를 변경하기 전에 작업을 멈췄습니다. 파일은 수정되지 않았습니다. 자세한 내용은 로그 파일에 있습니다. | Give 변경하기 an explicit object and remove an unnatural comma; preserve the no-modification assurance. |
| `errors.templates.partAlreadyAdded` | 이 공부 틀의 항목은 이미 성경 공부에 추가되어 있습니다. | 이 공부 틀의 항목은 이미 성경공부에 추가되어 있습니다. | Use the approved product term 성경공부. |
| `library.filter.archiveHelp` | 사용 중인 자료는 공부 옆에 나타납니다. 보관된 자료는 자료실에 남고, 설정에서 복원할 때까지 자료 창에는 나오지 않습니다. | 사용 중인 자료는 성경공부 옆에 나타납니다. 보관된 자료는 자료실에 남고, 설정에서 복원할 때까지 자료 창에는 나오지 않습니다. | Name the Study pane consistently without altering archive visibility. |
| `library.pane.noRecent` | 이번 사용 중에 연 자료가 여기에 표시됩니다. | 이번에 앱을 연 뒤에 읽은 자료가 여기에 표시됩니다. | Replace the literal 이번 사용 중에 with a natural description of the current session. |
| `library.passage.bookSection` | 책 단위 부분 {{passage}}(더 넓음) | 책 단위 구간 {{passage}}(더 넓은 범위) | Make the broader scope explicit and use the structure panel’s 구간 wording. |
| `library.structure.processing.background` | 이 창을 닫고 계속 읽거나 쓸 수 있습니다. 취소는 다음 안전한 처리 경계에서 적용되며 최종 저장 중에는 취소할 수 없습니다. | 이 창을 닫고 계속 읽거나 쓸 수 있습니다. 취소 요청은 현재 작업을 안전하게 멈출 수 있는 시점에 적용되며, 최종 저장 중에는 취소할 수 없습니다. | Replace processing-boundary jargon with a natural explanation while preserving deferred cancellation and the commit restriction. |
| `library.structure.processing.missing` | 저장된 제목 분석이 없습니다. 구조 만들기를 선택하세요. 분석 전에 일반 진단 보고서를 저장할 수도 있습니다. | 저장된 제목 분석이 없습니다. ‘구조 분석’을 선택하세요. 분석 전에 일반 진단 보고서를 저장할 수도 있습니다. | Match the actual 구조 분석 button instead of the nonexistent 구조 만들기 action. |
| `library.fileDialog.sources` | 자료실에 추가할 파일들 선택 | 자료실에 추가할 파일 선택 | Korean 파일 already permits multiple files; remove the unnecessary 들 suffix in the picker caption. |
| `scripture.addBible.words.not.tooLarge_other` | 어떤 성경보다도 단어 주석이 많아 하나도 넣지 않습니다. 본문은 그대로입니다. | 성경 한 권에 들어갈 수 있는 범위보다 단어 주석이 많아 하나도 넣지 않습니다. 본문은 그대로입니다. | Express the sanity limit naturally instead of the odd comparison with any Bible; preserve exclusion and unchanged text. |
| `scripture.addBible.guide.intro` | 일부 판본은 각 단어에 사전형, 문법, 어휘 번호를 제공합니다. 일부는 위에 포함되어 있거나 검증된 내려받기로 제공되며, 나머지는 발행처에서 내려받은 뒤 여기서 고를 수 있습니다. | 일부 판본은 각 단어에 원형, 문법, 사전 번호를 제공합니다. 일부는 위에 포함되어 있거나 검증된 내려받기로 제공되며, 나머지는 발행처에서 내려받은 뒤 여기서 고를 수 있습니다. | Use the same 원형 and 사전 번호 terms as the actual word-information controls. |
| `scripture.addBible.document.noWords` | 단어 정보(사전형, 문법, 사전 번호): 없음. 일반 문서에는 이런 정보가 없으며, 텍스트에서 지어내지 않습니다. | 단어 정보(원형, 문법, 사전 번호): 없음. 일반 문서에는 이런 정보가 없으며, 텍스트에서 지어내지 않습니다. | Match the 원형 word-information label; keep the no-invented-data warning. |
| `scripture.addBible.available.contains.lemma` | 사전형 | 원형 | Match the 원형 search and word-detail label. |
| `scripture.addBible.available.contains.strong` | 어휘 번호 | 사전 번호 | Match the 사전 번호 label used for lexical-number searches. |
| `settings.assistant.keyHelp.costA` | 대부분의 서비스는 사용한 만큼 조금 요금을 받으며, 키가 작동하기 전에 결제 정보나 선불 충전을 요구할 수 있습니다. 결제는 Abiding Word가 아니라 그 웹사이트에서 합니다. | 대부분의 서비스는 사용량에 따라 요금을 청구하며, 키를 사용하려면 결제 정보를 입력하거나 미리 충전해야 할 수 있습니다. 결제는 Abiding Word가 아니라 해당 웹사이트에서 합니다. | Replace the unnatural 조금 요금을 받으며 and key-works calque with ordinary Korean billing language; retain possible prepayment. |
| `settings.trash.studyHint` | 성경공부와 자료는 설정 → 데이터 및 백업 → 휴지통에서 복원할 수 있습니다. 연결된 가르침은 저장된 메모와 인용을 유지하며 성경공부 모음에 남지만 연결은 해제됩니다. 복원해도 자동으로 다시 연결되지 않습니다. | 성경공부와 자료는 설정 → 데이터와 백업 → 휴지통에서 복원할 수 있습니다. 연결된 가르침은 저장된 기록과 인용을 유지하며 성경공부 모음에 남지만 연결은 해제됩니다. 복원해도 자동으로 다시 연결되지 않습니다. | Match the exact 데이터와 백업 settings label and use 기록 for retained Study findings; preserve unlinking and restoration semantics. |
| `presentation.source.point` | 성경 가르침 {{number}}번째 부분 | 성경 가르침의 {{number}}번째 부분 | Add the missing possessive particle in the slide provenance label. |
| `studies.builtin.templates.topical-study.name` | 주제별 성경 공부 | 주제별 성경공부 | Use the approved 성경공부 product term in the template name. |
| `studies.builtin.templates.topical-research-study.name` | 주제별 성경 공부 | 주제별 성경공부 | Match the corresponding template and Study type name. |
| `studies.builtin.templates.passage-study.sections.related-scripture.guidance` | 이 본문을 설명하거나, 되울리거나, 성취하는 다른 구절들. | 이 본문을 설명하거나, 그 내용을 떠올리게 하거나, 성취하는 다른 구절들. | Replace the unnatural 되울리다 calque with idiomatic wording for echo/allusion; retain the existing explain/echo/fulfil distinctions. |
| `studies.builtin.templates.word-research-study.sections.occurrences.fields.concordance.label` | 성구 사전 | 용례 찾기 | Match the established concordance tool name; this is a live search tool, not a dictionary. |
| `studies.editor.options` | 성경 공부 옵션 | 성경공부 옵션 | Use the approved 성경공부 product term in the accessible menu name. |
| `studies.editor.filterOn` | 메모가 필터링되어 있습니다. | 기록에 필터가 적용되어 있습니다. | The code filters findings, whose approved Korean name is 기록; reserve 메모 for annotations. |
| `studies.templates.intro` | 공부 틀은 시작하는 구성입니다. 새 성경공부가 시작할 때의 부분들, 또는 가르침의 모양을 정합니다. 공부 틀을 바꿔도 이미 그 틀로 만든 성경공부는 바뀌지 않습니다. | 공부·가르침 틀은 새 성경공부나 가르침을 시작할 때의 구성을 정합니다. 틀을 바꿔도 이미 그 틀로 만든 성경공부는 바뀌지 않습니다. | Describe a starting structure naturally and include both template kinds shown in Settings; preserve existing-instance independence. |
| `studies.teaching.saveTemplate.keeps.points` | 가르침 틀에는 형식과 이 가르침의 대지 개수가 순서대로 담깁니다. 대지는 제목 없이 시작합니다. | 가르침 틀에는 형식과 대지의 수·순서가 담깁니다. 대지는 제목 없이 시작합니다. | Avoid saying a count is stored in order; preserve the number/order distinction and untitled points. |
| `studies.teaching.saveTemplate.keeps.generalPoints` | 가르침 틀에는 형식과 이 가르침의 주요 내용 개수가 순서대로 담깁니다. 주요 내용은 제목 없이 시작합니다. | 가르침 틀에는 형식과 주요 내용의 수·순서가 담깁니다. 주요 내용은 제목 없이 시작합니다. | Avoid saying a count is stored in order; preserve shape-specific 주요 내용 wording. |
| `studies.teaching.builtin.bibleLesson.hint` | 어린이와 청소년을 위한 하나의 시간입니다. 나이는 전달 방식을 바꾸지, 본문의 뜻을 바꾸지 않습니다. | 어린이와 청소년을 위한 성경 수업입니다. 나이에 따라 전달 방식은 달라지지만, 본문의 뜻은 바뀌지 않습니다. | Replace 하나의 시간 and the literal contrast construction with natural Korean; retain the exact age/meaning distinction. |
| `studies.teaching.builtin.bibleLesson.blocks.explain.guidance` | 본문의 뜻을 그들이 따라올 말로 말합니다. 더 쉬운 다른 뜻으로 바꾸지 않습니다. | 이해할 수 있는 말로 본문의 뜻을 설명합니다. 더 쉬운 다른 뜻으로 바꾸지 않습니다. | Replace 따라올 말로 말합니다 with natural Korean explanation wording; preserve the ban on changing the text’s meaning. |
| `studies.teaching.builtin.bibleLesson.blocks.questions.guidance` | 모둠이 듣는 질문과, 들을 대답에 대한 인도자의 메모. 메모는 인도자에게 남습니다. | 모임에서 나눌 질문과, 어떤 답에 귀 기울일지 적는 인도자 메모입니다. 메모는 인도자만 봅니다. | Replace the awkward group-hears/questions and listen-for-answers calques; retain leader-only notes. |
| `studies.teaching.fromStudy.open` | 성경 공부에서 가져오기… | 성경공부에서 가져오기… | Use the approved 성경공부 product term in the action label. |
| `studies.teaching.fromStudy.title` | 성경 공부에서 가져오기 | 성경공부에서 가져오기 | Match the action and Study title. |
| `studies.teaching.fromStudy.help` | 이 가르침에 복사할 내용을 고르세요. 복사본은 자유롭게 고칠 수 있고 출처가 표시되며, 처음에는 인도자 전용이고 성경 공부와 자동으로 맞춰지지 않습니다. | 이 가르침에 복사할 내용을 고르세요. 복사본은 자유롭게 고칠 수 있고 출처가 표시되며, 처음에는 인도자 전용이고 성경공부의 변경 내용이 자동으로 반영되지 않습니다. | Use approved terminology and replace 자동으로 맞춰지다 with a precise no-live-sync explanation; preserve initial privacy. |
| `studies.teaching.fromStudy.none` | 이 성경 공부에는 아직 가져올 내용이 없습니다. | 이 성경공부에는 아직 가져올 내용이 없습니다. | Use the approved 성경공부 product term. |
| `studies.teaching.fromStudy.added` | 성경 공부에서 인도자 전용으로 가져왔습니다. 준비되면 각 부분을 공유하세요. | 성경공부에서 인도자 전용으로 가져왔습니다. 준비되면 각 부분을 공유하세요. | Use the approved 성경공부 term; retain leader-only default and deliberate sharing. |
| `studies.teaching.fromStudy.provenance` | 성경 공부에서: {{label}} | 성경공부에서: {{label}} | Use the same Study name in generated provenance. |
| `studies.teaching.fromStudy.quotation` | 성경 공부의 인용 · {{citation}} | 성경공부의 인용 · {{citation}} | Use the same Study name in generated citation attribution. |
| `studies.teaching.fromStudy.groups.finding` | 메모 | 기록 | These are Study findings, not verse/source annotations; use the approved 기록 distinction. |
| `studies.inquiry.action.unresolved` | 열린 질문 함께 생각하기 | 남은 질문 함께 생각하기 | Replace the literal 열린 질문 with an idiomatic label for unresolved questions. |
| `studies.inquiry.proposal.used` | 새 표현을 글에 넣었고 저장됩니다. | 새 표현을 글에 넣었습니다. 변경 내용이 저장됩니다. | Repair the awkward tense/coordination while retaining that saving follows the insertion. |
| `studies.builder.guidance` | 선택 안내 | 안내(선택 사항) | Optional guidance means an optional field, not guidance about choosing something. |
| `studies.builder.kinds.concordance` | 성구 사전 | 용례 찾기 | Match the live concordance tool and template field, avoiding dictionary terminology. |
| `studies.qaExport.help` | 포함할 내용을 고르세요. 성경 공부 각 부분에 쓴 내용만 내보내며, 따로 적은 메모는 포함하지 않습니다. | 포함할 내용을 고르세요. 성경공부 각 부분에 쓴 내용만 내보내며, 따로 적은 기록은 포함하지 않습니다. | Use approved Study/finding terms and preserve the explicit exclusion of separate notes. |
| `studies.qaExport.modes.full` | 성경 공부 전체 | 성경공부 전체 | Use the approved 성경공부 term in the export choice. |
| `studies.repair.explain` | 원래 일곱 부분이 먼저 배치됩니다. 누락된 부분은 빈 상태로 추가됩니다. 주제, 이해, 결과와 중복 부분을 포함한 모든 기존 부분은 이름, 글, 메모와 연결을 유지합니다. 일곱 부분에 속하지 않는 부분은 기존 상대적 순서대로 뒤에 놓입니다. | 원래 일곱 부분이 먼저 배치됩니다. 누락된 부분은 빈 상태로 추가됩니다. 주제, 이해, 결과와 중복 부분을 포함한 모든 기존 부분은 이름, 글, 기록과 연결을 유지합니다. 일곱 부분에 속하지 않는 부분은 기존 상대적 순서대로 뒤에 놓입니다. | The repair preserves Study findings (기록), not verse/source annotations (메모); retain all seven-part/order/content-preservation semantics. |
| `studies.repair.finishDrafts` | 복구를 취소하기 전에 열린 메모와 부분 이름 편집을 완료하거나 닫으세요. | 복구를 취소하기 전에 열려 있는 기록과 부분 이름의 편집을 완료하거나 닫으세요. | The open data-capture editors are Study finding drafts; use 기록 and natural editing wording without weakening the undo gate. |
| `settings.assistant.imagesLeave` | 글쓰기 서비스와 달리, 이미지 설명은 인터넷으로 OpenAI에 전송됩니다. | 글쓰기 도우미가 이 컴퓨터에서 실행되더라도 이미지 설명은 인터넷으로 OpenAI에 전송됩니다. | Second AI consistency reviewer requested an explicit local-writing contrast: image descriptions still go to OpenAI even when writing runs on this computer; preserve the remote-image consent boundary. |

Three new `studies.inquiry.location` strings label generated metadata in AI consent/evidence previews: `pages` = `{{first}}–{{last}}쪽`, `paragraph` = `{{number}}번째 단락`, `paragraphs` = `{{first}}–{{last}}번째 단락`. Source titles, quoted text and persisted citation identities are unchanged.

## Verification, questions and limits

- All 4,016 baseline leaves retain their keys and placeholder multisets. The 46 edits preserve the exact interpolation names; no Korean plural branch was removed.
- Both Ten-Step variants, the Bible-name file and quoted presentation sample Scripture were compared with the baseline and remain unchanged. Source materials, migrations, saved writing and recogniser code were not edited by this reviewer.
- Final shared `npm run i18n:check` **passed**, six languages × nine areas, after the native enum, PowerPoint and Word display labels were integrated (4,046 Korean leaves). The final 4,046-row Korean ledger was independently compared with every actual locale leaf and matches exactly.
- A second, distinct AI consistency reviewer examines every Korean edit and retained high-risk warnings/privacy/method boundaries. Its results are recorded separately; no human-native approval is inferred from either pass.
- Private owner workshop/guide files were not available directly to this reviewer. POZETEK’s core **본문의 대상 / 본문의 진술 / 본문의 주제** wording and all supplied examples remain intact. Owner assessment may confirm this specialist terminology against the private original; no replacement explanation was invented.
- Specialist morphology names (for example 이태동사, 연계형 and 앗티카형) were retained rather than introducing an unapproved alternative vocabulary. No unresolved blocking Korean wording defect was identified in the inspected locale corpus; this does not certify the unavailable original method sources.
- Native OS-owned dialog chrome, actual Windows/WebView2/high-DPI layout, a real imported owner PDF and live DeepSeek Korean responses remain distinct owner/platform acceptance evidence. Application-owned labels are reviewed here; those external surfaces are not falsely claimed as run.

## Supplemental native enum display names

The source audit found English enum values embedded in otherwise localised native errors. The following **21 new Korean labels** were individually adjudicated. They are finite display labels selected by an existing format/book code, not arbitrary user text. The existing canonical registry and 66 Bible names, book IDs, recognition and imported Scripture remain unchanged. Greek/Latin and numbering qualifiers are explicit. These labels identify books in unsupported/additional-book import warnings; they make no new claim about canon status.

| Key | Korean display text | Decision |
| --- | --- | --- |
| `errors.importFormats.webPage` | 웹 페이지 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `errors.importFormats.osisUsx` | OSIS 및 USX | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.TOB` | 토빗기 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.JDT` | 유딧기 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.ESG` | 에스더(그리스어) | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.WIS` | 솔로몬의 지혜서 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.SIR` | 집회서 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.BAR` | 바룩서 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.LJE` | 예레미야의 편지 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.S3Y` | 세 청년의 노래 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.SUS` | 수산나 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.BEL` | 벨과 용 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.1MA` | 마카베오 1서 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.2MA` | 마카베오 2서 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.3MA` | 마카베오 3서 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.4MA` | 마카베오 4서 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.1ES` | 에스드라 1서(그리스어) | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.MAN` | 므낫세의 기도 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.PS2` | 시편 151편 | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.DAG` | 다니엘(그리스어) | Add localised enum display; preserve code, numbering and language qualifiers. |
| `scripture.importBookNames.2ES` | 에스드라 2서(라틴어) | Add localised enum display; preserve code, numbering and language qualifiers. |

Additional-book naming follows recognisable Korean titles while preserving the app’s existing 에스더/다니엘 spellings in Greek variants. 에스드라 in the numbered additional-book warnings is deliberately distinguished from the existing canonical 에스라 label. Owner assessment can confirm the preferred additional-book naming tradition; this warning-only localisation does not silently rename an installed Bible book.

The independent reviewer’s image-consent finding is resolved in `settings.assistant.imagesLeave`: remote image descriptions still go to OpenAI **even when the writing assistant runs on this computer**. This is a privacy clarification, not a behavioural or provider change.

## Supplemental generated PowerPoint accessibility labels

Three additional exported display/selection names were individually reviewed: `presentation.export.labels.band` = `색 띠` (coloured rectangle), `.blank` = `빈 화면` (blank slide layout), and `.slideImage` = `슬라이드 이미지` (notes-page thumbnail placeholder). Existing Korean colour-band/image terms and familiar blank-layout wording are used. OOXML identifiers, slide content, Scripture and speaker notes are unchanged.

## Supplemental generated Word custom-style display names

Three custom styles shown in Word’s Styles pane were individually reviewed: `studies.materials.wordStyles.label` = `항목 제목`, `.writingLine` = `답변 줄`, and `.writingLineLast` = `마지막 답변 줄`. These identify the small field heading, a ruled response-space line and its final line. Concise Korean display labels preserve their distinct roles. The native custom style IDs and user/document text are unchanged.
