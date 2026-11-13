import {createRequire} from 'node:module';import {mkdirSync,writeFileSync} from 'node:fs';
import {exerciseOriginal} from './original-workspace-flow.mjs';
const {chromium}=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE??'playwright');
const out=process.env.ORIGINAL_EVIDENCE_OUT??'work/original-workspace';mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH??'/usr/bin/chromium',args:['--no-sandbox']});
const results=[];
try {for(const kind of ['epub','docx'])for(const [width,height,scale]of JSON.parse(process.env.ORIGINAL_CASES??'[[1920,1009,1],[1440,800,1],[1440,800,1.5],[1440,800,2]]')){
 const page=await browser.newPage({viewport:{width,height},deviceScaleFactor:scale});page.setDefaultTimeout(10000);console.log('start',kind,width,height,scale);const errors=[];page.on('pageerror',e=>errors.push(e.message));await page.routeWebSocket('**',s=>s.close());
 await page.goto(`${process.env.ORIGINAL_URL??'http://localhost:1432'}/tests/screenshots/index.html?empty=1&assistant=off&viewportReview=1&a4Editors=1&sourcesOpen=1&lang=en&originalReview=${kind}`);
 const study=page.locator('#pane-study');await study.getByRole('button',{name:/INVENTED Viewport Project/}).first().click();await study.getByRole('button',{name:/INVENTED Viewport Study/}).first().click();
 const area=study.locator('section textarea').first();await area.fill('INVENTED writing retained across Original navigation.');
 await page.locator('#pane-sources [data-source-result]').filter({hasText:'INVENTED Original navigation'}).first().click();await page.getByTestId('reader-body').waitFor();
 const id=`${kind}-${width}-${height}-${scale}`;let run;
 try {run=await exerciseOriginal(page,{record:async(name,value)=>{writeFileSync(`${out}/${id}-${name}.json`,JSON.stringify(value,null,2)+'\n');await page.screenshot({path:`${out}/${id}-${name}.png`});}});}catch(error){run={error:String(error.stack)};console.log(id,run.error);await page.screenshot({path:`${out}/${id}-failure.png`});writeFileSync(`${out}/${id}-failure-dom.txt`,await page.locator('body').innerText());}
 const value=await area.inputValue();results.push({id,...run,writingPreserved:value==='INVENTED writing retained across Original navigation.',errors});await page.close();
 }}finally{await browser.close();}
writeFileSync(`${out}/results.json`,JSON.stringify({platform:'Linux Chromium; real React with memory API formatted EPUB/DOCX; not native WebView2',results},null,2)+'\n');

if(results.some(r=>r.error||!r.writingPreserved||r.errors.length))process.exitCode=1;
