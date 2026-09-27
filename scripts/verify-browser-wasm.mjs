// Artifact/vector integrity is necessary, never complete product acceptance.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {lstatSync, readFileSync, readdirSync, realpathSync} from 'node:fs';
import {join, resolve, sep} from 'node:path';
import {fileURLToPath, pathToFileURL} from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const requiredFiles = ['Foundry.holo', 'model.prism.json',
  'core-wasm/prism_foundry_web_core_wasm.wasm',
  'view/browser/prism_foundry_web_bg.wasm', 'view/browser/prism_foundry_web.js',
  'view/browser/app.css', 'view/browser/app.js', 'view/browser/index.html'];

function regularFile(base, relative) {
  assert.ok(!lstatSync(base).isSymbolicLink(), 'symlink artifact/source root is forbidden');
  assert.equal(typeof relative, 'string', 'artifact path must be a string');
  assert.ok(relative.length > 0 && !relative.startsWith('/')
    && !relative.includes('\\') && !relative.split('/').some((part) => !part || part === '..' || part === '.'),
  'artifact path must be confined');
  let path = base;
  for (const part of relative.split('/')) {
    path = join(path, part);
    assert.ok(!lstatSync(path).isSymbolicLink(), 'symlink artifact/source is forbidden');
  }
  assert.ok(realpathSync(path).startsWith(realpathSync(base) + sep), 'artifact escapes root');
  assert.ok(lstatSync(path).isFile(), 'artifact/source must be a regular file');
  assert.equal(lstatSync(path).nlink, 1, 'hardlinked artifact/source is forbidden');
  return readFileSync(path);
}

function sourcePaths(base, relative = 'src') {
  const paths = [];
  for (const entry of readdirSync(join(base, relative), {withFileTypes: true})) {
    assert.ok(!entry.isSymbolicLink(), 'source symlinks are forbidden');
    const path = `${relative}/${entry.name}`;
    if (entry.isDirectory()) paths.push(...sourcePaths(base, path));
    else if (entry.name.endsWith('.lex.tex')) paths.push(path);
  }
  return paths.sort();
}

function validateVectors(vectors) {
  assert.ok(Array.isArray(vectors) && vectors.length > 0, 'non-empty application vectors required');
  for (const vector of vectors) {
    for (const field of ['request', 'response']) {
      assert.ok(Array.isArray(vector[field]) && vector[field].every((byte) =>
        Number.isInteger(byte) && byte >= 0 && byte <= 255), 'vectors must contain exact bytes');
    }
  }
  assert.ok(vectors.some(({request, response}) => !Buffer.from(request).equals(Buffer.from(response))),
    'echo-only vectors do not establish Foundry behavior');
}

export function verifyExecutableVectors(vectors, invoke) {
  validateVectors(vectors);
  for (const {request, response} of vectors) {
    const output = invoke(Uint8Array.from(request));
    assert.ok(output instanceof Uint8Array, 'generated output must be bytes');
    assert.deepEqual([...output], response, 'generated output must match the declared vector');
  }
  return vectors.length;
}

// Static call reachability is a necessary anti-vacuity check, not evidence that
// every branch executes correctly or that any external standard is satisfied.
const runtimeCoverage = {
  'PrismFoundry.Foundry.dispatch': [
    'Foundry.Core.Identity', 'Foundry.Core.BackupCodes', 'Foundry.Core.Authority',
    'Foundry.Core.Organization', 'Foundry.Core.Site', 'Foundry.Storage.Kappa',
    'Foundry.Storage.ObjectSpace', 'Foundry.Network.Veilid', 'Foundry.Network.Resilience',
    'Foundry.Security.Holospaces', 'Foundry.Services.Workflows', 'Foundry.Services.AIInference',
    'Foundry.Services.Messaging', 'Foundry.Services.Governance', 'Foundry.Services.Finance',
    'Foundry.Services.Learning', 'Foundry.Services.Brand',
  ],
  'PrismFoundry.Foundry.present': ['Foundry.UI.App', 'Foundry.UI.Components'],
  'PrismFoundry.Foundry.replay': ['Foundry.Storage.Kappa', 'Foundry.Storage.ObjectSpace'],
};

export function verifyGeneratedReachability(snapshot, application) {
  assert.equal(application.entry_root, 'PrismFoundry.Foundry.dispatch');
  assert.equal(application.view?.presentation_root, 'PrismFoundry.Foundry.present');
  assert.equal(application.durability?.replay_root, 'PrismFoundry.Foundry.replay');
  const definitions = new Map();
  const roots = new Map();
  for (const module of snapshot.modules) {
    for (const declaration of module.declarations ?? []) {
      const ir = declaration.linked_ir;
      if (declaration.kind !== 'definition' || ir?.kind !== 'definition') continue;
      const key = `${module.name}.${ir.name}`;
      assert.ok(!definitions.has(key), 'duplicate generated definition');
      definitions.set(key, {module: module.name, body: ir.body});
      roots.set(`${module.lean_module}.${declaration.lean_name}`, key);
    }
  }
  const called = (body, module, result = []) => {
    if (!body || typeof body !== 'object') return result;
    if (body.kind === 'call') {
      assert.equal(typeof body.function?.name, 'string', 'generated call must resolve by name');
      result.push(`${body.function.module ?? module}.${body.function.name}`);
    }
    for (const value of Object.values(body)) {
      if (Array.isArray(value)) value.forEach((entry) => called(entry, module, result));
      else if (value && typeof value === 'object') called(value, module, result);
    }
    return result;
  };
  for (const [root, modules] of Object.entries(runtimeCoverage)) {
    assert.ok(application.library_roots?.includes(root), `missing generated export: ${root}`);
    assert.ok(roots.has(root), `missing generated root definition: ${root}`);
    const pending = [roots.get(root)];
    const seen = new Set();
    const reached = new Set();
    while (pending.length) {
      const key = pending.pop();
      if (seen.has(key)) continue;
      seen.add(key);
      const definition = definitions.get(key);
      assert.ok(definition, `unresolved generated call: ${key}`);
      reached.add(definition.module);
      pending.push(...called(definition.body, definition.module));
    }
    for (const module of modules) {
      assert.ok(reached.has(module), `${root} cannot reach required model: ${module}`);
    }
  }
}

