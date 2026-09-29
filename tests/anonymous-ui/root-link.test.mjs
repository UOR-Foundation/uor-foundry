// Structural source linkage only; generated runtime and release gates are separate.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';

const root = new URL('../../src/Foundry.lex.tex', import.meta.url);
const presentation = 'Foundry.UI.Presentation';
const canonical = value => Array.isArray(value) ? value.map(canonical)
  : value && typeof value === 'object'
    ? Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])])) : value;
const call = (name, args = []) => ({arguments: args, function: {module: presentation, name}, kind: 'call'});

function parse(text) {
  assert.equal((text.match(/\\begin\{lexlean\}\{Foundry\}/g) ?? []).length, 1, 'actual Foundry root module');
  const rows = [...text.matchAll(/^\\semanticdata\{(.*)\}$/gm)];
  assert.equal(rows.length, 1, 'one root semantic module');
  const model = JSON.parse(rows[0][1]);
  assert.equal(rows[0][1], JSON.stringify(canonical(model)), 'canonical unambiguous source data');
  assert.equal(model.spec, 'lexlean/semantic-module/1');
  assert.equal(new Set(model.declarations.map(row => row.name)).size, model.declarations.length);
  return model;
}

function field(record, name) {
  assert.equal(record.kind, 'record');
  const matches = record.fields.filter(row => row.field === name);
  assert.equal(matches.length, 1, 'one modeled field ' + name);
  return matches[0];
}

function verifyRootLink(text) {
  const model = parse(text);
  const present = model.declarations.find(row => row.name === 'present');
  assert.ok(present, 'actual present definition');
  assert.equal(present.kind, 'definition');
  assert.deepEqual(present.parameters, [{name: 'request', type: {kind: 'bytes'}}]);
  assert.deepEqual(present.result, {kind: 'bytes'});
  assert.deepEqual(present.body, call('anonymousPresentBytes', [{kind: 'var', name: 'request'}]),
    'present must call the modeled anonymous byte projection, not echo input');
  assert.deepEqual(present.axioms, ['Classical.choice', 'Quot.sound', 'propext']);
  assert.equal(text.split('\\importmodule{' + presentation + '}').length - 1, 1);
  const application = model.declarations.find(row => row.name === 'browserApplication');
  assert.ok(application, 'actual browserApplication definition');
  const view = field(application.body, 'view').value;
  assert.deepEqual(field(view, 'labels').value, call('anonymousLabels'), 'one source-owned label catalogue');
  for (const name of ['title', 'heading'])
    assert.deepEqual(field(view, name).value, call('anonymousLabelText', [{kind: 'nat', value: '0'}]));
  assert.deepEqual(field(view, 'presentationRoot').value,
    {kind: 'string', value: 'PrismFoundry.Foundry.present'}, 'BrowserView selects the checked root');
  assert.deepEqual(field(view, 'protocol').value, {kind: 'string', value: 'prismpm/browser-presentation/1'});
}

test('actual Foundry root links its modeled projection and label catalogue', () => {
  verifyRootLink(readFileSync(root, 'utf8'));
});

test('structural linkage rejects echo, substituted arguments, missing imports, labels and wrong source', () => {
  // This in-memory AST exercises the structural checker, not application behavior.
  const text = readFileSync(root, 'utf8'), model = parse(text);
  const present = model.declarations.find(row => row.name === 'present');
  present.body = call('anonymousPresentBytes', [{kind: 'var', name: 'request'}]);
  present.axioms = ['Classical.choice', 'Quot.sound', 'propext'];
  const view = field(model.declarations.find(row => row.name === 'browserApplication').body, 'view').value;
  field(view, 'labels').value = call('anonymousLabels');
  for (const name of ['title', 'heading'])
    field(view, name).value = call('anonymousLabelText', [{kind: 'nat', value: '0'}]);
  const base = text.includes('\\importmodule{' + presentation + '}') ? text
    : text.replace('\\title{', '\\importmodule{' + presentation + '}\n\\title{');
  const serialize = value => base.replace(/^\\semanticdata\{.*\}$/m,
    '\\semanticdata{' + JSON.stringify(canonical(value)) + '}');
  verifyRootLink(serialize(model));
  for (const mutate of [
    value => {value.declarations.find(row => row.name === 'present').body = {kind: 'var', name: 'request'};},
    value => {value.declarations.find(row => row.name === 'present').body.arguments = [{hex: '', kind: 'bytes'}];},
    value => {value.declarations.find(row => row.name === 'present').body.function.module = 'Foundry.UI.App';},
    value => {value.declarations.push(structuredClone(value.declarations.find(row => row.name === 'present')));},
    value => {field(field(value.declarations.find(row => row.name === 'browserApplication').body, 'view').value, 'labels').value = call('anonymousLabelText');},
    value => {field(field(value.declarations.find(row => row.name === 'browserApplication').body, 'view').value, 'presentationRoot').value = {kind: 'string', value: 'PrismFoundry.Foundry.dispatch'};},
  ]) {
    const changed = structuredClone(model); mutate(changed);
    assert.throws(() => verifyRootLink(serialize(changed)));
  }
  assert.throws(() => verifyRootLink(serialize(model).replace('\\importmodule{' + presentation + '}\n', '')));
  assert.throws(() => verifyRootLink(readFileSync(new URL('../../src/Foundry/UI/Journey.lex.tex', import.meta.url), 'utf8')));
  assert.throws(() => verifyRootLink(''));
});
