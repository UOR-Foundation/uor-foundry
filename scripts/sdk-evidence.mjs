import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';

const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const portableCases = ['attachment-assets', 'modeled-vectors', 'input-validation-recovery',
  'transport-failure-recovery', 'pre-init-privacy', 'delayed-init', 'intent-boundaries',
  'text-response-bounds', 'text-safe-rendering', 'detached-session'];

export function verifySdkEvidence({build, modelBytes, acceptanceBytes, manifestBytes, attestationId}) {
  assert.equal(sha256(manifestBytes), attestationId, 'manifest must bind the returned attestation');
  const manifest = JSON.parse(manifestBytes);
  const acceptance = JSON.parse(acceptanceBytes);
  assert.equal(manifest.schema, 'prismpm/application-verification-manifest/1');
  assert.equal(manifest.build_id, build.build_id);
  assert.equal(manifest.model_sha256, sha256(modelBytes));
  assert.equal(manifest.acceptance_sha256, sha256(acceptanceBytes));
  assert.equal(acceptance.schema, 'prismpm/application-acceptance/1');
  assert.equal(acceptance.build_id, build.build_id);
  assert.equal(acceptance.source_id, build.source_id);
  assert.equal(acceptance.status, 'verified');
  assert.equal(acceptance.hologram_oracle, 'verified');
  const processes = manifest.processes.filter((record) => record.tool === 'hologram-oracle');
  assert.equal(processes.length, 1, 'exactly one actual Hologram execution is required');
  const process = processes[0];
  assert.equal(process.exit_code, 0);
  assert.match(process.executable_sha256, /^[0-9a-f]{64}$/);
  const report = JSON.parse(process.stdout);
  assert.deepEqual(Object.keys(report).sort(), ['application_kappa', 'archive_fingerprint',
    'archive_kappa', 'direct_vectors', 'footer_verified', 'guest_allocation_boundary',
    'intent_vectors', 'portable_browser', 'resident_vectors', 'schema', 'view_attached', 'view_detached']);
  for (const field of ['application_kappa', 'archive_kappa', 'archive_fingerprint']) {
    assert.equal(report[field], acceptance.holo[field], 'oracle must execute this built Holo archive');
    assert.match(report[field], field === 'archive_fingerprint' ? /^[0-9a-f]{64}$/ : /^blake3:[0-9a-f]{64}$/);
  }
  assert.equal(report.schema, 'prismpm/hologram-oracle/2',
    'recording-only SDK evidence does not execute the portable View');
  assert.equal(report.footer_verified, true);
  assert.equal(report.guest_allocation_boundary, 'verified');
  assert.equal(report.direct_vectors, 10);
  assert.equal(report.resident_vectors, 10);
  assert.equal(report.intent_vectors, 7);
  assert.equal(report.view_attached, 1);
  assert.equal(report.view_detached, 1);
  assert.deepEqual(report.portable_browser, {
    schema: 'prismpm/portable-browser-oracle/1', profile: 'utf8-text', engine: 'chromium',
    browser_version: '151.0.7922.34', playwright: '1.62.1',
    cases: portableCases.map((name) => ({name, status: 'passed', attempts: 1})),
    vector_indexes: [0, 1, 2, 3, 4], skipped: 0, retries: 0, status: 'passed',
  }, 'every applicable Foundry vector and portable browser journey must execute');
}
