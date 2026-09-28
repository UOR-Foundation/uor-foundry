// Test-only semantic defects. Actual kernel/native/Wasm detection is required.
import assert from 'node:assert/strict';

export const mutations = Object.freeze([
  {id: 'ContextualRecovery', declaration: 'anonymousNavigate', suite: 'navigation', probe: 'Navigate-0-0-0-3'},
  {id: 'NavigationFocus', declaration: 'anonymousNavigationPresentation', suite: 'navigation-presentation', probe: 'Navigate-0-0-0-1'},
  {id: 'RestorationFocus', declaration: 'anonymousPresentation', suite: 'semantics', probe: 'Selector-0-0-Ready-Welcome'},
  {id: 'StaleIntent', declaration: 'anonymousNavigate', suite: 'navigation', probe: 'RevisionMismatch-0-7'},
  {id: 'RequestCap', declaration: 'anonymousNavigationSemanticBytes', suite: 'navigation-presentation', probe: 'InputOverflow'},
  {id: 'RecoveryLandmark', declaration: 'anonymousPresentation', suite: 'semantics', probe: 'Selector-0-0-Ready-Login'},
].map(Object.freeze));

const nat = value => ({kind: 'nat', value: String(value)});
const variable = name => ({kind: 'var', name});
const project = (name, field) => ({field, kind: 'project', value: variable(name)});
const equal = (left, right) => ({kind: 'beq', left, right});
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;

export function mutateAnonymousSource(bytes, id) {
  const registration = mutations.find(row => row.id === id); assert.ok(registration, 'registered mutation');
  const text = bytes.toString('utf8'), match = /^\\semanticdata\{(.*)\}$/m.exec(text);
  assert.ok(match, 'one semantic model');
  const model = JSON.parse(match[1]);
  assert.equal(JSON.stringify(canonical(model)), match[1], 'canonical original semantic source');
  const declarations = model.declarations.filter(row => row.name === registration.declaration);
  assert.equal(declarations.length, 1, 'exact live mutation target');
  const declaration = declarations[0]; assert.equal(declaration.kind, 'definition');
  if (id === 'ContextualRecovery' || id === 'StaleIntent') {
    const gate = declaration.body.condition.right.left;
    assert.equal(gate.kind, 'call');
    assert.deepEqual(gate.function, {module: 'Foundation.View.Browser.V1.Model', name: 'intentFits'});
    assert.deepEqual(gate.arguments, [{arguments: [variable('selector')], function: {name: 'anonymousPresentation'}, kind: 'call'}, variable('intent')]);
    if (id === 'ContextualRecovery') declaration.body.condition.right.left = {kind: 'or', left: gate,
      right: {kind: 'and', left: equal(project('selector', 'screen'), nat(0)),
        right: equal(project('intent', 'action'), nat(4))}};
    else gate.arguments[1] = {kind: 'record', type: {module: 'Foundation.View.Browser.V1.Model', name: 'Intent'}, fields: [
      {field: 'revision', value: project('selector', 'revision')},
      {field: 'action', value: project('intent', 'action')},
      {field: 'fields', value: project('intent', 'fields')},
    ]};
  } else if (id === 'NavigationFocus' || id === 'RestorationFocus') {
    const focus = declaration.body.fields.filter(row => row.field === 'focus');
    assert.equal(focus.length, 1);
    assert.deepEqual(focus[0].value, nat(id === 'NavigationFocus' ? 8 : 0));
    focus[0].value = nat(id === 'NavigationFocus' ? 0 : 8);
  } else if (id === 'RecoveryLandmark') {
    let changed = 0;
    const visit = node => {
      if (!node || typeof node !== 'object') return;
      if (node.kind === 'record' && node.type.name === 'Node') {
        const parent = node.fields.find(row => row.field === 'parent');
        const content = node.fields.find(row => row.field === 'content').value;
        if (parent.value.value === '7' && content.kind === 'constructor'
          && content.constructor.name === 'Content.Form' && content.arguments[0].value === '19') {
          content.arguments[0] = nat(14); changed++;
        }
      }
      for (const value of Object.values(node))
        if (Array.isArray(value)) value.forEach(visit); else visit(value);
    };
    visit(declaration.body); assert.equal(changed, 3, 'three contextual recovery form labels');
  } else {
    const gate = declaration.body.scrutinee;
    assert.equal(gate.kind, 'if'); assert.equal(gate.condition.kind, 'ble');
    assert.deepEqual(gate.condition.right, nat(64)); gate.condition.right = nat(65);
  }
  const result = Buffer.from(text.replace(match[0], '\\semanticdata{' + JSON.stringify(canonical(model)) + '}'));
  assert.notDeepEqual(result, bytes, 'real source defect required');
  return result;
}
