import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {allCases} from './corpus.mjs';
import {mutants, mutateSource} from './mutants.mjs';

test('UC-01 registered source mutants each name an exact independent runtime counterexample', () => {
  const source = readFileSync(new URL('../../src/Foundry/UI/CommandBridge.lex.tex', import.meta.url), 'utf8');
  const rows = allCases(); assert.equal(mutants.length, 9);
  assert.equal(new Set(mutants.map(row => row.name)).size, mutants.length);
  for (const mutant of mutants) {
    assert.equal(rows.filter(row => row.root === 'commandBridgeBytes' && row.name === mutant.vector).length, 1);
    const changed = mutateSource(source, mutant.name); assert.notEqual(changed, source);
    assert.equal(changed.split('\n').filter(line => line.startsWith('\\semanticdata{')).length, 1);
  }
  assert.throws(() => mutateSource(source, 'unknown'));
});
