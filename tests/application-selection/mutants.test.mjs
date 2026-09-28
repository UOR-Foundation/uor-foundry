import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {allCases} from './corpus.mjs';
import {mutants,mutate} from './mutants.mjs';

test('SC-01 mutation fixtures target distinct real source semantics and finite probes',()=>{
  const source=readFileSync(new URL('../../src/Foundry/Application/Selection.lex.tex',import.meta.url),'utf8');
  const cases=allCases(),names=new Set(cases.map(row=>row.name)),changed=new Set();
  assert.equal(mutants.length,13);
  assert.equal(new Set(mutants.map(row=>row.name)).size,13);
  for(const mutant of mutants){
    assert.ok(names.has(mutant.vector),mutant.vector);
    const value=mutate(source,mutant.name);assert.notEqual(value,source);changed.add(value);
    assert.equal(cases.find(row=>row.name===mutant.vector).root,'applicationSelectionBytes');
  }
  assert.equal(changed.size,13);
});
