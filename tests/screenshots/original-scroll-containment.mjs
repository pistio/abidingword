// Issue #49: adversarial browser test of scroll containment, not native Windows
// acceptance and not a full React Reading -> Original interaction test.
// Run from repository root using its TypeScript dependency and Playwright:
//   PLAYWRIGHT_MODULE=/path/to/playwright node tests/screenshots/original-scroll-containment.mjs
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { createHash } from 'node:crypto';
const require = createRequire(import.meta.url);
const { chromium } = require(process.env.PLAYWRIGHT_MODULE ?? 'playwright');
const ts = require(process.env.TYPESCRIPT_MODULE ?? 'typescript');
const sourcePath = resolve('src/lib/scrollInViewport.ts');
const source = readFileSync(sourcePath, 'utf8');
const compiled = ts.transpileModule(source, { compilerOptions: {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ES2022,
} }).outputText.replace('export function scrollInViewport', 'function scrollInViewport');
const output = process.env.SCROLL_EVIDENCE_OUT ?? 'work/original-scroll-containment.json';
const browser = await chromium.launch({ executablePath: process.env.CHROMIUM_PATH ?? '/usr/bin/chromium', args: ['--no-sandbox'] });
const results = [];
const html = `<!doctype html><html><head><style>
html { scroll-behavior: smooth; } body { margin:0; width:2400px; height:2400px; }
#shell { margin:400px 0 0 300px; width:700px; height:400px; overflow:hidden; scroll-behavior:smooth; }
#workspace { width:1400px; height:1200px; padding:180px 0 0 190px; box-sizing:border-box; }
#view { width:440px; height:210px; overflow:auto; scroll-behavior:smooth; border:2px solid; }
#other { width:50px; height:50px; overflow:auto; position:fixed; top:0; left:0; }
#other div { height:500px; width:500px; } #focus { position:fixed; top:60px; left:0; }
</style></head><body><input id="focus" value="INVENTED DRAFT"><div id="other"><div>Invented neighbouring pane</div></div>
<div id="shell"><div id="workspace"><div id="view"></div></div></div><div id="outside">Outside reader</div></body></html>`;

async function setup(page, depth) {
  await page.setContent(html);
  await page.addScriptTag({ content: `${compiled}\nwindow.scrollInViewport = scrollInViewport;` });
  await page.evaluate(depth => {
    let host = document.getElementById('view');
    for (let i = 0; i < depth; i++) {
      const wrapper = document.createElement('div');
      host.append(wrapper); host = wrapper.attachShadow({ mode: 'open' });
    }
    const content = document.createElement('div');
    content.innerHTML = '<div style="height:650px">INVENTED source material</div><mark id="target" style="display:block;height:30px">INVENTED footnote anchor</mark><div style="height:650px">INVENTED end</div>';
    host.append(content); window.target = content.querySelector('#target');
    document.getElementById('shell').scrollTo({ top: 37, left: 29, behavior:'instant' });
    document.getElementById('other').scrollTo({ top: 21, left: 13, behavior:'instant' });
    document.getElementById('view').scrollTo({ top: 19, left:0, behavior:'instant' });
    window.scrollTo({ top:100, left:17, behavior:'instant' });
    document.getElementById('focus').focus({ preventScroll:true });
    document.getElementById('focus').setSelectionRange(3,7);
  }, depth);
}

function snapshot() {
  const shell = document.getElementById('shell');
  const view = document.getElementById('view');
  const other = document.getElementById('other');
  const r = view.getBoundingClientRect(), t = window.target.getBoundingClientRect();
  return { shell: [shell.scrollLeft, shell.scrollTop], window: [scrollX, scrollY],
    other: [other.scrollLeft, other.scrollTop], viewTop: view.scrollTop,
    anchorOffset: t.top-r.top, anchorHeight:t.height, viewportHeight:view.clientHeight, borderTop:view.clientTop,
    active:document.activeElement.id, value:document.getElementById('focus').value,
    caret:[document.getElementById('focus').selectionStart,document.getElementById('focus').selectionEnd] };
}
try {
  for (const scale of [1, 1.25, 1.5, 2]) {
    const page = await browser.newPage({ viewport:{width:1440,height:1000}, deviceScaleFactor:scale });
    for (const depth of [0,1,2]) for (const block of ['start','center']) {
      await setup(page,depth);
      const beforeUnsafe = await page.evaluate(snapshot);
      await page.evaluate(block => window.target.scrollIntoView({block,inline:'nearest',behavior:'instant'}),block);
      const unsafe = await page.evaluate(snapshot);
      assert.notDeepEqual([unsafe.shell,unsafe.window],[beforeUnsafe.shell,beforeUnsafe.window], 'Negative control did not move any ancestor');
      await setup(page,depth);
      const before = await page.evaluate(snapshot);
      const applied = await page.evaluate(block => window.scrollInViewport(document.getElementById('view'),window.target,block),block);
      assert.equal(applied,true);
      // Also ensure smooth-scroll CSS cannot apply a delayed ancestor movement.
      await page.waitForTimeout(100);
      const after = await page.evaluate(snapshot);
      for (const key of ['shell','window','other','active','value','caret']) assert.deepEqual(after[key],before[key], `${key} moved for depth ${depth}, block ${block}`);
      assert.notEqual(after.viewTop,before.viewTop,'Reader did not move to target');
      const expectedOffset = after.borderTop + (block==='center' ? (after.viewportHeight-after.anchorHeight)/2 : 0);
      assert.ok(Math.abs(after.anchorOffset-expectedOffset)<=1, 'Target did not retain native alignment');
      results.push({ scale,depth,block,negativeControlMovedAncestor:true,passed:true,before,unsafe,after });
    }
    // Wrong-reader and absent targets must not alter any scroll/focus state.
    await setup(page,1); const before=await page.evaluate(snapshot);
    for (const kind of ['outside','missing','disconnected']) {
      const applied=await page.evaluate(kind=>window.scrollInViewport(document.getElementById('view'),kind==='outside'?document.getElementById('outside'):kind==='missing'?null:document.createElement('div'),'start'),kind);
      assert.equal(applied,false); assert.deepEqual(await page.evaluate(snapshot),before);
      results.push({scale,kind,passed:true});
    }
    // Restoration still runs if a browser/native shim throws after scrolling.
    const thrown=await page.evaluate(()=>{
      window.target.scrollIntoView=()=>{ document.getElementById('shell').scrollTo({top:100,left:100,behavior:'instant'}); throw Error('invented scroll failure'); };
      try {window.scrollInViewport(document.getElementById('view'),window.target,'start');return false;}catch(e){return e.message==='invented scroll failure';}
    });
    assert.equal(thrown,true); assert.deepEqual(await page.evaluate(snapshot),before);
    results.push({scale,kind:'restore-on-throw',passed:true});
    await page.close();
  }
} finally { await browser.close(); }
mkdirSync(dirname(output),{recursive:true});
writeFileSync(output,JSON.stringify({kind:'Linux Chromium synthetic scroll-containment qualification; NOT full app or Windows acceptance',helperSha256:createHash('sha256').update(source).digest('hex'),sourcePath:'src/lib/scrollInViewport.ts',results},null,2)+'\n');
console.log(`${results.length} synthetic browser checks passed; ${results.filter(r=>r.negativeControlMovedAncestor).length} unrestricted negative controls displaced ancestors. NOT Windows acceptance.`);
