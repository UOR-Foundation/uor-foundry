import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../../', import.meta.url));

test('UC-01 source owns the typed command bridge without a host implementation', () => {
  const source = readFileSync(`${root}src/Foundry/UI/CommandBridge.lex.tex`, 'utf8');
  assert.ok(source.startsWith('\\begin{lexlean}{Foundry.UI.CommandBridge}\n'));
  const rows = source.split('\n').filter(line => line.startsWith('\\semanticdata{'));
  assert.equal(rows.length, 1);
  const model = JSON.parse(rows[0].slice(14, -1));
  assert.equal(model.spec, 'lexlean/semantic-module/1');
  const declarations = new Map(model.declarations.map(row => [row.name, row]));
  assert.equal(declarations.size, model.declarations.length);
  for (const name of ['CommandRoute', 'CommandViewContext', 'CommandEffectContext',
    'CommandPlan', 'CommandContinuation', 'commandBridgePresentation',
    'prepareCommandBridge', 'settleCommandBridge', 'closeCommandBridge',
    'commandBridgeBytes']) {
    assert.ok(declarations.has(name), name);
  }
  for (const row of model.declarations) assert.notEqual(row.kind, 'axiom');
  assert.notEqual(declarations.get('commandBridgeBytes').body.kind, 'var');
  for (const module of ['Foundry.Application.Command',
    'Foundation.Browser.Application.V1.Effects', 'Foundation.View.Browser.V1.Model',
    'Foundation.Sec.V1.MailboxAdmission']) {
    assert.ok(source.includes(`\\importmodule{${module}}`), module);
  }
});
