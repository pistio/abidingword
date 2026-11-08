// Validate actual reviewer ledgers against the shipped corpus; never generate editorial decisions.
import {readFileSync,readdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
const base='64dadc0d1bb263362aa384951612e9395d705725';
function flatten(value,prefix='',out={}){for(const [key,child] of Object.entries(value)){if(typeof child==='string')out[prefix+key]=child;else flatten(child,prefix+key+'.',out);}return out;}
const results=[];
for(const language of ['en','de','ko','fr','es','pt']){
 const current={},before={};
 for(const file of readdirSync(`src/i18n/locales/${language}`).filter(f=>f.endsWith('.json'))){
  const path=`src/i18n/locales/${language}/${file}`;const ns=file.slice(0,-5)+'.';
  flatten(JSON.parse(readFileSync(path,'utf8')),ns,current);
  flatten(JSON.parse(execFileSync('git',['show',`${base}:${path}`],{encoding:'utf8'})),ns,before);
 }
 const ledger=JSON.parse(readFileSync(`docs/i18n/issue-42/review-${language}-decisions.json`,'utf8'));
 const decisions=new Map();const states={};
 for(const decision of ledger.decisions){
  assert.ok(!decisions.has(decision.key),`${language}: duplicate decision ${decision.key}`);
  decisions.set(decision.key,decision);const state=decision.decision??decision.state;
  assert.ok(state&&decision.reason&&decision.context,`${language}:${decision.key}: missing editorial state/reason/context`);
  states[state]=(states[state]??0)+1;
  assert.equal(decision.after,current[decision.key],`${language}:${decision.key}: stale decision after text`);
  if(Object.hasOwn(before,decision.key))assert.equal(decision.before,before[decision.key],`${language}:${decision.key}: incorrect before evidence`);
 }
 assert.deepEqual([...decisions.keys()].sort(),Object.keys(current).sort(),`${language}: incomplete individual review coverage`);
 results.push({language,leaves:Object.keys(current).length,states});
}
console.log(JSON.stringify({baseline:base,review:'AI individual editorial ledgers; no human native-speaker approval',results},null,2));
