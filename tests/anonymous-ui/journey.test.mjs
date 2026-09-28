// Source-registration guard; generated kernel/native/Wasm/browser evidence is separate.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';

function source() {
  const text = readFileSync(new URL('../../src/Foundry/UI/Presentation.lex.tex', import.meta.url), 'utf8');
  const rows = [...text.matchAll(/^\\semanticdata\{(.*)\}$/gm)];
  assert.equal(rows.length, 1);
  return JSON.parse(rows[0][1]).declarations;
}

test('navigation focus is projected only after the modeled intent has been admitted', () => {
  const rows = source(), declaration = name => {
    const matches = rows.filter(row => row.name === name);
    assert.equal(matches.length, 1, name);
    return matches[0];
  };
  const plain = declaration('anonymousPresentation');
  assert.deepEqual(plain.body.fields.find(field => field.field === 'focus').value, {kind:'nat', value:'0'});
  const focused = declaration('anonymousNavigationPresentation');
  assert.deepEqual(focused.body.fields.find(field => field.field === 'focus').value, {kind:'nat', value:'8'});
  const root = declaration('anonymousNavigationSemanticBytes');
  const bounded = root.body.scrutinee;
  assert.equal(bounded.kind, 'if');
  assert.equal(bounded.condition.kind, 'ble');
  assert.equal(bounded.condition.right.value, '64');
  const admitted = bounded.then_value;
  assert.equal(admitted.kind, 'match');
  assert.deepEqual(admitted.scrutinee.function, {name:'readAnonymousNavigation'});
  const ok = admitted.branches.find(branch => branch.constructor.name === 'Result.ok');
  assert.deepEqual(ok.body.function, {module:'Foundation.View.Browser.V1.SemanticsWire', name:'writeSemanticPresentation'});
  assert.deepEqual(ok.body.arguments[0].function, {name:'anonymousNavigationSemanticPresentation'});
  assert.deepEqual(ok.body.arguments[0].arguments, [{kind:'var', name:'next'}]);
  assert.deepEqual(root.parameters, [{name:'request', type:{kind:'bytes'}}]);
});
