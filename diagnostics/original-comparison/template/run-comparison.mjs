import {mkdirSync,readFileSync,writeFileSync,existsSync} from 'node:fs';
import {spawnSync,execFileSync} from 'node:child_process';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {randomUUID} from 'node:crypto';
import {verifyPackage,sha256} from './package-integrity.mjs';
import {classifyRun} from './classify-comparison.mjs';
const root=dirname(fileURLToPath(import.meta.url));
const manifest=verifyPackage(root);
if(process.argv.includes('--verify-only')){console.log('Package inventory and pinned source/executable bindings verified. No app launched.');process.exit(0);}
if(process.platform!=='win32')throw Error('This comparison requires the existing interactive Windows 11 desktop.');
const node=join(root,'tools/node/node.exe'),pwsh=join(root,'tools/pwsh/pwsh.exe');
const out=join(root,'results',new Date().toISOString().replace(/[:.]/g,'-')+'-'+randomUUID().slice(0,8));mkdirSync(out,{recursive:true});
const summary={protocolSourceSha:manifest.protocolSourceSha,protocolVariant:manifest.protocolVariant,startedUTC:new Date().toISOString(),expectedDpi:96,mode:'observe',scenario:'original',acceptance:'PENDING INDEPENDENT NATIVE SCREENSHOT REVIEW',runs:{}};
writeFileSync(join(out,'package-manifest.json'),JSON.stringify(manifest,null,2)+'\n');
const windows=JSON.parse(execFileSync(pwsh,['-NoProfile','-ExecutionPolicy','Bypass','-Command',"Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version,BuildNumber,ProductType | ConvertTo-Json -Compress"],{encoding:'utf8'}).replace(/^\uFEFF/,''));
summary.windows=windows;writeFileSync(join(out,'comparison-summary.json'),JSON.stringify(summary,null,2)+'\n');
if(Number(windows.ProductType)!==1||Number(windows.BuildNumber)<22000)throw Error('Native acceptance requires the existing Windows 11 desktop, not Windows Server or an older system.');
console.log('Abiding Word: targeted Reading -> Original comparison.');
console.log('Keep this clean test desktop at 100% scaling, with the application unobscured.');
console.log('Two disposable profiles will be created. Do not interact with the app during automation.');
for(const role of ['baseline','corrected']){
  const build=manifest.builds[role],runOut=join(out,role);
  console.log(`\nRunning ${role}: ${build.sourceSha}`);
  const result=spawnSync(node,[join(root,'scripts/windows/Test-WebviewViewport.mjs'),'--scenario','original','--mode','observe','--exe',join(root,build.executable),'--exe-sha256',build.sha256,'--source-sha',build.sourceSha,'--expected-dpi','96','--powershell',pwsh,'--port',role==='baseline'?'9227':'9228','--out',runOut],{cwd:root,encoding:'utf8',windowsHide:false,maxBuffer:16*1024*1024,env:{...process.env,PLAYWRIGHT_MODULE:join(root,'tools/playwright-core'),ABIDING_WORD_VIEWPORT_MODE:'observe'}});
  mkdirSync(runOut,{recursive:true});writeFileSync(join(runOut,'runner.log'),(result.stdout??'')+'\n'+(result.stderr??'')+(result.error?'\n'+result.error.stack:''));
  let report;try{report=JSON.parse(readFileSync(join(runOut,'report.json'),'utf8'));}catch{}
  let classification=classifyRun(report,runOut,role);
  if(report&&(report.sourceSha!==build.sourceSha||report.executableSha256!==build.sha256||report.protocolSourceSha!==manifest.protocolSourceSha))classification={classification:'SETUP_OR_MEASUREMENT_FAILURE',reproduced:false,reasons:['Report source/executable/protocol provenance does not match the package manifest.']};
  summary.runs[role]={sourceSha:build.sourceSha,executableSha256:build.sha256,exitStatus:result.status,signal:result.signal,error:result.error?.message,classification,webview2Version:report?.webview2Version,protocolSourceSha:report?.protocolSourceSha,readingNative:report?.cases?.find(c=>c.name==='reading-deep')?.native?.Main};
  console.log(classification.classification);
  for(const reason of classification.reasons)console.log('  '+reason);
  writeFileSync(join(out,'comparison-summary.json'),JSON.stringify(summary,null,2)+'\n');
}
const fixtureNames=['invented.usfm','invented-original.epub'];summary.identicalInventedContent={};
for(const name of fixtureNames){const a=join(out,'baseline',name),b=join(out,'corrected',name);summary.identicalInventedContent[name]=existsSync(a)&&existsSync(b)?{baseline:sha256(a),corrected:sha256(b),equal:sha256(a)===sha256(b)}:{equal:false,reason:'Fixture missing; setup failed.'};}
const before=summary.runs.baseline,after=summary.runs.corrected;
summary.sameRuntimeAndNativeClient=Boolean(before.webview2Version&&before.webview2Version===after.webview2Version&&before.protocolSourceSha===manifest.protocolSourceSha&&after.protocolSourceSha===manifest.protocolSourceSha&&before.readingNative&&after.readingNative&&['Dpi','ClientWidth','ClientHeight'].every(k=>before.readingNative[k]===after.readingNative[k]));
summary.nativeComparisonReadyForReview=summary.sameRuntimeAndNativeClient&&summary.runs.baseline.classification.reproduced===true&&summary.runs.baseline.classification.firstFailureRetained===true&&summary.runs.corrected.classification.classification==='CORRECTED_CHECKS_PASSED_PENDING_SCREENSHOT_REVIEW'&&Object.values(summary.identicalInventedContent).every(x=>x.equal);
summary.finishedUTC=new Date().toISOString();writeFileSync(join(out,'comparison-summary.json'),JSON.stringify(summary,null,2)+'\n');
const zip=out+'-evidence.zip';
const pack=spawnSync(pwsh,['-NoProfile','-ExecutionPolicy','Bypass','-File',join(root,'Zip-Evidence.ps1'),'-InputDirectory',out,'-OutputPath',zip],{encoding:'utf8',windowsHide:true});
if(pack.status!==0){console.error('Evidence ZIP creation failed. Evidence remains at '+out);console.error(pack.stderr);process.exitCode=1;}else console.log('\nEvidence ZIP: '+zip);
console.log(summary.nativeComparisonReadyForReview?'Measurements are ready for independent native screenshot review. This is not automatic Windows acceptance.':'Acceptance is blocked: see comparison-summary.json. A setup failure or a non-reproducing baseline is not evidence of a fix.');
if(!summary.nativeComparisonReadyForReview)process.exitCode=1;
