// A.4.3: a Study's secondary controls (information, passages, note filter,
// writing mode) are behind one "Study options" toggle, and each part's
// actions (hints, help, add, status, rename, move, delete) behind its own "…".
// These helpers open them as a person would; both do nothing when already open.
import { screen } from "@testing-library/react";
import type { UserEvent } from "@testing-library/user-event";
import i18n from "@/i18n";

export async function openStudyOptions(user: UserEvent): Promise<void> {
  const toggle = screen.queryByRole("button", { name: i18n.t("studies.editor.options") });
  if (toggle && toggle.getAttribute("aria-expanded") !== "true") await user.click(toggle);
}

/** Opens the options of the part called `title`, or of every part. */
export async function openPartOptions(user: UserEvent, title?: string): Promise<void> {
  const [before, after] = i18n.t("studies.sections.options", { title: "\u0000" }).split("\u0000") as [string, string];
  const toggles = screen.queryAllByRole("button").filter((b) => {
    const name = b.getAttribute("aria-label") ?? "";
    return title ? name === `${before}${title}${after}` : name.startsWith(before) && name.endsWith(after);
  });
  for (const toggle of toggles) if (toggle.getAttribute("aria-expanded") !== "true") await user.click(toggle);
}

/** Opens every part's options and shows its hints (guidance, goal, field help, example). */
export async function showPartHints(user: UserEvent): Promise<void> {
  await openPartOptions(user);
  for (const toggle of screen.queryAllByRole("button", { name: i18n.t("studies.flow.guidance") })) {
    if (toggle.getAttribute("aria-pressed") !== "true") await user.click(toggle);
  }
}

/** A.4.3: a Bible Teaching's secondary controls (Study link, information, sessions, passages, view). */
export async function openTeachingOptions(user: UserEvent): Promise<void> {
  // Waits for the open teaching (its toggle), so it can be called right after choosing one.
  const toggle = await screen.findByRole("button", { name: i18n.t("studies.teaching.options") });
  if (toggle.getAttribute("aria-expanded") !== "true") await user.click(toggle);
}

/** Opens the "…" of every teaching block (who it is for, heading, move, duplicate, delete, add). */
export async function openBlockOptions(user: UserEvent): Promise<void> {
  const [before, after] = i18n.t("studies.teaching.blocks.options", { name: "\u0000" }).split("\u0000") as [string, string];
  const [partBefore] = i18n.t("studies.sections.options", { title: "\u0000" }).split("\u0000") as [string];
  const toggles = screen.queryAllByRole("button").filter((b) => {
    const name = b.getAttribute("aria-label") ?? "";
    return name.startsWith(before) && name.endsWith(after) && !name.startsWith(partBefore);
  });
  for (const toggle of toggles) if (toggle.getAttribute("aria-expanded") !== "true") await user.click(toggle);
}

/** Opens every block's options and shows its guidance. */
export async function showBlockHints(user: UserEvent): Promise<void> {
  await openBlockOptions(user);
  for (const toggle of screen.queryAllByRole("button", { name: i18n.t("studies.teaching.blocks.guidance") })) {
    if (toggle.getAttribute("aria-pressed") !== "true") await user.click(toggle);
  }
}
