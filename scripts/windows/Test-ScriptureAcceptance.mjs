import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync, spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { parseArgs } from 'node:util';
import { setTimeout as delay } from 'node:timers/promises';
import { createInterface } from 'node:readline/promises';
import { scriptureFixtures, verseText } from './scripture-acceptance-fixtures.mjs';

const { values: args } = parseArgs({ options: {
  exe: { type: 'string' },
  'exe-sha256': { type: 'string' },
  'source-sha': { type: 'string' },
  out: { type: 'string' },
  help: { type: 'boolean' },
} });
if (args.help) {
  console.log('node Test-ScriptureAcceptance.mjs --exe INSTALLED_EXE --exe-sha256 SHA256 --source-sha GIT_SHA --out NEW_DIRECTORY');
  process.exit(0);
}
assert.equal(process.platform, 'win32', 'An interactive Windows desktop is required.');
assert.ok(args.exe && args.out && /^[a-f0-9]{64}$/.test(args['exe-sha256'] ?? '') && /^[a-f0-9]{40}$/.test(args['source-sha'] ?? ''));
const hash = (bytes) => createHash('sha256').update(bytes).digest('hex');
const exe = realpathSync(args.exe);
assert.equal(hash(readFileSync(exe)), args['exe-sha256']);
const out = resolve(args.out);
mkdirSync(dirname(out), { recursive: true });
mkdirSync(out);
const root = mkdtempSync(join(realpathSync(tmpdir()), 'aw-scripture-acceptance-'));
const data = join(root, 'app-data');
const webview = join(root, 'webview2');
mkdirSync(data);
mkdirSync(webview);
writeFileSync(join(root, 'acceptance-profile'), 'Abiding Word Scripture acceptance');
const exportsDir = join(out, 'exports');
mkdirSync(exportsDir);
const origins = Object.entries(scriptureFixtures()).map(([name, bytes]) => {
  const path = join(out, name);
  writeFileSync(path, bytes);
  return { path, sha256: hash(bytes), format: name.split('.').at(-1) };
});
const report = { sourceSha: args['source-sha'], executableSha256: args['exe-sha256'], root, data, webview, origins, cases: [], automated: false, ownerAccepted: false };
const reportPath = join(out, 'recognition-smoke.json');
let app;
let socket;

async function port() {
  const server = createServer().listen(0, '127.0.0.1');
  await once(server, 'listening');
  const result = server.address().port;
  await new Promise((done) => server.close(done));
  return result;
}

async function stop() {
  socket?.close();
  socket = undefined;
  if (!app || app.exitCode !== null || app.signalCode !== null) return;
  const closed = once(app, 'exit');
  app.kill();
  await closed;
}

async function launch() {
  const listener = await port();
  app = spawn(exe, [], { stdio: 'ignore', env: {
    ...process.env,
    ABIDING_WORD_DATA_DIR: data,
    WEBVIEW2_USER_DATA_FOLDER: webview,
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${listener} --remote-debugging-address=127.0.0.1`,
  } });
  await once(app, 'spawn');
  const expires = Date.now() + 45_000;
  while (Date.now() < expires) {
    assert.equal(app.exitCode, null, 'The acceptance application exited before connecting.');
    const targets = await fetch(`http://127.0.0.1:${listener}/json/list`, { signal: AbortSignal.timeout(1_000) })
      .then(response => response.json()).catch(() => []);
    const target = targets.find(t => t.type === 'page' && t.webSocketDebuggerUrl && t.url.includes('tauri.localhost'));
    if (target) {
      socket = new WebSocket(target.webSocketDebuggerUrl);
      await once(socket, 'open', { signal: AbortSignal.timeout(5_000) });
      const info = await invoke('app_info');
      assert.equal(info.ready, true);
      assert.equal(realpathSync(info.dataDirectory).toLowerCase(), realpathSync(data).toLowerCase());
      assert.equal(info.updatesEnabled, false);
      assert.equal(info.schemaVersion, 28);
      report.cases.push({ launch: info });
      return;
    }
    await delay(250);
  }
  throw new Error('The installed application did not expose its WebView2 target.');
}

let next = 0;
async function evaluate(expression) {
  const id = ++next;
  const connected = socket;
  const result = new Promise((accept, reject) => {
    const finish = () => {
      clearTimeout(timer);
      connected.removeEventListener('message', receive);
    };
    const receive = (event) => {
      const answer = JSON.parse(event.data);
      if (answer.id !== id) return;
      finish();
      if (answer.error || answer.result.exceptionDetails) {
        reject(new Error(JSON.stringify(answer.error ?? answer.result.exceptionDetails)));
      } else accept(answer.result.result.value);
    };
    const timer = setTimeout(() => {
      finish();
      reject(new Error('Timed out waiting for the native app.'));
    }, 180_000);
    connected.addEventListener('message', receive);
  });
  connected.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: { expression, awaitPromise: true, returnByValue: true } }));
  return result;
}
const invoke = (command, input = {}) =>
  evaluate(`window.__TAURI_INTERNALS__.invoke(${JSON.stringify(command)}, ${JSON.stringify(input)})`);
