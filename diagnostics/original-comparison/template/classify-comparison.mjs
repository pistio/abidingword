// This is an evidence classifier, never an automatic visual acceptance decision.
import {existsSync,readFileSync,readdirSync,statSync} from 'node:fs';
import {join} from 'node:path';
const close=(a,b)=>Number.isFinite(a)&&Number.isFinite(b)&&Math.abs(a-b)<=4;
function nativeSamples(directory){
  const logs=join(directory,'native-logs');if(!existsSync(logs))return [];
  return readdirSync(logs).filter(f=>f.endsWith('.log')).flatMap(f=>readFileSync(join(logs,f),'utf8').split('\n').flatMap(line=>{const at=line.indexOf('Viewport sample ');if(at<0)return [];try{return [JSON.parse(line.slice(at+16))];}catch{return [{malformed:true}];}}));
}
function measurement(case_,directory){
  const errors=[], n=case_?.native?.Main,t=case_?.tauri,d=case_?.dom,i=case_?.interaction;
  if(!n||!t?.controller?.rect||!t.stableClient||!d||!i){errors.push('Missing stable Win32/COM/DOM/interaction measurement.');return errors;}
  const finite=(object,keys)=>object&&keys.every(key=>Number.isFinite(object[key]));
  const rect=object=>finite(object,['top','bottom','left','right','width','height']);
  const offsets=object=>finite(object,['scrollTop','scrollLeft','scrollHeight','clientHeight','scrollWidth','clientWidth']);
  if(!finite(i,['innerWidth','innerHeight','devicePixelRatio'])||!rect(i.shell?.rect)||!rect(i.workspace?.rect)||!rect(i.document?.rect)||!offsets(i.shell)||!offsets(i.workspace)||!offsets(i.document)||!rect(i.toolbar)||!rect(i.appbar)||!Array.isArray(i.windowScroll)||i.windowScroll.length!==2||!i.windowScroll.every(Number.isFinite)||!Array.isArray(i.ancestors)||!i.ancestors.length||!i.ancestors.every(a=>rect(a.rect)&&offsets(a))||!finite(i.visualViewport,['width','height','offsetTop','pageTop']))errors.push('Missing finite DOM/toolbar/ancestor offsets and viewport coordinates.');
  if(!finite(n,['ClientWidth','ClientHeight','ClientScreenX','ClientScreenY','Dpi'])||!finite(t.controller,['zoomFactor'])||Math.abs(t.controller.zoomFactor-1)>0.01)errors.push('Missing native dimensions or controller zoom.');
  if(t.repairEnabled!==false)errors.push('COM logger does not confirm repair disabled.');
  if(!n.Visible||n.Minimized||!n.Maximized||n.Dpi!==96)errors.push('Wrong native window state or OS DPI.');
  if(d.layoutMode!=='columns'||d.devicePixelRatio!==1||i.devicePixelRatio!==1)errors.push('Wrong three-pane layout or browser DPI.');
  const [l,top,r,b]=t.controller.rect;
  if(![l,top,r,b].every(Number.isFinite)||!close(r-l,n.ClientWidth)||!close(b-top,n.ClientHeight)||!close(d.innerWidth,n.ClientWidth)||!close(d.innerHeight,n.ClientHeight))errors.push('Native, controller and browser dimensions do not match.');
  const host=[n,...(case_.native.Descendants??[])].find(h=>h.Hwnd===t.controller.parentHwnd);
  if(!host||!finite(host,['ClientWidth','ClientHeight','ClientScreenX','ClientScreenY'])||l < -4||top < -4||!close(host.ClientScreenX+l,n.ClientScreenX)||!close(host.ClientScreenY+top,n.ClientScreenY)||r>host.ClientWidth+4||b>host.ClientHeight+4)errors.push('Missing, offset or clipped controller host.');
  if(!Array.isArray(t.client)||!Array.isArray(t.webview)||!close(t.client[0],n.ClientWidth)||!close(t.client[1],n.ClientHeight)||!close(t.webview[0],r-l)||!close(t.webview[1],b-top))errors.push('Native and Tauri samples disagree.');
  if(i.panes?.length!==3||!i.shell?.rect||!i.workspace?.rect||!i.toolbar||!i.appbar||!Array.isArray(i.ancestors))errors.push('Incomplete pane/toolbar/ancestor measurements.');
  if(!['pane-scripture','pane-study','pane-sources'].every(id=>i.panes?.some(p=>p.id===id&&rect(p.rect)&&offsets(p))))errors.push('Missing measured three-pane rectangles.');
  const png=join(directory,`${case_.name}-native-client.png`);
  if(case_.nativeScreenshotError||!existsSync(png)||statSync(png).size<100)errors.push('Native screenshot missing or failed.');
  else {const bytes=readFileSync(png);if(!bytes.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10]))||bytes.toString('ascii',12,16)!=='IHDR'||bytes.readUInt32BE(16)!==n.ClientWidth||bytes.readUInt32BE(20)!==n.ClientHeight)errors.push('Native PNG signature or client dimensions invalid.');}
  return errors;
}
export function classifyRun(report,directory,role){
  if(!report)return {classification:'SETUP_OR_MEASUREMENT_FAILURE',reproduced:false,reasons:['No report.json; inspect runner.log.']};
  const samples=nativeSamples(directory);
  const common=[];
  if(report.mode!=='observe'||report.scenario!=='original'||report.sourceFormat!=='epub')common.push('Wrong scenario, mode or source format.');
  if(samples.length===0||samples.some(s=>s.repairEnabled!==false||s.phase==='after-request'))common.push('Native logs do not establish observation-only execution.');
  const reading=report.cases?.find(c=>c.name==='reading-deep'), original=report.cases?.find(c=>c.name==='original-deep');
  common.push(...measurement(reading,directory),...measurement(original,directory));
  if(reading?.interaction?.shell?.scrollTop!==0||reading?.interaction?.workspace?.scrollTop!==0||reading?.interaction?.document?.scrollTop!==0||reading?.interaction?.windowScroll?.some(x=>x!==0))common.push('Reading baseline already has displaced outer scroll offsets.');
  if(!reading?.comparison?.passed||!reading?.interactionComparison?.passed||!reading?.interaction?.toolbarVisible||!reading?.interaction?.appbarVisible||!(reading?.interaction?.reader?.scrollTop>0))common.push('Reading baseline was not a valid visible deep-reading state.');
  if(common.length)return {classification:'SETUP_OR_MEASUREMENT_FAILURE',reproduced:false,reasons:[...new Set(common)]};
  const before=reading.interaction,after=original.interaction;
  const gaps=after.panes.map(p=>after.innerHeight-p.rect.bottom);
  const sharedBand=gaps.every(g=>g>4)&&Math.max(...gaps)-Math.min(...gaps)<=4;
  const shellShift=after.shell.scrollTop-before.shell.scrollTop;
  const shifted=sharedBand&&shellShift>4&&gaps.every(g=>close(g,shellShift))&&after.panes.every(p=>{const old=before.panes.find(q=>q.id===p.id);return old&&close(old.rect.top-p.rect.top,shellShift);});
  const sameDimensions=close(reading.native.Main.ClientHeight,original.native.Main.ClientHeight)&&close(before.innerHeight,after.innerHeight)&&close(before.shell.rect.top,after.shell.rect.top)&&close(before.shell.rect.bottom,after.shell.rect.bottom);
  const lostControls=!after.toolbarVisible&&!after.appbarVisible;
  const reproduced=shifted&&sameDimensions&&lostControls;
  if(role==='baseline'&&reproduced)return {classification:'LAYOUT_DEFECT_REPRODUCED_PENDING_SCREENSHOT_REVIEW',reproduced:true,sharedBandPhysicalPixels:gaps,shellScrollShift:shellShift,firstFailureRetained:report.cases.at(-1)?.name==='original-deep',reasons:['Requires independent review of baseline native screenshot.']};
  const incomplete=report.cases.flatMap(c=>measurement(c,directory));
  if(incomplete.length)return {classification:'SETUP_OR_MEASUREMENT_FAILURE',reproduced:false,reasons:[...new Set(incomplete)]};
  if(['reading-deep','original-deep','keyboard-footnote','keyboard-backlink','history-back','history-forward','contents-footnotes','contents-passage','highlight-reveal','reading-return','original-return','keyboard-tab'].every((name,index)=>report.cases[index]?.name===name)&&report.geometryAndFunctionPassed&&report.savedEditVerified&&report.cases.length>=12&&report.originalScrolling?.length===3&&report.originalScrolling.every(s=>s.passed)&&report.originalEditing?.valuePreserved&&report.originalEditing?.caretPreserved&&report.cases.every(c=>c.comparison?.passed&&c.interactionComparison?.passed))return {classification:role==='baseline'?'BASELINE_DID_NOT_REPRODUCE':'CORRECTED_CHECKS_PASSED_PENDING_SCREENSHOT_REVIEW',reproduced:false,reasons:['Independent native screenshot review remains required.']};
  return {classification:reproduced?'LAYOUT_DEFECT_PRESENT':'FUNCTIONAL_OR_OTHER_FAILURE',reproduced,reasons:report.failures??['Run did not complete all required checks.']};
}
