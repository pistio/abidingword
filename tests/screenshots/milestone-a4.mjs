// A.4 review: the complete workspace with the invented handbook PDF (actual
// Rust import/index/reader metadata from tests/macarthur_shape.rs with
// AW_A4_REVIEW_DIR) and real PDF.js. Synthetic evidence; NOT the MacArthur PDF.
import { createRequire } from "node:module";
import { mkdirSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const base = process.env.SCREENSHOT_URL ?? "http://localhost:1430/tests/screenshots/index.html";
const out = process.env.SCREENSHOT_OUT ?? "docs/implementation/milestone-a4/images";
mkdirSync(out, { recursive: true });
const browser = await chromium.launch({ ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}), headless: true, args: ["--no-sandbox"] });
const results = [];
const scenes = (process.env.A4_SCENES ?? "toolbar").split(",");
try {
  if (scenes.includes("toolbar")) for (const [width, scale] of [[640, 1], [900, 1], [1100, 1], [1440, 1], [900, 1.25], [1100, 1.5], [1440, 2]]) {
    const page = await browser.newPage({ viewport: { width, height: 900 }, deviceScaleFactor: scale });
    const errors = []; page.on("pageerror", (e) => errors.push(e.message));
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3PdfReview=1&passage=ACT.6`);
    const sources = page.locator("#pane-sources");
    const result = sources.locator("[data-source-result]").filter({ hasText: "Invented handbook" }).first();
    await result.waitFor();
    const label = await result.textContent();
    assert.ok(label.includes("Book section"), label);
    const tag = `${width}-${scale}`;
    if (scale === 1) await page.screenshot({ path: `${out}/a41-sources-${tag}.png` });
    await result.click();
    const body = sources.getByTestId("reader-body"); await body.waitFor();
    await body.locator("[data-pdf-heading=\"p8-c0\"]").waitFor();
    const toolbar = sources.getByRole("toolbar", { name: "Reader tools" });
    const indicator = toolbar.getByTestId("pdf-page-indicator");
    await page.waitForFunction(() => document.querySelector("#pane-sources [data-testid='pdf-page-indicator']")?.textContent?.startsWith("8 /"));
    const box = await toolbar.boundingBox();
    const fits = await toolbar.evaluate((el) => el.scrollWidth <= el.clientWidth + 1);
    assert.ok(box.height <= 60, `toolbar is one row (${box.height}px)`);
    assert.ok(fits, "toolbar does not overflow horizontally");
    await page.screenshot({ path: `${out}/a42-reading-${tag}.png` });
    await toolbar.getByRole("button", { name: "More reader actions" }).click();
    await toolbar.getByRole("menu").waitFor();
    if (scale === 1) await page.screenshot({ path: `${out}/a42-menu-${tag}.png` });
    await page.keyboard.press("Escape");
    await toolbar.getByRole("button", { name: "Original", exact: true }).click();
    await sources.getByTestId("pdf-viewer").waitFor();
    await sources.locator('[data-page="8"] canvas').waitFor();
    await page.waitForTimeout(200);
    assert.equal(await sources.getByRole("textbox", { name: "Page number" }).inputValue(), "8");
    if (scale === 1) await page.screenshot({ path: `${out}/a42-original-${tag}.png` });
    await toolbar.getByRole("button", { name: "Reading", exact: true }).click();
    await indicator.click();
    const jump = sources.getByRole("dialog", { name: "Go to page" }); await jump.waitFor();
    if (width === 900 && scale === 1) await page.screenshot({ path: `${out}/a42-jump-${tag}.png` });
    await page.keyboard.press("Escape");
    const geometry = await page.locator("#pane-scripture,#pane-study,#pane-sources").evaluateAll((nodes) => nodes.map((n) => ({ id: n.id, width: n.clientWidth, scrollWidth: n.scrollWidth })));
    assert.ok(geometry.every((n) => n.scrollWidth <= n.width + 2), JSON.stringify(geometry));
    assert.deepEqual(errors, []);
    results.push({ scene: "toolbar", width, scale, toolbarHeight: box.height, status: "passed", geometry });
    await page.close();
  }
  if (scenes.includes("diagnostic")) {
    const page = await browser.newPage({ viewport: { width: 1100, height: 900 } });
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3PdfReview=1&passage=ACT.6`);
    const sources = page.locator("#pane-sources");
    await sources.locator("[data-source-result]").filter({ hasText: "Invented handbook" }).first().click();
    const toolbar = sources.getByRole("toolbar", { name: "Reader tools" }); await toolbar.waitFor();
    await toolbar.getByRole("button", { name: "More reader actions" }).click();
    await toolbar.getByRole("menuitem", { name: "Document structure" }).click();
    const panel = page.getByRole("dialog", { name: "Document structure", exact: true }); await panel.waitFor();
    await panel.getByText("Diagnostic report (metadata only)").click();
    await page.screenshot({ path: `${out}/a41-structure-diagnostic-1100.png` });
    results.push({ scene: "diagnostic", status: "passed" });
    await page.close();
  }
  if (scenes.includes("languages")) for (const lang of ["de", "ko"]) {
    const page = await browser.newPage({ viewport: { width: 900, height: 900 } });
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3PdfReview=1&passage=ACT.6&lang=${lang}`);
    const sources = page.locator("#pane-sources");
    await sources.locator("[data-source-result]").filter({ hasText: "Invented handbook" }).first().click();
    const toolbar = sources.getByRole("toolbar"); await toolbar.waitFor();
    await sources.getByTestId("reader-body").waitFor();
    const fits = await toolbar.evaluate((el) => el.scrollWidth <= el.clientWidth + 1);
    assert.ok(fits, `${lang}: toolbar fits`);
    await page.screenshot({ path: `${out}/a42-reading-${lang}-900.png` });
    results.push({ scene: "language", lang, status: "passed" });
    await page.close();
  }
} finally {
  await browser.close();
  writeFileSync(`${out}/checks-a4.json`, JSON.stringify(results, null, 2));
}
console.log(JSON.stringify(results.map((r) => `${r.scene} ${r.width ?? r.lang ?? ""} ${r.scale ?? ""} ${r.status}`)));

// A.4.3 editors: run with A4_SCENES=editors (also against the A.3 baseline for "before").
if ((process.env.A4_SCENES ?? "").split(",").includes("editors")) {
  const { readFileSync } = await import("node:fs");
  const de = JSON.parse(readFileSync(new URL("../../src/i18n/locales/de/studies.json", import.meta.url), "utf8"));
  const prefix = process.env.A4_PREFIX ?? "a43";
  const browser2 = await chromium.launch({ ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}), headless: true, args: ["--no-sandbox"] });
  const done = [];
  try {
    for (const width of [640, 900, 1100, 1440]) {
      const page = await browser2.newPage({ viewport: { width, height: 1000 } });
      const errors = []; page.on("pageerror", (e) => errors.push(e.message));
      await page.goto(`${base}?assistant=off&a4Editors=1&lang=de&passage=ROM.8`);
      const study = page.locator("#pane-study");
      await study.getByRole("button", { name: /Organisatorisches Projekt/ }).first().click();
      await study.getByRole("button", { name: /Römer 8/ }).first().click();
      await study.getByRole("heading", { level: 2 }).first().waitFor();
      await page.waitForTimeout(200);
      await page.screenshot({ path: `${out}/${prefix}-study-de-${width}.png` });
      await study.getByRole("button", { name: de.view.teaching, exact: true }).click();
      await study.getByRole("radio", { name: new RegExp(de.teaching.builtin.eva.name) }).check({ force: true });
      await study.getByRole("button", { name: de.teaching.start, exact: true }).click();
      await study.getByRole("heading", { name: de.teaching.builtin.eva.blocks.erkennen.title }).first().waitFor();
      await page.waitForTimeout(200);
      // The header first, as the teaching opens for the person (a new teaching may focus its first field).
      await page.evaluate(() => document.querySelectorAll("#pane-study, #pane-study *").forEach((el) => { if (el.scrollTop) el.scrollTop = 0; }));
      await page.screenshot({ path: `${out}/${prefix}-teaching-de-${width}.png` });
      const geometry = await page.locator("#pane-study").evaluate((n) => ({ width: n.clientWidth, scrollWidth: n.scrollWidth }));
      assert.ok(geometry.scrollWidth <= geometry.width + 2, JSON.stringify(geometry));
      assert.deepEqual(errors, []);
      done.push(`editors ${width} passed`);
      await page.close();
    }
  } finally {
    await browser2.close();
  }
  console.log(JSON.stringify(done));
}
