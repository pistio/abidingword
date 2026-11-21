// Real React Settings interface with invented in-memory data, not a SQLite or
// Windows acceptance test. Native database/file preservation has Rust tests.
// Start the existing Vite screenshot harness; an empty profile needs no web.json.
// PLAYWRIGHT_MODULE=/path/to/playwright SCREENSHOT_URL=http://localhost:1433/tests/screenshots/index.html \
//   node tests/screenshots/recovery-interface.mjs <evidence-directory>
import assert from "node:assert/strict";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";

const { chromium } = createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const base = process.env.SCREENSHOT_URL ?? "http://localhost:1433/tests/screenshots/index.html";
const output = resolve(process.argv[2] ?? "work/recovery-verification/browser");
mkdirSync(output, { recursive: true });
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? "/usr/bin/chromium", args: ["--no-sandbox"] });
const results = [];
const failures = [];
const sizes = [
  { width: 1280, height: 900, scale: 1 },
  { width: 480, height: 640, scale: 1 },
  // The same physical extent as 1280×900 at 200% scaling. Browser emulation
  // cannot establish Windows monitor scaling or native screen-reader behaviour.
  { width: 640, height: 450, scale: 2 },
];
const locale = (lang, area) => JSON.parse(readFileSync(new URL(`../../src/i18n/locales/${lang}/${area}.json`, import.meta.url), "utf8"));

async function fits(locator, label) {
  const geometry = await locator.evaluate(el => {
    const r = el.getBoundingClientRect();
    return { x: r.x, y: r.y, right: r.right, bottom: r.bottom, width: r.width, height: r.height,
      scrollWidth: el.scrollWidth, clientWidth: el.clientWidth, scrollHeight: el.scrollHeight,
      clientHeight: el.clientHeight, viewportWidth: innerWidth, viewportHeight: innerHeight };
  });
  assert.ok(geometry.x >= -1 && geometry.right <= geometry.viewportWidth + 1, `${label}: horizontal bounds ${JSON.stringify(geometry)}`);
  assert.ok(geometry.scrollWidth <= geometry.clientWidth + 1, `${label}: horizontal overflow ${JSON.stringify(geometry)}`);
  return geometry;
}

