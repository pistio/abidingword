// Final editorial context samples: invented profile; Linux Chromium, not Windows acceptance.
import {createRequire} from 'node:module';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const {chromium}=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE??'playwright');
const out='docs/i18n/issue-42/images/full-review';mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',args:['--no-sandbox']});
const results=[];
const fill=(s,vars)=>s.replace(/{{(\w+)}}/g,(_,k)=>vars[k]);
try {
 for(const language of ['en','de','ko','fr','es','pt'])for(const width of [900,1440]){
  const copy=Object.fromEntries(['studies','app','settings','library','scripture'].map(ns=>[ns,JSON.parse(readFileSync(`src/i18n/locales/${language}/${ns}.json`,'utf8'))]));
  const page=await browser.newPage({viewport:{width,height:1000},deviceScaleFactor:width===900?1.5:1});
  await page.routeWebSocket('**',socket=>socket.close());
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`http://localhost:1432/tests/screenshots/index.html?empty=1&assistant=off&a4Editors=1&sourcesOpen=1&lang=${language}`);
  const study=page.locator('#pane-study');
  await study.getByRole('button',{name:/Organisatorisches Projekt/}).first().click();
  await study.getByRole('button',{name:/Römer 8/}).first().click();
  await study.locator('textarea').first().waitFor();
  async function capture(scene,locator=page){
   const geometry=await page.locator('#pane-scripture,#pane-study,#pane-sources').evaluateAll(nodes=>nodes.map(n=>({id:n.id,width:n.clientWidth,scrollWidth:n.scrollWidth})));
   assert.ok(geometry.every(n=>n.scrollWidth<=n.width+2),JSON.stringify(geometry));
   assert.deepEqual(errors,[]);
   await locator.screenshot({path:`${out}/${language}-${scene}-${width}.png`});
   results.push({language,width,scale:width===900?1.5:1,scene,geometry,pageErrors:[...errors]});
  }
  await capture('study');
  const title=copy.studies.builtin.templates['pozetek-writing-study'].sections.time.title;
  const section=study.getByRole('heading',{name:title,exact:true}).locator('xpath=ancestor::section[1]');
  await section.getByRole('button',{name:fill(copy.studies.sections.options,{title}),exact:true}).click();
  const remove=section.getByRole('button',{name:fill(copy.studies.sections.delete,{title}),exact:true});await remove.click();
  const modal=page.getByRole('alertdialog');await modal.waitFor();assert.equal(await modal.getAttribute('aria-modal'),'true');
  await capture('delete',modal);await page.keyboard.press('Escape');await modal.waitFor({state:'hidden'});
  assert.ok(await remove.evaluate(e=>e===document.activeElement),'delete focus return');
  await study.getByRole('button',{name:copy.studies.view.teaching,exact:true}).first().click();
  await study.getByRole('heading',{name:copy.studies.teaching.empty.title,exact:true}).waitFor();
  await capture('teaching');
  await page.getByRole('button',{name:copy.app.toolbar.settings,exact:true}).click();
  await page.getByRole('heading',{name:copy.settings.title,exact:true}).waitFor();
  await capture('settings');
  const assistant=page.locator('#settings-section-assistant');
  await assistant.getByRole('button').first().click();
  await assistant.scrollIntoViewIfNeeded();await capture('ai-settings',assistant);
  const data=page.locator('#settings-section-data');
  await data.getByRole('button').first().click();
  await data.scrollIntoViewIfNeeded();await capture('backup-settings',data);
  await page.close();
 }
}finally{
 await browser.close();
 writeFileSync(`${out}/checks.json`,JSON.stringify({platform:'Linux Chromium; invented empty profile; 900px at 150% device scale /1440px at100%; not actual Windows, native OS dialog chrome or actual-file acceptance',results},null,2)+'\n');
}
console.log(`${results.length} editorial contexts passed`);
