# A.4.4a — "Dieses Element wurde nicht gefunden" with DeepSeek and Korean text (#19)

**Status: cause classified and fixed; mocked-provider regressions pass. Owner Windows check with the real DeepSeek key OPEN.**

## Classification

Not a DeepSeek, key, network, model or Korean-text problem. **The request never reached a provider.**

- Block teachings (the three current templates, EVA, sermon and children's lesson, plus personal block templates) render each writing
  block through `BlockCanvas`, which maps a block without a special role to the assistant part `pointExplanation`
  (`assistPart(block.role)`).
- Block teachings have no *points*, so the field sent no `pointId`.
- `assistant::context::assemble` answered a point part without a point with `AppError::not_found("teaching point", "")`. The
  interface shows NotFound as the generic `errors.kind.notFound`: "Dieses Element wurde nicht gefunden. Möglicherweise wurde es gelöscht."
- This happened for any text, in any language and with any provider, on a block body. The owner's report (DeepSeek, Korean, German UI)
  fits exactly; it would also have failed with OpenAI and English.

Two related gaps in block teachings were found on the way: the assistant's "nearby" context, its "has the person written anything"
check and its two-point structure check only looked at the older points/introduction/conclusion. Requests in a block teaching
therefore got thin context, or a wrong "write something first".

## Fix

- The field sends its **block id** (`AssistInput.blockId`, serde-defaulted, so older callers still deserialize).
- `assemble` resolves the block within **this** teaching. A missing, deleted or foreign block, or an unknown point, is refused with
  the actionable `assistant.staleTarget` (six languages): "This part has changed or was removed since you asked. Close the help, check
  the text and ask again." Nothing is sent and nothing is applied. The A.3 request shape (a point part with neither point nor block)
  gets the same message instead of the generic NotFound.
- The focus label is the block's own title (e.g. "Erkennen"). The neighbouring blocks (±1, bounded like points) are context; all blocks
  are context for a whole-teaching review; block writing counts as the person's material.
- Study help (`inquiry/study.rs`): a deleted or moved part/block is classified the same way, not as NotFound.
- Unchanged guarantees: a suggestion is only a proposal (word-by-word comparison, explicit **Use this wording**), and it is never applied
  if the text changed after asking (existing check). No key in request bodies or logs. Preview first, nothing sent before **Ask**.

## Tests (no network, no paid calls)

`src-tauri/tests/assistant_blocks.rs` (scripted DeepSeek transport, German UI, Korean text in an EVA-shaped block teaching):
correction reaches `https://api.deepseek.com/v1`, carries the Korean text, the block name and its neighbour, no key in the body,
proposal only, teaching unchanged; preview names the block; the A.3 request shape → `assistant.staleTarget`, nothing sent; deleted
block and foreign block → refused, nothing sent or applied; block writing satisfies a free request.
`tests/ui/a4-block-assist.test.tsx`: the German EVA field sends its `blockId`, shows the Korean correction as a comparison, keeps the
original until **Diese Formulierung übernehmen**; a vanished block is refused with a clear message and nothing is asked.
The memory API now mirrors the native rule.

## Owner check (Windows, real DeepSeek)

Disposable profile. Settings → AI: DeepSeek, model `deepseek-chat`, your key. German interface. In a Bible Teaching from **Bibelstudium in
der Gruppe — EVA**, write Korean in **Erkennen** → AI-Hilfe → **Grammatik und Rechtschreibung prüfen** → check the preview (what is sent,
to DeepSeek) → **Fragen**. Expected: a suggested wording with the changes; your text unchanged until **Diese Formulierung übernehmen**.
Repeat in a Bible Study part via **Hilfe** (Study help). If an error appears, note its exact wording: a key, network or model error now
reads differently from the old "nicht gefunden". Do not paste the key anywhere.