try {
  for (const lang of ["en", "de", "ko", "fr", "es", "pt"]) {
    const settings = locale(lang, "settings");
    const words = settings.data;
    const common = locale(lang, "common");
    const app = locale(lang, "app");
    for (const size of sizes) for (const kind of ["daily", "full"]) {
      const name = `${lang}-${size.width}x${size.height}-${size.scale}x-${kind}`;
      const page = await browser.newPage({ viewport: { width: size.width, height: size.height }, deviceScaleFactor: size.scale });
      const errors = [];
      page.on("pageerror", error => errors.push(error.message));
      try {
        await page.goto(`${base}?empty=1&assistant=off&lang=${lang}`);
        await page.waitForFunction(() => !!window.api);
        await page.evaluate(() => {
          const state = window.api.state.portability;
          state.dailySnapshots = [
            { id: "abiding-word-v27-2026-11-19T10-00-00.000Z.db", createdAt: "2026-11-19T10:00:00Z", bytes: 987654, schemaVersion: 27, usable: true },
            { id: "abiding-word-v27-2026-11-18T10-00-00.000Z.db", createdAt: "2026-11-18T10:00:00Z", bytes: 14, schemaVersion: 27, usable: false },
          ];
          state.nextOpen = "C:/Disposable/Invented.awbackup";
          const safetyCopy = "C:\\Disposable\\" + "InventedLongRecoveryFolder".repeat(6) + "\\backups\\pre-restore\\abiding-word-v27-2026-11-20T10-00-00.000Z.db";
          const delay = original => async (...args) => {
            await new Promise(done => { window.finishInventedRestore = done; });
            return { ...(await original(...args)), safetyCopy };
          };
          window.api.dailySnapshotRestore = delay(window.api.dailySnapshotRestore);
          window.api.backupRestore = delay(window.api.backupRestore);
        });
        await page.getByRole("button", { name: app.toolbar.settings, exact: true }).click();
        const section = page.getByRole("region", { name: settings.sections.data, exact: true });
        await section.getByRole("button", { name: settings.sections.data, exact: true }).click();
        const open = async () => {
          if (kind === "daily") {
            const list = section.getByRole("list", { name: words.dailyTitle });
            if (!(await list.count())) await section.getByRole("button", { name: words.dailyShow, exact: true }).click();
            const buttons = list.getByRole("button", { name: words.dailyChoose, exact: true });
            await buttons.first().waitFor();
            assert.equal(await buttons.count(), 2);
            assert.equal(await buttons.nth(1).isDisabled(), true, "unavailable recovery point must be disabled");
            assert.ok(await buttons.first().getAttribute("aria-describedby"), "date and availability must describe each choice");
            await buttons.first().click();
          } else {
            await section.getByRole("button", { name: words.restore, exact: true }).click();
          }
          return section.getByRole("region", { name: kind === "daily" ? words.dailyPreview : words.restorePreview, exact: true });
        };
        let preview = await open();
        const previewGeometry = await fits(preview, `${name} warning`);
        const cancel = preview.getByRole("button", { name: common.cancel, exact: true });
        await cancel.focus();
        await page.keyboard.press("Escape");
        await preview.waitFor({ state: "detached" });
        assert.deepEqual(await page.evaluate(() => [window.api.state.portability.restored, window.api.state.portability.dailyRestored]), [[], []], "Cancel must not restore");
        preview = await open();
        const confirm = preview.getByRole("button", { name: kind === "daily" ? words.dailyConfirm : words.restoreConfirm, exact: true });
        await confirm.focus();
        await page.keyboard.press("Enter");
        await page.waitForFunction(() => typeof window.finishInventedRestore === "function");
        const pending = page.getByRole("dialog", { name: words.restoreInProgress, exact: true });
        await pending.waitFor();
        const pendingGeometry = await fits(pending, `${name} pending`);
        assert.equal(await page.locator("#root").evaluate(el => el.inert), true);
        await page.keyboard.press("Escape");
        assert.equal(await pending.isVisible(), true, "Escape must not dismiss an active restore");
        await page.evaluate(() => window.finishInventedRestore());
        const complete = page.getByRole("dialog", { name: kind === "daily" ? words.dailyRestored : words.restored, exact: true });
        await complete.waitFor();
        const completeGeometry = await fits(complete, `${name} completed`);
        assert.ok(completeGeometry.y >= 0 && completeGeometry.bottom <= size.height + 1, "completion must fit vertically and allow internal scrolling");
        const reload = complete.getByRole("button", { name: words.restoredContinue, exact: true });
        await reload.scrollIntoViewIfNeeded();
        await reload.focus();
        for (const key of ["Tab", "Shift+Tab", "Escape"]) {
          await page.keyboard.press(key);
          assert.equal(await reload.evaluate(el => el === document.activeElement), true, `${key}: reload focus must stay in completion dialog`);
        }
        assert.equal(await page.locator("#root").evaluate(el => el.inert), true, "completed restore must keep stale workspace inert");
        const restoredCounts = await page.evaluate(() => [window.api.state.portability.restored.length, window.api.state.portability.dailyRestored.length]);
        assert.deepEqual(restoredCounts, kind === "daily" ? [0, 1] : [1, 0]);
        if (size.width === 480 || (lang === "en" && size.scale === 2)) await page.screenshot({ path: `${output}/${name}.png` });
        const navigation = page.waitForEvent("framenavigated", { predicate: frame => frame === page.mainFrame() });
        await page.keyboard.press("Enter");
        await navigation;
        assert.deepEqual(errors, []);
        results.push({ name, lang, kind, size, previewGeometry, pendingGeometry, completeGeometry, cancelledSafely: true, keyboardReload: true, errors });
      } catch (error) {
        failures.push({ name, error: String(error), errors });
        await page.screenshot({ path: `${output}/${name}-failure.png` }).catch(() => {});
      } finally { await page.close(); }
    }
  }
} finally {
  await browser.close();
  writeFileSync(`${output}/results.json`, JSON.stringify({ platform: "Linux Chromium; real React interface, invented in-memory backend; reduced viewport/DPR emulation, not native Windows acceptance", results, failures }, null, 2) + "\n");
}
console.log(JSON.stringify({ passed: results.length, failed: failures.length, failures }, null, 2));
assert.equal(failures.length, 0, "recovery interface browser cases failed");
