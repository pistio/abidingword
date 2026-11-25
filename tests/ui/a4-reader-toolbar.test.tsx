// A.4.2 (#26): one compact, adaptive reader toolbar for the pane and the full
// reader; icon-first with an accessible "…" menu; per-device pins with reset;
// never wraps into extra rows (overflow grows instead). Invented sources only.
import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { createFakePdfEngine } from "../support/fakePdfEngine";
import { createMemoryApi } from "../support/memoryApi";
import { renderAppInProject as renderApp } from "../support/render";
import { pinReaderActions, readerAction } from "../support/readerActions";

const sources = () => within(screen.getByRole("complementary", { name: "Sources" }));

async function openPdf() {
  const api = createMemoryApi();
  api.installFixtureSource({ title: "Toolbar fixture", format: "pdf", segments: [1, 2, 3].map((pageNumber) => ({ text: `Synthetic page ${pageNumber}`, pageNumber })) });
  const engine = createFakePdfEngine({ pages: 3 });
  const result = await renderApp(api, { pdfEngine: engine });
  await result.user.click(await sources().findByRole("button", { name: "Toolbar fixture" }));
  const toolbar = await screen.findByRole("toolbar", { name: "Reader tools" });
  return { ...result, api, toolbar };
}

const menuItems = async (user: ReturnType<typeof import("@testing-library/user-event").default.setup>, toolbar: HTMLElement) => {
  await user.click(within(toolbar).getByRole("button", { name: "More reader actions" }));
  const items = within(screen.getByRole("menu", {name:"More reader actions"})).getAllByRole("menuitem").map((m) => m.textContent ?? "");
  await user.keyboard("{Escape}");
  return items;
};

