// Real Chromium interaction and inert/focus verification using the existing
// React screenshot harness and invented memory data. Not native SQLite/Windows.
// PLAYWRIGHT_MODULE=/path/to/playwright node tests/screenshots/parent-deletion.mjs <evidence-directory>
import assert from "node:assert/strict";
import { mkdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";

const { chromium } = createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const output = resolve(process.argv[2] ?? "work/recovery-verification/parent-deletion/browser");
mkdirSync(output, { recursive: true });
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? "/usr/bin/chromium", args: ["--no-sandbox"] });
const page = await browser.newPage({ viewport: { width: 1280, height: 900 } });
const errors = [];
page.on("pageerror", error => errors.push(error.message));
const result = { platform: "Linux Chromium; actual React App with invented in-memory backend; no native Windows or SQLite acceptance", errors };

try {
  await page.addInitScript(() => {
    let api;
    Object.defineProperty(window, "api", { get: () => api, set: value => {
      api = value;
      const state = window.deletionReview = { writes: 0, deletes: 0, latePortalClicks: 0 };
      const originalInfo = api.appInfo;
      const ready = api.projectCreate({ workspaceId: api.state.workspace.id, title: "Invented disposable project" }).then(project => {
        state.projectId = project.id;
      });
      api.appInfo = async () => { await ready; return originalInfo(); };
      const update = api.projectUpdate;
      api.projectUpdate = async input => {
        state.writes++;
        if (state.writes === 1) await new Promise(done => { state.finishSave = done; });
        return update(input);
      };
      const remove = api.projectDelete;
      api.projectDelete = async (...args) => {
        state.deletes++;
        if (state.deletes === 1) await new Promise((_, reject) => { state.failDelete = () => reject({ kind: "database", message: "Invented deletion failure; project retained" }); });
        return remove(...args);
      };
    } });
  });
  await page.goto(`${process.env.SCREENSHOT_URL ?? "http://localhost:1433/tests/screenshots/index.html"}?empty=1&assistant=off&lang=en`);
  await page.getByRole("button", { name: "Invented disposable project Project folder", exact: true }).click();
  const field = page.getByRole("textbox", { name: "Project title", exact: true });
  await field.fill("Latest invented writing");
  await page.getByRole("button", { name: "Project information", exact: true }).click();
  const panel = page.getByRole("dialog", { name: "Project information", exact: true });
  await panel.getByRole("button", { name: "Move to Trash", exact: true }).click();
  const confirm = panel.getByRole("alertdialog").getByRole("button", { name: "Move to Trash", exact: true });
  await confirm.click();
  await page.waitForFunction(() => !!window.deletionReview.finishSave && !!document.querySelector("[data-writing-operation]"));
  const frozenGeometry = async () => page.evaluate(() => {
    const overlay = document.querySelector("[data-writing-operation]");
    const box = overlay.getBoundingClientRect();
    return { rootInert: document.querySelector("#root").inert, focusedStatus: document.activeElement === overlay,
      bodyChildren: [...document.body.children].map(el => ({ tag: el.tagName, inert: el.inert ?? false, status: el === overlay })),
      overlay: { x: box.x, y: box.y, width: box.width, height: box.height },
      writes: window.deletionReview.writes, deletes: window.deletionReview.deletes };
  });
  result.pendingSave = await frozenGeometry();
  assert.equal(result.pendingSave.deletes, 0, "delete must await the active save");
  assert.equal(result.pendingSave.rootInert, true);
  assert.equal(result.pendingSave.focusedStatus, true);
  const fieldBox = await field.boundingBox();
  await page.mouse.click(fieldBox.x + 8, fieldBox.y + 8);
  await page.keyboard.type("late input");
  await page.keyboard.press("Escape");
  await page.mouse.click(10, 10);
  assert.equal(await field.inputValue(), "Latest invented writing");
  assert.equal(await panel.count(), 1, "outside click and Escape must not dismiss the panel while saves settle");
  await page.evaluate(() => window.deletionReview.finishSave());
  await page.waitForFunction(() => !!window.deletionReview.failDelete);
  await page.evaluate(() => {
    const button = document.createElement("button");
    button.id = "invented-late-portal";
    button.textContent = "Invented late portal action";
    button.onclick = () => window.deletionReview.latePortalClicks++;
    document.body.append(button);
  });
  await page.waitForFunction(() => document.querySelector("#invented-late-portal").inert);
  await page.locator("#invented-late-portal").dispatchEvent("click");
  await page.keyboard.press("Tab");
  await page.keyboard.type("another late draft");
  await page.keyboard.press("Escape");
  assert.equal(await field.inputValue(), "Latest invented writing");
  result.pendingDelete = await frozenGeometry();
  assert.equal(result.pendingDelete.focusedStatus, true);
  assert.equal(await page.evaluate(() => window.deletionReview.latePortalClicks), 0);
  await page.screenshot({ path: `${output}/pending-delete.png` });
  await page.evaluate(() => window.deletionReview.failDelete());
  await panel.getByRole("alert").waitFor();
  await page.locator("[data-writing-operation]").waitFor({ state: "detached" });
  await page.waitForFunction(() => document.activeElement?.textContent === "Move to Trash");
  result.released = await page.evaluate(() => ({ rootInert: document.querySelector("#root").inert,
    latePortalInert: document.querySelector("#invented-late-portal").inert,
    focused: document.activeElement?.outerHTML.slice(0, 600) }));
  result.released.error = await panel.getByRole("alert").textContent();
  assert.equal(result.released.rootInert, false);
  assert.equal(result.released.latePortalInert, false);
  assert.equal(await confirm.evaluate(el => el === document.activeElement), true, "failed deletion must return keyboard focus to its enabled initiator");
  await page.screenshot({ path: `${output}/failure-released.png` });
  // Failed deletion releases the real title editor without reload.
  await field.click();
  await field.press("End");
  await page.keyboard.type(" remains editable");
  await page.getByRole("button", { name: "Project information", exact: true }).click();
  await page.waitForFunction(() => window.api.state.projects.some(p => p.title === "Latest invented writing remains editable"));
  assert.equal(await field.inputValue(), "Latest invented writing remains editable");
  await panel.getByRole("button", { name: "Move to Trash", exact: true }).click();
  await confirm.click();
  await panel.waitFor({ state: "detached" });
  assert.equal(await page.locator("[data-writing-operation]").count(), 0);
  assert.equal(await page.locator("#root").evaluate(el => el.inert), false);
  await page.getByRole("button", { name: "Settings", exact: true }).click();
  await page.getByRole("heading", { name: "Settings", exact: true }).waitFor();
  await page.screenshot({ path: `${output}/success-usable.png` });
  assert.deepEqual(errors, []);
  result.passed = true;
} catch (error) {
  result.passed = false;
  result.error = String(error);
  await page.screenshot({ path: `${output}/failure.png` }).catch(() => {});
  throw error;
} finally {
  writeFileSync(`${output}/results.json`, JSON.stringify(result, null, 2) + "\n");
  await browser.close();
}
console.log(JSON.stringify(result, null, 2));
