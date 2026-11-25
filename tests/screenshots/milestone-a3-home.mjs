// Complete workspace; invented identical profile on archived A.2 and current app.
import { createRequire } from "node:module";
import { mkdirSync, writeFileSync } from "node:fs";
import assert from "node:assert/strict";
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const out = process.env.SCREENSHOT_OUT ?? "docs/implementation/milestone-a3-home/images";
mkdirSync(`${out}/before`, { recursive: true });
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? "/usr/bin/chromium", headless: true, args: ["--no-sandbox"] });
const base = process.env.SCREENSHOT_URL ?? "http://localhost:1430/tests/screenshots/index.html";
const before = process.env.BEFORE_URL ?? "http://localhost:1431/tests/screenshots/index.html";
const results = [];
const pane = page => page.getByRole("main", { name: "Study", exact: true });
const bounds = page => page.locator("#pane-scripture,#pane-study,#pane-sources").evaluateAll(nodes => nodes.map(node => ({ id: node.id, width: node.clientWidth, scrollWidth: node.scrollWidth })));
try {
  for (const width of [900, 1440]) {
    const page = await browser.newPage({ viewport: { width, height: 1000 } });
    await page.goto(`${before}?assistant=off&sourcesOpen=1&a3Review=1`);
    await pane(page).getByRole("button", { name: /^Root Bible Study/ }).waitFor();
    await page.screenshot({ path: `${out}/before/home-${width}.png` });
    await page.close();
  }
  for (const width of [900, 1440]) for (const [url, prefix] of [[before, "before/owner"], [base, "owner"]]) {
    const page = await browser.newPage({ viewport: { width, height: 1000 } });
    await page.goto(`${url}?assistant=off&sourcesOpen=1&a3Review=1&a3Shape=owner`);
    await pane(page).getByRole("button", { name: /^Root Bible Study/ }).waitFor();
    assert.equal(await pane(page).getByRole("button", { name: "Standalone Teaching", exact: true }).count(), url === before ? 1 : 0);
    await page.screenshot({ path: `${out}/${prefix}-${width}.png` });
    await page.close();
  }
  for (const lang of ["de", "ko"]) {
    const page = await browser.newPage({ viewport: { width: 900, height: 1000 } });
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3Review=1&lang=${lang}`);
    await page.locator("#pane-study").getByRole("button", { name: /^Root Bible Study/ }).waitFor();
    assert.ok((await bounds(page)).every(p => p.scrollWidth <= p.width + 1));
    await page.screenshot({ path: `${out}/home-${lang}-900.png` });
    await page.close();
  }
  for (const [width, scale] of [[640, 1], [900, 1], [1100, 1], [1440, 1], [900, 1.25], [1100, 1.5], [1440, 2]]) {
    const page = await browser.newPage({ viewport: { width, height: 1000 }, deviceScaleFactor: scale });
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3Review=1`);
    const home = pane(page);
    await home.getByRole("button", { name: /^Root Bible Study/ }).waitFor();
    assert.equal(await home.getByRole("list", { name: "Studies", exact: true }).getByRole("listitem").count(), 3);
    assert.equal(await home.getByRole("button", { name: /^Abiding in the Vine/ }).count(), 0);
    assert.equal(await home.getByRole("button", { name: "Standalone Teaching", exact: true }).count(), 0);
    assert.equal(await home.getByRole("button", { name: "Organise in projects", exact: true }).count(), 0);
    assert.equal(await home.locator("footer").getByRole("button", { name: "Trash", exact: true }).count(), 1);
    await home.getByRole("button", { name: "New project", exact: true }).scrollIntoViewIfNeeded();
    const geometry = await bounds(page);
    assert.ok(geometry.every(p => p.scrollWidth <= p.width + 1), JSON.stringify(geometry));
    await page.screenshot({ path: `${out}/home-${width}-${scale}.png` });
    await home.getByRole("button", { name: "Gospel of John Project folder", exact: true }).click();
    await home.getByRole("button", { name: /^Abiding in the Vine/ }).waitFor();
    assert.equal(await home.getByRole("button", { name: /^Root Bible Study/ }).count(), 0);
    await page.screenshot({ path: `${out}/folder-${width}-${scale}.png` });
    results.push({ width, scale, panes: geometry });
    await page.close();
  }
  {
    const page = await browser.newPage({ viewport: { width: 900, height: 1000 } });
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3Review=1`);
    await pane(page).getByRole("button", { name: "New project", exact: true }).click();
    const dialog = page.getByRole("dialog", { name: "New project", exact: true });
    const long = "LongFolderNameWithoutSpaces".repeat(8);
    await dialog.getByRole("textbox", { name: "Name", exact: true }).fill(long);
    await dialog.getByRole("button", { name: "Save", exact: true }).click();
    await pane(page).getByRole("button", { name: `${long} Project folder`, exact: true }).click();
    await pane(page).getByRole("textbox", { name: "Project title", exact: true }).waitFor();
    const overflow = await pane(page).evaluate(node => Array.from(node.querySelectorAll("nav,[data-scroll-root]")).some(el => el.scrollWidth > el.clientWidth + 1));
    assert.equal(overflow, false, "Long folder breadcrumb stays inside the writing pane");
    await page.screenshot({ path: `${out}/long-folder-900.png` });
    await page.close();
  }
  const page = await browser.newPage({ viewport: { width: 1100, height: 1000 } });
  await page.goto(`${base}?assistant=off&sourcesOpen=1&a3Review=1`);
  const home = pane(page);
  const actions = home.getByRole("group", { name: "Actions for “Root Bible Study”", exact: true });
  await actions.getByRole("button", { name: "File actions", exact: true }).click();
  await page.getByRole("menuitem", { name: "Move", exact: true }).click();
  const move = page.getByRole("dialog", { name: "Move “Root Bible Study”", exact: true });
  await move.getByRole("combobox", { name: "Destination", exact: true }).selectOption({ label: "Gospel of John" });
  await page.screenshot({ path: `${out}/move.png` });
  await move.getByRole("button", { name: "Move", exact: true }).click();
  await home.getByRole("button", { name: /^Root Bible Study/ }).waitFor({ state: "detached" });
  await home.getByRole("button", { name: "Gospel of John Project folder", exact: true }).click();
  await home.getByRole("button", { name: /^Root Bible Study/ }).click();
  await home.getByText("Invented root research retained while filing this document.", { exact: true }).waitFor();
  await home.getByRole("button", { name: "Back to Gospel of John", exact: true }).click();
  await home.getByRole("navigation", { name: "Breadcrumb", exact: true }).getByRole("button", { name: "Studies", exact: true }).click();
  await home.getByRole("group", { name: "Actions for “Gospel of John”", exact: true }).getByRole("button", { name: "File actions", exact: true }).click();
  await page.getByRole("menuitem", { name: "Move to Trash", exact: true }).click();
  await page.getByRole("alertdialog", { name: "Move “Gospel of John” to Trash?", exact: true }).waitFor();
  await page.screenshot({ path: `${out}/folder-trash.png` });
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await page.getByRole("button", { name: "Settings", exact: true }).click();
  const data = page.getByRole("region", { name: "Data and backup", exact: true });
  await data.getByRole("button", { name: "Data and backup", exact: true }).click();
  await data.locator("summary", { hasText: "Legacy standalone Teaching" }).click();
  await data.getByRole("button", { name: "Historical independent message", exact: true }).click();
  const legacy = page.getByRole("dialog", { name: "Legacy standalone Teaching", exact: true });
  await legacy.getByRole("textbox", { name: "Main part", exact: true }).waitFor();
  assert.equal(await legacy.getByRole("textbox", { name: "Main part", exact: true }).inputValue(), "Invented private manuscript for compatibility review.");
  await page.screenshot({ path: `${out}/legacy-management.png` });
  await page.close();
  writeFileSync(`${out}/../workspace-results.json`, `${JSON.stringify(results, null, 2)}\n`);
  console.log("A.3 home: baseline/current complete workspace, seven width/DPR cases, folder navigation, move with retained writing, safe Trash and legacy management passed.");
} finally { await browser.close(); }
