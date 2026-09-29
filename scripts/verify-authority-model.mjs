// Scoped source/kernel evidence only; full application acceptance remains mandatory.
import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync, mkdtempSync, readFileSync, writeFileSync} from 'node:fs';
import {dirname, join} from 'node:path';
import {fileURLToPath} from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const relative = 'src/Foundry/Core/Authority.lex.tex';
const source = readFileSync(join(root, relative));
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const model = JSON.parse(/\\semanticdata\{(.*)\}/.exec(source.toString('utf8'))[1]);
const fixturePath = join(root, 'tests/fixtures/authority-regressions.json');
const fixtureBytes = readFileSync(fixturePath);
const fixture = JSON.parse(fixtureBytes);
assert.equal(fixture.schema, 'foundry/authority-regressions/1');
const verifierPath = fileURLToPath(import.meta.url);
const verifierBytes = readFileSync(verifierPath);
const required = [
  'executed_approval_rejected', 'rejected_approval_rejected', 'approved_approval_rejected',
  'approval_count_mismatch_rejected', 'duplicate_record_approval_rejected',
  'empty_record_approval_rejected', 'missing_proposer_approval_rejected',
  'zero_quorum_approval_rejected', 'empty_approver_rejected', 'duplicate_approver_rejected',
  'valid_approval_advances', 'insufficient_approval_remains_pending',
  'execute_count_mismatch_rejected', 'execute_duplicate_record_rejected',
  'execute_zero_quorum_rejected', 'execute_insufficient_approvals_rejected',
  'execute_pending_rejected', 'execute_executed_rejected', 'execute_rejected_rejected',
  'valid_proposal_executes',
];
assert.deepEqual(fixture.cases.map(row => row.id), required);
const named = name => ({kind: 'named', arguments: [], member: {name}});
const constructor = name => ({kind: 'constructor', constructor: {name}, arguments: []});
const text = value => ({kind: 'string', value});
const natural = value => ({kind: 'nat', value: String(value)});
const list = values => values.reduceRight((tail, value) => ({kind: 'cons', head: text(value), tail}),
  {kind: 'nil', element: {kind: 'string'}});
const proposal = value => ({kind: 'record', type: {name: 'ChangeProposal'}, fields: Object.entries({
  proposalId: text('proposal-a'), organizationId: text('organization-a'),
  scope: constructor('AdminScope.Organization'), proposer: text('alice'),
  approvals: list(value.approvals), status: constructor('ProposalStatus.' + value.status),
  approvalCount: natural(value.approvalCount), requiredQuorum: natural(value.requiredQuorum),
}).map(([field, value]) => ({field, value}))});
for (const testCase of fixture.cases) {
  const name = testCase.id;
  const declarations = model.declarations.filter(row => row.name === name);
  assert.equal(declarations.length, 1, `missing or duplicate regression: ${name}`);
  assert.equal(declarations[0].kind, 'theorem');
  const arguments_ = [proposal(testCase.proposal)];
  if (testCase.operation === 'castApproval') arguments_.push(text(testCase.approver));
  assert.deepEqual(declarations[0].statement, {kind: 'eq',
    left: {kind: 'call', function: {name: testCase.operation}, arguments: arguments_},
    right: {kind: 'constructor', constructor: {name: testCase.error ? 'Result.error' : 'Result.ok'},
      type_arguments: [named('ChangeProposal'), named('AuthorityError')],
      arguments: [testCase.error ? constructor('AuthorityError.' + testCase.error) : proposal(testCase.expected)]},
  }, `weakened or wrong regression case: ${name}`);
  assert.deepEqual(declarations[0].parameters, []);
  assert.deepEqual(declarations[0].axioms, []);
}
assert.equal(process.env.PRISMPM_SDK_INVENTORY, '/opt/prismpm/share/inventory.json',
  'run source verification inside the immutable SDK devcontainer');
const inventory = readFileSync(process.env.PRISMPM_SDK_INVENTORY);
const compiler = '/usr/local/bin/lexlean';
const compilerHash = sha(readFileSync(compiler));
mkdirSync(join(root, 'target'), {recursive: true});
const work = mkdtempSync(join(root, 'target/authority-kernel-'));
const project = join(work, 'project');
function run(args, report, expectedStatus = 0) {
  const result = spawnSync(compiler, args, {encoding: 'utf8', timeout: 300_000,
    maxBuffer: 16 * 1024 * 1024, cwd: root});
  writeFileSync(join(work, `${report}.stdout`), result.stdout ?? '', {flag: 'wx'});
  writeFileSync(join(work, `${report}.stderr`), result.stderr ?? '', {flag: 'wx'});
  assert.equal(result.error, undefined, `compiler invocation failed; evidence: ${work}`);
  assert.equal(result.signal, null, `compiler interrupted; evidence: ${work}`);
  assert.equal(result.status, expectedStatus, `source/kernel verification failed; evidence: ${work}`);
  return result.stdout;
}
run(['init', '--language', '1.1', '--name', 'foundry-authority-probe',
  '--module-prefix', 'PrismFoundry', project], 'init');
