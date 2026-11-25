// Real React + PDF.js, generated Rust PDF fixture; no actual owner PDF/Windows claim.
import {createRequire} from 'node:module';
import {readFileSync,mkdirSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const {chromium}=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE??'playwright');
const out='docs/i18n/issue-42/images/full-review';mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',args:['--no-sandbox']});const results=[];
try{
 for(const language of ['en','de','ko','fr','es','pt'])for(const width of [900,1440]){
  const copy=JSON.parse(readFileSync(`src/i18n/locales/${language}/library.json`,'utf8'));
  const page=await browser.newPage({viewport:{width,height:1000},deviceScaleFactor:width===900?1.5:1});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  // Seed an installed edition without adding any Scripture text. Missing-text warning remains real UI.
  await page.route('**/web.json',route=>route.fulfill({json:{}}));
  await page.goto(`http://localhost:1432/tests/screenshots/index.html?assistant=off&sourcesOpen=1&a3PdfReview=1&lang=${language}&passage=ACT.6`);
  await page.waitForFunction(()=>Boolean(window.api));
  await page.evaluate(()=>{
   const api=window.api;
   api.libraryPdfStructureProgress=async()=>({status:'failed',stage:'index',done:448,total:1689,elapsedMs:308917,timings:[{stage:'index',milliseconds:304681}],headingCount:null,indexedLinks:null,errorCode:'library.structure.timeout',canCancel:false});
   const original=api.libraryPdfStructure;
   api.libraryPdfStructure=async id=>({...await original(id),headings:[],analysis:2,rebuildRecommended:true});
   api.libraryPdfStructureBuild=async()=>{throw {kind:'invalidInput',code:'library.structure.timeout',message:'Heading analysis stopped. Reopen Chapters and headings for its stage and retry. Previous metadata remains available.'};};
  });
  const source=page.locator('#pane-sources');
  await source.locator('[data-source-result]').filter({hasText:/Invented/}).first().click();
  await source.getByTestId('reader-body').waitFor();
  await page.screenshot({path:`${out}/${language}-source-${width}.png`});
  await source.getByRole('toolbar',{name:copy.toolbar.label}).getByRole('button',{name:copy.toolbar.more}).click();
  await page.getByRole('menu').getByRole('menuitem',{name:copy.structure.title,exact:true}).click();
  const dialog=page.getByRole('dialog',{name:copy.structure.title,exact:true});await dialog.waitFor();
  await dialog.getByRole('button',{name:copy.structure.rebuild,exact:true}).click();
  await dialog.getByText(copy.structure.timeout,{exact:true}).first().waitFor();
  assert.ok(!(await dialog.innerText()).includes('Heading analysis stopped.'),'English native fallback leaked');
  assert.ok(await dialog.evaluate(el=>el.scrollWidth<=el.clientWidth+2),'dialog horizontal overflow');
  const geometry=await page.locator('#pane-scripture,#pane-study,#pane-sources').evaluateAll(nodes=>nodes.map(n=>({id:n.id,width:n.clientWidth,scrollWidth:n.scrollWidth})));
  assert.ok(geometry.every(n=>n.scrollWidth<=n.width+2),JSON.stringify(geometry));assert.deepEqual(errors,[]);
  await dialog.screenshot({path:`${out}/${language}-timeout-${width}.png`});
  results.push({language,width,scenes:['source','missing-Scripture-text','timeout'],geometry,pageErrors:errors});await page.close();
 }
}finally{
 await browser.close();writeFileSync(`${out}/source-checks.json`,JSON.stringify({platform:'Linux Chromium; synthetic native failure and generated PDF; empty installed Scripture fixture; not Windows or actual MacArthur acceptance',results},null,2)+'\n');
}
console.log(`${results.length} six-language source/Scripture/timeout cases passed`);
