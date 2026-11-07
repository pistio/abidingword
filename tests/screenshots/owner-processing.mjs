// Synthetic browser presentation only; native build behavior is tested in Rust.
// Chromium geometry is not Windows WebView2 acceptance.
import { createRequire } from 'node:module';
import {mkdirSync,writeFileSync} from 'node:fs';
import assert from 'node:assert/strict';
const require=createRequire(import.meta.url);
const {chromium}=require(process.env.PLAYWRIGHT_MODULE??'playwright');
const out=process.env.SCREENSHOT_OUT??'docs/implementation/windows-owner-corrections/images';
mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH??'/usr/bin/chromium',headless:true,args:['--no-sandbox']});
const results=[];
try {
 for (const [before,width] of [[true,900],[false,640],[false,900],[false,1440]]) {
  const page=await browser.newPage({viewport:{width,height:900}});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  const base=before?'http://localhost:1431':'http://localhost:1430';
  await page.goto(`${base}/tests/screenshots/index.html?assistant=off&sourcesOpen=1&a3PdfReview=1&passage=ACT.6${before?'':'&pdfProcessingReview=1'}`);
  const source=page.locator('#pane-sources');
  await source.locator('[data-source-result]').filter({hasText:'Invented handbook'}).first().click();
  const tools=source.getByRole('toolbar',{name:'Reader tools'});
  await tools.getByRole('button',{name:'More reader actions'}).click();
  await tools.getByRole('menuitem',{name:'Document structure',exact:true}).click();
  const dialog=page.getByRole('dialog',{name:'Document structure',exact:true});
  await dialog.waitFor();
  if(!before) {
   await dialog.getByText('Read PDF pages · 12 s').waitFor();
   assert.equal(await dialog.getByRole('progressbar').getAttribute('value'),'125');
   assert.equal(await dialog.getByRole('button',{name:'Rebuild structure',exact:true}).isDisabled(),true);
   assert.ok(await dialog.getByRole('button',{name:'Close',exact:true}).last().isEnabled());
   assert.ok(await dialog.evaluate(el=>el.scrollWidth<=el.clientWidth+2),'dialog overflow');
  }
  await page.screenshot({path:`${out}/35-${before?'before':'progress'}-${width}.png`});
  if(!before) {
   await dialog.getByRole('button',{name:'Cancel',exact:true}).click();
   await dialog.getByText(/Analysis cancelled/).waitFor();
   assert.ok(await dialog.getByRole('button',{name:'Rebuild structure',exact:true}).isEnabled());
   if(width===900) await page.screenshot({path:`${out}/35-cancelled-900.png`});
  }
  await dialog.getByRole('button',{name:'Close',exact:true}).last().click();
  await page.getByRole('dialog',{name:'Document structure',exact:true}).waitFor({state:'hidden'});
  assert.ok(await source.getByTestId('reader-body').isVisible());
  assert.deepEqual(errors,[]);
  results.push({before,width,progress:before?'unavailable':'visible, cancel retained, retry enabled',pageErrors:errors});
  await page.close();
 }
 writeFileSync(`${out}/35-browser-checks.json`,JSON.stringify({platform:'Linux Chromium; synthetic job state, invented PDF; not WebView2 acceptance',results},null,2)+'\n');
 console.log(JSON.stringify(results));
} finally {await browser.close();}