export async function verifyBrowserArtifacts(projectRoot, build) {
  assert.match(build?.build_id ?? '', /^[0-9a-f]{64}$/, 'SDK build identity is required');
  assert.match(build?.source_id ?? '', /^[0-9a-f]{64}$/, 'SDK source identity is required');
  const buildRoot = join(projectRoot, '.prism/build', build.build_id);
  const manifest = JSON.parse(regularFile(buildRoot, 'manifest.json'));
  assert.equal(manifest.schema, 'prismpm/build-manifest/1');
  assert.equal(manifest.inputs?.lexlean_source_id, build.source_id, 'stale build source identity');
  assert.ok(Array.isArray(manifest.files) && manifest.files.length > 0, 'non-empty artifact inventory required');
  const inventory = new Map();
  for (const record of manifest.files) {
    assert.ok(!inventory.has(record.path), 'duplicate artifact inventory entry');
    const bytes = regularFile(buildRoot, record.path);
    assert.equal(bytes.length, record.byte_length, 'artifact byte length mismatch');
    assert.equal(sha256(bytes), record.sha256, 'artifact digest mismatch');
    inventory.set(record.path, bytes);
  }
  for (const path of [...requiredFiles, 'lexlean/snapshot.json']) {
    assert.ok(inventory.get(path)?.length > 0, `required non-empty artifact missing: ${path}`);
  }
  const snapshot = JSON.parse(inventory.get('lexlean/snapshot.json'));
  assert.equal(snapshot.source_id, build.source_id, 'stale source snapshot');
  assert.ok(Array.isArray(snapshot.modules) && snapshot.modules.length > 0, 'source modules required');
  const declaredSources = [];
  for (const module of snapshot.modules) {
    assert.equal(sha256(regularFile(projectRoot, module.source.path)), module.source.sha256,
      `source changed since build: ${module.source.path}`);
    if (module.source.path.startsWith('src/')) declaredSources.push(module.source.path);
  }
  assert.deepEqual([...new Set(declaredSources)].sort(), sourcePaths(projectRoot),
    'all current application sources must be in the SDK snapshot');
  const model = JSON.parse(inventory.get('model.prism.json'));
  assert.equal(model.schema, 'prismpm/model-document/4');
  assert.equal(model.application?.profile, 'prismpm/browser-application/1');
  assert.equal(model.application?.entry_root, 'PrismFoundry.Foundry.dispatch');
  validateVectors(model.application.acceptance_vectors);
  verifyGeneratedReachability(snapshot, model.application);
  const binding = pathToFileURL(join(buildRoot, 'view/browser/prism_foundry_web.js'));
  binding.searchParams.set('build', build.build_id);
  const {initSync, invoke_bytes: invoke} = await import(binding.href);
  assert.equal(typeof initSync, 'function');
  assert.equal(typeof invoke, 'function');
  initSync({module: inventory.get('view/browser/prism_foundry_web_bg.wasm')});
  return verifyExecutableVectors(model.application.acceptance_vectors, invoke);
}

// Process orchestration is separately testable; synthetic replies establish no
// SDK or product acceptance. The public verifier below fixes the actual binary.
export function selectVerifiedBuild(run) {
  run(['lock', 'check']);
  const build = JSON.parse(run(['--json', 'build']));
  const verification = JSON.parse(run(['--json', 'verify']));
  assert.equal(build.build_id, verification.build_id, 'verification must bind this build');
  assert.match(verification.attestation_id ?? '', /^[0-9a-f]{64}$/, 'SDK verification is required');
  return build;
}

export function verifyThroughLockedSdk(projectRoot = root) {
  const run = (args) => execFileSync('/usr/local/bin/prismpm', args, {
    cwd: projectRoot, encoding: 'utf8', timeout: 1_800_000, maxBuffer: 64 * 1024 * 1024,
    stdio: ['ignore', 'pipe', 'inherit'],
  });
  // Do not select a cached directory or accept a caller-supplied artifact path.
  return verifyBrowserArtifacts(projectRoot, selectVerifiedBuild(run));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  await verifyThroughLockedSdk();
  console.log('Generated artifact/vector integrity passed; complete product and browser acceptance remain required.');
}
