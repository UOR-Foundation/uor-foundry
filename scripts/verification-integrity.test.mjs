import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {mkdirSync, mkdtempSync, readFileSync, rmSync, symlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {test} from 'node:test';
import {verifyBrowserArtifacts, verifyExecutableVectors, verifyGeneratedReachability} from './verify-browser-wasm.mjs';

const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const root = fileURLToPath(new URL('..', import.meta.url));
const build = {build_id: 'a'.repeat(64), source_id: 'b'.repeat(64)};

// Synthetic metadata is used only to exercise rejection boundaries, never as
// generated application or production acceptance evidence.
function fixture(t) {
  const project = mkdtempSync(join(tmpdir(), 'foundry-integrity-fixture-'));
  t.after(() => rmSync(project, {recursive: true, force: true}));
  const base = join(project, '.prism/build', build.build_id);
  const write = (path, bytes) => {
    mkdirSync(dirname(path), {recursive: true});
    writeFileSync(path, bytes);
  };
  write(join(project, 'src/Sample.lex.tex'), 'synthetic non-product source');
  const files = new Map([
    ['Foundry.holo', Buffer.from('not a product artifact')],
    ['model.prism.json', Buffer.from(JSON.stringify({schema: 'prismpm/model-document/4', application: {
      profile: 'prismpm/browser-application/1', entry_root: 'PrismFoundry.Foundry.dispatch',
      acceptance_vectors: [{request: [1], response: [1]}],
    }}))],
    ['core-wasm/prism_foundry_web_core_wasm.wasm', Buffer.from('not wasm')],
    ['view/browser/prism_foundry_web_bg.wasm', Buffer.from('not wasm')],
    ['view/browser/prism_foundry_web.js', Buffer.from('not javascript')],
    ['view/browser/app.css', Buffer.from('synthetic')],
    ['view/browser/app.js', Buffer.from('synthetic')],
    ['view/browser/index.html', Buffer.from('synthetic')],
    ['lexlean/snapshot.json', Buffer.from(JSON.stringify({source_id: build.source_id, modules: [{source: {
      path: 'src/Sample.lex.tex', sha256: sha256('synthetic non-product source'),
    }}]}))],
  ]);
  const manifest = {schema: 'prismpm/build-manifest/1',
    inputs: {lexlean_source_id: build.source_id}, files: []};
  for (const [path, bytes] of files) {
    write(join(base, path), bytes);
    manifest.files.push({path, byte_length: bytes.length, sha256: sha256(bytes)});
  }
  const saveManifest = () => write(join(base, 'manifest.json'), JSON.stringify(manifest));
  saveManifest();
  return {project, base, manifest, saveManifest, write};
}

test('missing generated artifacts fail rather than skip', async () => {
  const root = mkdtempSync(join(tmpdir(), 'foundry-missing-build-'));
  try {
    await assert.rejects(verifyBrowserArtifacts(root, {}), /build identity/);
  } finally {
    rmSync(root, {recursive: true, force: true});
  }
});

test('valid IDs cannot hide a missing build directory', async (t) => {
  const f = fixture(t);
  await assert.rejects(verifyBrowserArtifacts(f.project, {...build, build_id: 'c'.repeat(64)}), /ENOENT/);
});

test('stale SDK source identity is rejected', async (t) => {
  const f = fixture(t);
  await assert.rejects(verifyBrowserArtifacts(f.project, {...build, source_id: 'c'.repeat(64)}), /stale build/);
});

test('source changes invalidate retained build output', async (t) => {
  const f = fixture(t);
  f.write(join(f.project, 'src/Sample.lex.tex'), 'changed source');
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /source changed since build/);
});

test('uncompiled added sources invalidate retained build output', async (t) => {
  const f = fixture(t);
  f.write(join(f.project, 'src/Added.lex.tex'), 'added source');
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /all current application sources/);
});

test('substituted artifact bytes are rejected', async (t) => {
  const f = fixture(t);
  f.write(join(f.base, 'Foundry.holo'), 'not a product artifacT');
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /artifact digest mismatch/);
});

test('missing artifact bytes are rejected', async (t) => {
  const f = fixture(t);
  rmSync(join(f.base, 'Foundry.holo'));
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /ENOENT/);
});

test('empty Wasm is rejected even when its metadata agrees', async (t) => {
  const f = fixture(t);
  const path = 'view/browser/prism_foundry_web_bg.wasm';
  f.write(join(f.base, path), '');
  Object.assign(f.manifest.files.find((entry) => entry.path === path), {byte_length: 0, sha256: sha256('')});
  f.saveManifest();
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /required non-empty artifact missing/);
});

test('duplicate artifact inventory entries are rejected', async (t) => {
  const f = fixture(t);
  f.manifest.files.push({...f.manifest.files[0]});
  f.saveManifest();
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /duplicate artifact/);
});

test('artifact paths cannot escape the selected build', async (t) => {
  const f = fixture(t);
  f.manifest.files[0].path = '../other/Foundry.holo';
  f.saveManifest();
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /artifact path must be confined/);
});

test('artifact symlinks cannot substitute outside content', async (t) => {
  const f = fixture(t);
  rmSync(join(f.base, 'Foundry.holo'));
  symlinkSync(join(f.project, 'src/Sample.lex.tex'), join(f.base, 'Foundry.holo'));
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /symlink artifact/);
});

test('bound echo-only metadata is rejected before executing untrusted bindings', async (t) => {
  const f = fixture(t);
  await assert.rejects(verifyBrowserArtifacts(f.project, build), /echo-only/);
});

