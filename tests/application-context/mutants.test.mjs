import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import {allCases} from './corpus.mjs';
import {mutants,mutateSource} from './mutants.mjs';

test('each closed semantic mutant has an independent exact runtime counterexample',()=>{
  const rows=allCases();assert.equal(mutants.length,9);
  assert.equal(new Set(mutants.map(row=>row.name)).size,mutants.length);
  for(const mutant of mutants) {
    const source=readFileSync(new URL(`../../src/Foundry/Application/${mutant.module}.lex.tex`,import.meta.url),'utf8');
    const output=mutateSource(source,mutant.name);
    assert.notEqual(output,source);
    assert.equal(rows.filter(row=>row.root===mutant.root&&row.name===mutant.vector).length,1);
    assert.equal(output.split('\n').filter(line=>line.startsWith('\\semanticdata{')).length,1);
    assert.equal(JSON.parse(output.split('\n').find(line=>line.startsWith('\\semanticdata{')).slice(14,-1)).spec,'lexlean/semantic-module/1');
  }
  assert.throws(()=>mutateSource('','unknown'));
});
