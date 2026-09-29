// Independent canonical vectors: this module never reads or executes product source.
import assert from 'node:assert/strict';

export const maximumFrame = 16384;
export const maximumCounter = 4294967295;
export function head(major, value) {
  assert.ok(Number.isInteger(value) && value >= 0 && value <= maximumCounter);
  if (value < 24) return Buffer.from([major * 32 + value]);
  if (value <= 255) return Buffer.from([major * 32 + 24, value]);
  if (value <= 65535) return Buffer.from([major * 32 + 25, value >>> 8, value & 255]);
  return Buffer.from([major * 32 + 26, value >>> 24, (value >>> 16) & 255, (value >>> 8) & 255, value & 255]);
}
export function cbor(value) {
  if (typeof value === 'number') return head(0, value);
  if (Buffer.isBuffer(value)) return Buffer.concat([head(2, value.length), value]);
  if (typeof value === 'string') {
    const bytes = Buffer.from(value, 'utf8');
    return Buffer.concat([head(3, bytes.length), bytes]);
  }
  assert.ok(Array.isArray(value));
  return Buffer.concat([head(4, value.length), ...value.map(cbor)]);
}
export const cborFailure = code => cbor([1, 2, code]);
export const contextFailure = code => cbor([1, 1, code]);
export const success = value => cbor([1, 0, value]);

export function referenceCases() {
  const rows = [];
  const add = (name, input, output) => rows.push({name, root:'referenceBytes', input, output});
  const digest = Buffer.from(Array.from({length:32}, (_, i) => i));
  for (let octet = 0; octet <= 255; octet++) {
    const value = Buffer.alloc(32, octet);
    value[31] = 255 - octet;
    for (const [algorithm, prefix] of [[0, 'blake3:'], [1, 'sha256:']]) {
      const text = prefix + value.toString('hex');
      add(`reference-${algorithm}-${octet}-encode`, cbor([1,0,[algorithm,value]]), success(text));
      add(`reference-${algorithm}-${octet}-decode`, cbor([1,1,text]), success([algorithm,value]));
    }
    const account = 'sha256:' + value.toString('hex');
    add(`account-${octet}-encode`, cbor([1,2,value]), success(account));
    add(`account-${octet}-decode`, cbor([1,3,account]), success(value));
  }
  for (const [name, text] of [
    ['missing-prefix', digest.toString('hex')],
    ['wrong-prefix', 'sha512:' + digest.toString('hex')],
    ['uppercase-prefix', 'SHA256:' + digest.toString('hex')],
    ['uppercase-digit', 'sha256:' + 'A'.repeat(64)],
    ['short', 'sha256:' + '0'.repeat(63)],
    ['long', 'sha256:' + '0'.repeat(65)],
    ['bad-digit', 'sha256:' + 'g'.repeat(64)],
    ['space', 'sha256:' + ' '.repeat(64)],
    ['embedded-zero', 'sha256:' + '\0'.repeat(64)],
    ['unicode-lookalike', 'sha256:' + '\uff10'.repeat(64)],
    ['empty', ''],
  ]) {
    for (const operation of [1,3]) add(`text-${operation}-${name}`, cbor([1,operation,text]), cborFailure(6));
  }
  add('account-disallows-blake3', cbor([1,3,'blake3:' + digest.toString('hex')]), cborFailure(6));
  add('unknown-algorithm', cbor([1,0,[2,digest]]), cborFailure(6));
  for (const size of [0,1,31,33]) {
    add(`binary-${size}`, cbor([1,2,Buffer.alloc(size)]), cborFailure(6));
    add(`digest-${size}`, cbor([1,0,[1,Buffer.alloc(size)]]), cborFailure(6));
  }
  for (const input of [cbor([1,0,[1,digest]]),cbor([1,1,'sha256:'+digest.toString('hex')]),cbor([1,2,digest]),cbor([1,3,'sha256:'+digest.toString('hex')])]) {
    for (let size=0; size<input.length; size++) add(`truncated-${input[2]}-${size}`,input.subarray(0,size),cborFailure(2));
    add(`trailing-${input[2]}`,Buffer.concat([input,Buffer.from([0])]),cborFailure(8));
  }
  add('unknown-version',cbor([2,2,digest]),cborFailure(3));
  add('unknown-operation',cbor([1,4,digest]),cborFailure(3));
  add('short-array',cbor([1,2]),cborFailure(3));
  add('extra-field',cbor([1,2,digest,0]),cborFailure(3));
  add('oversized-array',head(4,17),cborFailure(6));
  add('indefinite-array',Buffer.from([0x9f]),cborFailure(4));
  add('noncanonical-version',Buffer.concat([Buffer.from([0x83,0x18,0x01,0x02]),cbor(digest)]),cborFailure(5));
  add('invalid-utf8',Buffer.from([0x83,0x01,0x01,0x61,0xff]),cborFailure(7));
  add('exact-malformed-frame',Buffer.alloc(maximumFrame),cborFailure(3));
  add('one-over-frame',Buffer.alloc(maximumFrame+1),cborFailure(6));
  assert.equal(new Set(rows.map(row=>row.name)).size,rows.length);
  return rows;
}

