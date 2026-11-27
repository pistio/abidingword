// Opens the pinned installed candidate in the same disposable owner-test
// profile on each run. Environment changes apply only to the child process.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { parseArgs } from 'node:util';

const { values: args } = parseArgs({ options: {
  exe: { type: 'string' },
  help: { type: 'boolean' },
} });
if (args.help) {
  console.log('node Launch-ScriptureAcceptance.mjs [--exe INSTALLED_EXE]');
  console.log('Checks acceptance-build.json and opens the installed candidate in its disposable owner-test profile.');
  process.exit(0);
}
assert.equal(process.platform, 'win32', 'This launcher requires Windows.');
const build = JSON.parse(readFileSync(join(dirname(fileURLToPath(import.meta.url)), 'acceptance-build.json'), 'utf8').replace(/^\uFEFF/, ''));
assert.match(build.sourceSha ?? '', /^[a-f0-9]{40}$/);
assert.match(build.executableSha256 ?? '', /^[a-f0-9]{64}$/);
assert.equal(build.identifier, 'app.abidingword.scripture.acceptance');
assert.equal(build.updater, false);
assert.ok(args.exe || process.env.LOCALAPPDATA, 'LOCALAPPDATA is unavailable; pass --exe.');
const executable = realpathSync(args.exe ?? join(process.env.LOCALAPPDATA, 'Abiding Word Scripture Acceptance', 'abiding-word-scripture-acceptance.exe'));
assert.equal(createHash('sha256').update(readFileSync(executable)).digest('hex'), build.executableSha256, 'The installed executable does not match this candidate. Install the matching setup file first.');

const root = join(realpathSync(tmpdir()), 'aw-scripture-acceptance-owner-test');
const marker = join(root, 'acceptance-profile');
if (existsSync(root)) {
  assert.equal(realpathSync(root).toLowerCase(), resolve(root).toLowerCase(), 'The test profile must not redirect to another folder.');
  assert.equal(readFileSync(marker, 'utf8'), 'Abiding Word Scripture acceptance', 'An existing folder is not a marked acceptance profile.');
} else {
  mkdirSync(root);
  writeFileSync(marker, 'Abiding Word Scripture acceptance', { flag: 'wx' });
}
const data = join(root, 'app-data');
const webview = join(root, 'webview2');
for (const path of [data, webview]) {
  mkdirSync(path, { recursive: true });
  assert.equal(realpathSync(path).toLowerCase(), resolve(path).toLowerCase(), 'Test data must remain inside its marked profile.');
}
const child = spawn(executable, [], {
  detached: true,
  stdio: 'ignore',
  env: { ...process.env, ABIDING_WORD_DATA_DIR: data, WEBVIEW2_USER_DATA_FOLDER: webview },
});
await once(child, 'spawn');
child.unref();
console.log(`Opened Scripture acceptance candidate ${build.sourceSha}.`);
console.log(`Disposable test data: ${data}`);
console.log('This launcher preserves this test profile for reopening. It does not mark Windows acceptance as passed.');
