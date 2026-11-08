#!/usr/bin/env node
// Full source inventory; candidate detection is deliberately not editorial approval.
import {readFileSync,readdirSync,mkdirSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {join} from 'node:path';
import ts from 'typescript';
import {readLocales,LANGUAGES} from './i18n-corpus.mjs';
const root='docs/i18n/issue-42';mkdirSync(root,{recursive:true});
const walk=d=>readdirSync(d,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(join(d,e.name)):[join(d,e.name)]).sort();
const hash=text=>createHash('sha256').update(text).digest('hex');
const locales=readLocales();
const sources=[];const candidates=[];const nativeCodes=new Map();
for(const file of [...walk('src').filter(f=>/\.(tsx?|html)$/.test(f)),...walk('src-tauri/src').filter(f=>f.endsWith('.rs'))]){
 const text=readFileSync(file,'utf8');sources.push({file,sha256:hash(text)});
 if(file.endsWith('.rs')){
  for(const m of text.matchAll(/UserMessage::new\(\s*"([\w.]+)"/g)){
   if(!nativeCodes.has(m[1]))nativeCodes.set(m[1],[]);
   nativeCodes.get(m[1]).push({file,line:text.slice(0,m.index).split('\n').length});
  }
  for(const m of text.matchAll(/"((?:[^"\\]|\\.)*)"/g))if(/[A-Za-z]{2,} [A-Za-z]{2,}/.test(m[1])){
   candidates.push({file,line:text.slice(0,m.index).split('\n').length,text:m[1],classification:'native candidate: may be log, provider prompt, metadata or visible copy; inspect call path'});
  }
 }else{
  const source=ts.createSourceFile(file,text,ts.ScriptTarget.Latest,true);
  const visit=n=>{
   const literal=ts.isStringLiteral(n)||ts.isNoSubstitutionTemplateLiteral(n)||ts.isJsxText(n);
   const visibleAttribute=n.parent&&ts.isJsxAttribute(n.parent)&&['aria-label','title','placeholder','alt','label'].includes(n.parent.name.getText(source));
   if(literal&&(/[A-Za-z]{2,} [A-Za-z]{2,}/.test(n.text)||((ts.isJsxText(n)||visibleAttribute)&&/\p{L}/u.test(n.text)))){
    const parent=n.parent;let classification='requires context review';
    if(ts.isJsxAttribute(parent)&&parent.name.getText(source)==='className')classification='CSS attribute';
    else if(ts.isImportDeclaration(parent)||ts.isExportDeclaration(parent))classification='module path';
    else if(ts.isPropertyAssignment(parent)&&parent.name.getText(source)==='defaultValue')classification='localised fallback: verify key and call path';
    else if(ts.isJsxText(n))classification='JSX text candidate';
    candidates.push({file,line:source.getLineAndCharacterOfPosition(n.getStart()).line+1,text:n.text.trim(),classification});
   }
   ts.forEachChild(n,visit);
  };visit(source);
 }
}
const base='38c785be723f4ff4ca7ba8c885aeabff3a89df7a';
function flatten(d,p='',out={}){for(const[k,v]of Object.entries(d)){if(typeof v==='string')out[p+k]=v;else flatten(v,p+k+'.',out);}return out;}
const counts={};
const escape=s=>String(s).replaceAll('|','\\|').replaceAll('\n','<br>');
for(const lang of LANGUAGES){
 const areas={};const edited=[];
 for(const file of readdirSync(`src/i18n/locales/${lang}`).filter(f=>f.endsWith('.json')).sort()){
  const path=`src/i18n/locales/${lang}/${file}`;const current=flatten(JSON.parse(readFileSync(path,'utf8')));const before=flatten(JSON.parse(execFileSync('git',['show',`${base}:${path}`],{encoding:'utf8'})));
  areas[file.slice(0,-5)]={keys:Object.keys(current).length,sha256:hash(readFileSync(path,'utf8'))};
  for(const[key,value]of Object.entries(current))if(before[key]!==value)edited.push({key:file.slice(0,-5)+'.'+key,before:before[key]??'(new localised message)',after:value});
 }
 counts[lang]={areas,keys:locales[lang].size,changedKeys:edited.length};
 const lines=[`# ${lang}: AI editorial change log`, '',`Base: \`${base}\`. This is AI linguistic review, not human native-speaker approval.`, '', 'Register: '+({en:'British English; preserve established product terms.',de:'Informal du; Bibelstudium / Bibellehre stay distinct.',ko:'Respectful Korean; preserve 성경공부, 성경 가르침 and 틀 terminology.',fr:'Consistent vous; idiomatic French, curly apostrophes.',es:'Consistent tú; broadly understandable Spanish. Biblical quotations retain their original register.',pt:'Brazilian Portuguese, você; arquivo, salvar and gerund progress labels.'}[lang]),'','These rows are the individually edited/reviewed keys. The full application is inventoried in inventory.json and the complete six-language corpus; unchanged keys are not claimed to have all received a new individual linguistic adjudication.','', '| Key/context | Before | After | Editorial reason |','| --- | --- | --- | --- |'];
 for(const e of edited){const reason=e.before==='(new localised message)'?'Remove an English-only native/UI path; preserve meaning and uncertainty.':e.key.includes('timeout')?'Replace technical processing-boundary jargon with outcome, unchanged-data assurance and retry.':lang==='pt'?'Use natural Brazilian vocabulary/register and match the visible action names.':e.key.includes('workspace')?'Apply approved terminology; remove duplication.':'Correct register or localise diagnostic evidence without changing the method or source data.';lines.push(`| \`${e.key}\` | ${escape(e.before)} | ${escape(e.after)} | ${reason} |`);}
 writeFileSync(`${root}/${lang}.md`,lines.join('\n')+'\n');
}
const missing=[];
for(const code of [...nativeCodes.keys()].sort())for(const language of LANGUAGES)if(![...locales[language].keys()].some(key=>key.replace(/_(zero|one|two|few|many|other)$/,'')==='errors.'+code))missing.push({language,code});
const out={scope:'Complete locale/source inventory; not a claim of complete individual editorial approval',base,languages:counts,sources,nativeCodes:[...nativeCodes].sort().map(([code,locations])=>({code,locations})),missingNativeCodes:missing,candidates};
writeFileSync(`${root}/inventory.json`,JSON.stringify(out,null,2)+'\n');
console.log(JSON.stringify({languages:counts,sourceFiles:sources.length,staticallyNamedNativeCodes:nativeCodes.size,missingNativeCodes:missing.length,sourceCandidates:candidates.length},null,2));
if(missing.length)process.exitCode=1;
