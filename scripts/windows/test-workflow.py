"""Offline packaging contract/security checks; no Actions dispatch or app edits."""
import json
import re
import subprocess
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[2]
SHA = '571b8fc0633b6909b6532e1c6b254e2dbb357f45'
workflow = ROOT / '.github/workflows/windows-owner-corrections-acceptance-build.yml'
text = workflow.read_text()
w = yaml.load(text, Loader=yaml.BaseLoader)
assert list(w['on']) == ['workflow_dispatch']
assert w['permissions'] == {'contents': 'read'}
assert 'secrets.' not in text and 'workflow_run' not in text and 'schedule:' not in text
inputs = w['on']['workflow_dispatch']['inputs']['source_sha']
assert inputs['options'] == [SHA] and inputs['default'] == SHA
job = w['jobs']['acceptance']
assert job['if'] == "github.event_name == 'workflow_dispatch'"
assert job['runs-on'] == 'windows-2025'
steps = job['steps']
assert SHA in steps[0]['run']
for step in steps:
    if 'uses' in step:
        assert re.search(r'@[a-f0-9]{40}$', step['uses']), step
checkouts = [s for s in steps if s.get('uses', '').startswith('actions/checkout@')]
assert [(s['with']['ref'], s['with']['path']) for s in checkouts] == [
    ('${{ inputs.source_sha }}', 'app'), ('${{ github.workflow_sha }}', 'packaging')]
assert all(s['with']['persist-credentials'] == 'false' for s in checkouts)
positions = {s.get('id'): i for i, s in enumerate(steps) if 'id' in s}
assert positions['nsis-preflight'] < positions['nsis-probe'] < positions['compile'] < positions['diagnostic']
assert steps[positions['nsis-preflight']]['timeout-minutes'] == '6'
assert steps[positions['nsis-probe']]['timeout-minutes'] == '2'
assert 'SkipExecutableProbe' not in text
compile_step = steps[positions['compile']]
assert compile_step['working-directory'] == 'app'
assert '--no-bundle --no-sign --ci -- --locked' in compile_step['run']
assert steps[positions['diagnostic']]['if'].startswith('always()')
bundle = next(s for s in steps if 'Package existing unsigned executable' in s.get('name', ''))
assert 'tauri bundle --bundles nsis --no-sign --ci' in bundle['run']
assert 'nsis-probe.outcome' in bundle['if']
assert bundle['env']['TAURI_BUNDLER_TOOLS_GITHUB_MIRROR_TEMPLATE'].startswith('https://127.0.0.1:9/')
assert positions['diagnostic'] < steps.index(bundle)
diagnostic_upload = next(s for s in steps if 'not a portable' in s.get('name', ''))
assert diagnostic_upload['if'].startswith('always()')
evidence = steps[-1]
assert evidence['if'] == 'always()' and evidence['with']['if-no-files-found'] == 'error'
assert 'packaging-evidence/*.log' in evidence['with']['path']
probe = (ROOT / 'scripts/windows/Test-NsisCache.ps1').read_text()
assert '--no-binary-patching' in probe and 'Assert-NoToolDownload $log' in probe
assert "$env:CARGO_NET_OFFLINE = 'true'" in probe
assert "'fn main() {}'" in probe
assert 'Remove-Item -Recurse -Force $probe' in probe
manifest = json.loads((ROOT / 'scripts/windows/nsis-toolchain.json').read_text())
lock = json.loads(subprocess.check_output(['git', 'show', f'{SHA}:package-lock.json'], cwd=ROOT))
assert manifest['cliVersion'] == lock['packages']['node_modules/@tauri-apps/cli']['version'] == '2.12.0'
assert manifest['bundlerVersion'] == '2.10.0'
assert manifest['tools'][0]['sha1'] == 'ef7ff767e5cbd9edd22add3a32c9b8f4500bb10d'
assert manifest['tools'][1]['sha1'] == '75197fee3c6a814fe035788d1c34ead39349b860'
assert len(manifest['requiredFiles']) == 13 and len(manifest['pluginFiles']) == 5
config = json.loads(subprocess.check_output(['git', 'show', f'{SHA}:src-tauri/tauri.conf.json'], cwd=ROOT))
assert not config['bundle'].get('useLocalToolsDir', False)
assert not config.get('plugins', {}).get('updater') and not config['bundle'].get('createUpdaterArtifacts')
assert len(config['bundle']['resources']) == 13
# Packaging-only lineage: application, dependency locks, templates and SQL identical.
assert not subprocess.check_output(['git', 'diff', SHA, '--', 'src', 'src-tauri', 'resources', 'package.json', 'package-lock.json'], cwd=ROOT)
print('Workflow order, exact source/CLI pins, immutable actions, read-only permissions, failure artifacts, probe guard and unchanged application/schema: passed.')
