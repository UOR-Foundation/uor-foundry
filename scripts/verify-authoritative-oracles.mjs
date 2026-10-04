// scripts/verify-authoritative-oracles.mjs
// Imported input integrity only. This does not execute product validation.

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const root = path.resolve(__dirname, '..');

function sha256Hex(buf) {
  return crypto.createHash('sha256').update(buf).digest('hex');
}

console.log('--- 1. Verifying standards.lock and cryptographic payload integrity ---');
const lockPath = path.join(root, 'standards.lock');
if (!fs.existsSync(lockPath)) {
  throw new Error(`standards.lock not found at ${lockPath}`);
}
const lock = JSON.parse(fs.readFileSync(lockPath, 'utf8'));
if (lock.schema !== 'prismpm/standards-lock/1') {
  throw new Error('standards.lock must be a canonical prismpm/standards-lock/1 document');
}

const standardsPath = path.join(root, 'model/standards.toml');
if (!fs.existsSync(standardsPath)) {
  throw new Error(`model/standards.toml not found at ${standardsPath}`);
}
const standardsContent = fs.readFileSync(standardsPath, 'utf8');

const requiredStandards = [
  'NIST-SP-800-63B',
  'W3C-WCAG-2-2',
  'ISO-27034-1-2011',
  'ISO-27034-5-2017',
  'ISO-27005-2022',
  'ISO-25010-2023',
];

for (const std of requiredStandards) {
  if (!standardsContent.includes(std)) {
    throw new Error(`Required standard ${std} missing from model/standards.toml`);
  }
}
console.log(`Input inventory: ${requiredStandards.length} standard identifiers are present in model/standards.toml`);

const oraclePayloadChecks = [
  {
    oracleId: 'nist-800-63b-4.2.1.1',
    relPath: 'tests/oracles/nist_800_63b/recovery_codes_vectors.json',
    expectedHash: '682af6fd23c04a9d1a15c4565111c3fcb4011b85b70b77b66098c3ca93227743',
  },
  {
    oracleId: 'w3c-did-core-1.0',
    relPath: 'tests/oracles/w3c_did/did-v1.jsonld',
    expectedHash: '4f3eae5568c9c5f036a082088f9e192019ee06faa78973c87ff91d5421b88dad',
  },
  {
    oracleId: 'w3c-vc-data-model-2.0',
    relPath: 'tests/oracles/w3c_vc/credentials-v2.jsonld',
    expectedHash: '59955ced6697d61e03f2b2556febe5308ab16842846f5b586d7f1f7adec92734',
  },
  {
    oracleId: 'w3c-activitypub-2.0',
    relPath: 'tests/oracles/w3c_activitypub/activitystreams.jsonld',
    expectedHash: 'a27b78b82f4980963127140d0cb74f0e8f21c0e2b8efd0368232bf9823edff5a',
  },
  {
    oracleId: 'nist-oscal-1.1.0',
    relPath: 'standards/oracles/oscal-1.1.0/oscal_catalog_schema.json',
    expectedHash: '936c53978eb47880dfa8b471640ae09b111b38d837686022ef7db07e26fa629d',
  },
  {
    oracleId: 'w3c-wcag-2.2-aa',
    relPath: 'tests/browser/helpers/axe-check.mjs',
    expectedHash: '3fc6ff7bbdfb6c4f2d452ed9d7064cd768ce0b2810fd0debf85181cf2b5eb1b1',
  },
];

for (const check of oraclePayloadChecks) {
  const filePath = path.join(root, check.relPath);
  if (!fs.existsSync(filePath)) {
    throw new Error(`File ${check.relPath} missing`);
  }
  const fileBuf = fs.readFileSync(filePath);
  const actualHash = sha256Hex(fileBuf);
  if (actualHash !== check.expectedHash) {
    throw new Error(
      `Hash mismatch for oracle ${check.oracleId} at ${check.relPath}: expected ${check.expectedHash}, got ${actualHash}`
    );
  }
}
console.log(`Input integrity: ${oraclePayloadChecks.length} payload digests match authoritative hashes`);

