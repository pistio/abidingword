// Real React/CSS with invented memory data. NOT Win32/WebView2 acceptance.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { domSnapshot } from '../../scripts/windows/viewport-evidence.mjs';
const { chromium } = createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? 'playwright');
const output = process.env.VIEWPORT_EVIDENCE_OUT ?? 'docs/implementation/issue-49/browser-contract.json';
const base = process.env.VIEWPORT_URL ?? 'http://localhost:1432/tests/screenshots/index.html';
mkdirSync(dirname(output), { recursive: true });
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium', args: ['--no-sandbox'] });
const results = [];
try {
  for (const scale of [1, 1.25, 1.5, 2]) for (const width of [900, 1440]) {
    const page = await browser.newPage({ viewport: { width, height: 800 }, deviceScaleFactor: scale });
    await page.routeWebSocket('**', socket => socket.close()); // freeze HMR during measurement
    const errors = [];page.on('pageerror', e => errors.push(e.message));
    await page.goto(`${base}?empty=1&assistant=off&a4Editors=1&viewportReview=1&sourcesOpen=1&lang=en`);
    const study = page.locator('#pane-study');
    await study.getByRole('button', { name: /INVENTED Viewport Project/ }).first().click();
    await study.getByRole('button', { name: /INVENTED Viewport Study/ }).first().click();
    await page.locator('#pane-sources [data-source-result]').filter({ hasText: 'INVENTED Viewport Source' }).first().click();
    await page.locator('#pane-sources [data-testid=reader-body]').waitFor();
    await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
    const area = study.locator('section textarea').first();await area.waitFor();await area.focus();
    await area.press('End');await area.pressSequentially(' Unsaved resize draft.');
    const draft = await area.inputValue();
    const caret = await area.evaluate(el => [el.selectionStart, el.selectionEnd]);
    for (const height of [800, 1019, 670, 1130, 800]) {
      await page.setViewportSize({ width, height });
      await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
      const m = await page.evaluate(domSnapshot);
      for (const [name, rect] of Object.entries({ shell: m.shell, workspace: m.workspace, ...Object.fromEntries(Object.entries(m.panes).map(([k, v]) => [k, v.rect])) })) {
        assert.ok(rect, `missing ${name}`);assert.ok(Math.abs(rect.bottom - m.innerHeight) <= 2, `${name} clipped: ${JSON.stringify(m)}`);
      }
      assert.equal(m.documentHeight, m.innerHeight);
      assert.equal(await area.inputValue(), draft);assert.ok(await area.evaluate(el => el === document.activeElement));
      assert.deepEqual(await area.evaluate(el => [el.selectionStart, el.selectionEnd]), caret);
      const scrolling = await page.evaluate(async () => {
        const results = [];
        for (const name of ['scripture', 'study', 'sources']) {
          const root = [...document.querySelector(`#pane-${name}`).querySelectorAll('*')].find(el => /^(auto|scroll)$/.test(getComputedStyle(el).overflowY) && el.clientHeight > 50 && el.scrollHeight > el.clientHeight + 10);
          if (!root) { results.push({ name, passed: false });continue; }
          const old = root.scrollTop;root.scrollTop = root.scrollHeight;
          await new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));
          results.push({ name, passed: Math.abs(root.scrollHeight - root.clientHeight - root.scrollTop) <= 2, scrollTop: root.scrollTop, scrollHeight: root.scrollHeight, clientHeight: root.clientHeight });root.scrollTop = old;
        }
        return results;
      });
      assert.ok(scrolling.every(s => s.passed), JSON.stringify(scrolling));assert.deepEqual(errors, []);
      // Both 800 cases get distinct filenames so restore cannot overwrite initial evidence.
      const index = results.length;
      await page.screenshot({ path: join(dirname(output), `browser-${scale}-${width}-${height}-${index}.png`) });
      results.push({ scale, width, height, ...m, scrolling, draftAndCaretPreserved: true, pageErrors: errors });
    }
    await page.close();
  }
} finally { await browser.close(); }
writeFileSync(output, JSON.stringify({ kind: 'Chromium DOM-only; emulated DPR is NOT Windows OS DPI or WebView2 evidence', note: 'No CSS/application layout repair made. Existing browser DOM contract already passed before changes. Native Windows cause and before/after remain unverified.', results }, null, 2) + '\n');
console.log(`Verified ${results.length} browser geometry/scroll/edit cases (NOT native WebView2).`);
