// Complete workspace regression, with synthetic sources only.
// Start the screenshot harness as documented in capture.mjs, then run:
// PLAYWRIGHT_MODULE=<installed playwright> CHROMIUM_PATH=<chromium> node tests/screenshots/milestone-a1.mjs
// SCREENSHOT_URL and SCREENSHOT_OUT override the server and output directory.
import { createRequire } from "node:module";
import { mkdirSync } from "node:fs";
import assert from "node:assert/strict";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const base = process.env.SCREENSHOT_URL ?? "http://localhost:1430/tests/screenshots/index.html";
const out = process.env.SCREENSHOT_OUT ?? "docs/implementation/milestone-a1/images";
mkdirSync(out, { recursive: true });
const browser = await chromium.launch({
  ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
  headless: true, args: ["--no-sandbox"],
});
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  await page.goto(`${base}?assistant=off&sourcesOpen=1`);
  await page.getByRole("main").getByRole("button", { name: /^Abiding in the Vine/ }).first().click();
  await page.getByRole("button", { name: "Step-by-step", exact: true }).click();
  await page.getByRole("navigation", { name: "Study sections" }).waitFor();
  for (const width of [1440, 1100, 900]) {
    await page.setViewportSize({ width, height: 1000 });
    const boxes = await page.locator("[data-study-steps] button,[data-study-steps] select").evaluateAll(nodes => nodes.map(node => {
      const rect = node.getBoundingClientRect();
      return { bottom: rect.bottom, width: rect.width, right: rect.right };
    }));
    assert.equal(boxes.length, 3);
    assert.ok(Math.abs(boxes[0].bottom - boxes[2].bottom) <= 2, "Previous and Next alignment");
    assert.ok(boxes[1].width >= 180, "Selector remains usable in a combined pane");
    assert.ok(boxes.every(box => box.width >= 40 && box.right <= width));
    await page.screenshot({ path: `${out}/study-${width}.png` });
  }
  await page.getByRole("button", { name: "Bible Teaching", exact: true }).click();
  await page.getByRole("article", { name: "Sunday sermon · Abiding in the Vine" }).waitFor();
  await page.getByRole("button", { name: "New Teaching", exact: true }).click();
  await page.getByRole("button", { name: "Start", exact: true }).click();
  const writing = page.getByRole("textbox", { name: "Main part", exact: true });
  await writing.waitFor();
  for (const width of [1440, 1100, 900]) {
    await page.setViewportSize({ width, height: 1000 });
    const bounds = await writing.evaluate(node => ({
      field: node.getBoundingClientRect().width,
      group: node.closest('[role="group"]').getBoundingClientRect().width,
      right: node.getBoundingClientRect().right,
    }));
    assert.ok(bounds.field >= bounds.group - 8 && bounds.field >= 180 && bounds.right <= width, JSON.stringify(bounds));
    const pane = await page.getByRole("main", { name: "Study" }).boundingBox();
    const select = await page.getByRole("combobox", { name: "Teaching:" }).boundingBox();
    assert.ok(select.x >= pane.x && select.x + select.width <= pane.x + pane.width, "Teaching selector stays inside its pane");
    await writing.scrollIntoViewIfNeeded();
    await page.screenshot({ path: `${out}/teaching-${width}.png` });
  }
  await page.getByRole("button", { name: "Change language", exact: true }).click();
  await page.getByRole("combobox", { name: "Interface language", exact: true }).waitFor();
  await page.screenshot({ path: `${out}/language.png` });
  await page.close();

  const reader = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  await reader.goto(`${base}?assistant=off&a1Review=1&sourcesOpen=1`);
  const sources = reader.getByRole("complementary", { name: "Sources" });
  await sources.getByRole("button", { name: "Synthetic EPUB layout review", exact: true }).click();
  await sources.getByRole("button", { name: "Contents", exact: true }).waitFor();
  const snapshot = () => sources.evaluate(root => Array.from(root.querySelectorAll("*")).filter(node =>
    node instanceof HTMLElement && node.scrollHeight > node.clientHeight + 1 && /auto|scroll/.test(getComputedStyle(node).overflowY),
  ).map(node => ({ tag: node.tagName, testid: node.dataset.testid ?? "", top: node.scrollTop, width: node.clientWidth, height: node.clientHeight })));
  for (const width of [1440, 1100, 900]) {
    await reader.setViewportSize({ width, height: 1000 });
    for (const mode of ["Reading", "Original"]) {
      await sources.getByRole("button", { name: mode, exact: true }).click();
      const body = sources.getByTestId(mode === "Reading" ? "reader-body" : "epub-viewer");
      await body.waitFor();
      await sources.evaluate(root => {
        for (const node of root.querySelectorAll("*")) {
          if (node instanceof HTMLElement && /auto|scroll/.test(getComputedStyle(node).overflowY)) {
            node.scrollTop = Math.min(300, Math.max(0, node.scrollHeight - node.clientHeight));
          }
        }
      });
      await reader.evaluate(() => new Promise(resolve => requestAnimationFrame(resolve)));
      const before = await snapshot();
      const bounds = await body.boundingBox();
      const opener = sources.getByRole("button", { name: "Contents", exact: true });
      await opener.click();
      await sources.getByTestId("contents-panel").waitFor();
      const afterBounds = await body.boundingBox();
      assert.ok(Math.abs(bounds.width - afterBounds.width) <= 1 && Math.abs(bounds.x - afterBounds.x) <= 1, "Contents must not resize the reader");
      const after = await snapshot();
      for (const item of before) {
        const match = after.find(value => value.tag === item.tag && value.testid === item.testid && value.width === item.width && value.height === item.height);
        assert.ok(match && Math.abs(match.top - item.top) <= 1, "Contents must not scroll reader ancestors");
      }
      await reader.screenshot({ path: `${out}/contents-${mode.toLowerCase()}-${width}.png` });
      await sources.getByRole("button", { name: "Close contents", exact: true }).click();
      assert.ok(await opener.evaluate(node => node === document.activeElement), "Restore Contents opener focus");
    }
  }
  const review = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  await review.goto(`${base}?assistant=off`);
  await review.getByRole("main").getByRole("button", { name: /^Abiding in the Vine/ }).first().click();
  await review.getByRole("button", { name: "Bible Study information", exact: true }).click();
  const information = review.getByRole("dialog", { name: "Bible Study information" });
  await information.getByRole("button", { name: "Move to Trash" }).click();
  const confirmation = information.getByRole("alertdialog");
  await confirmation.scrollIntoViewIfNeeded();
  await review.screenshot({ path: `${out}/danger-zone.png` });
  await confirmation.getByRole("button", { name: "Move to Trash" }).click();
  await information.waitFor({ state: "detached" });
  await review.getByRole("button", { name: "Settings", exact: true }).click();
  const openSettings = async name => {
    const region = review.getByRole("region", { name, exact: true });
    const toggle = region.getByRole("button", { name, exact: true });
    if (await toggle.getAttribute("aria-expanded") !== "true") await toggle.click();
    return region;
  };
  const data = await openSettings("Data and backup");
  const trash = data.getByRole("region", { name: "Trash", exact: true });
  await trash.getByRole("heading", { name: "Abiding in the Vine", exact: true }).waitFor();
  await trash.scrollIntoViewIfNeeded();
  await review.screenshot({ path: `${out}/trash.png` });
  const templates = await openSettings("Templates");
  const teachingTemplates = templates.getByRole("list", { name: "Bible Teaching templates" });
  assert.equal(await teachingTemplates.getByRole("listitem").count(), 3);
  await teachingTemplates.scrollIntoViewIfNeeded();
  await review.screenshot({ path: `${out}/templates.png` });
  const library = await openSettings("Sources");
  await library.getByRole("button", { name: "Add Sources…", exact: true }).click();
  await review.getByRole("dialog", { name: "Add Sources", exact: true }).waitFor();
  await review.screenshot({ path: `${out}/import.png` });
  // Minimum supported desktop width: the default Scripture + Study layout.
  const minimum = await browser.newPage({ viewport: { width: 640, height: 800 } });
  await minimum.goto(`${base}?assistant=off`);
  await minimum.getByRole("main").getByRole("button", { name: /^Abiding in the Vine/ }).first().click();
  await minimum.getByRole("button", { name: "Step-by-step", exact: true }).click();
  const selector = minimum.locator("[data-study-steps] select");
  await selector.waitFor();
  const minimumBounds = await selector.boundingBox();
  assert.ok(minimumBounds.width >= 180 && minimumBounds.x + minimumBounds.width <= 640);
  for (const name of ["Change language", "Settings", "Show Sources pane"]) {
    const button = minimum.getByRole("button", { name, exact: true });
    const bounds = await button.boundingBox();
    assert.ok(bounds && bounds.x >= 0 && bounds.x + bounds.width <= 640, name);
  }
  await minimum.screenshot({ path: `${out}/study-640.png` });
  console.log("A.1 full-workspace checks passed at 900, 1100 and 1440 pixels, including EPUB Reading and Original; default layout at 640 pixels.");
} finally {
  await browser.close();
}
