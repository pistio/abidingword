// Invented memory profile; Linux Chromium geometry, not Windows WebView2 acceptance.
import {createRequire} from 'node:module';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const {chromium}=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE??'playwright');
const out=process.env.SCREENSHOT_OUT??'docs/implementation/owner-followup/images';mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
const results=[];
try {
 for(const language of ['en','de','ko','fr','es','pt'])for(const before of [true,false])for(const width of [900,1440]){
  const copy=JSON.parse(readFileSync(`src/i18n/locales/${language}/studies.json`,'utf8'));
  const page=await browser.newPage({viewport:{width,height:1000}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`http://localhost:${before?1431:1430}/tests/screenshots/index.html?assistant=off&a4Editors=1&lang=${language}&sourcesOpen=1`);
  const study=page.locator('#pane-study');
  await study.getByRole('button',{name:/Organisatorisches Projekt/}).first().click();
  await study.getByRole('button',{name:/Römer 8/}).first().click();
  const core=study.getByRole('heading',{name:copy.builtin.templates['pozetek-writing-study'].sections.core.title,exact:true}).locator('xpath=ancestor::section[1]');
  await core.getByRole('textbox').waitFor();await core.scrollIntoViewIfNeeded();
  const paragraphs=await core.locator('[data-part-hints] p').count();
  assert.equal(await core.getByRole('textbox').count(),1);
  if(!before){assert.equal(paragraphs,3);assert.equal(await core.getByRole('textbox').inputValue(),'');}
  assert.ok(await study.evaluate(e=>e.scrollWidth<=e.clientWidth+2),'study horizontal overflow');
  await core.screenshot({path:`${out}/${language}-${before?'before':'after'}-core-${width}.png`});
  assert.deepEqual(errors,[]);results.push({language,before,width,hintParagraphs:paragraphs,fields:1,pageErrors:errors});await page.close();
 }
 writeFileSync(`${out}/browser-checks.json`,JSON.stringify({platform:'Linux Chromium; invented profile; not Windows acceptance',results},null,2)+'\n');console.log(JSON.stringify(results));
}finally{await browser.close();}
