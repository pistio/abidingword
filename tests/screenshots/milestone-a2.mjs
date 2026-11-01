// Complete workspace stress checks with real PDF.js and invented source bytes.
// Start the screenshot harness as in capture.mjs. No Windows acceptance claim.
import { createRequire } from "node:module";
import { mkdirSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const base = process.env.SCREENSHOT_URL ?? "http://localhost:1430/tests/screenshots/index.html";
const out = process.env.SCREENSHOT_OUT ?? "docs/implementation/milestone-a2/images";
mkdirSync(out, { recursive: true });
const browser = await chromium.launch({ ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}), headless: true, args: ["--no-sandbox"] });
const results = [];
const geometry = page => page.locator("#pane-scripture,#pane-study,#pane-sources").evaluateAll(nodes => nodes.map(node => {
  const rect = node.getBoundingClientRect();
  return { id: node.id, x: rect.x, y: rect.y, width: rect.width, height: rect.height, scrollWidth: node.scrollWidth, clientWidth: node.clientWidth };
}));
const stable = (before, after) => {
  for (const old of before) {
    const next = after.find(node => node.id === old.id);
    assert.ok(next, old.id);
    for (const key of ["x", "y", "width", "height"]) assert.ok(Math.abs(next[key] - old[key]) < 2, `${old.id} moved: ${JSON.stringify({ old, next })}`);
  }
};
try {
  for (const [width, scale] of [[640, 1], [900, 1], [1100, 1], [1440, 1], [900, 1.25], [1100, 1.5], [1440, 2]]) {
    const page = await browser.newPage({ viewport: { width, height: 1000 }, deviceScaleFactor: scale });
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a1Review=1&a2Review=1`);
    await page.getByRole("main").getByRole("button", { name: /^Abiding in the Vine/ }).first().click();
    const sources = page.locator("#pane-sources");
    const pdf = sources.getByRole("button", { name: /^Invented PDF navigation review/ }).first();
    await pdf.scrollIntoViewIfNeeded();
    const initial = await geometry(page);
    assert.equal(initial.length, 3);
    assert.ok(initial.every(node => node.width >= 239 && node.scrollWidth <= node.clientWidth + 2), JSON.stringify(initial));
    assert.equal(await page.locator("[data-workspace-layout]").getAttribute("data-workspace-layout"), width === 640 ? "stacked" : "columns");
    const studyBefore = await page.evaluate(() => JSON.stringify(window.api.state.blocks));
    await pdf.click();
    await sources.getByTestId("reader-body").waitFor();
    stable(initial, await geometry(page));
    await sources.getByRole("button", { name: "Next PDF page", exact: true }).click();
    assert.equal(await sources.getByRole("spinbutton", { name: "PDF page", exact: true }).inputValue(), "2");
    await sources.getByRole("button", { name: "Original", exact: true }).click();
    await sources.locator('[data-page="2"] canvas').waitFor();
    stable(initial, await geometry(page));
    const contents = sources.getByRole("button", { name: "Contents", exact: true });
    await contents.click();
    await sources.getByTestId("contents-panel").waitFor();
    stable(initial, await geometry(page));
    await page.screenshot({ path: `${out}/pdf-contents-${width}-${scale}.png` });
    await sources.getByRole("button", { name: "Close contents", exact: true }).click();
    assert.ok(await contents.evaluate(node => node === document.activeElement));
    await sources.getByRole("button", { name: "Back to Sources", exact: true }).click();
    await sources.getByRole("heading", { name: /Sources for John/ }).waitFor();
    assert.ok(await page.evaluate(() => !!document.activeElement?.getAttribute("data-source-result")), "Back returns focus to a live result");
    // Reopening an indexed hit must beat the cached page-2 position.
    await sources.locator('[data-source-result]').filter({ hasText: "Invented PDF navigation review" }).click();
    await sources.getByTestId("pdf-viewer").waitFor();
    await page.waitForFunction(() => document.querySelector('#pane-sources input[type="number"]')?.value === "1");
    await sources.getByRole("button", { name: "Back to Sources", exact: true }).click();
    await sources.getByRole("button", { name: /^Synthetic EPUB layout review/ }).first().click();
    await sources.getByRole("button", { name: "Reading", exact: true }).click();
    const body = sources.getByTestId("reader-body");
    await body.waitFor();
    await body.evaluate(node => { node.scrollTop = 450; });
    const scroll = await body.evaluate(node => node.scrollTop);
    const longBefore = await geometry(page);
    await sources.getByRole("button", { name: "Contents", exact: true }).click();
    await sources.getByTestId("contents-panel").waitFor();
    stable(longBefore, await geometry(page));
    assert.equal(await body.evaluate(node => node.scrollTop), scroll);
    await sources.getByRole("button", { name: "Close contents", exact: true }).click();
    const passage = page.locator("#pane-scripture").getByRole("textbox", { name: "Go to passage", exact: true });
    await passage.scrollIntoViewIfNeeded();
    const passageBefore = await geometry(page);
    await passage.fill("John 10");
    await passage.press("Enter");
    await page.locator("#pane-scripture").getByRole("heading", { name: "John 10", exact: true }).waitFor();
    stable(passageBefore, await geometry(page));
    assert.equal(await body.evaluate(node => node.scrollTop), scroll, "A Scripture change must not jump the open source");
    if (width !== 640) {
      const divider = page.getByRole("separator", { name: "Resize Sources pane", exact: true });
      const before = (await sources.boundingBox()).width;
      assert.ok(Math.abs(Number(await divider.getAttribute("aria-valuenow")) - before) <= 1);
      await divider.press("ArrowRight");
      const after = (await sources.boundingBox()).width;
      assert.ok(Math.abs((before - after) - 24) <= 1, `Resize jump: ${before} → ${after}`);
    }
    assert.equal(await page.evaluate(() => JSON.stringify(window.api.state.blocks)), studyBefore, "Source browsing must not rewrite Study writing");
    await sources.getByRole("button", { name: "Back to Sources", exact: true }).click();
    await page.screenshot({ path: `${out}/workspace-${width}-${scale}.png` });
    results.push({ width, deviceScaleFactor: scale, panes: await geometry(page), originalPdf: true, cachedTarget: "page 1", studyWritingPreserved: true });
    await page.close();
  }
  const review = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
  await review.goto(`${base}?assistant=off&sourcesOpen=1&a2Review=1`);
  const actions = review.getByRole("group", { name: "Actions for “POZETEK recovery review”", exact: true });
  await actions.getByRole("button", { name: "Move to Trash", exact: true }).click();
  await review.getByRole("menuitem", { name: "Move to Trash", exact: true }).click();
  const confirmation = review.getByRole("alertdialog", { name: "Move “POZETEK recovery review” to Trash?", exact: true });
  await confirmation.waitFor();
  await review.screenshot({ path: `${out}/listing-confirmation.png` });
  await confirmation.getByRole("button", { name: "Cancel", exact: true }).click();
  await review.getByRole("button", { name: /^POZETEK recovery review/ }).click();
  const restore = review.getByText("Restore missing sections", { exact: true });
  await restore.scrollIntoViewIfNeeded();
  await restore.click();
  await review.screenshot({ path: `${out}/restore-missing.png` });
  await review.getByRole("button", { name: "Restore “Time”", exact: true }).click();
  await review.getByRole("heading", { name: "Time", exact: true }).waitFor();
  await review.getByRole("heading", { name: "Time", exact: true }).scrollIntoViewIfNeeded();
  await review.screenshot({ path: `${out}/restored-time.png` });
  assert.ok(await review.evaluate(() => window.api.state.blocks.some(block => block.content === "Invented research to retain while restoring the method.")));
  writeFileSync(`${out}/../workspace-results.json`, JSON.stringify(results, null, 2) + "\n");
  console.log("A.2 complete workspace checks passed: 640/900/1100/1440, DPR 1/1.25/1.5/2; real invented PDF, EPUB, Contents, splitter and exact reopens.");
} finally { await browser.close(); }
