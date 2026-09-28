import assert from 'node:assert/strict';
import test from 'node:test';
import {allCases,cbor} from './corpus.mjs';

test('SC-01 independent encoder preserves canonical RFC 8949 boundary bytes',()=>{
  for(const [value,hex]of [[0,'00'],[23,'17'],[24,'1818'],[255,'18ff'],[256,'190100'],
    [65535,'19ffff'],[65536,'1a00010000'],[4294967295,'1affffffff']])assert.equal(cbor(value).toString('hex'),hex);
  assert.equal(cbor([1,Buffer.from([0,255]),'界']).toString('hex'),'83014200ff63e7958c');
});

test('SC-01 corpus retains all selection outcome and phase cross-products',()=>{
  const rows=allCases(),names=new Set(rows.map(row=>row.name));
  assert.equal(rows.length,13000,'complete closed selection corpus');
  assert.equal(names.size,rows.length);
  assert.equal(rows.filter(row=>row.name.startsWith('matrix-')).length,3840);
  assert.equal(rows.filter(row=>/^unsupported-[0-9]+-[0-9]+$/u.test(row.name)).length,88);
  assert.equal(rows.filter(row=>row.name.startsWith('counter-')).length,225);
  for(const name of ['same-account-clears-scope','same-organization-clears-workspace','clear-empty-advances',
    'account-namespace','workspace-parent','observation-request','current-revision',
    'observation-frame-one-over','transition-frame-one-over','matrix-4-2-3-8-7',
    'cancelled-observation-request','cancelled-current-revision','unsupported-foreign-observation',
    'malformed-outcome-before-stale-context','maximum-observation-4','maximum-transition-4',
    'maximum-admitted-workspace'])assert.ok(names.has(name),name);
  for(const row of rows)assert.ok(Buffer.isBuffer(row.input)&&Buffer.isBuffer(row.output));
});
