import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createMemoryApi } from "../support/memoryApi";
import { renderApp } from "../support/render";

// Document Bibles made navigable (v0.24): using only the chapters found without
// doubt, finding a passage in the document, and marking by hand where one is.

async function documentBible(file: string) {
  const api = createMemoryApi();
  api.installFixtureBible("WEB", "web");
  const preview = await api.bibleImportPreview({ paths: [`/home/you/Documents/${file}`] });
  await api.bibleImportCommit({ previewId: preview.previewId, name: "Unclear Bible", shortName: "UNC", language: "English", asDocument: true });
  const rendered = await renderApp(api);
  const pane = screen.getByRole("complementary", { name: "Scripture" });
  await rendered.user.selectOptions(within(pane).getByRole("combobox", { name: "Translation" }), "UNC");
  await within(pane).findByText("Read as a document");
  return { ...rendered, api, pane };
}

async function usePartial(user: Awaited<ReturnType<typeof renderApp>>["user"], pane: HTMLElement) {
  await user.click(within(pane).getByText("What can I do with this Bible?"));
  await user.click(within(pane).getByRole("button", { name: "Look for verses…" }));
  const dialog = await screen.findByRole("dialog", { name: "Look for verses in Unclear Bible" });
  expect(await within(dialog).findByText(/1 chapter has problems and is held back/)).toBeInTheDocument();
  expect(within(dialog).queryByRole("button", { name: "Use these verses" })).not.toBeInTheDocument();
  await user.click(within(dialog).getByRole("button", { name: "Use the 13 verses found without doubt" }));
  expect(await within(dialog).findByText(/The verses found can now be looked up by passage in Unclear Bible/)).toBeInTheDocument();
  await user.click(within(dialog).getByRole("button", { name: "Close" }));
}

describe("Passages in document Bibles", () => {
  it("uses only the chapters found without doubt, and the Bible stays a document", async () => {
    const { api, user, pane } = await documentBible("psalms-unclear.md");
    expect(within(pane).getByText(/No passages can be found in this document yet/)).toBeInTheDocument();
    await usePartial(user, pane);
    const bible = api.state.bibles.find((b) => b.shortName === "UNC")!;
    expect(bible.kind).toBe("document");
    expect([...api.state.text.get(bible.id)!.keys()].some((k) => k.startsWith("PSA|"))).toBe(false);
    expect(await within(pane).findByText("13 verses can be found by passage: John (3 of 21 chapters) · Romans (1 of 16 chapters)")).toBeInTheDocument();
  });

  it("shows a passage found in the document, and says plainly when one is not identified", async () => {
    const { user, pane } = await documentBible("psalms-unclear.md");
    await usePartial(user, pane);
    const find = within(pane).getByRole("textbox", { name: "Passage to find in this document" });
    await user.type(find, "John 15:1-3");
    await user.click(within(pane).getByRole("button", { name: "Find" }));
    const panel = await within(pane).findByRole("region", { name: /John 15:1.3 in this document/ });
    expect(within(panel).getByText(/JHN 15:1 about the vine/)).toBeInTheDocument();
    expect(within(panel).getByRole("button", { name: "Show it in the document" })).toBeInTheDocument();
    // Another Bible is offered only where the passage is not all there.
    expect(within(panel).queryByRole("button", { name: /^Read it in/ })).not.toBeInTheDocument();

    await user.clear(find);
    await user.type(find, "Psalm 23");
    await user.click(within(pane).getByRole("button", { name: "Find" }));
    const unknown = await within(pane).findByRole("region", { name: "Psalm 23 in this document" });
    expect(within(unknown).getByText("Where Psalm 23 is in this document has not been identified yet.")).toBeInTheDocument();
    expect(within(unknown).queryByText(/Fixture/)).not.toBeInTheDocument();
    await user.click(within(unknown).getByRole("button", { name: "Read it in WEB" }));
    await waitFor(() => expect(within(pane).getByRole("combobox", { name: "Translation" })).toHaveDisplayValue("WEB"));
  });

  it("marks where a passage is by hand, finds it there, and removes the mark", async () => {
    const { api, user, pane } = await documentBible("psalms-unclear.md");
    const bible = api.state.bibles.find((b) => b.shortName === "UNC")!;
    const find = within(pane).getByRole("textbox", { name: "Passage to find in this document" });
    await user.type(find, "Psalm 23");
    await user.click(within(pane).getByRole("button", { name: "Find" }));
    const panel = await within(pane).findByRole("region", { name: "Psalm 23 in this document" });
    await user.click(within(panel).getByRole("button", { name: "Mark where it is" }));
    const form = within(pane).getByRole("form", { name: "Mark where a passage is" });
    expect(within(form).getByText(/The place: in the text in psalms-unclear\.md\./)).toBeInTheDocument();
    expect(within(form).getByRole("textbox", { name: "Passage at this place" })).toHaveValue("Psalm 23");
    await user.click(within(form).getByRole("button", { name: "Save the mark" }));
    const marked = await within(pane).findByRole("list", { name: "Places marked by hand" });
    expect(within(marked).getByRole("button", { name: "Psalm 23, in the text" })).toBeInTheDocument();
    expect(api.state.places.get(bible.id)).toHaveLength(1);
    expect(api.state.places.get(bible.id)![0]).toMatchObject({ reference: "PSA.23", page: null, segment: 0 });
    expect(await within(pane).findByText(/1 place marked by hand/)).toBeInTheDocument();
    // Nothing else changed: the Bible is still a document with no verses.
    expect(bible.kind).toBe("document");
    await user.click(within(marked).getByRole("button", { name: /Remove the mark for/ }));
    await waitFor(() => expect(api.state.places.get(bible.id)).toHaveLength(0));
  });

  it("a mark needs a passage that can be read", async () => {
    const { user, pane } = await documentBible("psalms-unclear.md");
    await user.click(within(pane).getByRole("button", { name: "Mark a passage here" }));
    const form = within(pane).getByRole("form", { name: "Mark where a passage is" });
    expect(within(form).getByRole("button", { name: "Save the mark" })).toBeDisabled();
    await user.type(within(form).getByRole("textbox", { name: "Passage at this place" }), "not a passage");
    await user.click(within(form).getByRole("button", { name: "Save the mark" }));
    expect(await within(form).findByRole("alert")).toBeInTheDocument();
    await user.click(within(form).getByRole("button", { name: "Cancel" }));
    expect(within(pane).queryByRole("form")).not.toBeInTheDocument();
  });
});
