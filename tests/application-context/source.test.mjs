import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';

const root = fileURLToPath(new URL('../../', import.meta.url));
const modules = {
  Reference: ['accountReferenceText', 'accountReferenceBytes', 'referenceBytes'],
  State: ['ApplicationState', 'readApplicationState', 'writeApplicationState', 'applicationStateBytes'],
  Command: ['ApplicationCommand', 'readApplicationCommand', 'writeApplicationCommand',
    'checkApplicationCommand', 'applicationCommandBytes', 'applicationContextBytes'],
};

test('AC-01 source owns every registered component root without a Foundation shadow', () => {
  for (const [module, expected] of Object.entries(modules)) {
    const source = readFileSync(`${root}src/Foundry/Application/${module}.lex.tex`, 'utf8');
    assert.ok(source.startsWith(`\\begin{lexlean}{Foundry.Application.${module}}\n`));
    const encoded = source.split('\n').filter(line => line.startsWith('\\semanticdata{'));
    assert.equal(encoded.length, 1);
    const model = JSON.parse(encoded[0].slice(14, -1));
    assert.equal(model.spec, 'lexlean/semantic-module/1');
    const names = model.declarations.map(row => row.name);
    assert.equal(new Set(names).size, names.length);
    for (const name of expected) assert.ok(names.includes(name), `${module}.${name}`);
    assert.ok(!/\\begin\{lexlean\}\{Foundation\./u.test(source));
    for (const row of model.declarations) {
      if (row.name.endsWith('Bytes')) assert.notEqual(row.body?.kind, 'var', `${row.name} must not echo input`);
      assert.notEqual(row.kind, 'axiom');
    }
  }
});