const chapter = translationId => invoke('scripture_get_chapter', { input: { translationId, book: 'RUT', chapter: 1 } });

async function checkEdition(fixture, bible) {
  const text = await chapter(bible.id);
  assert.equal(text.verses.find(v => v.verse === 1)?.text, verseText(1, 1));
  const verified = await invoke('scripture_search', { input: { query: 'c1v1', translationId: bible.id } });
  assert.equal(verified.hits[0]?.text, verseText(1, 1));
  if (fixture.format === 'epub') {
    assert.ok(text.unchecked.some(v => v.verse === 2));
    assert.ok(!text.verses.some(v => v.verse === 2));
    const results = await invoke('scripture_search', { input: { query: 'c1v2', translationId: bible.id } });
    assert.equal(results.hits.length, 0);
  }
  const origin = await invoke('bible_verse_origin', { translationId: bible.id, book: 'RUT', chapter: 1, verse: 1 });
  assert.equal(origin.sourceSha256, fixture.sha256);
  if (fixture.format === 'epub') {
    assert.equal(origin.entryPath, 'OEBPS/c1.xhtml');
    assert.ok(origin.provenance.spans.length > 0);
    assert.equal(origin.exclusions[0].text, '[a]');
  }
  const found = await invoke('material_scripture', { input: { translationId: bible.id, references: ['RUT.1.1-RUT.1.2'] } });
  assert.equal(found.passages[0].verses[0].text, verseText(1, 1));
  if (fixture.format === 'epub') assert.equal(found.passages[0].verses.length, 1);
  const files = readdirSync(join(data, 'sources', 'bibles', bible.id));
  assert.equal(files.length, 1);
  assert.equal(hash(readFileSync(join(data, 'sources', 'bibles', bible.id, files[0]))), fixture.sha256);
}

async function checkProfileGuard() {
  const invalid = join(root, 'invalid-app-data');
  mkdirSync(invalid);
  const probe = spawn(exe, [], { stdio: 'ignore', env: {
    ...process.env,
    ABIDING_WORD_DATA_DIR: invalid,
    WEBVIEW2_USER_DATA_FOLDER: webview,
  } });
  try {
    const [code] = await once(probe, 'exit', { signal: AbortSignal.timeout(10_000) });
    assert.equal(code, 78, 'The binary did not reject the mismatched disposable profile.');
    report.cases.push({ mismatchedProfileRejected: true });
  } finally {
    if (probe.exitCode === null && probe.signalCode === null) probe.kill();
  }
}