test('empty vectors cannot establish application acceptance', () => {
  assert.throws(() => verifyExecutableVectors([], (value) => value), /non-empty/);
});

test('the actual declared Foundry echo vectors are rejected', () => {
  const text = (value) => [...Buffer.from(value)];
  const vectors = ['Hello, Foundry.', 'Status: Ready'].map((value) => ({
    request: text(value), response: text(value),
  }));
  assert.throws(() => verifyExecutableVectors(vectors, (value) => value), /echo-only/);
});

test('an echo implementation cannot satisfy a non-echo vector', () => {
  assert.throws(() => verifyExecutableVectors([{request: [1], response: [2]}],
    (value) => value), /generated output/);
});

test('a non-echo vector checks exact generated output without claiming product completion', () => {
  assert.equal(verifyExecutableVectors([{request: [1], response: [2]}],
    () => Uint8Array.of(2)), 1);
});

function disconnectedRuntime() {
  const entry = 'PrismFoundry.Foundry.';
  const declarations = ['dispatch', 'present', 'replay'].map((name) => ({
    kind: 'definition', lean_name: name,
    linked_ir: {kind: 'definition', name, body: {kind: 'var', name: 'request'}},
  }));
  return {
    snapshot: {modules: [{name: 'Foundry', lean_module: 'PrismFoundry.Foundry', declarations}]},
    application: {entry_root: `${entry}dispatch`, view: {presentation_root: `${entry}present`},
      durability: {replay_root: `${entry}replay`},
      library_roots: ['dispatch', 'present', 'replay'].map((name) => entry + name)},
  };
}

test('entry definitions must reach the modeled implementation, not just compile', () => {
  const {snapshot, application} = disconnectedRuntime();
  assert.throws(() => verifyGeneratedReachability(snapshot, application), /cannot reach required model/);
});

test('imports, source descriptions, and caller completion flags cannot establish reachability', () => {
  const {snapshot, application} = disconnectedRuntime();
  snapshot.modules[0].imports = ['Foundry.Core.Identity'];
  snapshot.modules[0].declarations.push({kind: 'theorem', linked_ir: {
    kind: 'theorem', statement: {kind: 'call', function: {module: 'Foundry.Core.Identity', name: 'enroll'}},
  }});
  Object.assign(application, {complete: true, standards_verified: true, accessible: true});
  assert.throws(() => verifyGeneratedReachability(snapshot, application), /cannot reach required model/);
});

test('unresolved domain calls cannot count as generated behavior', () => {
  const {snapshot, application} = disconnectedRuntime();
  snapshot.modules[0].declarations[0].linked_ir.body = {
    kind: 'call', function: {module: 'Foundry.Core.Identity', name: 'enroll'}, arguments: [],
  };
  assert.throws(() => verifyGeneratedReachability(snapshot, application), /unresolved generated call/);
});

test('presentation and replay must bind actual exported model roots', () => {
  const {snapshot, application} = disconnectedRuntime();
  application.view.presentation_root = application.entry_root;
  assert.throws(() => verifyGeneratedReachability(snapshot, application));
});

test('transitive generated call coverage is accepted only as static coverage', () => {
  const {snapshot, application} = disconnectedRuntime();
  const groups = {
    dispatch: ['Core.Identity', 'Core.BackupCodes', 'Core.Authority', 'Core.Organization',
      'Core.Site', 'Storage.Kappa', 'Storage.ObjectSpace', 'Network.Veilid', 'Network.Resilience',
      'Security.Holospaces', 'Services.Workflows', 'Services.AIInference', 'Services.Messaging',
      'Services.Governance', 'Services.Finance', 'Services.Learning', 'Services.Brand'],
    present: ['UI.App', 'UI.Components'], replay: ['Storage.Kappa', 'Storage.ObjectSpace'],
  };
  const call = (module, name) => ({kind: 'call', function: {module, name}, arguments: []});
  for (const [name, modules] of Object.entries(groups)) {
    snapshot.modules[0].declarations.find((declaration) => declaration.lean_name === name)
      .linked_ir.body = call('Foundry', `${name}Helper`);
    snapshot.modules[0].declarations.push({kind: 'definition', lean_name: `${name}Helper`, linked_ir: {
      kind: 'definition', name: `${name}Helper`, body: {fields: modules.map((module) =>
        ({value: call(`Foundry.${module}`, 'operation')}))},
    }});
  }
  for (const name of new Set(Object.values(groups).flat())) {
    snapshot.modules.push({name: `Foundry.${name}`, lean_module: `PrismFoundry.Foundry.${name}`,
      declarations: [{kind: 'definition', lean_name: 'operation', linked_ir: {
        kind: 'definition', name: 'operation', body: {kind: 'var', name: 'synthetic'},
      }}]});
  }
  assert.equal(verifyGeneratedReachability(snapshot, application), undefined);
  // No product acceptance or source behavior is inferred from the synthetic graph.
});

test('oracle ingestion reports its limited scope instead of product acceptance', () => {
  const output = execFileSync(process.execPath, ['scripts/verify-authoritative-oracles.mjs'], {
    cwd: root, encoding: 'utf8', timeout: 30_000,
  });
  assert.match(output, /Oracle input integrity only; product, standards conformance and accessibility are not established/);
  assert.doesNotMatch(output, /ALL AUTHORITATIVE ORACLES AND TEST VECTORS VERIFIED SUCCESSFULLY/);
  assert.ok(readFileSync(join(root, 'SPEC.md'), 'utf8').includes('complete applicable authoritative oracle/assessment coverage'));
});