describe("A.4.2 compact reader toolbar", () => {
  it("shows Back, the view switch, the page indicator and Find/Contents icons; everything else is in the menu, once", async () => {
    const { user, toolbar } = await openPdf();
    expect(within(toolbar).getByRole("button", { name: "Back to Sources" })).toBeInTheDocument();
    expect(within(toolbar).getByRole("group", { name: "View" })).toBeInTheDocument();
    expect(within(toolbar).getByTestId("pdf-page-indicator")).toHaveTextContent("1 / 3");
    const find = within(toolbar).getByRole("button", { name: "Find" });
    expect(find).toHaveAttribute("title", "Find");
    expect(find.className).toMatch(/size-11/); // 44 px target
    for (const name of ["Marks", "Chapters and headings", "Open full reader", "About this source", "Back to the previous place"]) {
      expect(within(toolbar).queryByRole("button", { name })).toBeNull();
    }
    const items = await menuItems(user, toolbar);
    expect(items).toEqual(expect.arrayContaining(["Back to the previous place", "Forward to the next place", "Bookmark this place", "Add note here", "Marks", "Chapters and headings", "Open full reader", "About this source", "Customise toolbar…"]));
    expect(items).not.toContain("Find");
    expect(new Set(items).size).toBe(items.length);
    // No separate rows of verbose page controls or a permanent About bar remain.
    expect(screen.queryByRole("button", { name: "Go to page" })).toBeNull();
    expect(screen.queryByText("About this source")).toBeNull();
  });

  it("disabled menu actions are announced as unavailable and do nothing", async () => {
    const { user, toolbar } = await openPdf();
    await user.click(within(toolbar).getByRole("button", { name: "More reader actions" }));
    const back = screen.getByRole("menuitem", { name: "Back to the previous place" });
    expect(back).toHaveAttribute("aria-disabled", "true");
    await user.click(back);
    expect(screen.getByRole("menu", {name:"More reader actions"})).toBeInTheDocument(); // still open: nothing happened
  });

  it("pins and unpins actions per device, keeps them after reopening, and resets to the default", async () => {
    const { user, toolbar } = await openPdf();
    await readerAction(user, "Customise toolbar…");
    const dialog = await screen.findByRole("dialog", { name: "Reader toolbar" });
    await user.click(within(dialog).getByRole("checkbox", { name: "Marks" }));
    await user.click(within(dialog).getByRole("checkbox", { name: "Find in this source" }));
    expect(JSON.parse(window.localStorage.getItem("abiding-word.reader.toolbar.pins")!)).toEqual(["contents", "marks"]);
    await user.click(within(dialog).getByRole("button", { name: "Close" }));
    expect(within(toolbar).getByRole("button", { name: "Marks" })).toBeInTheDocument();
    expect(within(toolbar).queryByRole("button", { name: "Find" })).toBeNull();
    expect(await menuItems(user, toolbar)).toContain("Find");

    // Reopen the source: the choice is remembered on this device.
    await user.click(within(toolbar).getByRole("button", { name: "Back to Sources" }));
    await user.click(await sources().findByRole("button", { name: "Toolbar fixture" }));
    const again = await screen.findByRole("toolbar", { name: "Reader tools" });
    expect(within(again).getByRole("button", { name: "Marks" })).toBeInTheDocument();

    await readerAction(user, "Customise toolbar…", again);
    await user.click(within(await screen.findByRole("dialog", { name: "Reader toolbar" })).getByRole("button", { name: "Reset to default" }));
    expect(window.localStorage.getItem("abiding-word.reader.toolbar.pins")).toBeNull();
    await user.click(screen.getByRole("button", { name: "Close" }));
    expect(within(again).getByRole("button", { name: "Find" })).toBeInTheDocument();
    expect(within(again).queryByRole("button", { name: "Marks" })).toBeNull();
  });

  it("moves pinned icons into the menu when the row is narrow instead of wrapping", async () => {
    pinReaderActions(["find", "contents", "history", "marks", "structure"]);
    const rect = vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (this: HTMLElement) {
      if (this.dataset.testid === "reader-toolbar") return new DOMRect(0, 0, 360, 48);
      return new DOMRect(0, 0, 220, 44); // Back + view switch + page indicator
    });
    try {
      const { user, toolbar } = await openPdf();
      expect(toolbar.className).toMatch(/flex-nowrap/);
      await waitFor(() => expect(within(toolbar).queryByRole("button", { name: "Chapters and headings" })).toBeNull());
      const shown = within(toolbar).getAllByRole("button").filter((b) => b.dataset.readerAction);
      expect(shown.length).toBeLessThanOrEqual(2);
      const items = await menuItems(user, toolbar);
      expect(items).toEqual(expect.arrayContaining(["Marks", "Chapters and headings"]));
    } finally {
      rect.mockRestore();
    }
  });

  it("offers a name on a long touch press without making it the only way in", async () => {
    vi.useFakeTimers({ shouldAdvanceTime: true });
    try {
      const { toolbar } = await openPdf();
      const find = within(toolbar).getByRole("button", { name: "Find" });
      const hint = find.nextElementSibling as HTMLElement;
      expect(hint).toHaveAttribute("data-visible", "false");
      fireEvent.pointerDown(find, { pointerType: "touch" });
      vi.advanceTimersByTime(600);
      await waitFor(() => expect(hint).toHaveAttribute("data-visible", "true"));
      fireEvent.pointerUp(find, { pointerType: "touch" });
      fireEvent.click(find);
      expect(find).toHaveAttribute("aria-expanded", "true"); // the find bar opened
    } finally {
      vi.useRealTimers();
    }
  });

  it("uses the same toolbar in the full reader, with the title beside Back", async () => {
    const { user, toolbar } = await openPdf();
    await readerAction(user, "Open full reader", toolbar);
    const full = await screen.findByRole("toolbar", { name: "Reader tools" });
    expect(within(full).getByRole("heading", { level: 1, name: "Toolbar fixture" })).toBeInTheDocument();
    expect(within(full).getByRole("button", { name: "Back" })).toBeInTheDocument();
    expect(within(full).getByTestId("pdf-page-indicator")).toHaveTextContent("1 / 3");
    const items = await menuItems(user, full);
    expect(items).not.toContain("Open full reader");
  });
});
