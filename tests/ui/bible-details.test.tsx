import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createMemoryApi } from "../support/memoryApi";
import { openSettingsSection, renderAppInProject as renderApp } from "../support/render";

describe("Bible import details and editing", () => {
  it("explains missing required fields while keeping edition confirmation optional", async () => {
    const api = createMemoryApi();
    api.state.nextChosenFiles = ["/home/me/invented.docx"];
    const { user } = await renderApp(api);
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    await user.click(await screen.findByRole("button", { name: "Choose Bible files…" }));
    await screen.findByRole("heading", { name: "New Bible" });
    const add = screen.getByRole("button", { name: "Add this Bible" });
    expect(add).toBeDisabled();
    expect(screen.getByRole("status")).toHaveTextContent("Short name");
    expect(screen.getByRole("status")).toHaveTextContent("Language");
    expect(screen.getByRole("status")).toHaveTextContent("Edition confirmation is optional");
    const short = screen.getByRole("textbox", { name: "Short name" });
    expect(short).toBeRequired();
    expect(screen.getByRole("combobox", { name: "Language" })).toBeRequired();
    await user.type(short, "IRD");
    await user.selectOptions(screen.getByRole("combobox", { name: "Language" }), "en");
    expect(add).toBeEnabled();
    expect(screen.getByRole("checkbox", { name: "I confirm the edition identified above." })).not.toBeChecked();
    await user.click(add);
    await waitFor(() => expect(api.state.bibles.some((b) => b.shortName === "IRD")).toBe(true));
    expect(api.state.bibles.find((b) => b.shortName === "IRD")?.editionConfirmed).toBe(false);
  });

  it("offers Edit details directly after importing and saves metadata without touching the edition", async () => {
    const api = createMemoryApi();
    api.state.nextChosenFiles = ["/home/me/invented.docx"];
    const { user } = await renderApp(api);
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    await user.click(await screen.findByRole("button", { name: "Choose Bible files…" }));
    await screen.findByRole("heading", { name: "New Bible" });
    await user.type(screen.getByRole("textbox", { name: "Short name" }), "IRD");
    await user.selectOptions(screen.getByRole("combobox", { name: "Language" }), "en");
    await user.click(screen.getByRole("button", { name: "Add this Bible" }));
    expect(await screen.findByRole("button", { name: "Edit details" })).toBeInTheDocument();
    const before = { ...api.state.bibles.find((b) => b.shortName === "IRD")! };
    await user.click(screen.getByRole("button", { name: "Edit details" }));
    const editor = screen.getByRole("form", { name: "Edit details" });
    expect(within(editor).getByRole("combobox", { name: "Language" })).toBeRequired();
    const name = within(editor).getByRole("textbox", { name: "Name" });
    const short = within(editor).getByRole("textbox", { name: "Short name" });
    await user.clear(name);
    await user.type(name, "Invented Ruth Updated");
    await user.clear(short);
    await user.type(short, "IRU");
    await user.type(within(editor).getByRole("textbox", { name: "Publisher" }), "Test publisher");
    await user.type(within(editor).getByRole("textbox", { name: "Copyright or licence (optional)" }), "Invented test only");
    await user.click(within(editor).getByRole("button", { name: "Save details" }));
    await waitFor(() => expect(api.state.bibles.find((b) => b.id === before.id)?.shortName).toBe("IRU"));
    const after = api.state.bibles.find((b) => b.id === before.id)!;
    expect(after).toMatchObject({
      name: "Invented Ruth Updated", shortName: "IRU", publisher: "Test publisher",
      copyrightNotice: "Invented test only", sourceType: before.sourceType,
      sourcePath: before.sourcePath, originalFileName: before.originalFileName,
      editionConfirmed: before.editionConfirmed, verseCount: before.verseCount,
    });
    expect(screen.getByRole("status")).toHaveTextContent("Bible details saved");
  });

  it("edits installed Bibles from Settings, refuses duplicate short names and permits Cancel", async () => {
    const api = createMemoryApi();
    const first = api.installFixtureBible("FXA", "alpha");
    api.installFixtureBible("FXB", "beta");
    const { user } = await renderApp(api);
    const section = await openSettingsSection(user, "Bible texts");
    await user.click(within(section).getAllByRole("button", { name: "Edit details" })[0]!);
    let editor = within(section).getByRole("form", { name: "Edit details" });
    const short = within(editor).getByRole("textbox", { name: "Short name" });
    await user.clear(short);
    await user.type(short, "fxb");
    await user.click(within(editor).getByRole("button", { name: "Save details" }));
    expect(await within(editor).findByRole("alert")).toHaveTextContent("already belongs");
    expect(api.state.bibles.find((b) => b.id === first.id)?.shortName).toBe("FXA");
    await user.click(within(editor).getByRole("button", { name: "Cancel" }));
    await user.click(within(section).getAllByRole("button", { name: "Edit details" })[0]!);
    editor = within(section).getByRole("form", { name: "Edit details" });
    expect(within(editor).getByRole("textbox", { name: "Short name" })).toHaveValue("FXA");
    await user.clear(within(editor).getByRole("textbox", { name: "Name" }));
    await user.type(within(editor).getByRole("textbox", { name: "Name" }), "Updated label");
    await user.click(within(editor).getByRole("button", { name: "Save details" }));
    await waitFor(() => expect(api.state.bibles.find((b) => b.id === first.id)?.name).toBe("Updated label"));
    expect(within(section).getByText("Bible details saved.")).toBeInTheDocument();
  });

  it("reports an import error separately from missing metadata instead of requiring edition confirmation", async () => {
    const api = createMemoryApi();
    api.state.nextChosenFiles = ["/home/me/broken.zip"];
    const { user } = await renderApp(api);
    await user.click(screen.getByRole("button", { name: "Add a Bible…" }));
    await user.click(await screen.findByRole("button", { name: "Choose Bible files…" }));
    await screen.findByRole("heading", { name: "New Bible" });
    expect(screen.getByRole("button", { name: "Add this Bible" })).toBeDisabled();
    expect(screen.getByText(/These files cannot be added yet/)).toBeInTheDocument();
    expect(screen.getByText(/Edition confirmation is optional/)).toBeInTheDocument();
  });
});