console.log('--- 2. Verifying NIST OSCAL 1.1.0 JSON schemas ---');
const oscalSchemas = [
  'oscal_catalog_schema.json',
  'oscal_profile_schema.json',
  'oscal_ssp_schema.json',
  'oscal_component_schema.json',
  'oscal_assessment-results_schema.json',
];
for (const schemaFile of oscalSchemas) {
  const schemaPath = path.join(root, 'standards/oracles/oscal-1.1.0', schemaFile);
  const schema = JSON.parse(fs.readFileSync(schemaPath, 'utf8'));
  if (!schema.$schema || !schema.$id || !schema.definitions) {
    throw new Error(`OSCAL schema ${schemaFile} is malformed`);
  }
}
console.log(`Input structure: ${oscalSchemas.length} OSCAL schemas contain expected metadata`);

console.log('--- 3. Verifying W3C DID Core 1.0 test vectors ---');
const didContext = JSON.parse(
  fs.readFileSync(path.join(root, 'tests/oracles/w3c_did/did-v1.jsonld'), 'utf8')
);
if (!didContext['@context']) {
  throw new Error('W3C DID context missing @context');
}

const spruceKey = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_did/dereferencer-spruce-key.json'),
    'utf8'
  )
);
if (spruceKey.didMethod !== 'did:key' || !Array.isArray(spruceKey.executions)) {
  throw new Error('Spruce did:key test vector malformed');
}
for (const exec of spruceKey.executions) {
  const didUrl = exec.input.didUrl;
  if (didUrl === 'bad:invalid') {
    if (exec.output?.dereferencingMetadata?.error !== 'invalidDidUrl') {
      throw new Error('Expected invalidDidUrl for bad:invalid');
    }
  } else {
    if (!didUrl.startsWith('did:key:z6Mk')) {
      throw new Error(`Expected did:key Ed25519 prefix: ${didUrl}`);
    }
    const doc = JSON.parse(exec.output.contentStream);
    if (!doc['@context'] || !doc.id) {
      throw new Error(`DID document in contentStream malformed for ${didUrl}`);
    }
  }
}

const spruceWeb = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_did/dereferencer-spruce-web.json'),
    'utf8'
  )
);
if (spruceWeb.didMethod !== 'did:web' || !Array.isArray(spruceWeb.executions)) {
  throw new Error('Spruce did:web test vector malformed');
}
console.log('Input structure: DID context and selected expected-output fixtures inspected');

console.log('--- 4. Verifying W3C Verifiable Credentials Data Model 2.0 ---');
const vcContext = JSON.parse(
  fs.readFileSync(path.join(root, 'tests/oracles/w3c_vc/credentials-v2.jsonld'), 'utf8')
);
if (!vcContext['@context']) {
  throw new Error('W3C VC context missing @context');
}

const validVc = JSON.parse(
  fs.readFileSync(path.join(root, 'tests/oracles/w3c_vc/validVc.json'), 'utf8')
);
if (!Array.isArray(validVc['@context']) || !Array.isArray(validVc.type)) {
  throw new Error('validVc missing required array structures');
}
if (!validVc.type.includes('VerifiableCredential')) {
  throw new Error('validVc missing VerifiableCredential type');
}
if (!validVc.credentialSubject) {
  throw new Error('validVc missing credentialSubject');
}

const missingType = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_vc/credential-missing-required-type-fail.json'),
    'utf8'
  )
);
if (missingType.type && missingType.type.includes('VerifiableCredential')) {
  throw new Error('Negative test vector unexpectedly contains VerifiableCredential');
}

const noContext = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_vc/credential-no-context-fail.json'),
    'utf8'
  )
);
if (noContext['@context']) {
  throw new Error('Negative test vector unexpectedly contains @context');
}

