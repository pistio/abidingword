// Assembles already-built debug executables and portable diagnostic tools.
// No build, installer, network operation, workflow dispatch or app launch.
import assert from 'node:assert/strict';
import {parseArgs} from 'node:util';
import {readFileSync,writeFileSync,mkdirSync,readdirSync,statSync,lstatSync,copyFileSync,cpSync,existsSync} from 'node:fs';
import {resolve,join,dirname,relative,isAbsolute} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
const CORRECTED='32e8a52567c3863f9b2930125f711fba6b7bd9ef';
const BASELINE='624ef9f357bcc956275e70bbd8ae371f1eba89f9';
const BASELINE_HASH='0413927c456653deb2a30406a79ba192ae795af475928a837729ab73440716dc';
const hash=file=>createHash('sha256').update(readFileSync(file)).digest('hex');
const json=file=>JSON.parse(readFileSync(file,'utf8').replace(/^\uFEFF/,''));
const {values:a}=parseArgs({options:{'source-root':{type:'string'},'baseline-root':{type:'string'},'corrected-root':{type:'string'},'tools-root':{type:'string'},'template-root':{type:'string'},'frontend-result':{type:'string'},'frontend-proof':{type:'string'},'tool-provenance':{type:'string'},out:{type:'string'},help:{type:'boolean'}}});
if(a.help){console.log('node assemble.mjs --source-root PINNED_SOURCE --baseline-root OLD_ARTIFACT --corrected-root NEW_BUILD --tools-root TOOLS --template-root TEMPLATE --frontend-result VITEST_JSON --frontend-proof CLEAN_SOURCE_JSON --tool-provenance TOOLS_JSON --out NEW_DIRECTORY');process.exit(0);}
for(const key of ['source-root','baseline-root','corrected-root','tools-root','frontend-result','frontend-proof','out'])assert.ok(a[key],`Missing --${key}`);
const source=resolve(a['source-root']),baseline=resolve(a['baseline-root']),corrected=resolve(a['corrected-root']),tools=resolve(a['tools-root']),template=resolve(a['template-root']??join(dirname(fileURLToPath(import.meta.url)),'package')),out=resolve(a.out);
assert.equal(execFileSync('git',['-C',source,'rev-parse','HEAD'],{encoding:'utf8'}).trim(),CORRECTED,'Package resources must come from exact approved source.');
assert.equal(execFileSync('git',['-C',source,'status','--porcelain','--untracked-files=no'],{encoding:'utf8'}).trim(),'','Approved source must remain tracked-clean.');
assert.ok(!existsSync(out),'Output directory must be new; never overwrite a previous kit or evidence.');
const frontend=json(resolve(a['frontend-result'])),gate=json(resolve(a['frontend-proof']));
assert.equal(frontend.success,true,'Complete frontend gate did not pass.');
assert.equal(frontend.numFailedTests,0,'Unexpected frontend failure remains.');
assert.equal(frontend.numPassedTests,1298,'Unexpected full suite count; inspect before packaging.');
assert.equal(gate.sourceSha,CORRECTED,'Frontend gate used the wrong source.');assert.equal(gate.clean,true,'Frontend source was not clean.');
function buildAt(root,expected){
 const proofPath=join(root,'work/viewport-source.json'),environmentPath=join(root,'work/viewport-build-environment.txt'),executable=join(root,'src-tauri/target/debug/abiding-word.exe');
 const proof=json(proofPath);const env=Object.fromEntries(readFileSync(environmentPath,'utf8').replace(/^\uFEFF/,'').split(/\r?\n/).filter(Boolean).map(line=>{const i=line.indexOf('=');return[line.slice(0,i),line.slice(i+1)];}));
 assert.equal(proof.commit,expected,'Build source proof mismatch.');assert.equal(proof.dirty,false,'Build proof source was dirty.');assert.equal(proof.schemaVersion,27);assert.equal(proof.migrations.length,27);
 assert.equal(env.SourceSHA,expected,'Build environment source mismatch.');assert.equal(env.InstallerCreated,'false','Expected plain executable provenance.');assert.equal(env.RunnerOS,'Windows');
 const bytes=readFileSync(executable);assert.ok(bytes.length>1024&&bytes[0]===0x4d&&bytes[1]===0x5a,'Executable is not a nonempty Windows PE file.');
 const sha256=hash(executable);assert.equal(sha256,env.ExecutableSHA256?.toLowerCase(),'Executable SHA256 differs from build proof.');assert.match(env.GitHubRunID,/^\d+$/);
 if(expected===BASELINE){assert.equal(sha256,BASELINE_HASH,'Wrong already-built baseline.');assert.equal(env.GitHubRunID,'43405180688','Baseline run mismatch.');}
 return{proof,env,proofPath,environmentPath,executable,sha256};
}
const old=buildAt(baseline,BASELINE),fixed=buildAt(corrected,CORRECTED);
assert.equal(JSON.stringify(old.proof.migrations),JSON.stringify(fixed.proof.migrations),'Historical migrations differ across builds.');
assert.equal(json(join(tools,'playwright-core/package.json')).version,'1.65.0','Unexpected Playwright protocol version.');
for(const path of ['node/node.exe','pwsh/pwsh.exe'])assert.ok(statSync(join(tools,path)).size>1024,`Missing portable ${path}`);
const config=json(join(source,'src-tauri/tauri.conf.json'));assert.ok(!config.plugins?.updater&&!config.bundle?.createUpdaterArtifacts,'Updater configuration must be absent.');
assert.equal(Object.keys(config.bundle.resources).length,13,'Unexpected bundled-resource inventory.');
mkdirSync(out,{recursive:true});
function copy(file,dest){mkdirSync(dirname(dest),{recursive:true});copyFileSync(file,dest);}
function copyTree(from,to){
 for(const entry of readdirSync(from,{withFileTypes:true})){
  if(['package-manifest.json','results'].includes(entry.name))continue;
  const src=join(from,entry.name),dst=join(to,entry.name);assert.ok(!lstatSync(src).isSymbolicLink(),`Do not ship symbolic link: ${src}`);
  if(entry.isDirectory()){mkdirSync(dst,{recursive:true});copyTree(src,dst);}else if(entry.isFile())copy(src,dst);else throw Error(`Unsupported package entry ${src}`);
 }
}
copyTree(template,out);copyTree(tools,join(out,'tools'));
const builds={};
for(const [role,b,sha] of [['baseline',old,BASELINE],['corrected',fixed,CORRECTED]]){
 const path=`binaries/${role}/abiding-word.exe`;copy(b.executable,join(out,path));
 copy(b.proofPath,join(out,`provenance/${role}/viewport-source.json`));copy(b.environmentPath,join(out,`provenance/${role}/viewport-build-environment.txt`));
 builds[role]={sourceSha:sha,sha256:b.sha256,executable:path,runId:b.env.GitHubRunID,buildProfile:'debug',installerCreated:false};
}
const resources=[];
for(const [from,to] of Object.entries(config.bundle.resources)){
 assert.ok(!isAbsolute(to)&&!to.split(/[\\/]/).includes('..'),'Unsafe resource destination.');
 const file=resolve(source,'src-tauri',from),rel=relative(source,file);assert.ok(!isAbsolute(rel)&&!rel.startsWith('..'),'Resource outside pinned checkout.');
 const sha256=hash(file);for(const role of ['baseline','corrected'])copy(file,join(out,'binaries',role,to));
 resources.push({source:rel.replaceAll('\\','/'),destination:to,sha256,bytes:statSync(file).size});
}
copy(resolve(a['frontend-result']),join(out,'provenance/frontend-results.json'));copy(resolve(a['frontend-proof']),join(out,'provenance/clean-source.json'));
if(a['tool-provenance'])copy(resolve(a['tool-provenance']),join(out,'provenance/tool-archives.json'));
writeFileSync(join(out,'provenance/resources.json'),JSON.stringify({sourceSha:CORRECTED,identicalForBothBuilds:true,resources},null,2)+'\n');
const protocolPaths=['scripts/windows/Test-WebviewViewport.mjs','scripts/windows/Measure-WebviewGeometry.ps1','scripts/windows/viewport-evidence.mjs','scripts/windows/original-epub-fixture.mjs','tests/screenshots/original-workspace-flow.mjs'];
const protocolOrigin=protocolPaths.map(path=>({path,sourceSha:CORRECTED,sourceFileSHA256:hash(join(source,path)),packagedFileSHA256:hash(join(out,path)),adaptedForPortablePackage:hash(join(source,path))!==hash(join(out,path))}));
writeFileSync(join(out,'provenance/protocol-origin.json'),JSON.stringify({sourceSha:CORRECTED,files:protocolOrigin,note:'Both executables use exactly the same packaged protocol; portable wrapper adaptations are inventoried separately from application source.'},null,2)+'\n');
const manifest={formatVersion:1,protocolSourceSha:CORRECTED,protocolVariant:'portable comparison: original/observe; manifest verification replaces Git preflight; diagnostic logs preserved; fail-closed comparison classification',createdUTC:new Date().toISOString(),builds,frontendGate:{sourceSha:CORRECTED,clean:true,passed:1298,failed:0},resourcesIdentical:true,files:[]};
function inventory(dir){for(const entry of readdirSync(dir,{withFileTypes:true}).sort((a,b)=>a.name.localeCompare(b.name))){const path=join(dir,entry.name);if(entry.isDirectory())inventory(path);else{const rel=relative(out,path).replaceAll('\\','/');if(rel!=='package-manifest.json')manifest.files.push({path:rel,bytes:statSync(path).size,sha256:hash(path)});}}}
inventory(out);writeFileSync(join(out,'package-manifest.json'),JSON.stringify(manifest,null,2)+'\n');
const {verifyPackage}=await import(pathToFileURL(join(out,'package-integrity.mjs')).href);verifyPackage(out);
console.log(`Verified comparison package: ${out}`);console.log(`Baseline ${BASELINE} ${old.sha256}`);console.log(`Corrected ${CORRECTED} ${fixed.sha256}`);console.log(`${manifest.files.length} inventoried files; identical ${resources.length} resource files for each executable.`);
