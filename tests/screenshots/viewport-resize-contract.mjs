// Issue #49: vertical fill contract during viewport resizes.
// This exercises the real React pane layout in Chromium, with invented data.
// It CANNOT establish native Windows/WebView2 client/controller bounds.
// Run against the existing screenshot Vite fixture on port 1432.
import { createRequire } from 'node:module';
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
const { chromium } = createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE ?? 'playwright');
const output = process.env.VIEWPORT_EVIDENCE_OUT ?? 'docs/implementation/windows-viewport-p0-browser.json';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium', args: ['--no-sandbox'] });
const results = [];

async function measure(page) {
  return page.evaluate(() => {
    const rect = (selector) => {
      const node = document.querySelector(selector);
      if (!node) return null;
      const r = node.getBoundingClientRect();
      return { x: r.x, y: r.y, width: r.width, height: r.height, top: r.top, bottom: r.bottom };
    };
    return {
      innerHeight: window.innerHeight,
      innerWidth: window.innerWidth,
      devicePixelRatio: window.devicePixelRatio,
      documentHeight: document.documentElement.clientHeight,
      shell: rect('[data-testid="app-shell"]'),
      workspace: rect('[data-workspace-layout]'),
      scripture: rect('#pane-scripture'),
      study: rect('#pane-study'),
      sources: rect('#pane-sources')
    };
  });
}

try {
  for (const scale of [1, 1.5]) {
    const page = await browser.newPage({ viewport: { width: 1440, height: 800 }, deviceScaleFactor: scale });
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    await page.goto('http://localhost:1432/tests/screenshots/index.html?empty=1&assistant=off&a4Editors=1&sourcesOpen=1&lang=de');
    await page.locator('#pane-study').waitFor();

    for (const height of [800, 1019, 670, 1130, 800]) {
      await page.setViewportSize({ width: 1440, height });
      const m = await measure(page);
      const expected = m.innerHeight;
      for (const name of ['shell', 'workspace', 'scripture', 'study', 'sources']) {
        assert.ok(m[name], `missing ${name}`);
        assert.ok(Math.abs(m[name].bottom - expected) <= 2,
          `${name} bottom=${m[name].bottom}, expected=${expected}; ${JSON.stringify(m)}`);
      }
      assert.equal(m.documentHeight, m.innerHeight);
      assert.deepEqual(errors, []);
      results.push({ scale, height, ...m });
    }
    await page.close();
  }
} finally {
  await browser.close();
}
mkdirSync(output.slice(0, output.lastIndexOf('/')), { recursive: true });
writeFileSync(output, JSON.stringify({
  kind: 'Chromium DOM-only vertical-height contract; no Windows WebView2/native acceptance',
  note: 'This tests React/CSS resize response. A passing result cannot confirm the user-observed 193px native Windows gap is fixed.',
  results
}, null, 2) + '\n');
console.log(`Verified ${results.length} browser DOM-height cases (NOT native WebView2).`);
