// Native-style wording in real React UI; synthetic API failures, not Windows acceptance.
import {createRequire} from 'node:module';
import {readFileSync,mkdirSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const {chromium}=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE??'playwright');
const out='docs/i18n/issue-42/images';mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',args:['--no-sandbox']});const results=[];
try{
 for(const language of ['en','de','ko','fr','es','pt'])for(const before of [true,false]){
  const copy=JSON.parse(readFileSync(`src/i18n/locales/${language}/library.json`,'utf8'));
  const page=await browser.newPage({viewport:{width:900,height:1000}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`http://localhost:${before?1431:1430}/tests/screenshots/index.html?assistant=off&sourcesOpen=1&a3PdfReview=1&lang=${language}&passage=ACT.6`);
  await page.waitForFunction(()=>Boolean(window.api));
  await page.evaluate(()=>{
   const api=window.api;
   api.libraryPdfStructureProgress=async()=>({status:'failed',stage:'index',done:448,total:1689,elapsedMs:308917,timings:[{stage:'index',milliseconds:304681}],headingCount:null,indexedLinks:null,errorCode:'library.structure.timeout',canCancel:false});
   const original=api.libraryPdfStructure;
   api.libraryPdfStructure=async id=>({...await original(id),headings:[],analysis:2,rebuildRecommended:true});
   api.libraryPdfStructureBuild=async()=>{throw {kind:'invalidInput',code:'library.structure.timeout',message:'Heading analysis stopped. Reopen Chapters and headings for its stage and retry. Previous metadata remains available.'};};
  });
  const source=page.locator('#pane-sources');
  await source.locator('[data-source-result]').filter({hasText:'Invented handbook'}).first().click();
  await source.getByRole('toolbar',{name:copy.toolbar.label}).getByRole('button',{name:copy.toolbar.more}).click();
  await page.getByRole('menu').getByRole('menuitem',{name:copy.structure.title,exact:true}).click();
  const dialog=page.getByRole('dialog',{name:copy.structure.title,exact:true});await dialog.waitFor();
  await dialog.getByRole('button',{name:copy.structure.rebuild,exact:true}).click();
  if(!before){
   await dialog.getByText(copy.structure.timeout,{exact:true}).first().waitFor();
   assert.ok(!(await dialog.innerText()).includes('Heading analysis stopped.'),'English native fallback leaked');
   assert.ok(await dialog.evaluate(el=>el.scrollWidth<=el.clientWidth+2),'dialog horizontal overflow');
  }
  await dialog.screenshot({path:`${out}/${language}-${before?'before':'after'}-timeout-900.png`});assert.deepEqual(errors,[]);
  results.push({language,before,width:900,scenario:'synthetic native timeout with legacy empty metadata',pageErrors:errors});await page.close();
 }
 writeFileSync(`${out}/browser-checks.json`,JSON.stringify({platform:'Linux Chromium; synthetic failure; not Windows WebView2 or actual-file acceptance',results},null,2)+'\n');console.log(JSON.stringify(results));
}finally{await browser.close();}
