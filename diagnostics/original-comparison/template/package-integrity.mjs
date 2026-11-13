import {readFileSync,statSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve,relative,isAbsolute} from 'node:path';
export const sha256=file=>createHash('sha256').update(readFileSync(file)).digest('hex');
export function verifyPackage(root){
  const manifest=JSON.parse(readFileSync(resolve(root,'package-manifest.json'),'utf8'));
  if(manifest.protocolSourceSha!=='32e8a52567c3863f9b2930125f711fba6b7bd9ef')throw Error('Unexpected protocol source.');
  if(manifest.builds?.baseline?.sourceSha!=='624ef9f357bcc956275e70bbd8ae371f1eba89f9'||manifest.builds?.corrected?.sourceSha!=='32e8a52567c3863f9b2930125f711fba6b7bd9ef')throw Error('Unexpected executable source pair.');
  if(!Array.isArray(manifest.files)||manifest.files.length<8)throw Error('Incomplete package inventory.');
  for(const entry of manifest.files){
    const file=resolve(root,entry.path), rel=relative(root,file);
    if(!rel||isAbsolute(rel)||rel.startsWith('..'))throw Error('Unsafe package path.');
    if(statSync(file).size!==entry.bytes||sha256(file)!==entry.sha256)throw Error(`Package file changed or incomplete: ${entry.path}`);
  }
  for(const build of Object.values(manifest.builds)){
    const entry=manifest.files.find(f=>f.path===build.executable);
    if(!entry||entry.sha256!==build.sha256)throw Error('Executable missing from package inventory.');
  }
  return manifest;
}
