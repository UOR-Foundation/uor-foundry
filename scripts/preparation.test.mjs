import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../', import.meta.url));
const source = readFileSync(join(root, 'Justfile'), 'utf8');

function preparationOrder(justfile) {
  const result = spawnSync('just', ['--justfile', justfile, '--dry-run', 'vv'], {
    cwd: root, encoding: 'utf8', timeout: 10_000,
  });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stderr);
  const commands = result.stderr.split('\n').map(line => line.trim()).filter(Boolean);
  const fetch = commands.indexOf('prismpm fetch --locked');
  assert.equal(commands.filter(line => line === 'prismpm fetch --locked').length, 1,
    'complete gate acquires locked SDK inputs exactly once');
  for (const before of ['prismpm template check', 'prismpm lock check']) {
    assert.ok(commands.indexOf(before) >= 0 && commands.indexOf(before) < fetch,
      'SDK and template admission precede acquisition');
  }
  for (const after of ['cargo clippy --workspace --all-targets -- -D warnings',
    'cargo test --workspace', 'cargo test -p repo-conformance --test bdd']) {
    assert.ok(commands.indexOf(after) > fetch, 'locked inputs precede every product gate');
  }
}

test('complete acceptance acquires exact SDK inputs before compiler and product checks', () => {
  preparationOrder(join(root, 'Justfile'));
});

test('actual recipe defects cannot omit, unlock or delay acquisition', t => {
  const directory = mkdtempSync(join(tmpdir(), 'foundry-preparation-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  assert.equal(source.split('vv: prepare ').length, 2);
  assert.equal(source.split('    prismpm fetch --locked').length, 2);
  const defects = [
    source.replace('vv: prepare ', 'vv: template-check '),
    source.replace('    prismpm fetch --locked', '    prismpm fetch'),
    source.replace('vv: prepare fmt-check model lint test features bdd deny',
      'vv: template-check fmt-check model lint test features bdd deny prepare'),
  ];
  for (const [index, defect] of defects.entries()) {
    assert.notEqual(defect, source, 'the actual owning recipe is mutated');
    const path = join(directory, 'Justfile-' + index);
    writeFileSync(path, defect, {flag: 'wx'});
    assert.throws(() => preparationOrder(path), error => error.code === 'ERR_ASSERTION'
      && /complete gate acquires locked SDK inputs|locked inputs precede every product gate/.test(error.message));
  }
});
