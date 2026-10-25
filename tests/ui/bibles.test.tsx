import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import i18n from "@/i18n";
import { createMemoryApi } from "../support/memoryApi";
import { openSettingsSection, renderApp } from "../support/render";

describe("adding and managing Bibles", () => {
  it("adds the included Bible after reviewing it", async () => {
    const { api, user } = await renderApp();
    const scripture = screen.getByRole("complementary", { name: "Scripture" });
    await user.click(within(scripture).getByRole("button", { name: "Add a Bible…" }));
    await user.selectOptions(await screen.findByRole("combobox", { name: "Choose a Bible" }), "World English Bible (WEB)");
    await user.click(screen.getByRole("button", { name: "Add…" }));
    const dialog = screen.getByRole("dialog", { name: "Add World English Bible?" });
    await user.click(within(dialog).getByRole("button", { name: "Continue" }));

    expect(await screen.findByRole("heading", { level: 1, name: "New Bible" })).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toHaveValue("World English Bible");
    expect(screen.getByText("3 books detected")).toBeInTheDocument();
    expect(within(screen.getByRole("list", { name: "Books" })).getByText("Romans")).toBeInTheDocument();
    expect(screen.getByText(/1 warning — worth reviewing/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Add this Bible" }));
    expect(await screen.findByText(/is ready to read/)).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Read it now" }));

    expect(api.state.bibles.map((b) => b.shortName)).toEqual(["WEB"]);
    const pane = await screen.findByRole("complementary", { name: "Scripture" });
    expect(within(pane).getByRole("combobox", { name: "Translation" })).toHaveDisplayValue("WEB");
  });

  it("offers the included Bibles in one drop-down, grouped by language", async () => {
    const { user } = await renderApp();
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    const select = await screen.findByRole("combobox", { name: "Choose a Bible" });
    const groups = within(select).getAllByRole("group").map((g) => g.getAttribute("label"));
    expect(groups).toEqual(["English", "Deutsch", "한국어", "Français", "Español", "Português", "Ελληνικά (Koine Greek)"]);
    expect(within(select).getAllByRole("option").map((o) => o.textContent)).toEqual([
      "Berean Standard Bible (BSB)",
      "World English Bible (WEB)",
      "Lutherbibel 1912 (LUT1912)",
      "Schlachter-Bibel 1951 (SCH1951)",
      "Elberfelder 1905 (unrevidiert) (ELB1905)",
      "성경전서 개역한글판 (KRV)",
      "Louis Segond 1910 (LSG)",
      "Santa Biblia — Reina Valera 1909 (RV1909)",
      "Bíblia Livre (BLIVRE)",
      "SBL Greek New Testament (SBLGNT)",
      "Robinson–Pierpont Byzantine Textform 2018 (RP2018)",
      "Textus Receptus (Boyd 2022) (TR)",
      "1904 Patriarchal Greek New Testament (PT1904)",
    ]);
    // the first Bible in the interface language is chosen; its language and licence are shown
    expect(select).toHaveDisplayValue("Berean Standard Bible (BSB)");
    expect(screen.getByText("English · Public Domain")).toBeInTheDocument();
    await user.selectOptions(select, "Bíblia Livre (BLIVRE)");
    expect(screen.getByText("Português · CC BY 4.0")).toBeInTheDocument();
  });

  it("asks for confirmation in a dialog, and Cancel or Escape adds nothing", async () => {
    const { api, user } = await renderApp();
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    await user.selectOptions(await screen.findByRole("combobox", { name: "Choose a Bible" }), "Lutherbibel 1912 (LUT1912)");
    const add = screen.getByRole("button", { name: "Add…" });
    await user.click(add);
    let dialog = screen.getByRole("dialog", { name: "Add Lutherbibel 1912?" });
    expect(dialog).toHaveAttribute("aria-modal", "true");
    expect(within(dialog).getByText("Language: Deutsch")).toBeInTheDocument();
    expect(within(dialog).getByText("Licence: Public Domain")).toBeInTheDocument();
    // focus starts in the dialog, on Cancel
    expect(within(dialog).getByRole("button", { name: "Cancel" })).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("dialog", { name: "Add Lutherbibel 1912?" })).not.toBeInTheDocument();
    // Escape closed only the question; Add a Bible stays open.
    expect(screen.getByRole("dialog", { name: "Add a Bible" })).toBeInTheDocument();
    expect(add).toHaveFocus();

    await user.click(add);
    dialog = screen.getByRole("dialog", { name: "Add Lutherbibel 1912?" });
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(screen.queryByRole("dialog", { name: "Add Lutherbibel 1912?" })).not.toBeInTheDocument();
    expect(api.state.pendingPreview).toBeNull();
    expect(screen.getByRole("heading", { level: 1, name: "Add a Bible" })).toBeInTheDocument();
  });

  it("chooses the German Bible first for a German interface, and leaves out what is already added", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("WEB", "web").isBuiltin = true;
    await i18n.changeLanguage("de");
    try {
      const { user } = await renderApp(api);
      await openSettingsSection(user, "Bibelübersetzungen");
      await user.click(await screen.findByRole("button", { name: /Bibel hinzufügen/ }));
      const select = await screen.findByRole("combobox", { name: "Bibel auswählen" });
      expect(within(select).getAllByRole("group")[0]).toHaveAttribute("label", "Deutsch");
      expect(select).toHaveDisplayValue("Lutherbibel 1912 (LUT1912)");
      const options = within(select).getAllByRole("option").map((o) => o.textContent);
      expect(options).toContain("Berean Standard Bible (BSB)");
      expect(options).not.toContain("World English Bible (WEB)"); // already added
    } finally {
      await i18n.changeLanguage("en");
    }
  });

  it("chooses the Korean Bible first for a Korean interface", async () => {
    await i18n.changeLanguage("ko");
    try {
      const { user } = await renderApp();
      await user.click(screen.getAllByRole("button", { name: /성경 추가/ })[0]!);
      expect(await screen.findByRole("combobox", { name: "성경 선택" })).toHaveDisplayValue("성경전서 개역한글판 (KRV)");
    } finally {
      await i18n.changeLanguage("en");
    }
  });

  it("will not add a Bible with problems, and says why", async () => {
    const api = createMemoryApi();
    api.state.nextChosenFiles = ["/home/me/broken.zip"];
    const { user } = await renderApp(api);
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    await user.click(await screen.findByRole("button", { name: "Choose Bible files…" }));

    expect(await screen.findByText(/1 problem must be fixed/)).toBeInTheDocument();
    expect(screen.getByText("John 15:1 appears twice.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add this Bible" })).toBeDisabled();
  });

  it("refuses unsupported files with a plain message", async () => {
    const api = createMemoryApi();
    api.state.nextChosenFiles = ["/home/me/bible.rtf"];
    const { user } = await renderApp(api);
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    await user.click(await screen.findByRole("button", { name: "Choose Bible files…" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Word files can’t be imported");
  });

  it("lists installed Bibles in Settings, moves one to Trash after confirmation, and deletes it for good from Trash", async () => {
    const api = createMemoryApi();
    api.installFixtureBible("FXA", "alpha");
    api.installFixtureBible("FXB", "beta");
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");

    const list = await screen.findByRole("list", { name: "Installed Bibles" });
    expect(within(list).getAllByRole("listitem")).toHaveLength(2);
    await user.click(within(list).getByRole("button", { name: "More for Fixture FXA" }));
    await user.click(screen.getByRole("menuitem", { name: /^Move to Trash/ }));
    const confirm = screen.getByRole("alertdialog", { name: "Move Fixture FXA to Trash?" });
    expect(confirm).toHaveTextContent("Your notes, highlights and the passages in your studies belong to the verses");
    // Focus starts on Cancel; Escape cancels.
    expect(within(confirm).getByRole("button", { name: "Cancel" })).toHaveFocus();
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("alertdialog")).not.toBeInTheDocument();
    expect(api.state.bibles.find((b) => b.shortName === "FXA")?.state).toBe("active");

    await user.click(within(list).getByRole("button", { name: "More for Fixture FXA" }));
    await user.click(screen.getByRole("menuitem", { name: /^Move to Trash/ }));
    await user.click(within(screen.getByRole("alertdialog")).getByRole("button", { name: "Move to Trash" }));
    await waitFor(() => expect(within(list).getAllByRole("listitem")).toHaveLength(1));
    expect(within(section).getByRole("status")).toHaveTextContent("Fixture FXA was moved to Trash.");

    // From Trash: restore is possible; deleting for good asks again.
    await user.click(within(section).getByText("Trash (1)"));
    await user.click(within(section).getByRole("button", { name: "Delete permanently" }));
    const really = screen.getByRole("alertdialog", { name: "Delete Fixture FXA permanently?" });
    expect(really).toHaveTextContent("This cannot be undone.");
    await user.click(within(really).getByRole("button", { name: "Delete permanently" }));
    await waitFor(() => expect(api.state.bibles.map((b) => b.shortName)).toEqual(["FXB"]));
  });

  it("archives a Bible and restores it, and never deletes one that is not in Trash", async () => {
    const api = createMemoryApi();
    const fxa = api.installFixtureBible("FXA", "alpha");
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    await user.click(within(section).getByRole("button", { name: "More for Fixture FXA" }));
    await user.click(screen.getByRole("menuitem", { name: /^Archive/ }));
    await within(section).findByText("Archived (1)");
    expect((await api.bibleList()).map((b) => b.shortName)).toEqual([]);
    await user.click(within(section).getByRole("button", { name: "Undo" }));
    await waitFor(() => expect(api.state.bibles[0]?.state).toBe("active"));
    await expect(api.bibleRemovePermanently([fxa.id])).rejects.toThrow(/Only items in Trash/);
  });

  it("offers the Greek New Testaments as included Bibles, saying they hold 27 of the 66 books", async () => {
    const { user } = await renderApp();
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    await user.selectOptions(await screen.findByRole("combobox", { name: "Choose a Bible" }), "SBL Greek New Testament (SBLGNT)");
    await user.click(screen.getByRole("button", { name: "Add…" }));
    const dialog = screen.getByRole("dialog", { name: "Add SBL Greek New Testament?" });
    expect(within(dialog).getByText("27 of the 66 books")).toBeInTheDocument();
  });

  it("downloads an available Bible only after confirming its size, then reviews it as usual", async () => {
    const { api, user } = await renderApp();
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    const list = await screen.findByRole("list", { name: "Available to download" });
    const oshb = within(list).getByText(/Open Scriptures Hebrew Bible/).closest("li")!;
    expect(within(oshb).getByText("עברית (Biblical Hebrew) · 39 of the 66 books · lexical numbers · grammar")).toBeInTheDocument();
    expect(within(oshb).getByText("Public Domain text; CC BY 4.0 word data · 28.6 MB")).toBeInTheDocument();
    await user.click(within(oshb).getByRole("button", { name: "Download OSHB…" }));
    const dialog = screen.getByRole("dialog", { name: "Download Open Scriptures Hebrew Bible (Westminster Leningrad Codex)?" });
    expect(within(dialog).getByText(/40 files \(28.6 MB\) are downloaded from GitHub and each is checked/)).toBeInTheDocument();
    // Cancel downloads nothing.
    await user.click(within(dialog).getByRole("button", { name: "Cancel" }));
    expect(api.state.pendingPreview).toBeNull();
    await user.click(within(oshb).getByRole("button", { name: "Download OSHB…" }));
    await user.click(within(screen.getByRole("dialog", { name: /^Download Open Scriptures/ })).getByRole("button", { name: "Download (28.6 MB)" }));
    expect(await screen.findByRole("heading", { level: 1, name: "New Bible" })).toBeInTheDocument();
    expect(screen.getByLabelText("Name")).toHaveValue("Open Scriptures Hebrew Bible (Westminster Leningrad Codex)");
    expect(screen.getByLabelText("Short name")).toHaveValue("OSHB");
    expect(api.state.bibles).toHaveLength(0);
  });

  it("a download that fails its check keeps nothing and says why", async () => {
    const api = createMemoryApi();
    api.state.catalogueFails = true;
    const { user } = await renderApp(api);
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    const list = await screen.findByRole("list", { name: "Available to download" });
    await user.click(within(list).getByRole("button", { name: "Download MorphGNT…" }));
    await user.click(within(screen.getByRole("dialog", { name: "Download MorphGNT: SBLGNT Edition?" })).getByRole("button", { name: "Download (8.9 MB)" }));
    expect(await screen.findByText(/did not match the verified copy, so nothing was kept/)).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 1, name: "Add a Bible" })).toBeInTheDocument();
    expect(api.state.bibles).toHaveLength(0);
  });
});