try {
  await checkProfileGuard();
  await launch();
  const editions = [];
  for (const fixture of origins.slice(0, 3)) {
    const preview = await invoke('bible_import_preview', { input: { paths: [fixture.path] } });
    assert.equal(preview.format, fixture.format);
    let reviewed = preview;
    if (fixture.format === 'epub') {
      const heading = preview.document.books.find(book => book.book === 'RUT');
      assert.ok(heading);
      assert.ok(preview.document.unverifiedVerses > 0);
      reviewed = await invoke('bible_import_correct', { previewId: preview.previewId, books: [{ index: heading.index, book: 'RUT' }] });
      assert.equal(reviewed.document.unverifiedVerses, preview.document.unverifiedVerses);
    }
    if (fixture.format === 'docx') {
      const heading = preview.document.books.find(book => book.heading === 'The Story of the Moabite Woman');
      assert.ok(heading);
      reviewed = await invoke('bible_import_correct', { previewId: preview.previewId, books: [{ index: heading.index, book: 'RUT' }] });
    }
    assert.equal(reviewed.document.versesUsable, true);
    const bible = await invoke('bible_import_commit', { input: {
      previewId: reviewed.previewId, name: `Invented Ruth ${fixture.format.toUpperCase()}`,
      shortName: fixture.format.toUpperCase(), language: 'English', languageCode: 'en',
      confirmEdition: fixture.format === 'epub',
    } });
    assert.equal(bible.editionConfirmed, fixture.format === 'epub');
    await checkEdition(fixture, bible);
    editions.push({ fixture, bible });
    report.cases.push({ imported: fixture.format, bible: bible.id, verifiedVerses: bible.verseCount });
  }
  const scan = await invoke('bible_import_preview', { input: { paths: [origins[3].path] } });
  assert.equal(scan.document.files[0].scanned, true);
  assert.equal(scan.document.versesUsable, false);
  await invoke('bible_import_cancel', { previewId: scan.previewId });

  const workspace = await invoke('workspace_get_default');
  const project = await invoke('project_create', { input: { workspaceId: workspace.id, title: 'Invented Scripture Acceptance' } });
  const study = await invoke('study_create', { input: { projectId: project.id, title: 'Invented Ruth Study', primaryPassage: 'RUT.1.1' } });
  const note = study.sections.flatMap(section => section.blocks)[0];
  assert.ok(note, 'The invented Study has no writable section.');
  await invoke('study_block_update', { input: { id: note.id, content: 'INVENTED_STUDY_NOTE_ACCEPTANCE' } });
  const teaching = await invoke('teaching_create', { input: {
    studyId: study.study.id, title: 'Invented Ruth Teaching', kind: 'bible_study', templateVersion: 2,
    blocks: [
      { key: 'reading', blockType: 'scripture', title: 'Public reading', participantFacing: true },
      { key: 'leader', blockType: 'teaching_notes', title: 'Leader notes', body: 'PRIVATE_LEADER_ACCEPTANCE', participantFacing: false },
      { key: 'private', blockType: 'scripture', title: 'Private reading', participantFacing: false },
    ],
  } });
  for (const [index, reference] of ['RUT.1.1-RUT.1.2', 'RUT.1.3'].entries()) {
    const block = teaching.blocks[index === 0 ? 0 : 2];
    await invoke('teaching_passage_add', { input: { preparationId: teaching.id, blockId: block.id, reference } });
  }
  await invoke('settings_update', { input: { scriptureTranslationId: editions[0].bible.id, scripturePassage: 'RUT.1' } });
  report.cases.push({ study: study.study.id, teaching: teaching.id, scannedRefusal: true });
  await stop();
  await launch();
  const bibles = await invoke('bible_list');
  for (const { fixture, bible } of editions) {
    assert.equal(bibles.find(candidate => candidate.id === bible.id)?.editionConfirmed, bible.editionConfirmed);
    await checkEdition(fixture, bible);
  }
  const docx = editions.find(({ fixture }) => fixture.format === 'docx');
  const restoredCorrection = await invoke('bible_import_preview', { input: { paths: [docx.fixture.path] } });
  assert.equal(restoredCorrection.document.versesUsable, true);
  assert.ok(restoredCorrection.document.saved.some(choice => choice.chosen === 'RUT'));
  await invoke('bible_import_cancel', { previewId: restoredCorrection.previewId });
  const reopenedStudy = await invoke('study_get', { id: study.study.id });
  assert.equal(reopenedStudy.study.id, study.study.id);
  assert.equal(reopenedStudy.sections.flatMap(section => section.blocks).find(block => block.id === note.id)?.content, 'INVENTED_STUDY_NOTE_ACCEPTANCE');
  assert.equal((await invoke('teaching_get', { id: teaching.id })).id, teaching.id);
  report.automated = true;
  writeFileSync(reportPath, JSON.stringify(report, null, 2));
  console.log(`Automated checks passed. Use the open application for the owner checks in the acceptance checklist. Save outputs under ${exportsDir}.`);
  const prompt = createInterface({ input: process.stdin, output: process.stdout });
  try {
    const answer = await prompt.question('Type PASS only after completing all owner checks, or Enter to leave acceptance pending. ');
    if (answer === 'PASS') {
      const markdown = readFileSync(join(exportsDir, 'participant.md'), 'utf8');
      const office = execFileSync('powershell.exe', ['-NoProfile', '-Command', [
        'Add-Type -AssemblyName System.IO.Compression.FileSystem;',
        '$zip = [IO.Compression.ZipFile]::OpenRead($env:AW_ACCEPTANCE_DOCX);',
        'try {',
        "$entry = $zip.GetEntry('word/document.xml');",
        '$reader = [IO.StreamReader]::new($entry.Open());',
        'try { [xml]$xml = $reader.ReadToEnd() } finally { $reader.Dispose() };',
        '$xml.DocumentElement.InnerText',
        '} finally { $zip.Dispose() }',
      ].join(' ')], { encoding: 'utf8', env: { ...process.env, AW_ACCEPTANCE_DOCX: join(exportsDir, 'participant.docx') } });
      for (const exported of [markdown, office]) {
        assert.ok(exported.includes(verseText(1, 1)), 'Verified public Scripture is missing from participant output.');
        for (const privateValue of ['PRIVATE_LEADER_ACCEPTANCE', 'c1v2', 'c1v3']) {
          assert.ok(!exported.includes(privateValue), `Unexpected participant export content: ${privateValue}`);
        }
      }
      report.exports = ['participant.md', 'participant.docx'].map(name => ({ name, sha256: hash(readFileSync(join(exportsDir, name))) }));
      report.ownerAccepted = true;
    }
  } finally {
    prompt.close();
  }
  for (const fixture of origins) assert.equal(hash(readFileSync(fixture.path)), fixture.sha256);
} catch (error) {
  report.ownerAccepted = false;
  report.error = String(error);
  process.exitCode = 1;
  console.error(error);
} finally {
  writeFileSync(reportPath, JSON.stringify(report, null, 2));
  await stop();
}