// These fixtures encode the reviewed public contract, not product AST or helpers.
export const octets = seed => Buffer.from(Array.from({length:32}, (_, i) => (seed+i)%256));
export const reference = (seed, algorithm=1) => [algorithm,octets(seed)];
export const copy = value => Buffer.isBuffer(value) ? Buffer.from(value) : Array.isArray(value) ? value.map(copy) : value;
export const account = (counter=0, phase=1) => [octets(1),octets(2),counter,counter,counter,reference(3),reference(4),reference(5),phase];
export const organization = (counter=0, phase=1, label='Community') => [reference(6),counter,counter,reference(7),reference(8),reference(9),phase,label];
export const workspace = (counter=0, label='Workspace') => [reference(6),reference(10),counter,reference(11),label];
export function state(selection=4,counter=0,accountPhase=1,organizationPhase=1,label='Community') {
  return [1,octets(0),counter,
    [1,3,4].includes(selection)?[1,account(counter,accountPhase)]:[0],
    selection>=2?[1,organization(counter,organizationPhase,label)]:[0],
    selection>=4?[1,workspace(counter,label)]:[0]];
}
export function scope(current) {
  return current[4][0]===0?[0]:current[5][0]===0?[1,copy(current[4][1])]:[2,copy(current[4][1]),copy(current[5][1])];
}
export function action(tag, counter=0, label='Community') {
  return [
    [0],[1,0,'citizen@example.test',reference(12)],[2,reference(13)],[3,reference(14)],
    [4,reference(15),counter,reference(16),reference(17),label],
    [5,reference(18),reference(19)],[6,reference(18)],[7,reference(20),label],
    [8,reference(20)],[9,reference(21)],[10,reference(22),reference(23),reference(24)],
    [11],[12,octets(25),reference(26)],[13,reference(27)],[14],[15,octets(1),octets(2)],
  ][tag];
}
export function command(current, selectedAction=action(0)) {
  return [1,copy(current[1]),reference(28),current[2],copy(current[3]),scope(current),copy(selectedAction)];
}
const boundaries=[0,23,24,255,256,65535,65536,maximumCounter];
function requireUnique(rows) {
  assert.equal(new Set(rows.map(row=>row.name)).size,rows.length);
  return rows;
}
function structuralFailures(add, valid, prefix) {
  const encoded=cbor(valid);
  for(let size=0;size<encoded.length;size++) add(`${prefix}-truncated-${size}`,encoded.subarray(0,size),cborFailure(2));
  add(`${prefix}-trailing`,Buffer.concat([encoded,Buffer.from([0])]),cborFailure(8));
  add(`${prefix}-indefinite`,Buffer.from([0x9f]),cborFailure(4));
  add(`${prefix}-oversized-array`,head(4,17),cborFailure(6));
  add(`${prefix}-exact-malformed-frame`,Buffer.alloc(maximumFrame),cborFailure(3));
  add(`${prefix}-one-over-frame`,Buffer.alloc(maximumFrame+1),cborFailure(6));
  const version=copy(valid);version[0]=2;
  add(`${prefix}-unknown-version`,cbor(version),cborFailure(3));
  add(`${prefix}-noncanonical-version`,Buffer.concat([head(4,valid.length),Buffer.from([0x18,0x01]),...valid.slice(1).map(cbor)]),cborFailure(5));
  add(`${prefix}-extra-field`,cbor([...valid,0]),cborFailure(3));
  add(`${prefix}-missing-field`,cbor(valid.slice(0,-1)),cborFailure(3));
  add(`${prefix}-noncanonical-array`,Buffer.concat([Buffer.from([0x98,valid.length]),...valid.map(cbor)]),cborFailure(5));
  add(`${prefix}-unsupported-version-width`,Buffer.concat([head(4,valid.length),Buffer.from([0x1b,0,0,0,0,0,0,0,1]),...valid.slice(1).map(cbor)]),cborFailure(4));
}
export function stateCases() {
  const rows=[],add=(name,input,output)=>rows.push({name,root:'applicationStateBytes',input,output});
  for(const counter of boundaries)for(let ap=0;ap<4;ap++)for(let op=0;op<4;op++)for(let selection=0;selection<6;selection++) {
    const value=state(selection,counter,ap,op);
    add(`state-${selection}-${counter}-${ap}-${op}`,cbor(value),success(value));
  }
  for(const label of ['x','a'.repeat(256),'é'.repeat(128),'🚀'.repeat(64),'UOR Foundation']) {
    const value=state(4,maximumCounter,3,3,label);
    add(`state-label-${Buffer.from(label).toString('hex')}`,cbor(value),success(value));
  }
  const blake=state();
  for(const [group,indexes]of [[3,[5,6,7]],[4,[0,3,4,5]],[5,[0,1,3]]])for(const index of indexes)blake[group][1][index][0]=0;
  add('all-blake3-object-references',cbor(blake),success(blake));
  const baseline=state(),cases=[
    ['instance-short',s=>s[1]=Buffer.alloc(31),6],
    ['instance-long',s=>s[1]=Buffer.alloc(33),6],
    ['account-option-tag',s=>s[3][0]=2,3],
    ['account-option-empty',s=>s[3]=[],3],
    ['account-option-extra',s=>s[3].push(0),3],
    ['organization-option-tag',s=>s[4][0]=2,3],
    ['organization-option-empty',s=>s[4]=[],3],
    ['workspace-option-tag',s=>s[5][0]=2,3],
    ['workspace-option-empty',s=>s[5]=[],3],
    ['account-namespace-short',s=>s[3][1][0]=Buffer.alloc(31),6],
    ['account-id-short',s=>s[3][1][1]=Buffer.alloc(31),6],
    ['account-phase',s=>s[3][1][8]=4,6],
    ['organization-phase',s=>s[4][1][6]=4,6],
    ['workspace-without-organization',s=>s[4]=[0],6],
    ['workspace-wrong-organization',s=>s[5][1][0]=reference(99),6],
    ['workspace-wrong-algorithm',s=>s[5][1][0][0]=0,6],
    ['organization-empty-label',s=>s[4][1][7]='',6],
    ['organization-long-label',s=>s[4][1][7]='x'.repeat(257),6],
    ['organization-long-utf8-label',s=>s[4][1][7]='é'.repeat(129),6],
    ['workspace-empty-label',s=>s[5][1][4]='',6],
    ['workspace-long-label',s=>s[5][1][4]='x'.repeat(257),6],
    ['account-record-extra',s=>s[3][1].push(0),3],
    ['organization-record-extra',s=>s[4][1].push(0),3],
    ['workspace-record-extra',s=>s[5][1].push(0),3],
  ];
  for(const [name,change,error] of cases){const value=copy(baseline);change(value);add(name,cbor(value),cborFailure(error));}
  for(const [group,indexes] of [[3,[5,6,7]],[4,[0,3,4,5]],[5,[0,1,3]]])for(const index of indexes) {
    for(const [name,change] of [['algorithm',r=>r[0]=2],['length',r=>r[1]=Buffer.alloc(31)]]) {
      const value=copy(baseline);change(value[group][1][index]);add(`reference-${group}-${index}-${name}`,cbor(value),cborFailure(6));
    }
  }
  for(const [name,tail,error]of [
    ['unsupported-revision-width',Buffer.from([0x1b,0,0,0,1,0,0,0,0]),4],
    ['noncanonical-revision',Buffer.from([0x18,0]),5],
    ['wrong-revision-type',Buffer.from([0xf4]),3],
  ])add(name,Buffer.concat([head(4,6),cbor(1),cbor(baseline[1]),tail,...baseline.slice(3).map(cbor)]),cborFailure(error));
  const beforeInstance=Buffer.from([0x86,1,0x58,33]);
  add('announced-instance-over-before-truncation',beforeInstance,cborFailure(6));
  structuralFailures(add,baseline,'state');
  return requireUnique(rows);
}
export function commandCases() {
  const rows=[],add=(name,input,output)=>rows.push({name,root:'applicationCommandBytes',input,output});
  for(const counter of boundaries)for(let selection=0;selection<6;selection++)for(let tag=0;tag<16;tag++) {
    const value=command(state(selection,counter),action(tag,counter));
    add(`command-${selection}-${counter}-${tag}`,cbor(value),success(value));
  }
  for(let operation=0;operation<4;operation++)for(const mailbox of ['x','x'.repeat(320),'é'.repeat(160),'🚀'.repeat(80)]) {
    const value=command(state(),[1,operation,mailbox,reference(12)]);
    add(`mailbox-${operation}-${Buffer.from(mailbox).toString('hex')}`,cbor(value),success(value));
  }
  for(const tag of [4,7]) {
    const value=command(state(4,maximumCounter,3,3,'x'.repeat(256)),action(tag,maximumCounter,'x'.repeat(256)));
    add(`command-maximum-structural-action-${tag}`,cbor(value),success(value));
  }
  for(let tag=0;tag<16;tag++) {
    const short=command(state(),action(tag));short[6].pop();add(`action-${tag}-missing`,cbor(short),cborFailure(3));
    const extra=command(state(),[...action(tag),0]);add(`action-${tag}-extra`,cbor(extra),cborFailure(3));
  }
  for(const [name,value,error] of [
    ['unknown-action',[16],3],['negative-action',Buffer.from([0x81,0x20]),3],
    ['unknown-operation',[1,4,'x',reference(12)],6],
    ['empty-mailbox',[1,0,'',reference(12)],6],
    ['long-mailbox',[1,0,'x'.repeat(321),reference(12)],6],
    ['long-utf8-mailbox',[1,0,'é'.repeat(161),reference(12)],6],
    ['empty-create-label',[4,reference(15),0,reference(16),reference(17),''],6],
    ['long-workspace-label',[7,reference(20),'x'.repeat(257)],6],
    ['short-genesis-namespace',[12,Buffer.alloc(31),reference(26)],6],
    ['short-selected-namespace',[15,Buffer.alloc(31),octets(2)],6],
    ['short-selected-account',[15,octets(1),Buffer.alloc(31)],6],
  ]) {
    const row=command(state(),Buffer.isBuffer(value)?[0]:value);
    const input=Buffer.isBuffer(value)?Buffer.concat([head(4,7),...row.slice(0,6).map(cbor),value]):cbor(row);
    add(name,input,cborFailure(error));
  }
  for(const [name,change,error] of [
    ['unknown-scope',s=>s[5]=[3],3],
    ['empty-scope',s=>s[5]=[],3],
    ['none-scope-extra',s=>s[5]=[0,0],3],
    ['scope-hierarchy',s=>s[5][2][0]=reference(99),6],
    ['instance-length',s=>s[1]=Buffer.alloc(31),6],
    ['request-algorithm',s=>s[2][0]=2,6],
    ['request-length',s=>s[2][1]=Buffer.alloc(33),6],
  ]) {const value=command(state());change(value);add(name,cbor(value),cborFailure(error));}
  const mailboxPrefix=Buffer.concat([head(4,7),...command(state()).slice(0,6).map(cbor),Buffer.from([0x84,1,0])]);
  add('mailbox-invalid-utf8',Buffer.concat([mailboxPrefix,Buffer.from([0x61,0xff]),cbor(reference(12))]),cborFailure(7));
  add('mailbox-noncanonical-length',Buffer.concat([mailboxPrefix,Buffer.from([0x78,1,0x61]),cbor(reference(12))]),cborFailure(5));
  add('mailbox-announced-one-over-before-truncation',Buffer.concat([mailboxPrefix,head(3,321)]),cborFailure(6));
  structuralFailures(add,command(state(),action(4)),'command');
  return requireUnique(rows);
}
export function contextCases() {
  const rows=[],add=(name,current,requested,output)=>rows.push({name,root:'applicationContextBytes',input:cbor([1,current,requested]),output});
  for(const counter of boundaries)for(let selection=0;selection<6;selection++)for(let tag=0;tag<16;tag++) {
    const current=state(selection,counter),requested=command(current,action(tag,counter));
    const missing=[1,2,3,4,11,13].includes(tag)?![1,3,4].includes(selection):[5,6,7,8].includes(tag)?selection<2:[9,10].includes(tag)?selection<4:false;
    add(`context-${selection}-${counter}-${tag}`,current,requested,counter===maximumCounter&&tag!==0?contextFailure(6):missing?contextFailure(7):success(requested));
  }
  const current=state(),base=command(current,action(10));
  const substitutions=[
    ['instance',r=>r[1]=octets(99),2],
    ['account-absent',r=>r[4]=[0],3],
    ...Array.from({length:9},(_,i)=>[`account-field-${i}`,r=>{const a=r[4][1];if(i<2)a[i]=octets(99);else if(i<5)a[i]=1;else if(i<8)a[i]=reference(99);else a[i]=2;},3]),
    ['organization-absent',r=>r[5]=[0],4],
    ['workspace-absent',r=>r[5]=[1,r[5][1]],4],
    ...Array.from({length:8},(_,i)=>[`organization-field-${i}`,r=>{const o=r[5][1];if(i===0){o[0]=reference(99);r[5][2][0]=reference(99);}else if(i===1||i===2)o[i]=1;else if(i<6)o[i]=reference(99);else if(i===6)o[i]=2;else o[i]='Changed';},4]),
    ...[1,2,3,4].map(i=>[`workspace-field-${i}`,r=>{r[5][2][i]=i===2?1:i===4?'Changed':reference(99);},4]),
    ['revision',r=>r[3]=1,5],
  ];
  for(const [name,change,error] of substitutions){const requested=copy(base);change(requested);add(name,current,requested,contextFailure(error));}
  const invalidState=copy(current);invalidState[4]=[0];add('invalid-state',invalidState,base,contextFailure(0));
  const invalidCommand=copy(base);invalidCommand[5][2][0]=reference(99);add('invalid-command',current,invalidCommand,contextFailure(1));
  const manyWrong=copy(base);manyWrong[1]=octets(99);manyWrong[4]=[0];manyWrong[5]=[0];manyWrong[3]=1;
  add('failure-order-instance',current,manyWrong,contextFailure(2));
  manyWrong[1]=copy(current[1]);add('failure-order-account',current,manyWrong,contextFailure(3));
  manyWrong[4]=copy(current[3]);add('failure-order-scope',current,manyWrong,contextFailure(4));
  manyWrong[5]=scope(current);add('failure-order-revision',current,manyWrong,contextFailure(5));
  const empty=state(0,maximumCounter);add('failure-order-exhaustion-before-selection',empty,command(empty,action(1)),contextFailure(6));
  for(const phase of [0,1,2,3]) {
    const phased=state(4,0,phase,phase),requested=command(phased,action(0));
    add(`phase-is-data-not-authorization-${phase}`,phased,requested,success(requested));
  }
  // Labels, names and request references confer no special privileges at this boundary.
  const named=state(4,0,1,1,'UOR Foundation'),namedCommand=command(named,action(4));
  add('nonunique-organization-name',named,namedCommand,success(namedCommand));
  const maximum=state(4,maximumCounter,3,3,'x'.repeat(256));
  const maximumCommand=command(maximum,action(4,maximumCounter,'x'.repeat(256)));
  add('maximum-structural-context',maximum,maximumCommand,contextFailure(6));
  maximum[2]=maximumCounter-1;maximumCommand[3]=maximumCounter-1;
  add('maximum-admitted-structural-context',maximum,maximumCommand,success(maximumCommand));
  const rawAdd=(name,input,output)=>rows.push({name,root:'applicationContextBytes',input,output});
  structuralFailures(rawAdd,[1,current,base],'context');
  return requireUnique(rows);
}

export function allCases() {
  const rows=[...referenceCases(),...stateCases(),...commandCases(),...contextCases()];
  assert.equal(new Set(rows.map(row=>`${row.root}/${row.name}`)).size,rows.length);
  return rows;
}
