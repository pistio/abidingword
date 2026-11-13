// Shared actual-UI interaction for browser and native WebView2 evidence.
// Callers provide a fresh invented formatted source already open in Reading.
export async function mode(page, source, name) {
  const direct=source.getByRole('button',{name,exact:true});
  if(await direct.count())await direct.click();
  else {await source.getByRole('button',{name:/^View:/}).click();await page.getByRole('menu',{name:'View',exact:true}).getByRole('menuitem',{name,exact:true}).click();}
}
export async function action(page, source, name) {
  const direct=source.getByRole('button',{name,exact:true});
  if(await direct.count())await direct.click();
  else {await source.getByRole('button',{name:'More reader actions'}).click();await page.getByRole('menu',{name:'More reader actions',exact:true}).getByRole('menuitem',{name,exact:true}).click();}
}
export function snapshot() {
  const rect=el=>{if(!el)return null;const r=el.getBoundingClientRect();return {top:r.top,bottom:r.bottom,left:r.left,right:r.right,width:r.width,height:r.height};};
  const record=el=>({tag:el.tagName,id:el.id,testid:el.getAttribute('data-testid'),className:typeof el.className==='string'?el.className:'',rect:rect(el),scrollTop:el.scrollTop,scrollLeft:el.scrollLeft,scrollHeight:el.scrollHeight,clientHeight:el.clientHeight,scrollWidth:el.scrollWidth,clientWidth:el.clientWidth,overflow:getComputedStyle(el).overflow});
  const reader=document.querySelector('[data-testid=epub-viewer] [role=region]')??document.querySelector('[data-testid=reader-body]');
  const ancestors=[];for(let el=reader?.parentElement;el;el=el.parentElement)ancestors.push(record(el));
  const shadow=[...document.querySelectorAll('[data-testid=epub-viewer] *')].map(el=>el.shadowRoot).find(Boolean);
  const anchors=shadow?[...shadow.querySelectorAll('#p40,#p65,mark[data-mark-id]')].map(el=>({id:el.id,tag:el.tagName,markId:el.getAttribute('data-mark-id'),rect:rect(el)})):[];
  const readingParagraphs=[...document.querySelectorAll('[data-testid=reader-body] [data-segment-id]')].filter(el=>{const r=el.getBoundingClientRect();const v=reader.getBoundingClientRect();return r.bottom>v.top&&r.top<v.bottom;}).map(el=>({textPrefix:el.textContent.slice(0,60),rect:rect(el)}));
  const toolbar=document.querySelector('[data-testid=reader-header]');const appbar=document.querySelector('[data-testid=app-shell] > header');
  const visible=el=>{if(!el)return false;const r=el.getBoundingClientRect();const x=r.left+r.width/2,y=r.top+r.height/2;return r.top>=0&&r.bottom<=innerHeight&&el.contains(document.elementFromPoint(x,y));};
  return {innerWidth,innerHeight,devicePixelRatio,windowScroll:[scrollX,scrollY],visualViewport:visualViewport?{width:visualViewport.width,height:visualViewport.height,offsetTop:visualViewport.offsetTop,pageTop:visualViewport.pageTop}:null,document:record(document.documentElement),shell:record(document.querySelector('[data-testid=app-shell]')),workspace:record(document.querySelector('[data-workspace-layout]')),panes:[...document.querySelectorAll('#pane-scripture,#pane-study,#pane-sources')].map(record),reader:reader?record(reader):null,toolbar:rect(toolbar),toolbarVisible:visible(toolbar),appbar:rect(appbar),appbarVisible:visible(appbar),ancestors,anchors,readingParagraphs};
}
async function settle(page){await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));}
export function compareOriginal(before, current) {
  const errors=[];
  for(const key of ['shell','workspace'])for(const edge of ['top','bottom','left','right'])if(Math.abs(before[key].rect[edge]-current[key].rect[edge])>2)errors.push(`${key}.${edge} moved`);
  for(let i=0;i<before.panes.length;i++)for(const edge of ['top','bottom','left','right'])if(Math.abs(before.panes[i].rect[edge]-current.panes[i].rect[edge])>2)errors.push(`${before.panes[i].id}.${edge} moved`);
  for(const key of ['shell','workspace','document'])for(const offset of ['scrollTop','scrollLeft'])if(current[key][offset]!==before[key][offset])errors.push(`${key}.${offset} moved`);
  for(const old of before.ancestors){const now=current.ancestors.find(a=>a.tag===old.tag&&a.id===old.id&&a.className===old.className);if(now&&(old.scrollTop!==now.scrollTop||old.scrollLeft!==now.scrollLeft))errors.push(`Ancestor ${old.tag} ${old.id||old.className} scrolled`);}
  if(JSON.stringify(current.windowScroll)!==JSON.stringify(before.windowScroll))errors.push('Window scrolled');
  if(!current.toolbarVisible)errors.push('Reader toolbar disappeared');
  if(!current.appbarVisible)errors.push('Application toolbar disappeared');
  return {passed:errors.length===0,errors};
}
export async function exerciseOriginal(page,{record}) {
  const source=page.locator('#pane-sources');
  const entries=[];let baseline;
  const capture=async (name,target)=>{await settle(page);const value=await page.evaluate(snapshot);entries.push({name,...value});await record?.(name,value);if(baseline){const check=compareOriginal(baseline,value);if(!check.passed)throw Error(`${name}: ${check.errors.join('; ')}`);}if(target){const el=value.anchors.find(a=>target==='mark'?a.markId:a.id===target);if(!el||el.rect.bottom<=value.reader.rect.top||el.rect.top>=value.reader.rect.bottom)throw Error(`${name}: expected ${target} visible in reader`);}return value;};
  const contents=async name=>{await action(page,source,'Contents');await source.getByRole('navigation',{name:/Contents of/}).getByRole('button',{name:new RegExp(`^${name}(?:\\s|$)`)}).click();await settle(page);};
  await mode(page,source,'Reading');await source.getByTestId('reader-body').waitFor();
  await settle(page);await contents('Deep passage');
  // The actual reading pane advances, so switching mode must map a deep place.
  await source.getByTestId('reader-body').evaluate(el=>el.scrollTop+=75);
  const before=await capture('reading-deep');baseline=before;if(before.reader.scrollTop<=0)throw Error('Reading precondition must have nonzero scroll position');
  await mode(page,source,'Original');await source.getByTestId('epub-viewer').waitFor();await source.locator('#p40').waitFor();
  await capture('original-deep','p40');
  const note=source.getByRole('link',{name:'Footnote 1',exact:true});await note.focus();await page.keyboard.press('Enter');
  await capture('keyboard-footnote','p65');
  const backlink=source.getByRole('link',{name:'Return to passage',exact:true});await backlink.focus();await page.keyboard.press('Enter');
  await capture('keyboard-backlink','p40');
  await action(page,source,'Back to the previous place');await capture('history-back','p65');
  await action(page,source,'Forward to the next place');await capture('history-forward','p40');
  await contents('Footnotes');await capture('contents-footnotes','p65');
  await contents('Deep passage');await capture('contents-passage','p40');
  await action(page,source,'Marks');
  const mark=source.getByRole('button').filter({hasText:'INVENTED formatted paragraph 41'});
  await mark.first().click();await capture('highlight-reveal','mark');
  await mode(page,source,'Reading');await source.getByTestId('reader-body').waitFor();const readingReturn=await capture('reading-return');if(!readingReturn.readingParagraphs.some(p=>p.textPrefix.includes('INVENTED formatted paragraph 41.')))throw Error('Reading return lost passage 41');
  await mode(page,source,'Original');await source.getByTestId('epub-viewer').waitFor();await capture('original-return','p40');
  // Traverse the real toolbar and reader using Tab, without programmatic link focus.
  const originalButton=source.getByRole('button',{name:'Original',exact:true});
  if(await originalButton.count())await originalButton.focus();
  else await source.getByRole('button',{name:/^View:/}).focus();
  let reached=false;
  for(let n=0;n<20;n++){
    await page.keyboard.press('Tab');
    reached=await note.evaluate(el=>el.getRootNode().activeElement===el);
    if(reached)break;
  }
  if(!reached)throw Error('Tab navigation did not reach the footnote link');
  await capture('keyboard-tab','p40');
  return {before,entries};
}
