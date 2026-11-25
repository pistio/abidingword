// A.4.2: reader actions live in the compact toolbar as icons (when pinned and
// there is room) or in its "More reader actions" menu. Tests reach them by
// their accessible name either way, as a keyboard or screen-reader user would.
import { screen, within } from "@testing-library/react";
import type { UserEvent } from "@testing-library/user-event";

type Root = HTMLElement | undefined;
const scope = (root: Root) => (root ? within(root) : screen);
const toolbarIn = (root: Root): HTMLElement | undefined => {
  const base = root ?? document.body;
  if (base.dataset.testid === "reader-toolbar") return base;
  const all = base.querySelectorAll<HTMLElement>("[data-testid='reader-toolbar']");
  return all[all.length - 1];
};

/** Uses the reader action called `name`: its toolbar icon, or the item in the "…" menu. */
export async function readerAction(user: UserEvent, name: string | RegExp, root?: Root): Promise<void> {
  const visible = scope(root).queryAllByRole("button", { name }).find((b) => b.closest("[data-testid='reader-toolbar']"));
  if (visible) return user.click(visible);
  const toolbar = toolbarIn(root);
  if (!toolbar) throw new Error("No reader toolbar");
  await user.click(within(toolbar).getByRole("button", { name: "More reader actions" }));
  await user.click(await within(screen.getByRole("menu", { name: "More reader actions" })).findByRole("menuitem", { name }));
}

/** Whether the reader action called `name` can be used now (menu items say so with aria-disabled). */
export async function readerActionEnabled(user: UserEvent, name: string | RegExp, root?: Root): Promise<boolean> {
  const visible = scope(root).queryAllByRole("button", { name }).find((b) => b.closest("[data-testid='reader-toolbar']"));
  if (visible) return !(visible as HTMLButtonElement).disabled;
  const toolbar = toolbarIn(root);
  if (!toolbar) throw new Error("No reader toolbar");
  await user.click(within(toolbar).getByRole("button", { name: "More reader actions" }));
  const item = await within(screen.getByRole("menu", { name: "More reader actions" })).findByRole("menuitem", { name });
  const enabled = item.getAttribute("aria-disabled") !== "true";
  await user.keyboard("{Escape}");
  return enabled;
}

/** Pins reader actions to the toolbar on this (test) device, as Customise toolbar… would. */
export function pinReaderActions(ids: string[]): void {
  window.localStorage.setItem("abiding-word.reader.toolbar.pins", JSON.stringify(ids));
}

/** The same compact physical-page navigation in Reading and Original. */
export function readerPage(root?: HTMLElement): HTMLElement {
  return (root ? within(root) : screen).getByTestId("pdf-page-indicator");
}
export const readerPageName = (page: string | number) => new RegExp(`^PDF page ${page} of `);
export async function jumpReaderPage(user: UserEvent, page: number, root?: HTMLElement): Promise<void> {
  await user.click(readerPage(root));
  const dialog = screen.getByRole("dialog", { name: "Go to page" });
  const field = within(dialog).getByRole("spinbutton", { name: "PDF page" });
  await user.clear(field); await user.type(field,String(page));
  await user.click(within(dialog).getByRole("button", {name:"Go to page"}));
}
