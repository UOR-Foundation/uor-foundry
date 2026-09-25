// scripts/verify-authoritative-oracles.mjs
// Verification of authoritative external standards, test vectors, and oracles.

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

const requiredAuthorities = [
  'NIST-SP-800-63B',
  'NIST-OSCAL-1-1-0',
  'W3C-DID-CORE-1-0',
  'W3C-VC-2-0',
  'W3C-ACTIVITYPUB-2-0',
  'W3C-WCAG-2-2',
];

for (const authId of requiredAuthorities) {
  const found = lock.authorities.find((a) => a.id === authId);
  if (!found) {
    throw new Error(`Required authority ${authId} missing from standards.lock`);
  }
}
console.log(`✓ All ${requiredAuthorities.length} authoritative standards bodies verified in standards.lock`);

const oraclePayloadChecks = [
  {
    oracleId: 'nist-800-63b-4.2.1.1',
    relPath: 'tests/oracles/nist_800_63b/recovery_codes_vectors.json',
  },
  {
    oracleId: 'w3c-did-core-1.0',
    relPath: 'tests/oracles/w3c_did/did-v1.jsonld',
  },
  {
    oracleId: 'w3c-vc-data-model-2.0',
    relPath: 'tests/oracles/w3c_vc/credentials-v2.jsonld',
  },
  {
    oracleId: 'w3c-activitypub-2.0',
    relPath: 'tests/oracles/w3c_activitypub/activitystreams.jsonld',
  },
  {
    oracleId: 'nist-oscal-1.1.0',
    relPath: 'standards/oracles/oscal-1.1.0/oscal_catalog_schema.json',
  },
  {
    oracleId: 'w3c-wcag-2.2-aa',
    relPath: 'tests/browser/helpers/axe-check.mjs',
  },
];

for (const check of oraclePayloadChecks) {
  const oracle = lock.oracles.find((o) => o.id === check.oracleId);
  if (!oracle) {
    throw new Error(`Oracle ${check.oracleId} missing from standards.lock`);
  }
  const filePath = path.join(root, check.relPath);
  if (!fs.existsSync(filePath)) {
    throw new Error(`File ${check.relPath} missing`);
  }
  const fileBuf = fs.readFileSync(filePath);
  const actualHash = sha256Hex(fileBuf);
  if (actualHash !== oracle.upstream_payload_sha256) {
    throw new Error(
      `Hash mismatch for oracle ${check.oracleId} at ${check.relPath}: expected ${oracle.upstream_payload_sha256}, got ${actualHash}`
    );
  }
}
console.log(`✓ All ${oraclePayloadChecks.length} oracle payload cryptographic digests verified against standards.lock`);

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
console.log(`✓ All ${oscalSchemas.length} NIST OSCAL 1.1.0 official JSON schemas verified`);

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
console.log('✓ W3C DID Core 1.0 JSON-LD context, did:key, and did:web vectors verified');

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
console.log('✓ W3C VC 2.0 context, positive validVc, and negative defect vectors verified');

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
console.log('✓ W3C ActivityPub / ActivityStreams 2.0 context and test vectors verified');

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
console.log('✓ W3C WCAG 2.2 Level AA accessibility helper and @axe-core/playwright dependency verified');

console.log('ALL AUTHORITATIVE ORACLES AND TEST VECTORS VERIFIED SUCCESSFULLY');
