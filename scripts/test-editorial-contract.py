"""Offline preservation gates for the Issue 42 editorial branch."""
import json
from pathlib import Path
import re
import subprocess

BASE = '38c785be723f4ff4ca7ba8c885aeabff3a89df7a'

def flatten(value, prefix=''):
    result = {}
    for key, child in value.items():
        if isinstance(child, dict):
            result.update(flatten(child, prefix + key + '.'))
        else:
            result[prefix + key] = child
    return result

for path in Path('src/i18n/locales').glob('*/*.json'):
    before = flatten(json.loads(subprocess.check_output(['git', 'show', f'{BASE}:{path}'])))
    after = flatten(json.loads(path.read_text()))
    assert before.keys() <= after.keys(), f'Removed keys or plural branches: {path}'
    for key, old in before.items():
        assert set(re.findall(r'{{\s*([^}]+)\s*}}', old)) == set(re.findall(r'{{\s*([^}]+)\s*}}', after[key])), (path, key)
        if 'ten-step-' in key:
            assert after[key] == old, f'Accepted Ten-Step wording changed: {path}:{key}'
for pattern in ['src-tauri', 'resources', 'src/i18n/locales/*/books.json']:
    assert not subprocess.check_output(['git', 'diff', BASE, '--', pattern]), pattern
print('All existing keys, plural branches, placeholder names, Ten-Step guidance, native source, resources, Bible names and migrations preserved.')
