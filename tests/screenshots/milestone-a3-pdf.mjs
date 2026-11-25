// Actual Rust-generated synthetic metadata + real PDF.js, complete workspace.
import { createRequire } from "node:module";
import { mkdirSync, writeFileSync, readFileSync } from "node:fs";
import assert from "node:assert/strict";
const require=createRequire(import.meta.url);
const { chromium }=require(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const base=process.env.SCREENSHOT_URL ?? "http://localhost:1430/tests/screenshots/index.html";
const out=process.env.SCREENSHOT_OUT ?? "docs/implementation/milestone-a3-pdf/images";
mkdirSync(out,{recursive:true});
const browser=await chromium.launch({executablePath:process.env.CHROMIUM_PATH ?? "/usr/bin/chromium",headless:true,args:["--no-sandbox"]});
const results=[];
try {
  for(const [width,scale] of (process.env.SCREENSHOT_ONLY_RAW ? [] : [[640,1],[900,1],[1100,1],[1440,1],[900,1.25],[1100,1.5],[1440,2]])) {
    const page=await browser.newPage({viewport:{width,height:1000},deviceScaleFactor:scale});
    const errors=[];page.on("pageerror",e=>errors.push(e.message));
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3PdfReview=1`);
    const sources=page.locator("#pane-sources");
    const result=sources.locator('[data-source-result]').filter({hasText:"Invented commentary"}).first();
    await result.waitFor();
    assert.ok((await result.textContent()).includes("Kapitel 6"));
    assert.ok((await result.textContent()).includes("Embedded outline"));
    await page.screenshot({fullPage:true,path:`${out}/sources-${width}-${scale}.png`});
    await result.click();
    const body=sources.getByTestId("reader-body");await body.waitFor();
    const heading=body.getByRole("heading",{name:"Kapitel 6",exact:true});await heading.waitFor();
    assert.ok(await heading.isVisible());
    assert.ok((await body.textContent()).includes("PDF page 2"));
    await page.screenshot({fullPage:true,path:`${out}/reading-${width}-${scale}.png`});
    await sources.getByRole("button",{name:"Original",exact:true}).click();
    await sources.getByTestId("pdf-viewer").waitFor();
    await page.waitForTimeout(150);
    assert.equal(await sources.getByRole("spinbutton",{name:"PDF page",exact:true}).inputValue(),"2","Direct Reading-to-Original retains the opened section");
    await sources.getByRole("button",{name:"Reading",exact:true}).click();
    await sources.getByRole("button",{name:"Contents",exact:true}).click();
    const contents=sources.getByTestId("contents-panel");await contents.waitFor();
    await contents.getByRole("button",{name:/^Kapitel 6/}).click();
    await sources.getByRole("button",{name:"Original",exact:true}).click();
    await sources.locator('[data-page="2"] canvas').waitFor();
    assert.equal(await sources.getByRole("spinbutton",{name:"PDF page",exact:true}).inputValue(),"2");
    await page.screenshot({fullPage:true,path:`${out}/original-${width}-${scale}.png`});
    await sources.getByRole("button",{name:"Contents",exact:true}).click();
    await contents.getByRole("button",{name:/^Kapitel 7/}).click();
    await sources.locator('[data-page="3"] canvas').waitFor();
    await page.waitForFunction(()=>document.querySelector('#pane-sources input[type="number"]')?.value==="3");
    await sources.getByRole("button",{name:"Reading",exact:true}).click();
    await body.getByRole("heading",{name:"Kapitel 7",exact:true}).waitFor();
    await sources.getByRole("button",{name:"Document structure",exact:true}).click();
    const panel=page.getByRole("dialog",{name:"Document structure",exact:true});await panel.waitFor();
    assert.ok((await panel.textContent()).includes("not a judgement of commentary relevance"));
    await page.screenshot({fullPage:true,path:`${out}/structure-${width}-${scale}.png`});
    await panel.getByRole("button",{name:"Close",exact:true}).click();
    const geometry=await page.locator("#pane-scripture,#pane-study,#pane-sources").evaluateAll(nodes=>nodes.map(n=>({id:n.id,width:n.clientWidth,scrollWidth:n.scrollWidth})));
    assert.ok(geometry.every(n=>n.scrollWidth<=n.width+2),JSON.stringify(geometry));
    assert.deepEqual(errors,[]);
    results.push({width,scale,status:"passed",geometry});await page.close();
  }
  for(const lang of (process.env.SCREENSHOT_ONLY_RAW ? [] : ["de","ko"])) {
    const labels=JSON.parse(readFileSync(new URL(`../../src/i18n/locales/${lang}/library.json`,import.meta.url),"utf8"));
    const page=await browser.newPage({viewport:{width:900,height:1000}});
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3PdfReview=1&lang=${lang}`);
    const sources=page.locator("#pane-sources");
    await sources.locator('[data-source-result]').filter({hasText:"Invented commentary"}).first().click();
    await sources.getByRole("heading",{name:"Kapitel 6",exact:true}).waitFor();
    await sources.getByRole("button",{name:labels.structure.title,exact:true}).click();
    await page.getByRole("dialog",{name:labels.structure.title,exact:true}).waitFor();
    await page.screenshot({fullPage:true,path:`${out}/structure-${lang}-900.png`});
    results.push({lang,width:900,status:"passed"});await page.close();
  }
  if(process.env.BEFORE_URL && !process.env.SCREENSHOT_ONLY_RAW) for(const width of [900,1440]) {
    const page=await browser.newPage({viewport:{width,height:1000}});
    await page.goto(`${process.env.BEFORE_URL}?assistant=off&sourcesOpen=1&a3PdfReview=1`);
    const sources=page.locator("#pane-sources");
    await sources.getByRole("button",{name:"Invented commentary",exact:true}).click();
    await sources.getByRole("spinbutton",{name:"PDF page",exact:true}).fill("2");
    await sources.getByRole("button",{name:"Go to page",exact:true}).click();
    await sources.getByTestId("reader-body").getByText(/SYNTHETIC COMMENTARY/).first().waitFor();
    assert.equal(await sources.getByRole("heading",{name:"Kapitel 6",exact:true}).count(),0);
    await page.screenshot({fullPage:true,path:`${out}/before-reading-${width}.png`});
    results.push({before:"5fab020012d0db42899dd7ff0b3d1b47f249fc04",width,status:"plain Reading captured"});await page.close();
  }
  {
    const page=await browser.newPage({viewport:{width:900,height:1000}});
    await page.goto(`${base}?assistant=off&sourcesOpen=1&a3PdfReview=1`);
    const sources=page.locator("#pane-sources");
    await sources.locator('[data-source-result]').filter({hasText:"Invented commentary"}).first().click();
    await sources.getByRole("button",{name:"Document structure",exact:true}).click();
    const panel=page.getByRole("dialog",{name:"Document structure",exact:true});
    await panel.getByText("Original PDF bookmarks",{exact:true}).click();
    const bookmark=panel.getByRole("button",{name:/Apostelgeschichte › Kapitel 6.*PDF page 2/});
    await bookmark.waitFor();
    await page.screenshot({fullPage:true,path:`${out}/original-bookmarks-900.png`});
    await bookmark.click();
    await page.waitForFunction(()=>document.querySelector('#pane-sources input[type="number"]')?.value==="2");
    await sources.getByRole("button",{name:"Contents",exact:true}).click();
    await sources.getByTestId("contents-panel").getByRole("button",{name:/^Kapitel 6/}).waitFor();
    results.push({scenario:"separate original PDF page bookmarks retain corrected Contents",width:900,status:"passed"});await page.close();
  }
  writeFileSync(`${out}/${process.env.SCREENSHOT_ONLY_RAW ? "raw-bookmarks-check" : "checks"}.json`,JSON.stringify(results,null,2)+"\n");
} finally {await browser.close();}
