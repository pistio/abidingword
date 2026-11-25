// A.4.4 (#19): AI writing help on a block teaching's field (all six interfaces,
// Korean writing, DeepSeek configured). The field names its block, so the
// request reaches the provider; the suggestion is shown for review and the
// original text stays until the person uses it. Memory API mirrors the native
// rule; no network or paid call.
import { act, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import i18n, { LANGUAGES } from "@/i18n";
import type { AssistReply } from "@/types/teaching";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";

const KOREAN = "우리는 열매를 맺기 위하여 더 열심히 노력하지 않는다";
const FIXED = "우리는 열매를 맺기 위하여 더 열심히 노력하지 않습니다.";
const studyPane = () => screen.getByRole("main", { name: i18n.t("app.panes.name.study") });

afterEach(async () => {
  await act(async () => { await i18n.changeLanguage("en"); });
});

async function evaWithAssistant(language: string) {
  const api = createMemoryApi();
  const project = await api.projectCreate({ workspaceId: api.state.workspace.id, title: "Hauskreis" });
  await api.studyCreate({ projectId: project.id, title: "요한복음 15" });
  await api.assistantSettingsSet({ ...api.state.teaching.assistant.config, enabled: true, provider: "deepseek", model: "deepseek-chat" });
  await api.assistantKeySet("provider", "sk-deepseek-test");
  await act(async () => { await i18n.changeLanguage(language); });
  const { user } = await renderApp(api);
  await user.click(within(await screen.findByRole("list", { name: i18n.t("studies.list.listLabel") })).getByRole("button", { name: /요한복음 15/ }));
  await within(studyPane()).findByRole("textbox", { name: i18n.t("studies.editor.titleLabel") });
  await user.click(within(studyPane()).getByRole("button", { name: i18n.t("studies.view.teaching") }));
  const form = within(studyPane()).getByRole("form");
  await user.click(within(form).getByRole("radio", { name: new RegExp(i18n.t("studies.teaching.builtin.eva.name")) }));
  await user.click(within(form).getByRole("button", { name: i18n.t("studies.teaching.start") }));
  const erkennen = i18n.t("studies.teaching.builtin.eva.blocks.erkennen.title");
  // The Study article contains the named Teaching article. Wait for its actual
  // field, then scope commands to that editor rather than any article ancestor.
  const field = await within(studyPane()).findByRole("textbox", { name: erkennen });
  const article = within(studyPane()).getByRole("article", { name: api.state.teaching.preparations[0]!.title });
  return { api, user, article, field, erkennen };
}

describe.each(LANGUAGES)("A.4.4 AI help on a block teaching (#19), $code", ({ code }) => {
  it("sends the block, shows the Korean correction for review and changes nothing until it is used", async () => {
    const { api, user, article, field, erkennen } = await evaWithAssistant(code);
    await user.type(field, KOREAN);
    await waitFor(() => expect(api.state.teaching.preparations[0]!.blocks!.some((b) => b.body === KOREAN)).toBe(true));
    const reply: AssistReply = { result: { type: "proposal", suggestion: FIXED, changes: "높임말", placement: "replace" }, assistant: "DeepSeek · deepseek-chat", remote: true };
    api.state.teaching.assistant.replies.push(reply);
    const menuName = i18n.t("studies.teaching.assist.menuLabel", { part: erkennen });
    await user.click(within(article).getByRole("button", { name: menuName }));
    const menu = await within(article).findByRole("menu", { name: menuName });
    const items = within(menu).getAllByRole("menuitem");
    // The wording group, then grammar (the owner's "correct my text").
    const grammar = i18n.t("studies.teaching.assist.action.grammar");
    let item = items.find((i) => i.textContent === grammar);
    if (!item) {
      await user.click(items[0]!);
      item = await within(article).findByRole("menuitem", { name: grammar });
    }
    await user.click(item);
    const panel = await screen.findByTestId("assist-panel");
    const askButton = await within(panel).findByRole("button", { name: i18n.t("studies.teaching.assist.ask") });
    await waitFor(() => expect(askButton).toBeEnabled());
    // The preview already names what is shared; no "not found".
    expect(within(panel).queryByText(i18n.t("errors.kind.notFound"))).toBeNull();
    await user.click(askButton);
    // The suggestion is shown as a comparison for review (removed / added words).
    await waitFor(() => expect(panel).toHaveTextContent("않습니다."));
    expect(panel).toHaveTextContent("않는다");
    const asked = api.state.teaching.assistant.asked.at(-1)!;
    const block = api.state.teaching.preparations[0]!.blocks!.find((b) => b.body === KOREAN)!;
    expect(asked.blockId).toBe(block.id);
    expect(asked.pointId ?? null).toBeNull();
    expect(asked.text).toBe(KOREAN);
    expect(asked.uiLanguage).toBe(code);
    // Still the original until the person chooses.
    expect(field).toHaveValue(KOREAN);
    await user.click(within(panel).getByRole("button", { name: i18n.t("studies.teaching.assist.proposal.use") }));
    await waitFor(() => expect(field).toHaveValue(FIXED));
  });

  it("refuses safely, with a clear message, when the block is gone", async () => {
    const { api, user, article, field, erkennen } = await evaWithAssistant(code);
    await user.type(field, KOREAN);
    const menuName = i18n.t("studies.teaching.assist.menuLabel", { part: erkennen });
    // The block disappears elsewhere (another window, a restore) before the request.
    const prep = api.state.teaching.preparations[0]!;
    prep.blocks = prep.blocks!.filter((b) => b.body !== KOREAN && b.title !== erkennen);
    await user.click(within(article).getByRole("button", { name: menuName }));
    const menu = await within(article).findByRole("menu", { name: menuName });
    const grammar = i18n.t("studies.teaching.assist.action.grammar");
    let item = within(menu).getAllByRole("menuitem").find((i) => i.textContent === grammar);
    if (!item) {
      await user.click(within(menu).getAllByRole("menuitem")[0]!);
      item = await within(article).findByRole("menuitem", { name: grammar });
    }
    await user.click(item);
    const panel = await screen.findByTestId("assist-panel");
    // (The memory API skips the native message localisation; the key is translated in all six languages.)
    await waitFor(() => expect(panel).toHaveTextContent(/changed or was removed since you asked/));
    expect(api.state.teaching.assistant.asked).toHaveLength(0);
    expect(field).toHaveValue(KOREAN);
  });
});
