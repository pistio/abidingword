# ko: AI editorial change log

Base: `38c785be723f4ff4ca7ba8c885aeabff3a89df7a`. This is AI linguistic review, not human native-speaker approval.

Register: Respectful Korean; preserve 성경공부, 성경 가르침 and 틀 terminology.

These rows are the individually edited/reviewed keys. The full application is inventoried in inventory.json and the complete six-language corpus; unchanged keys are not claimed to have all received a new individual linguistic adjudication.

| Key/context | Before | After | Editorial reason |
| --- | --- | --- | --- |
| `errors.validation.entryInvalid` | (new localised message) | 이 항목을 저장하지 못했습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.validation.unknownField` | (new localised message) | 이 부분에는 해당 입력란이 없습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.structure.timeout` | 안전한 처리 경계에서 분석 시간 제한에 도달했습니다. 이전 메타데이터는 변경되지 않았습니다. 다시 시도하거나 진단 보고서를 저장하세요. | 시간 제한을 넘겨 분석이 중단되었습니다. 이번 시도에서는 변경 사항을 저장하지 않았습니다. 다시 시도하거나 진단 보고서를 저장해 주세요. | Replace technical processing-boundary jargon with outcome, unchanged-data assurance and retry. |
| `errors.library.nothingChosen` | (new localised message) | 항목을 하나 이상 선택해 주세요. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.notInTrash` | (new localised message) | 휴지통에 있는 항목만 영구 삭제할 수 있습니다. 먼저 휴지통으로 옮겨 주세요. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.badYear` | (new localised message) | 연도는 0에서 3000 사이의 숫자여야 합니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.collectionCycle` | (new localised message) | 자료 모음을 자기 자신이나 그 안의 하위 모음으로 옮길 수 없습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.linkUnsupported` | (new localised message) | 이 유형의 링크는 열 수 없습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.tooMany` | (new localised message) | 한 번에 최대 {{count}}개 항목을 변경할 수 있습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.badName` | (new localised message) | 이름은 1자 이상 {{max}}자 이하로 입력해 주세요. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.collectionExists` | (new localised message) | 여기에 “{{name}}” 자료 모음이 이미 있습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.library.tagExists` | (new localised message) | “{{name}}” 태그가 이미 있습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.material.tooManyPassages` | (new localised message) | 이 자료에는 성경 구절이 너무 많아 본문을 모두 넣을 수 없습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.words.documentBible` | (new localised message) | 문서에서 추가한 성경에는 단어 정보를 추가할 수 없습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `errors.messageUnavailable` | (new localised message) | 작업을 완료하지 못했습니다. 다시 시도해 주세요. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.invalidPassage` | (new localised message) | 구절을 읽을 수 없습니다. 사도행전 6장처럼 책 이름과 장을 입력해 주세요. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.noCandidates` | (new localised message) | 이 자료에는 해당 구절과 겹치는 성경 참조가 없습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.structuralResult` | (new localised message) | 첫 결과는 PDF {{page}}쪽의 문서 구조에 근거합니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.literalResult` | (new localised message) | 일치하는 단락을 찾지 못했습니다. 첫 결과는 PDF {{page}}쪽에 있는 구절 언급입니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.section` | (new localised message) | 단락 제목 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.book` | (new localised message) | 책 단위 단락 (더 넓은 범위) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.chapter` | (new localised message) | 장 제목 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.manual` | (new localised message) | 직접 연결한 구절 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.contents` | (new localised message) | 목차 항목 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.note` | (new localised message) | 각주·미주에서 언급 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.literal` | (new localised message) | 구절 언급 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.inherited` | (new localised message) | 대응 구절이 있는 단락 안 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.unknown` | (new localised message) | 구절 연결 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.diagnostic.manualResult` | (new localised message) | 첫 결과는 직접 추가한 연결이며, PDF {{page}}쪽에 있습니다. | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.structure.timeout` | 안전한 처리 경계에서 분석 시간 제한에 도달했습니다. 이전 메타데이터는 변경되지 않았습니다. 다시 시도하거나 진단 보고서를 저장하세요. | 시간 제한을 넘겨 분석이 중단되었습니다. 이번 시도에서는 변경 사항을 저장하지 않았습니다. 다시 시도하거나 진단 보고서를 저장해 주세요. | Replace technical processing-boundary jargon with outcome, unchanged-data assurance and retry. |
| `library.fileDialog.source` | (new localised message) | 자료실에 추가할 파일 선택 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.sources` | (new localised message) | 자료실에 추가할 파일들 선택 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.folder` | (new localised message) | 자료실에 추가할 폴더 선택 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.sourceTypes` | (new localised message) | 책과 기록 (PDF, EPUB, Word, Markdown, 텍스트) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.bible` | (new localised message) | 추가할 성경 선택 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.bibleTypes` | (new localised message) | 성경 파일 (USFM, .zip, PDF, EPUB, Word, Markdown, 텍스트) | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `library.fileDialog.allFiles` | (new localised message) | 모든 파일 | Remove an English-only native/UI path; preserve meaning and uncertainty. |
| `studies.teaching.workspace.basedOn` | 기반 성경 성경공부 — | 연결된 성경공부 — | Apply approved terminology; remove duplication. |
| `studies.teaching.workspace.linkStudy` | 성경 성경공부에 연결 | 성경공부에 연결 | Apply approved terminology; remove duplication. |
