// Actual PDF.js + invented Rust-derived Source metadata. Linux Chromium is
// independent browser evidence; native Windows WebView2 remains an owner gate.
import {createRequire} from 'node:module';import {mkdirSync,writeFileSync} from 'node:fs';import assert from 'node:assert/strict';
const require=createRequire(import.meta.url);const {chromium}=require(process.env.PLAYWRIGHT_MODULE??'playwright');
const out='docs/implementation/windows-owner-corrections/images';mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox']});const results=[];
async function mode(page,source,name) {
 const direct=source.getByRole('button',{name,exact:true});
 if(await direct.count()) {await direct.click();return;}
 await source.getByRole('button',{name:/^View:/}).click();await page.getByRole('menu',{name:'View',exact:true}).getByRole('menuitem',{name,exact:true}).click();
}
async function action(page,source,name) {
 const direct=source.getByRole('button',{name,exact:true});
 if(await direct.count()) {await direct.click();return;}
 await source.getByRole('button',{name:'More reader actions'}).click();await page.getByRole('menu',{name:'More reader actions',exact:true}).getByRole('menuitem',{name,exact:true}).click();
}
const geometry=page=>page.locator('#pane-scripture,#pane-study,#pane-sources').evaluateAll(nodes=>nodes.map(n=>({id:n.id,x:n.getBoundingClientRect().x,width:n.clientWidth,scrollWidth:n.scrollWidth})));
try {
 for(const [before,physicalWidth,scale] of [[true,900,1],[false,640,1],[false,900,1],[false,1100,1],[false,1440,1],[false,1100,1.25],[false,1440,1.5],[false,1440,2]]) {
  console.log({before,physicalWidth,scale});const width=Math.round(physicalWidth/scale);const page=await browser.newPage({viewport:{width,height:900},deviceScaleFactor:scale});const errors=[];page.on('console',m=>{if(m.type()==='error')console.log(m.text());});page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`http://localhost:${before?1431:1430}/tests/screenshots/index.html?assistant=off&sourcesOpen=1&a3PdfReview=1&passage=ACT.6`);
  const study=page.locator('#pane-study');
  await study.getByRole('button',{name:/Gospel of John/}).first().click();
  await study.getByRole('button',{name:/Abiding in the Vine/}).first().click();
  await study.locator('textarea').first().waitFor();
  const source=page.locator('#pane-sources');await source.getByRole('button',{name:/Invented handbook/}).first().click();
  await source.getByTestId('reader-body').waitFor();const initial=await geometry(page);
  const writing=await study.locator('textarea').evaluateAll(fields=>fields.map(f=>f.value));assert.ok(writing.some(Boolean),'must test actual Study writing');
  await mode(page,source,'Original');await source.locator('canvas').first().waitFor();
  const header=source.getByTestId('reader-header');const height=(await header.boundingBox()).height;
  if(before) {
   assert.equal(await source.getByRole('toolbar').count(),2);await page.screenshot({path:`${out}/34-before-original-900.png`});
   results.push({before:true,width,toolbars:2,platform:'Chromium'});await page.close();continue;
  }
  assert.equal(await source.getByRole('toolbar').count(),1,'one entire reader toolbar');assert.ok(height<=60,`header ${height}px`);
  assert.ok(await header.evaluate(el=>el.scrollWidth<=el.clientWidth+1),'header overflow');
  const compare=async()=>{
   const now=await geometry(page);assert.deepEqual(now.map(({id,x,width})=>({id,x,width})),initial.map(({id,x,width})=>({id,x,width})),'Source interactions moved panes');
   assert.ok(now.every(p=>p.scrollWidth<=p.width+2),JSON.stringify(now));
   assert.deepEqual(await page.locator('#pane-study textarea').evaluateAll(fields=>fields.map(f=>f.value)),writing,'Study writing changed');
  };
  await page.screenshot({path:`${out}/34-original-${physicalWidth}-${scale}.png`});
  await compare();
  for(let i=0;i<8;i++)await action(page,source,'Zoom in');
  await page.waitForFunction(()=>document.querySelector('[data-testid="pdf-zoom"]')?.textContent?.includes('400'));
  await compare();
  await mode(page,source,'Reading');await source.getByTestId('reader-body').waitFor();await compare();
  await mode(page,source,'Original');await source.locator('canvas').first().waitFor();
  assert.match(await source.getByTestId('pdf-zoom').textContent(),/400/,'zoom survives mode switches');await compare();
  await action(page,source,'Fit width');await compare();
  await action(page,source,'Contents');await source.getByRole('navigation',{name:/Contents of/}).waitFor();await compare();await page.keyboard.press('Escape');
  await source.getByTestId('pdf-page-indicator').click();const jump=page.getByRole('dialog',{name:'Go to page',exact:true});await jump.waitFor();
  const rect=await jump.boundingBox();assert.ok(rect.x>=0&&rect.x+rect.width<=width+1,'jump clipped by pane');
  await jump.getByRole('spinbutton',{name:'PDF page'}).fill('9');await jump.getByRole('button',{name:'Go to page',exact:true}).click();
  await page.waitForFunction(()=>document.querySelector('[data-testid="pdf-page-indicator"]')?.getAttribute('aria-label')?.startsWith('PDF page 9 of'));
  await compare();assert.ok(await source.getByTestId('pdf-page-indicator').evaluate(el=>document.activeElement===el),'page focus lost');
  const oldViewport=page.viewportSize();await page.setViewportSize({width:width+120,height:900});await page.waitForTimeout(200);
  assert.ok((await geometry(page)).every(p=>p.scrollWidth<=p.width+2),'resized pane overflow');
  await page.setViewportSize(oldViewport);await page.waitForTimeout(200);await compare();
  assert.deepEqual(errors,[]);results.push({physicalWidth,scale,cssWidth:width,headerHeight:height,toolbars:1,zoomRestored:400,geometry:initial,pageErrors:errors});await page.close();
 }
 writeFileSync(`${out}/34-browser-checks.json`,JSON.stringify({platform:'Linux Chromium; actual PDF.js, invented Source; not WebView2 acceptance',results},null,2)+'\n');console.log(JSON.stringify(results));
} finally {await browser.close();}
