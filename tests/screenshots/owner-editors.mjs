// Real editor UI, invented memory profile. Linux Chromium is not Windows acceptance.
import {createRequire} from 'node:module';import {writeFileSync} from 'node:fs';import assert from 'node:assert/strict';
const {chromium}=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE??'playwright');
const browser=await chromium.launch({executablePath:'/usr/bin/chromium',args:['--no-sandbox']});const results=[];
const out='docs/implementation/windows-owner-corrections/images';
try {
 for(const [before,width,legacy] of [[true,1440,false],[false,1440,false],[false,900,false],[false,1440,true]]){
  console.log({before,width,legacy});const page=await browser.newPage({viewport:{width,height:1000}});const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`http://localhost:${before?1431:1430}/tests/screenshots/index.html?assistant=off&a4Editors=1&lang=de&sourcesOpen=1${legacy?'&pozetekLegacy=1':''}`);
  const study=page.locator('#pane-study');await study.getByRole('button',{name:/Organisatorisches Projekt/}).first().click();await study.getByRole('button',{name:/Römer 8/}).first().click();
  const field=study.locator('section textarea').first();await field.waitFor();
  await page.screenshot({path:`${out}/33-${before?'before':'after'}-${width}${legacy?'-legacy':''}.png`});
  if(before){assert.equal(await field.getAttribute('rows'),'10');results.push({before,width,rows:10});await page.close();continue;}
  assert.equal(await field.getAttribute('rows'),'5');assert.equal(await field.evaluate(el=>getComputedStyle(el).resize),'vertical');
  const rect=await field.boundingBox();await page.mouse.move(rect.x+rect.width-4,rect.y+rect.height-4);await page.mouse.down();await page.mouse.move(rect.x+rect.width-4,rect.y+rect.height+80,{steps:8});await page.mouse.up();
  const grown=(await field.boundingBox()).height;assert.ok(grown>rect.height+40,'manual resize did not grow');await field.press('End');await field.pressSequentially(' Ergänzung');await field.blur();
  assert.equal((await field.boundingBox()).height,grown,'typing reset explicit resize');
  if(legacy){
   const original=await page.evaluate(()=>window.api.state.blocks.map(b=>({id:b.id,content:b.content,data:b.data})));
   await study.getByRole('button',{name:'POZETEK-Teile prüfen',exact:true}).click();const dialog=page.getByRole('dialog',{name:'POZETEK-Teile prüfen',exact:true});await dialog.waitFor();
   assert.match(await dialog.innerText(),/time \(2\)/);await page.screenshot({path:`${out}/33-repair-preview-1440.png`});
   await dialog.getByRole('button',{name:'Fehlende Teile ergänzen und ordnen',exact:true}).click();await study.getByRole('button',{name:'POZETEK-Reparatur rückgängig',exact:true}).waitFor();
   const after=await page.evaluate(()=>window.api.state.blocks.map(b=>({id:b.id,content:b.content,data:b.data})));
   for(const block of original)assert.deepEqual(after.find(b=>b.id===block.id),block);
   await study.getByRole('button',{name:'POZETEK-Reparatur rückgängig',exact:true}).click();
  }else{
   const time=study.getByRole('heading',{name:'Z — Zeit',exact:true}).locator('xpath=ancestor::section[1]');await time.getByRole('button',{name:'Hinweise ausblenden',exact:true}).click();
   const options=time.getByRole('button',{name:'Optionen für Teil „Z — Zeit“',exact:true});await options.click();
   assert.equal(await time.getByRole('combobox').count(),0,'ordinary phase control remains');
   const remove=time.getByRole('button',{name:'Teil „Z — Zeit“ löschen',exact:true});await remove.click();
   const modal=page.getByRole('alertdialog');await modal.waitFor();assert.equal(await modal.getAttribute('aria-modal'),'true');
   await page.screenshot({path:`${out}/33-delete-modal-${width}.png`});await page.keyboard.press('Escape');await modal.waitFor({state:'hidden'});assert.ok(await remove.evaluate(el=>el===document.activeElement),'delete focus did not return');
  }
  assert.deepEqual(errors,[]);results.push({before:false,width,legacy,rows:5,initialHeight:rect.height,resizedHeight:grown,pageErrors:errors});await page.close();
 }
 writeFileSync(`${out}/33-browser-checks.json`,JSON.stringify({platform:'Linux Chromium; invented profile; not Windows acceptance',results},null,2)+'\n');console.log(JSON.stringify(results));
}finally{await browser.close();}
