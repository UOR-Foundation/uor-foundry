import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';

test('SC-01 source owns typed selection and canonical observation roots', () => {
  const source=readFileSync(new URL('../../src/Foundry/Application/Selection.lex.tex',import.meta.url),'utf8');
  assert.ok(source.startsWith('\\begin{lexlean}{Foundry.Application.Selection}\n'));
  for(const dependency of ['Reference','State','Command'])
    assert.ok(source.includes(`\\importmodule{Foundry.Application.${dependency}}`));
  const rows=source.split('\n').filter(line=>line.startsWith('\\semanticdata{'));
  assert.equal(rows.length,1);
  const model=JSON.parse(rows[0].slice(14,-1));
  assert.equal(model.spec,'lexlean/semantic-module/1');
  const names=model.declarations.map(row=>row.name);
  assert.equal(new Set(names).size,names.length);
  for(const name of ['SelectionOutcome','SelectionObservation','SelectionTransition','SelectionError',
    'reduceApplicationSelection','readSelectionObservation','writeSelectionObservation',
    'selectionObservationBytes','applicationSelectionBytes']) assert.ok(names.includes(name),name);
  for(const row of model.declarations) {
    assert.notEqual(row.kind,'axiom');
    if(row.name.endsWith('Bytes')) assert.notEqual(row.body?.kind,'var','observers cannot echo input');
  }
  assert.ok(!source.includes('\\begin{lexlean}{Foundation.'));
});
