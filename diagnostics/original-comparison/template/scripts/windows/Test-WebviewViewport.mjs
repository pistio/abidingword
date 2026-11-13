// Actual Windows executable + WebView2 CDP + independent Win32 collector.
// No installer, workflow dispatch, production profile or app permissions added.
import assert from 'node:assert/strict';
import { parseArgs } from 'node:util';
import { spawn, execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createServer } from 'node:net';
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync, mkdirSync, readdirSync } from 'node:fs';
import { resolve, join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { setTimeout as delay } from 'node:timers/promises';
import { release, version } from 'node:os';
import { domSnapshot, compareGeometry } from './viewport-evidence.mjs';
import { originalEpub, originalTitle } from './original-epub-fixture.mjs';
import { verifyPackage } from '../../package-integrity.mjs';
const packageRoot = fileURLToPath(new URL('../../', import.meta.url));
const packageManifest = verifyPackage(packageRoot);

const { values: args } = parseArgs({ options: {
  exe: { type: 'string' }, 'exe-sha256': { type: 'string' }, 'source-sha': { type: 'string' },
  scenario: { type: 'string', default: 'resize' },
  out: { type: 'string' }, mode: { type: 'string' }, 'expected-dpi': { type: 'string' },
  port: { type: 'string', default: '9227' }, powershell: { type: 'string', default: 'pwsh.exe' }, help: { type: 'boolean' },
} });
if (args.help) {
  console.log('Windows only: node scripts/windows/Test-WebviewViewport.mjs --exe PATH --exe-sha256 SHA256 --source-sha GIT_SHA --mode observe|repair --expected-dpi 96|120|144|192 --out NEW_DIRECTORY [--scenario original|resize] [--port 9227]');
  process.exit(0);
}
assert.equal(process.platform, 'win32', 'Native acceptance requires an interactive Windows desktop; Linux/Chromium is not a substitute.');
assert.ok(args.exe && args.out && /^[a-f0-9]{64}$/i.test(args['exe-sha256'] ?? '') && /^[a-f0-9]{40}$/i.test(args['source-sha'] ?? ''), 'Pin executable hash, source SHA and new output directory.');
assert.ok(['observe', 'repair'].includes(args.mode), 'Choose explicit observation or measured correction.');
assert.ok(['original', 'resize'].includes(args.scenario), 'Choose the exact Original interaction or historical resize scenario.');
if (args.scenario === 'original') assert.equal(args.mode, 'observe', 'Original acceptance observes native dimensions without the speculative size repair.');
const expectedDpi = Number(args['expected-dpi']);
assert.ok([96, 120, 144, 192].includes(expectedDpi), 'Use actual OS display scaling, not browser emulation.');
const exe = resolve(args.exe), out = resolve(args.out), port = Number(args.port);
assert.ok(Number.isInteger(port) && port > 1024 && port < 65536);
assert.equal(createHash('sha256').update(readFileSync(exe)).digest('hex'), args['exe-sha256'].toLowerCase(), 'Wrong executable.');
assert.ok(Object.values(packageManifest.builds).some(build => build.sourceSha === args['source-sha'] && build.sha256 === args['exe-sha256'].toLowerCase() && resolve(packageRoot, build.executable) === exe), 'Executable/source pair is not in the verified comparison manifest.');
execFileSync(args.powershell, ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-Command', 'if ($PSVersionTable.PSVersion -lt [version]"7.2") { exit 1 }']);
// Refuse a port already used by any other browser/profile.
await new Promise((accept, reject) => {
  const server = createServer();server.once('error', reject);server.listen(port, '127.0.0.1', () => server.close(accept));
});
mkdirSync(dirname(out), { recursive: true });mkdirSync(out); // must be NEW: never reuse owner data
const profile = join(out, 'invented-profile');mkdirSync(profile);
const mark = 'INVENTED VIEWPORT DATA — NOT SCRIPTURE OR OWNER WRITING';
const counts = [51,25,36,54,47,71,53,59,41,42,57,50,38,31,27,33,26,40,42,31,25];
const fixture = ['\\id JHN', '\\ide UTF-8', `\\h ${mark}`];
counts.forEach((n, c) => { fixture.push(`\\c ${c+1}`);for(let v=1;v<=n;v++)fixture.push(`\\v ${v} ${(mark + ` row ${c+1}:${v}. `).repeat(5)}`); });
const bibleFile = join(out, 'invented.usfm'), sourceFile = join(out, args.scenario === 'original' ? 'invented-original.epub' : 'invented.md');
writeFileSync(bibleFile, fixture.join('\n'));
writeFileSync(sourceFile, `# INVENTED Viewport Source\n\n${Array.from({length:80},(_,i)=>`Invented paragraph ${i+1}. John 15:1. ${'Geometry test data only. '.repeat(15)}`).join('\n\n')}`);
if (args.scenario === 'original') writeFileSync(sourceFile, originalEpub());
const sourceTitle = args.scenario === 'original' ? originalTitle : 'INVENTED Viewport Source';
const protocolSourceSha = packageManifest.protocolSourceSha;
const child = spawn(exe, [], { windowsHide: false, stdio: 'ignore', env: {
  ...process.env, ABIDING_WORD_DATA_DIR: profile,
  WEBVIEW2_USER_DATA_FOLDER: join(profile, 'webview2'),
  WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port} --remote-debugging-address=127.0.0.1`,
  ABIDING_WORD_VIEWPORT_MODE: args.mode, ABIDING_WORD_VIEWPORT_TRACE: '1',
} });
const report = { protocolSourceSha, sourceSha: args['source-sha'], executableSha256: args['exe-sha256'], mode: args.mode, scenario: args.scenario, expectedDpi, platform: process.platform, architecture: process.arch, osRelease: release(), osVersion: version(), cases: [], stage: 'spawn', nativeAcceptance: 'NOT AUTOMATIC: requires independent native screenshot review; this launcher performs no build or installation', failures: [] };
let browser;
let diagnosticLogDirectory;
let startupError;child.once('error', error => { startupError=error; });
const collector = fileURLToPath(new URL('./Measure-WebviewGeometry.ps1', import.meta.url));
function native(name, action = 'observe', size, screenshot = false) {
  const output = join(out, `${name}-native.json`);
  const command = ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', collector, '-ProcessId', String(child.pid), '-OutputPath', output, '-Action', action];
  if(size)command.push('-Width', String(size[0]), '-Height', String(size[1]));
  if(screenshot)command.push('-ScreenshotPath', join(out, `${name}-native-client.png`));
  execFileSync(args.powershell, command, { windowsHide: true, stdio: 'pipe' });
  return JSON.parse(readFileSync(output, 'utf8').replace(/^\uFEFF/, ''));
}
function latestSample(logDirectory) {
  const samples = readdirSync(logDirectory).filter(f=>f.endsWith('.log')).flatMap(f=>readFileSync(join(logDirectory,f),'utf8').split('\n').flatMap(line=>{
    const start=line.indexOf('Viewport sample ');if(start<0)return [];
    try{return [JSON.parse(line.slice(start+'Viewport sample '.length))];}catch{return [];}
  }));
  // Rotated log directory iteration is not chronologically ordered.
  // Select the newest sequence explicitly, not the last filename scanned.
  return samples.filter(s => Number.isSafeInteger(s?.sequence))
    .reduce((latest, sample) => !latest || sample.sequence > latest.sequence ? sample : latest, null);
}
async function scrollAll(page) {
  return page.evaluate(async()=>{
    const results=[];
    for(const name of ['scripture','study','sources']) {
      const pane=document.querySelector(`#pane-${name}`);
      const node=[...pane.querySelectorAll('*')].find(el=>/^(auto|scroll)$/.test(getComputedStyle(el).overflowY)&&el.clientHeight>50&&el.scrollHeight>el.clientHeight+10);
      if(!node){results.push({pane:name,passed:false,reason:'No overflowing real pane scroller'});continue;}
      const previous=node.scrollTop;node.scrollTop=node.scrollHeight;
      await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
      const passed=Math.abs(node.scrollHeight-node.clientHeight-node.scrollTop)<=2;
      results.push({pane:name,passed,scrollTop:node.scrollTop,scrollHeight:node.scrollHeight,clientHeight:node.clientHeight});node.scrollTop=previous;
    }
    return results;
  });
}
try {
  for(let i=0;i<100;i++) {
    if(startupError)throw startupError;
    if(child.exitCode!==null)throw Error('Test executable exited before CDP became available.');
    try { if((await fetch(`http://127.0.0.1:${port}/json/version`)).ok)break; }catch{}
    if(i===99)throw Error('No WebView2 CDP endpoint; use a plain local build on an interactive Windows desktop.');
    await delay(200);
  }
  report.stage='webview2-connect';
  const {chromium}=createRequire(import.meta.url)(process.env.PLAYWRIGHT_MODULE??'playwright');
  browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`);report.webview2Version=browser.version();
  const page=browser.contexts()[0].pages()[0];assert.ok(page, 'No application WebView2 target');
  await page.locator('[data-testid="app-shell"]').waitFor();
  report.stage='app-info';
  const info=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('app_info'));
  assert.equal(resolve(info.dataDirectory).toLowerCase(), profile.toLowerCase(), 'App is not using the disposable profile; stop before seeding.');
  assert.equal(info.updatesEnabled,false,'Require a plain build without updater.');assert.equal(info.schemaVersion,27);
  assert.equal(resolve(info.logDirectory).toLowerCase(),join(profile,'logs').toLowerCase(),'Logs must belong to the disposable profile.');
  diagnosticLogDirectory=info.logDirectory;
  report.stage='seed-disposable-profile';
  const seeded=await page.evaluate(async({bibleFile,sourceFile,mark,sourceTitle,scenario})=>{
    const invoke=window.__TAURI_INTERNALS__.invoke;
    const workspace=await invoke('workspace_get_default');
    const project=await invoke('project_create',{input:{workspaceId:workspace.id,title:'INVENTED Viewport Project'}});
    const detail=await invoke('study_create',{input:{projectId:project.id,title:'INVENTED Viewport Study',templateKey:'pozetek-writing-study',primaryPassage:'JHN.15'}});
    const body=(`${mark}. Writing and scrolling must survive native resizing.\n`).repeat(50);
    const section=detail.sections.find(s=>s.blocks.length);const block=section.blocks[0];
    await invoke('study_block_update',{input:{id:block.id,content:body}});
    // All Study sections must be populated: an empty/short Study misses the
    // real overflow condition behind the Reading -> Original regression.
    if(scenario==='original')for(const part of detail.sections)for(const writing of part.blocks) {
      if(writing.id!==block.id)await invoke('study_block_update',{input:{id:writing.id,content:body}});
    }
    const preview=await invoke('bible_import_preview',{input:{paths:[bibleFile]}});
    assertNoImportError(preview);
    function assertNoImportError(p){if(!p.canImport)throw Error('Invented USFM fixture was rejected; do not bypass import validation.');}
    const bible=await invoke('bible_import_commit',{input:{previewId:preview.previewId,name:mark,shortName:'INVENTED',language:'English',languageCode:'en'}});
    const source=await invoke('library_import_preview',{path:sourceFile});
    if(!source.canImport)throw Error('Invented source fixture rejected.');
    const imported=await invoke('library_import_commit',{input:{previewId:source.previewId,title:sourceTitle,allowDuplicate:false}});
    if(scenario==='original') {
      if(imported.format!=='epub')throw Error('Targeted Original verification requires a real EPUB import.');
      const structure=await invoke('library_epub_structure',{id:imported.id});
      if(!structure.some(item=>Object.hasOwn(item.anchors,'p40') && Object.hasOwn(item.anchors,'p65')))throw Error('Imported formatted EPUB lacks deep passage/footnote anchors.');
      const window=await invoke('source_get_segments',{input:{sourceId:imported.id,offset:40,limit:1}});
      const segment=window.segments[0];
      if(!segment?.text.startsWith('INVENTED formatted paragraph 41.'))throw Error('Real import paragraph mapping differs; inspect before proceeding.');
      await invoke('source_annotation_create',{input:{sourceId:imported.id,kind:'highlight',color:'amber',start:{segmentId:segment.id,offset:0},end:{segmentId:segment.id,offset:31},quote:segment.text.slice(0,31)}});
      localStorage.setItem('abiding-word.reader.toolbar.pins',JSON.stringify(['contents','history','marks']));
    }
    await invoke('settings_update',{input:{scriptureTranslationId:bible.id,scripturePassage:'JHN.15',theme:'light'}});
    localStorage.setItem('abiding-word.ui.language',JSON.stringify('en'));
    localStorage.setItem('abiding-word.layout.sourcesOpen','true');
    return {studyId:detail.study.id,blockId:block.id,body,sourceId:imported.id,sourceFormat:imported.format};
  },{bibleFile,sourceFile,mark,sourceTitle,scenario:args.scenario});
  report.sourceFormat=seeded.sourceFormat;
  report.stage='open-three-real-panes';
  await page.reload();await page.locator('[data-testid="app-shell"]').waitFor();
  const study=page.locator('#pane-study');
  await study.getByRole('button',{name:/INVENTED Viewport Project/}).first().click();
  await study.getByRole('button',{name:/INVENTED Viewport Study/}).first().click();
  const area=study.locator('section textarea').first();await area.waitFor();
  const editStudy=async()=>{await area.fill(seeded.body+'Edited in disposable test.');await area.focus();await area.evaluate(el=>el.setSelectionRange(el.value.length,el.value.length));};
  // Mirror the regression: edit Study before opening the Source, so test
  // automation does not scroll the surrounding workspace after Reading opens.
  if(args.scenario==='original')await editStudy();
  // SourcesPane's current overview renders recent/other imported Sources
  // as [data-source-link] buttons; [data-source-result] is used in search
  // results and may be absent in the ordinary overview after import.
  const sourceLink=page.locator('#pane-sources [data-source-link], #pane-sources [data-source-result]')
    .filter({hasText:sourceTitle}).first();
  await sourceLink.waitFor({state:'visible'});
  await sourceLink.click();
  await page.locator('#pane-sources [data-testid=reader-body]').waitFor();
  await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
  if(args.scenario==='resize')await editStudy();
  const expected=await area.inputValue();
  const caret=await area.evaluate(el=>[el.selectionStart,el.selectionEnd]);
  report.stage='initial-native-sample';
  const first=native('initial');assert.equal(first.Main.Dpi,expectedDpi,'Actual Windows DPI differs from requested run; emulation is not acceptance.');
  const scale=expectedDpi/96;
  if(args.scenario==='original') {
    report.stage='original-interaction';
    native('original-maximise','maximise');
    const { exerciseOriginal, compareOriginal } = await import('../../tests/screenshots/original-workspace-flow.mjs');
    let interactionBaseline;
    let previousInteractionSequence;
    await exerciseOriginal(page,{record:async(name,interaction)=>{
      interactionBaseline ??= interaction;
      const interactionComparison=compareOriginal(interactionBaseline,interaction);
      const dom=await page.evaluate(domSnapshot);
      let measured,sample,measurementError;
      try { measured=native(`${name}-measure`);sample=latestSample(info.logDirectory); }
      catch(error){measurementError=String(error?.stack??error);report.failures.push(`${name}: native measurement failed`);}
      let comparison=measured?compareGeometry(measured,sample,dom):{passed:false,diagnosis:'missing-native-measurement',errors:[measurementError]};
      if(measured&&(!measured.Main.Visible || measured.Main.Minimized || !measured.Main.Maximized))comparison={
        passed:false,diagnosis:'native-transition-unconfirmed',
        errors:[...comparison.errors,'Original verification requires a visible, maximised, non-minimised native window.']
      };
      let nativeScreenshotError;
      try { native(name,'observe',undefined,true); } catch(error) { nativeScreenshotError=String(error.message);report.failures.push(`${name}: native screenshot unavailable`); }
      await page.screenshot({path:join(out,`${name}-DOM.png`)});
      const tauriFreshness={kind:'Latest logged UI-thread COM sample from a native event; not a new COM read caused by this DOM interaction.',sequence:sample?.sequence,phase:sample?.phase,reusedFromPreviousCapture:sample?.sequence===previousInteractionSequence};
      previousInteractionSequence=sample?.sequence;
      report.cases.push({name,captureUTC:new Date().toISOString(),native:measured,tauri:sample,tauriFreshness,dom,comparison,interaction,interactionComparison,nativeScreenshotError,measurementError});
      if(!comparison.passed)report.failures.push(`${name}: native geometry`);
      if(!interactionComparison.passed)report.failures.push(`${name}: Original interaction containment`);
    }});
    report.originalScrolling=await scrollAll(page);
    if(report.originalScrolling.some(result=>!result.passed))report.failures.push('Pane scrolling failed after Original navigation.');
    report.originalEditing={valuePreserved:(await area.inputValue())===expected,caretPreserved:JSON.stringify(await area.evaluate(el=>[el.selectionStart,el.selectionEnd]))===JSON.stringify(caret)};
    if(Object.values(report.originalEditing).includes(false))report.failures.push('Study writing or caret changed during Original navigation.');
  } else {
  let previousSequence=0;
  for(const [name,action,size] of [ ['initial','observe'],['maximised','maximise'],['restored','restore'],['shorter','resize',[Math.round(1440*scale),Math.round(620*scale)]],['taller','resize',[Math.round(1440*scale),Math.round(850*scale)]],['maximised-again','maximise'],['restored-again','restore'] ]) {
    // An old Tauri sample must never be accepted as evidence for a new
    // maximise/restore/resize transition, even if the dimensions happen
    // to match. Only an observed native event can advance the sequence.
    report.stage=`window-transition:${name}`;
    native(`${name}-action`,action,size);
    let measured,dom,sample,comparison;
    for(let i=0;i<20;i++) {
      measured=native(`${name}-measure`);dom=await page.evaluate(domSnapshot);sample=latestSample(info.logDirectory);
      comparison=compareGeometry(measured,sample,dom);
      // A green pane-to-client comparison proves nothing if ShowWindow/
      // SetWindowPos failed and the window never entered the requested state.
      const stateErrors=[];
      if(!measured.Main.Visible || measured.Main.Minimized)
        stateErrors.push('The main native window is hidden or minimised.');
      if(action==='maximise' && !measured.Main.Maximized)
        stateErrors.push('Native IsZoomed is false after maximise.');
      if((action==='restore' || action==='resize') && measured.Main.Maximized)
        stateErrors.push('Native IsZoomed is true after restore/resize.');
      if(action==='resize' && size &&
         (Math.abs(measured.Main.ScreenWidth-size[0])>4 ||
          Math.abs(measured.Main.ScreenHeight-size[1])>4))
        stateErrors.push('Native outer window size differs from the requested physical size.');
      if(action !== 'observe' && (!Number.isSafeInteger(sample?.sequence) || sample.sequence <= previousSequence))
        stateErrors.push('No new native Tauri geometry sample after the requested window transition.');
      if(stateErrors.length)comparison={
        passed:false,diagnosis:'native-transition-unconfirmed',
        errors:[...comparison.errors,...stateErrors]
      };
      if(comparison.passed)break;
      await delay(100); // observation polling, never a resize/paint workaround
    }
    const editing={valuePreserved:(await area.inputValue())===expected,focusPreserved:await area.evaluate(el=>el===document.activeElement),caretPreserved:JSON.stringify(await area.evaluate(el=>[el.selectionStart,el.selectionEnd]))===JSON.stringify(caret)};
    const scroll=await scrollAll(page);
    await page.screenshot({path:join(out,`${name}-DOM.png`)});
    let nativeScreenshotError;
    try{native(name,'observe',undefined,true);}catch(error){nativeScreenshotError=String(error.message);report.failures.push(`${name}: native screenshot unavailable`);}
    const eventCounterGrowth=(sample?.sequence??0)-previousSequence;
    // Do not move the baseline backwards or hide a missing event with an
    // older snapshot. A failed transition remains failed in report.json.
    previousSequence=Math.max(previousSequence,sample?.sequence??0);
    if(eventCounterGrowth>100)report.failures.push(`${name}: excessive native event growth`);
    const entry={name,native:measured,tauri:sample,dom,comparison,editing,scroll,eventCounterGrowth,nativeScreenshotError};report.cases.push(entry);
    if(!comparison.passed||Object.values(editing).includes(false)||scroll.some(s=>!s.passed))report.failures.push(name);
  }
  }
  report.stage='persistent-edit-readback';
  await area.blur();
  await page.waitForFunction(async({id,expected,blockId})=>{
    const detail=await window.__TAURI_INTERNALS__.invoke('study_get',{id});return detail.sections.flatMap(s=>s.blocks).find(b=>b.id===blockId)?.content===expected;
  },{id:seeded.studyId,expected,blockId:seeded.blockId});
  report.savedEditVerified=true;
  report.geometryAndFunctionPassed=report.failures.length===0;
  report.stage='finished';
} catch(error) {
  report.failures.push(String(error?.stack ?? error));report.geometryAndFunctionPassed=false;
} finally {
  if(diagnosticLogDirectory){
    const logOut=join(out,'native-logs');mkdirSync(logOut,{recursive:true});
    try { const lines=readdirSync(diagnosticLogDirectory).filter(f=>f.endsWith('.log')).flatMap(file=>readFileSync(join(diagnosticLogDirectory,file),'utf8').split('\n').filter(line=>line.includes('Viewport sample ')));writeFileSync(join(logOut,'viewport-samples.log'),lines.join('\n')+'\n'); }
    catch(error){report.failures.push('Native log preservation failed: '+String(error));report.geometryAndFunctionPassed=false;}
  }
  writeFileSync(join(out,'report.json'),JSON.stringify(report,null,2)+'\n');
  if(browser)await browser.close();if(child.exitCode===null)child.kill();
}
assert.equal(report.geometryAndFunctionPassed,true,'Native evidence contains a failed gate; inspect report and native screenshots. Do not call the issue fixed.');
console.log('Native measurements/scroll/edit checks passed. Independently review native client screenshots and compare observe/repair runs before any fix claim.');
