import assert from 'node:assert/strict';
import test from 'node:test';
import {allCases,cbor,domain,planned,resource,screenTags,screens} from './corpus.mjs';

test('UC-01 independent corpus binds exact canonical framing and full route inventory', () => {
  assert.equal(cbor([1,true,false,'x',Buffer.from([255])]).toString('hex'),'8501f5f4617841ff');
  assert.equal(domain.toString('hex'),Buffer.from(resource+'\0').toString('hex'));
  assert.equal(domain.length,28);
  assert.equal(screens.length,19);
  assert.deepEqual([...new Set(screenTags)].sort((a,b)=>a-b),Array.from({length:16},(_,i)=>i));
  const p=planned();
  assert.equal(p.request[5][0],2);
  assert.equal(p.material.subarray(0,domain.length).toString(),resource+'\0');
  assert.equal(p.digest.length,32);
  const rows=allCases();
  const inventory = [
    [/^prepare-\d+-\d+$/,152], [/^resolve-\d+-\d+$/,152], [/^view-\d+-\d+$/,152],
    [/^pure-cloned-pending-\d+-\d+$/,152], [/^close-\d+-\d+-(pending|resolved|failed|unknown|closed)$/,760],
    [/^terminal-\d+-\d+-(resolved|failed|unknown|closed)$/,608], [/^unknown-\d+-\d+$/,152],
    [/^stale-view-\d+-\d+$/,152], [/^wrong-action-\d+-\d+$/,152],
    [/^text-/,36], [/^bad-fields-/,42], [/^wrong-screen-/,18], [/^effect-width-/,9],
    [/^effect-counter-exhausted$/,1], [/^mailbox-needs-account$/,1], [/^application-counter-/,4],
    [/^empty-(prepare|resolve)-/,6],
    [/^selection-before-exhaustion$/,1],
    [/^changed-context-/,25], [/^wrong-completion-/,6], [/^substituted-plan-/,6],
    [/^tampered-plan-completion-/,6], [/^bad-digest-/,5], [/^wrong-outcome-/,7],
    [/^rejected-outcome-/,13], [/^label-/,28], [/^unknown-label$/,1], [/^truncated-prepare-/,682],
    [/^trailing$/,1], [/^unknown-version$/,1], [/^unknown-operation$/,1], [/^wrong-root-arity$/,1],
    [/^noncanonical-version$/,1], [/^exact-malformed-frame$/,1], [/^over-frame$/,1],
  ];
  assert.equal(rows.length,3336);
  for(const [pattern,count]of inventory)assert.equal(rows.filter(row=>pattern.test(row.name)).length,count,String(pattern));
  for(const row of rows)assert.equal(inventory.filter(([pattern])=>pattern.test(row.name)).length,1,row.name);
  assert.equal(new Set(rows.map(row=>row.name)).size,rows.length);
  for(const row of rows){assert.ok(row.input.length<=16385);assert.ok(row.output.length<=16384);}
});
