import assert from 'node:assert/strict';
import test from 'node:test';
import {action, allCases, cbor, command, commandCases, contextCases, referenceCases, state, stateCases} from './corpus.mjs';

test('independent encoder fixes RFC 8949 boundaries and complete strings', () => {
  for (const [value, expected] of [[0,'00'],[23,'17'],[24,'1818'],[255,'18ff'],[256,'190100'],[65535,'19ffff'],[65536,'1a00010000'],[4294967295,'1affffffff']]) {
    assert.equal(cbor(value).toString('hex'),expected);
  }
  assert.equal(cbor([1,Buffer.from([0,255]),'\u00e9']).toString('hex'),'83014200ff62c3a9');
  assert.throws(()=>cbor(4294967296));
  assert.throws(()=>cbor(-1));
});

test('reference corpus independently covers every octet and exact malformed boundaries', () => {
  const rows=referenceCases();
  assert.ok(rows.length>1700);
  assert.equal(rows.filter(row=>row.name.startsWith('reference-')).length,1024);
  assert.equal(rows.filter(row=>/^account-\d+-/u.test(row.name)).length,512);
  assert.ok(rows.some(row=>row.name==='one-over-frame'&&row.input.length===16385));
  assert.ok(rows.some(row=>row.name==='account-disallows-blake3'));
});

test('state and command fixtures cover every lifecycle and selection without seeded authority', () => {
  const states=stateCases(),commands=commandCases(),contexts=contextCases();
  assert.equal(states.filter(row=>/^state-\d/u.test(row.name)).length,8*4*4*6);
  assert.equal(commands.filter(row=>/^command-\d/u.test(row.name)).length,8*6*16);
  assert.equal(contexts.filter(row=>/^context-\d/u.test(row.name)).length,8*6*16);
  for(let tag=0;tag<16;tag++)assert.ok(contexts.some(row=>row.name===`context-0-0-${tag}`));
  assert.deepEqual(state(0),[1,Buffer.from(Array.from({length:32},(_,i)=>i)),0,[0],[0],[0]]);
  assert.equal(command(state(0),action(15))[6][0],15);
  const maximum=state(4,4294967295,3,3,'x'.repeat(256));
  const maximumCommand=command(maximum,action(4,4294967295,'x'.repeat(256)));
  assert.equal(cbor(maximum).length,1028);
  assert.equal(cbor(maximumCommand).length,1436);
  assert.equal(cbor([1,maximum,maximumCommand]).length,2466);
  assert.equal(allCases().length,referenceCases().length+states.length+commands.length+contexts.length);
});
