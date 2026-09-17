import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import test from 'node:test';
import {verifySdkEvidence} from './sdk-evidence.mjs';

const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const encode = (value) => Buffer.from(JSON.stringify(value));
const cases = ['attachment-assets', 'modeled-vectors', 'input-validation-recovery',
  'transport-failure-recovery', 'pre-init-privacy', 'delayed-init', 'intent-boundaries',
  'text-response-bounds', 'text-safe-rendering', 'detached-session'];
const report = () => ({schema: 'prismpm/hologram-oracle/2', footer_verified: true,
  guest_allocation_boundary: 'verified', direct_vectors: 10, resident_vectors: 10,
  intent_vectors: 7, view_attached: 1, view_detached: 1,
  application_kappa: `blake3:${'a'.repeat(64)}`, archive_kappa: `blake3:${'b'.repeat(64)}`,
  archive_fingerprint: 'c'.repeat(64), portable_browser: {
    schema: 'prismpm/portable-browser-oracle/1', profile: 'utf8-text', engine: 'chromium',
    browser_version: '151.0.7922.34', playwright: '1.62.1',
    cases: cases.map((name) => ({name, status: 'passed', attempts: 1})),
    vector_indexes: [0, 1, 2, 3, 4], skipped: 0, retries: 0, status: 'passed',
  }});
const fixture = (changeReport = () => {}, changeManifest = () => {}) => {
  const build = {build_id: 'd'.repeat(64), source_id: 'e'.repeat(64)};
  const modelBytes = encode({schema: 'prismpm/model-document/2'});
  const acceptanceBytes = encode({schema: 'prismpm/application-acceptance/1',
    build_id: build.build_id, source_id: build.source_id, status: 'verified',
    hologram_oracle: 'verified', holo: {application_kappa: `blake3:${'a'.repeat(64)}`,
      archive_kappa: `blake3:${'b'.repeat(64)}`, archive_fingerprint: 'c'.repeat(64)}});
  const value = report();
  changeReport(value);
  const manifest = {schema: 'prismpm/application-verification-manifest/1',
    build_id: build.build_id, model_sha256: sha256(modelBytes),
    acceptance_sha256: sha256(acceptanceBytes), processes: [{tool: 'hologram-oracle',
      exit_code: 0, executable_sha256: 'f'.repeat(64),
      stdout: JSON.stringify(value), stderr: ''}]};
  changeManifest(manifest);
  const manifestBytes = encode(manifest);
  return {build, modelBytes, acceptanceBytes, manifestBytes,
    attestationId: sha256(manifestBytes)};
};

test('current build evidence contains the complete portable Text View execution', () => {
  verifySdkEvidence(fixture());
});

test('recording-only, incomplete, wrong-profile and skipped SDK evidence fails closed', () => {
  const mutations = [
    (value) => {value.schema = 'prismpm/hologram-oracle/1'; delete value.portable_browser;},
    (value) => {delete value.application_kappa;},
    (value) => {delete value.archive_kappa;},
    (value) => {delete value.archive_fingerprint;},
    (value) => {value.application_kappa = `blake3:${'0'.repeat(64)}`;},
    (value) => {value.archive_kappa = `blake3:${'0'.repeat(64)}`;},
    (value) => {value.archive_fingerprint = '0'.repeat(64);},
    (value) => {value.extra = true;},
    (value) => {value.footer_verified = false;},
    (value) => {value.direct_vectors = 9;},
    (value) => {value.resident_vectors = 9;},
    (value) => {value.intent_vectors = 6;},
    (value) => {value.view_attached = 0;},
    (value) => {value.view_detached = 0;},
    (value) => {value.guest_allocation_boundary = 'skipped';},
    (value) => {value.portable_browser.profile = 'legacy-numeric';},
    (value) => {value.portable_browser.engine = 'firefox';},
    (value) => {value.portable_browser.browser_version = 'unverified';},
    (value) => {value.portable_browser.playwright = '1.61.0';},
    (value) => {value.portable_browser.cases.pop();},
    (value) => {value.portable_browser.cases[0].status = 'skipped';},
    (value) => {value.portable_browser.cases[0].attempts = 2;},
    (value) => {value.portable_browser.vector_indexes.pop();},
    (value) => {value.portable_browser.skipped = 1;},
    (value) => {value.portable_browser.retries = 1;},
    (value) => {value.portable_browser.status = 'failed';},
  ];
  for (const mutate of mutations) {
    assert.throws(() => verifySdkEvidence(fixture(mutate)), mutate.toString());
  }
});

test('manifest identity, build binding and actual successful process are required', () => {
  for (const mutate of [
    (value) => {value.schema = 'other';},
    (value) => {value.build_id = '0'.repeat(64);},
    (value) => {value.model_sha256 = '0'.repeat(64);},
    (value) => {value.acceptance_sha256 = '0'.repeat(64);},
    (value) => {value.processes = [];},
    (value) => {value.processes.push(structuredClone(value.processes[0]));},
    (value) => {value.processes[0].exit_code = 1;},
    (value) => {value.processes[0].tool = 'hologram-oracle-build';},
    (value) => {value.processes[0].executable_sha256 = 'invalid';},
    (value) => {value.processes[0].stdout = '{}';},
  ]) assert.throws(() => verifySdkEvidence(fixture(undefined, mutate)), mutate.toString());
  const stale = fixture();
  stale.attestationId = '0'.repeat(64);
  assert.throws(() => verifySdkEvidence(stale));
});