mkdirSync(dirname(join(project, relative)), {recursive: true});
writeFileSync(join(project, relative), source, {flag: 'wx'});
const result = JSON.parse(run(['--project', join(project, 'lexlean.toml'),
  '--diagnostic-format', 'json', 'verify', join(project, relative)], 'verify'));
assert.equal(result.success, true);
assert.deepEqual(result.diagnostics, []);
assert.deepEqual(result.modules, ['Foundry.Core.Authority']);
assert.match(result.attestation_id, /^[0-9a-f]{64}$/);
const verified = join(project, '.lexlean/verified', result.attestation_id);
const attestation = JSON.parse(readFileSync(join(verified, 'attestation.json')));
const manifestBytes = readFileSync(join(verified, 'build-manifest.json'));
const manifest = JSON.parse(manifestBytes);
assert.equal(attestation.status, 'verified');
assert.equal(attestation.attestation_id, result.attestation_id);
assert.equal(attestation.build_manifest.sha256, sha(manifestBytes));
assert.equal(attestation.lexlean.executable_sha256, compilerHash);
assert.deepEqual(manifest.inputs.filter(row => row.kind === 'source'),
  [{byte_length: source.length, kind: 'source', path: relative, sha256: sha(source)}]);
const expected = model.declarations.map(row => `PrismFoundry.Foundry.Core.Authority.${row.name}`).sort();
assert.deepEqual(attestation.declarations.map(row => row.name).sort(), expected);
for (const row of attestation.declarations) {
  assert.equal(row.result, 'ok');
  assert.deepEqual(row.observed, []);
  assert.deepEqual(row.policy, {axioms: [], kind: 'none'});
}
const branch = (declaration, state) => declaration.body.branches.find(row =>
  row.constructor.name === `ProposalStatus.${state}`);
const mutations = [
  ['reopen-executed', declarations => {
    const target = declarations.find(row => row.name === 'castApproval');
    branch(target, 'Executed').body = branch(target, 'Pending').body;
  }],
  ['trust-record', declarations => {
    const target = declarations.find(row => row.name === 'approvalRecordValid');
    target.body = {kind: 'or', left: target.body, right: {kind: 'bool', value: true}};
  }],
  ['bypass-execution-quorum', declarations => {
    const target = branch(declarations.find(row => row.name === 'executeProposal'), 'Approved');
    const guard = target.body.else_value.else_value;
    assert.equal(guard.condition.kind, 'blt');
    guard.condition = {kind: 'and', left: guard.condition, right: {kind: 'bool', value: false}};
  }],
  ['allow-zero-quorum', declarations => {
    const target = branch(declarations.find(row => row.name === 'castApproval'), 'Pending');
    assert.equal(target.body.condition.kind, 'beq');
    target.body.condition = {kind: 'and', left: target.body.condition, right: {kind: 'bool', value: false}};
  }],
];
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
for (const [name, mutate] of mutations) {
  const mutant = structuredClone(model);
  mutate(mutant.declarations);
  const mutated = source.toString('utf8').replace(/\\semanticdata\{(.*)\}/,
    () => '\\semanticdata{' + JSON.stringify(canonical(mutant)) + '}');
  const mutationProject = join(work, name);
  run(['init', '--language', '1.1', '--name', 'foundry-authority-probe',
    '--module-prefix', 'PrismFoundry', mutationProject], `${name}-init`);
  mkdirSync(dirname(join(mutationProject, relative)), {recursive: true});
  writeFileSync(join(mutationProject, relative), mutated, {flag: 'wx'});
  const rejected = JSON.parse(run(['--project', join(mutationProject, 'lexlean.toml'),
    '--diagnostic-format', 'json', 'verify', join(mutationProject, relative)], name, 1));
  assert.equal(rejected.success, false);
  assert.ok(rejected.diagnostics.length > 0, `mutation ${name} produced no failure`);
  for (const diagnostic of rejected.diagnostics) {
    assert.equal(diagnostic.code, 'LLV7002', `mutation ${name} did not fail its regression proof`);
    assert.match(diagnostic.message, /Tactic `rfl` failed/);
  }
}
assert.deepEqual(readFileSync(join(root, relative)), source, 'source changed during verification');
assert.deepEqual(readFileSync(join(project, relative)), source, 'captured source changed');
assert.equal(sha(readFileSync(compiler)), compilerHash, 'compiler changed');
assert.deepEqual(readFileSync(process.env.PRISMPM_SDK_INVENTORY), inventory, 'SDK inventory changed');
assert.deepEqual(readFileSync(fixturePath), fixtureBytes, 'regression fixture changed');
assert.deepEqual(readFileSync(verifierPath), verifierBytes, 'verifier changed');
console.log(JSON.stringify(canonical({scope: 'authority-source-kernel', source_sha256: sha(source),
  fixture_sha256: sha(fixtureBytes), verifier_sha256: sha(verifierBytes),
  regression_ids: required,
  regression_theorems: required.length, declarations: expected.length,
  rejected_mutations: mutations.map(([name]) => name),
  attestation_id: result.attestation_id, evidence: work})));
