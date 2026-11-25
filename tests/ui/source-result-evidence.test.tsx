import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { expect, it, vi } from "vitest";
import { PassageSourceList } from "@/features/library/PassageSourceList";
import { setLanguage } from "@/i18n";
import type { PassageSourceGroup } from "@/types/library";

it("labels printed-index evidence in the interface language and keeps its heading target", async () => {
  const group: PassageSourceGroup = {
    sourceId: "invented-handbook",
    sourceTitle: "Invented handbook",
    author: null,
    moreLocations: 0,
    locations: [{
      segmentId: "acts-introduction",
      locator: "p. 8",
      origin: "section",
      matchKind: "contained",
      matchedText: "Apostelgeschichte",
      sectionHeading: "Apostelgeschichte",
      structureNodeId: "acts-heading",
      structureBasis: "printed",
      structureScope: "book",
      reference: {
        canonical: "ACT.1-ACT.28",
        display: "Acts 1–28",
        book: "ACT",
        startChapter: 1,
        startVerse: null,
        endChapter: 28,
        endVerse: null,
      },
    }],
  };
  const onOpen = vi.fn();
  const user = userEvent.setup();
  render(<PassageSourceList groups={[group]} label="Sources" passageLabel={reference => reference.display} onOpen={onOpen} />);

  const result = screen.getByRole("button", { name: /^Invented handbook/ });
  expect(result).toHaveTextContent("Book section Acts (broader)");
  expect(result).toHaveTextContent("Printed book index");
  await user.click(result);
  expect(onOpen).toHaveBeenCalledExactlyOnceWith("invented-handbook", "acts-introduction", "acts-heading");

  await act(() => setLanguage("de"));
  expect(result).toHaveTextContent("Gedrucktes Buchverzeichnis");
  expect(result).not.toHaveTextContent("Printed book index");
});