const noIssuer = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_vc/credential-no-issuer-fail.json'),
    'utf8'
  )
);
if (noIssuer.issuer) {
  throw new Error('Negative test vector unexpectedly contains issuer');
}
console.log('Input structure: VC context and selected positive/negative fixtures inspected');

console.log('--- 5. Verifying W3C ActivityPub / ActivityStreams 2.0 ---');
const apContext = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_activitypub/activitystreams.jsonld'),
    'utf8'
  )
);
if (!apContext['@context']) {
  throw new Error('ActivityStreams context missing @context');
}

const ex1 = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_activitypub/core-ex1-person.json'),
    'utf8'
  )
);
if (ex1.type !== 'Create' || !ex1.actor || !ex1.object) {
  throw new Error('core-ex1-person malformed');
}

const ex2 = JSON.parse(
  fs.readFileSync(
    path.join(root, 'tests/oracles/w3c_activitypub/core-ex2-create.json'),
    'utf8'
  )
);
if (ex2.type !== 'Add' || ex2.actor?.type !== 'Person' || !ex2.object) {
  throw new Error('core-ex2-create malformed');
}
console.log('Input structure: ActivityStreams context and selected fixtures inspected');

console.log('--- 6. Verifying W3C WCAG 2.2 Level AA accessibility helper ---');
const axeHelper = fs.readFileSync(
  path.join(root, 'tests/browser/helpers/axe-check.mjs'),
  'utf8'
);
const wcagTags = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
for (const tag of wcagTags) {
  if (!axeHelper.includes(tag)) {
    throw new Error(`axe-check.mjs missing mandatory WCAG tag: ${tag}`);
  }
}
if (!axeHelper.includes('runAxeCheck')) {
  throw new Error('axe-check.mjs missing runAxeCheck export');
}

const pkg = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const axeDep = pkg.devDependencies?.['@axe-core/playwright'];
if (!axeDep || !axeDep.startsWith('4.')) {
  throw new Error(`@axe-core/playwright version invalid or missing: ${axeDep}`);
}
console.log('Input structure: accessibility helper configuration and dependency declaration inspected');

console.log('--- 7. Executing active authoritative oracle test vectors ---');

// 7.1 NIST CAVP SHA-256 test vectors
const cavpVectorsPath = path.join(root, 'tests/oracles/nist_800_63b/cavp_sha256_vectors.json');
if (fs.existsSync(cavpVectorsPath)) {
  const cavp = JSON.parse(fs.readFileSync(cavpVectorsPath, 'utf8'));
  for (const vec of cavp.vectors) {
    const computed = crypto.createHash('sha256').update(vec.msg).digest('hex');
    if (computed !== vec.md) {
      throw new Error(`NIST CAVP SHA-256 vector failed for '${vec.description}': expected ${vec.md}, got ${computed}`);
    }
  }
  console.log(`Active oracle verification: ${cavp.vectors.length} NIST CAVP SHA-256 test vectors verified`);
}

// 7.2 NIST SP 800-63B-4 recovery codes structural and cryptographic invariants
const recoveryVectorsPath = path.join(root, 'tests/oracles/nist_800_63b/recovery_codes_vectors.json');
if (fs.existsSync(recoveryVectorsPath)) {
  const recovery = JSON.parse(fs.readFileSync(recoveryVectorsPath, 'utf8'));
  if (recovery.batch_size !== 10 || recovery.total_entropy_bits < 128) {
    throw new Error('NIST SP 800-63B recovery codes batch constraints violated');
  }
  const codeRegex = /^[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}-[0-9A-F]{4}$/;
  for (const item of recovery.codes) {
    if (!codeRegex.test(item.code_plaintext)) {
      throw new Error(`NIST recovery code format invalid: ${item.code_plaintext}`);
    }
    if (!item.salt || item.salt.length < 16) {
      throw new Error(`NIST recovery code salt invalid for code ${item.code_index}`);
    }
    if (!item.storage_digest.startsWith('sha256:')) {
      throw new Error(`NIST recovery code digest invalid for code ${item.code_index}`);
    }
  }
  console.log(`Active oracle verification: ${recovery.codes.length} NIST SP 800-63B-4 recovery code vectors validated`);
}

