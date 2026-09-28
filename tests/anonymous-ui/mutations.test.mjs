import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {navigationCorpus, navigationPresentationCorpus, presentationCorpus} from './corpus.mjs';
import {mutations, mutateAnonymousSource} from './mutations.mjs';

const source = () => readFileSync(new URL('../../src/Foundry/UI/Presentation.lex.tex', import.meta.url));
const suites = {navigation: navigationCorpus, semantics: () => presentationCorpus(true),
  'navigation-presentation': navigationPresentationCorpus};

test('all six navigation defects bind exact independent finite counterexamples', () => {
  assert.deepEqual(mutations.map(row => row.id),
    ['ContextualRecovery', 'NavigationFocus', 'RestorationFocus', 'StaleIntent', 'RequestCap', 'RecoveryLandmark']);
  const original = source(), distinct = new Set();
  for (const row of mutations) {
    const probe = suites[row.suite]().filter(value => value.id === row.probe);
    assert.equal(probe.length, 1, row.id);
    const changed = mutateAnonymousSource(original, row.id);
    assert.notDeepEqual(changed, original);
    const text = changed.toString('utf8'); distinct.add(text);
    const before = JSON.parse(/\\semanticdata\{(.*)\}/.exec(original.toString())[1]);
    const after = JSON.parse(/\\semanticdata\{(.*)\}/.exec(text)[1]);
    assert.deepEqual(after.declarations.map(value => value.name), before.declarations.map(value => value.name));
    const differences = before.declarations.filter((value, index) =>
      JSON.stringify(value) !== JSON.stringify(after.declarations[index])).map(value => value.name);
    assert.deepEqual(differences, [row.declaration], 'one targeted live definition ' + row.id);
    assert.deepEqual(source(), original, 'never edit the authoritative source');
  }
  assert.equal(distinct.size, 6);
});

test('mutation construction rejects unknown IDs and changed guard shapes', () => {
  const original = source();
  assert.throws(() => mutateAnonymousSource(original, 'Unregistered'));
  for (const row of mutations) {
    const once = mutateAnonymousSource(original, row.id);
    assert.throws(() => mutateAnonymousSource(once, row.id), 'exact pristine guard required ' + row.id);
  }
});
