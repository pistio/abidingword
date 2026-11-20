import { fireEvent, screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { createMemoryApi, type SourceFixture } from "../support/memoryApi";
import { createFakePdfEngine } from "../support/fakePdfEngine";
import { renderApp } from "../support/render";
import { readerPageName } from "../support/readerActions";

// Invented pages; no private source text is involved.
const BOOK: SourceFixture = {
  title: "A Durable Reader",
  format: "pdf",
  language: "English",
  segments: [1, 2, 3, 4, 5, 6].map((pageNumber) => ({
    text: "Invented text for page " + pageNumber,
    pageNumber,
  })),
};

const original = {
  mode: "original" as const,
  page: 4,
  history: { entries: [], index: -1 },
  place: { page: 4, fraction: 0.25, position: 3, offset: 0, item: null, fragment: null, itemOffset: 0 },
};

describe("durable source reading", () => {
  it("restores a saved PDF page and mode on cold application startup", async () => {
    const api = createMemoryApi();
    const source = api.installFixtureSource(BOOK);
    await api.sourceReaderPositionSave({ sourceId: source.id, position: original });
    const { user } = await renderApp(api, { pdfEngine: createFakePdfEngine({ pages: 6 }) });
    const pane = screen.getByRole("complementary", { name: "Sources" });
    await user.click(await within(pane).findByRole("button", { name: "A Durable Reader" }));
    const reader = await within(pane).findByRole("region", { name: "A Durable Reader" });
    expect(await within(reader).findByTestId("pdf-viewer")).toBeInTheDocument();
    await waitFor(() => expect(within(reader).getByTestId("pdf-page-indicator")).toHaveAccessibleName(readerPageName("4")));
  });

  it("remembers an open source on pagehide and keeps other sources' records", async () => {
    const api = createMemoryApi();
    const source = api.installFixtureSource(BOOK);
    const other = api.installFixtureSource({ title: "Another book", format: "text", segments: [{ text: "Unrelated text." }] });
    await api.sourceReaderPositionSave({ sourceId: other.id, position: { mode: "reading", page: null } });
    const { user } = await renderApp(api, { pdfEngine: createFakePdfEngine({ pages: 6 }) });
    const pane = screen.getByRole("complementary", { name: "Sources" });
    await user.click(await within(pane).findByRole("button", { name: "A Durable Reader" }));
    const reader = await within(pane).findByRole("region", { name: "A Durable Reader" });
    await within(reader).findByText("Invented text for page 1");
    fireEvent(window, new Event("pagehide"));
    await waitFor(() => expect(api.state.readerPositions.some(row => row.sourceId === source.id)).toBe(true));
    expect(api.state.readerPositions.some(row => row.sourceId === other.id)).toBe(true);
  });
});