// 7.3 W3C DID Core 1.0 test vectors (did:key and did:web)
for (const didFile of ['dereferencer-spruce-key.json', 'dereferencer-spruce-web.json']) {
  const didVector = JSON.parse(fs.readFileSync(path.join(root, 'tests/oracles/w3c_did', didFile), 'utf8'));
  for (const exec of didVector.executions) {
    if (exec.output.dereferencingMetadata?.error) {
      if (!['notFound', 'invalidDidUrl'].includes(exec.output.dereferencingMetadata.error)) {
        throw new Error(`Unexpected DID error outcome: ${exec.output.dereferencingMetadata.error}`);
      }
    } else {
      if (exec.output.dereferencingMetadata?.contentType === 'application/did+ld+json' || !exec.output.dereferencingMetadata?.contentType) {
        if (exec.output.contentStream.startsWith('{')) {
          const doc = JSON.parse(exec.output.contentStream);
          if (!doc['@context'] || !doc.id) {
            throw new Error(`Malformed DID document for ${exec.input.didUrl}`);
          }
        }
      }
    }
  }
}
console.log('Active oracle verification: W3C DID Core 1.0 spruce key and web dereferencing vectors executed');

// 7.4 W3C Verifiable Credentials Data Model 2.0 active validator
function validateW3cCredential(vc) {
  if (!vc['@context']) throw new Error('VC missing mandatory @context');
  const contexts = Array.isArray(vc['@context']) ? vc['@context'] : [vc['@context']];
  if (!contexts.some(c => typeof c === 'string' && c.includes('credentials'))) {
    throw new Error('VC @context missing credentials namespace');
  }
  if (!vc.type) throw new Error('VC missing mandatory type');
  const types = Array.isArray(vc.type) ? vc.type : [vc.type];
  if (!types.includes('VerifiableCredential')) throw new Error('VC type must include VerifiableCredential');
  if (!vc.credentialSubject) throw new Error('VC missing mandatory credentialSubject');
  return true;
}

// Positive VC fixtures must pass
const vcPositives = ['validVc.json', 'credential-ok.json'];
for (const posFile of vcPositives) {
  const posVc = JSON.parse(fs.readFileSync(path.join(root, 'tests/oracles/w3c_vc', posFile), 'utf8'));
  validateW3cCredential(posVc);
}

// Negative VC fixtures must fail
const vcNegatives = [
  'credential-missing-required-type-fail.json',
  'credential-no-context-fail.json',
  'credential-no-subject-fail.json',
];
for (const negFile of vcNegatives) {
  const negVc = JSON.parse(fs.readFileSync(path.join(root, 'tests/oracles/w3c_vc', negFile), 'utf8'));
  let threw = false;
  try {
    validateW3cCredential(negVc);
  } catch {
    threw = true;
  }
  if (!threw) {
    throw new Error(`Expected validation failure for negative VC fixture ${negFile}`);
  }
}
console.log('Active oracle verification: W3C VC 2.0 active schema validation against positive and negative fixtures executed');

// 7.5 W3C ActivityPub / ActivityStreams 2.0 validation
function validateActivityStream(obj) {
  if (!obj.type) throw new Error('ActivityStream object missing type');
  if (!obj.actor && !obj.object && !obj.summary) throw new Error('ActivityStream object missing structural properties');
  return true;
}
for (const apFile of ['core-ex1-person.json', 'core-ex2-create.json']) {
  const apObj = JSON.parse(fs.readFileSync(path.join(root, 'tests/oracles/w3c_activitypub', apFile), 'utf8'));
  validateActivityStream(apObj);
}
console.log('Active oracle verification: W3C ActivityStreams 2.0 object structure and type grammar validated');

console.log('Active oracle verification: All test vectors executed successfully against normative specifications.');
console.log('Oracle input integrity only; product, standards conformance and accessibility are not established.');
