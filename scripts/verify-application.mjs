import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdirSync, readFileSync, rmSync} from 'node:fs';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {verifyBrowserReport} from './browser-report.mjs';
import {copyRegularTree, createReproductionDirectory, verifySameTree} from './reproduction.mjs';
import {verifySdkEvidence} from './sdk-evidence.mjs';

process.chdir(fileURLToPath(new URL('../', import.meta.url)));
assert.ok(readFileSync('src/Foundry.lex.tex').length, 'Foundry source is required');
assert.equal(process.env.PRISMPM_SDK_INVENTORY, '/opt/prismpm/share/inventory.json',
  'run the full acceptance gate in the immutable prismpm.lock SDK');
const run = (command, args) => execFileSync(command, args, {
  encoding: 'utf8', timeout: 1_800_000, maxBuffer: 64 * 1024 * 1024,
  stdio: ['ignore', 'pipe', 'inherit'],
});
run('/usr/local/bin/prismpm', ['template', 'check']);
run('/usr/local/bin/prismpm', ['lock', 'check']);
run(process.execPath, ['--test', 'scripts/browser-report.test.mjs', 'scripts/reproduction.test.mjs',
  'scripts/sdk-evidence.test.mjs']);
const build = JSON.parse(run('/usr/local/bin/prismpm', ['--json', 'build']));
const verification = JSON.parse(run('/usr/local/bin/prismpm', ['--json', 'verify']));
assert.equal(build.build_id, verification.build_id);
assert.match(build.build_id, /^[0-9a-f]{64}$/);
assert.match(verification.attestation_id, /^[0-9a-f]{64}$/);
const buildRoot = `.prism/build/${build.build_id}`;
const reproduction = createReproductionDirectory(process.cwd());
try {
  for (const name of ['first', 'second']) {
    const destination = join(reproduction, name);
    mkdirSync(destination);
    for (const path of ['src', 'lake-manifest.json', 'lakefile.toml', 'lean-toolchain', 'lexlean.lock', 'lexlean.toml',
      'prismpm.lock', 'prismpm.toml', 'standards.lock']) {
      copyRegularTree(path, join(destination, path));
    }
    // These are exact inputs already verified by the SDK, never build outputs.
    mkdirSync(join(destination, '.prism/sdk'), {recursive: true});
    for (const path of ['inputs', 'stdlib-manifest.json']) {
      copyRegularTree(`.prism/sdk/${path}`, join(destination, '.prism/sdk', path));
    }
    const repeated = JSON.parse(run('/usr/local/bin/prismpm', [
      '--project', destination, '--json', 'build']));
    assert.equal(repeated.build_id, build.build_id, 'clean absolute roots must reproduce');
    verifySameTree(buildRoot, join(destination, '.prism/build', repeated.build_id));
  }
} finally {
  rmSync(reproduction, {recursive: true, force: true});
}
const modelBytes = readFileSync(`${buildRoot}/model.prism.json`);
const verifiedRoot = `.prism/verified/${verification.attestation_id}`;
const acceptanceBytes = readFileSync(`${verifiedRoot}/application-acceptance.json`);
verifySdkEvidence({build, modelBytes, acceptanceBytes,
  manifestBytes: readFileSync(`${verifiedRoot}/manifest.json`), attestationId: verification.attestation_id});
const model = JSON.parse(modelBytes);
const acceptance = JSON.parse(acceptanceBytes);
assert.equal(model.schema, 'prismpm/model-document/4');
assert.equal(model.application.profile, 'prismpm/browser-application/1');
assert.equal(model.application.name, 'Foundry');
assert.equal(model.application.cargo_name, 'prism-foundry-web');
assert.equal(model.application.cargo_version, '0.1.0');
assert.equal(model.application.cargo_repository, 'https://github.com/UOR-Foundation/uor-foundry');
assert.equal(model.application.cargo_homepage, 'https://uor.foundation/foundry-web/');
assert.equal(model.application.entry_root, 'PrismFoundry.Foundry.dispatch');
assert.deepEqual(model.application.library_roots, [
  'PrismFoundry.Foundry.dispatch',
  'PrismFoundry.Foundry.present',
  'PrismFoundry.Foundry.replay'
]);
assert.equal(model.application.core_contract, 'hologram:guest/core-wasm@1');
assert.equal(model.application.capabilities_empty, true);
assert.equal(model.application.fat_archive, true);
assert.equal(model.application.primary_layer, 0);
assert.equal(model.application.view_layer, 1);
assert.equal(model.application.request_maximum, 4096);
assert.equal(model.application.response_maximum, 8192);
assert.equal(model.application.guest_allocation_maximum, 8192);
const expected = [
  {
    request: [...Buffer.from('Hello, Foundry.')],
    response: [...Buffer.from('Hello, Foundry.')],
  },
  {
    request: [...Buffer.from('Status: Ready')],
    response: [...Buffer.from('Status: Ready')],
  },
];
assert.deepEqual(model.application.acceptance_vectors, expected,
  'all independent browser-application acceptance vectors are required');
assert.equal(acceptance.application, model.application.name);
assert.equal(acceptance.build_id, build.build_id);
assert.equal(acceptance.artifact_closure, 'verified');
assert.equal(acceptance.browser_projection, 'verified');
assert.equal(acceptance.core_wasm.status, 'verified');
assert.equal(acceptance.cargo_package.name, model.application.cargo_name);
assert.equal(acceptance.cargo_package.version, model.application.cargo_version);
assert.equal(acceptance.hologram_oracle, 'verified');
assert.equal(acceptance.modeled_vectors, expected.length);
const browserStartedAt = Date.now();
const browserReport = JSON.parse(execFileSync('npm', ['exec', '--offline', '--',
  'playwright', 'test', '--reporter=json'], {
  env: {...process.env, FOUNDRY_BUILD_ROOT: buildRoot},
  encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], timeout: 180_000,
  maxBuffer: 16 * 1024 * 1024,
}));
verifyBrowserReport(browserReport, process.cwd(), browserStartedAt, Date.now());
console.log(`FW-01 verified build ${build.build_id}`);
