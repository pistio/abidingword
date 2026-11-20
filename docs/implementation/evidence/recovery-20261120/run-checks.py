import datetime,hashlib,json,os,pathlib,subprocess,sys,time
root=pathlib.Path.cwd(); out=root/'work/recovery-verification'; mode=sys.argv[1]
checks={
'frontend':[
 ('npm-ci',['npm','ci','--cache','/workspace/.npm']),
 ('typecheck',['npm','run','typecheck']),
 ('eslint',['npm','run','lint']),
 ('i18n',['npm','run','i18n:check']),
 ('i18n-corpus',['npm','run','i18n:corpus','--','--check']),
 ('resources',['npm','run','release:resources']),
 ('audit',['npm','audit','--omit=dev','--audit-level=high','--cache','/workspace/.npm','--json']),
 ('frontend-tests',['npm','test','--','--maxWorkers=2','--reporter=default','--reporter=json','--outputFile=work/recovery-verification/frontend-tests.json']),
 ('frontend-build',['npm','run','build'])],
'native':[
 ('rust-fmt',['/workspace/work/recovery-native-env.sh','cargo','fmt','--manifest-path','src-tauri/Cargo.toml','--','--check']),
 ('rust-clippy',['/workspace/work/recovery-native-env.sh','cargo','clippy','--manifest-path','src-tauri/Cargo.toml','--all-targets','--locked','--','-D','warnings']),
 ('rust-tests',['/workspace/work/recovery-native-env.sh','cargo','test','--manifest-path','src-tauri/Cargo.toml','--locked','--no-fail-fast'])]}
sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
assert not subprocess.check_output(['git','status','--porcelain'],text=True).strip(), 'source must start clean'
files=subprocess.check_output(['git','ls-files','-z']).split(b'\0')
inputs={p.decode():hashlib.sha256((root/p.decode()).read_bytes()).hexdigest() for p in files if p and not p.startswith(b'docs/') and (root/p.decode()).is_file()}
record={'sourceSha':sha,'startedUTC':datetime.datetime.now(datetime.timezone.utc).isoformat(),'mode':mode,'cleanBefore':True,'inputHashes':inputs,'checks':[]}
path=out/f'{mode}-checks.json'
for name,command in checks[mode]:
 start=time.monotonic(); utc=datetime.datetime.now(datetime.timezone.utc).isoformat(); log=out/f'final-{name}.log'
 with log.open('w') as handle: status=subprocess.run(command,stdout=handle,stderr=subprocess.STDOUT).returncode
 item={'name':name,'command':command,'startedUTC':utc,'durationSeconds':round(time.monotonic()-start,2),'exitCode':status,'log':str(log.relative_to(root)),'logSha256':hashlib.sha256(log.read_bytes()).hexdigest()}
 record['checks'].append(item);path.write_text(json.dumps(record,indent=2)+'\n')
 print(json.dumps(item),flush=True)
 if status: print(log.read_text(errors='replace')[-7000:],flush=True);break
record['sourceUnchanged']=all(hashlib.sha256((root/p).read_bytes()).hexdigest()==h for p,h in inputs.items())
record['headUnchanged']=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()==sha
record['cleanAfter']=not subprocess.check_output(['git','status','--porcelain'],text=True).strip()
record['completedUTC']=datetime.datetime.now(datetime.timezone.utc).isoformat();path.write_text(json.dumps(record,indent=2)+'\n')
sys.exit(0 if all(c['exitCode']==0 for c in record['checks']) and len(record['checks'])==len(checks[mode]) and record['sourceUnchanged'] and record['headUnchanged'] and record['cleanAfter'] else 1)
