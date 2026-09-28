// Independent contract vectors. No product source, generated output or runtime imports.
import assert from 'node:assert/strict';
import {account, action, cbor, cborFailure, command, copy, head, maximumCounter,
  maximumFrame, octets, organization, reference, state, success, workspace} from '../application-context/corpus.mjs';
export {cbor, copy, maximumFrame};

export const selectionFailure=code=>cbor([1,1,[code]]);
export const contextFailure=code=>cbor([1,1,[0,code]]);
export const observation=(request,outcome)=>[1,copy(request),copy(outcome)];
const same=(left,right)=>cbor(left).equals(cbor(right));
export const transition=(tag,current)=>success([tag,current]);
export function outcome(tag,current,request,accountPhase=1,organizationPhase=1) {
  if(tag===1) {
    const value=account(17,accountPhase);
    if(request[6][0]===15) {value[0]=copy(request[6][1]);value[1]=copy(request[6][2]);}
    return [1,value];
  }
  if(tag===2) {
    const value=organization(19,organizationPhase,'Unreserved organization');
    if(request[6][0]===13)value[0]=copy(request[6][1]);
    return [2,value];
  }
  if(tag===3) {
    const value=workspace(23,'Selected workspace');
    if(current[4][0]===1)value[0]=copy(current[4][1][0]);
    if(request[6][0]===8)value[1]=copy(request[6][1]);
    return [3,value];
  }
  return [tag];
}

// Valid-input oracle; malformed framing and binding cases have explicit expectations below.
function expected(current,request,observed) {
  const tag=request[6][0],lookup=observed[2],kind=lookup[0];
  if(tag!==0&&current[2]===maximumCounter)return contextFailure(6);
  if([1,2,3,4,11,13].includes(tag)&&current[3][0]===0)return contextFailure(7);
  if([5,6,7,8].includes(tag)&&current[4][0]===0)return contextFailure(7);
  if([9,10].includes(tag)&&current[5][0]===0)return contextFailure(7);
  if(!same(request,observed[1]))return selectionFailure(1);
  if(![0,8,13,14,15].includes(tag))return selectionFailure(2);
  if([8,13,15].includes(tag)&&kind===7)return transition(2,current);
  if([8,13].includes(tag)&&(current[3][0]===0||current[3][1][8]!==1))return selectionFailure(8);
  if(tag===8&&![0,1].includes(current[4][1][6]))return selectionFailure(9);
  if([8,13,15].includes(tag)&&[4,5,6].includes(kind))return selectionFailure(kind+6);
  if(kind!==({0:0,8:3,13:2,14:0,15:1}[tag]))return selectionFailure(3);
  if(tag===0)return transition(0,current);
  const next=copy(current);next[2]++;
  if(tag===14) {next[4]=[0];next[5]=[0];}
  if(tag===15) {next[3]=[1,copy(lookup[1])];next[4]=[0];next[5]=[0];}
  if(tag===13) {next[4]=[1,copy(lookup[1])];next[5]=[0];}
  if(tag===8)next[5]=[1,copy(lookup[1])];
  return transition(1,next);
}

function framingCases(add,root,valid,prefix) {
  const encoded=cbor(valid);
  for(let size=0;size<encoded.length;size++)add(`${prefix}-truncated-${size}`,root,encoded.subarray(0,size),cborFailure(2));
  for(const [name,input,error]of [
    ['trailing',Buffer.concat([encoded,Buffer.from([0])]),8],
    ['indefinite',Buffer.from([0x9f]),4],['oversized-array',head(4,17),6],
    ['frame-exact-malformed',Buffer.alloc(maximumFrame),3],['frame-one-over',Buffer.alloc(maximumFrame+1),6],
    ['unknown-version',cbor([2,...valid.slice(1)]),3],['missing-field',cbor(valid.slice(0,-1)),3],
    ['extra-field',cbor([...valid,0]),3],
    ['noncanonical-version',Buffer.concat([head(4,valid.length),Buffer.from([0x18,1]),...valid.slice(1).map(cbor)]),5],
    ['noncanonical-array',Buffer.concat([Buffer.from([0x98,valid.length]),...valid.map(cbor)]),5],
  ])add(`${prefix}-${name}`,root,input,cborFailure(error));
}

export function allCases() {
  const rows=[],add=(name,root,input,output)=>rows.push({name,root,input,output});
  const reduce=(name,current,request,observed,output=expected(current,request,observed))=>
    add(name,'applicationSelectionBytes',cbor([1,current,request,observed]),output);
  for(let selected=0;selected<6;selected++)for(let ap=0;ap<4;ap++)for(let op=0;op<4;op++)
    for(const tag of [0,8,13,14,15])for(let kind=0;kind<8;kind++) {
      const current=state(selected,7,ap,op),request=command(current,action(tag));
      reduce(`matrix-${selected}-${ap}-${op}-${tag}-${kind}`,current,request,
        observation(request,outcome(kind,current,request,ap,op)));
    }
  for(let tag=0;tag<16;tag++)if(![0,8,13,14,15].includes(tag))for(let kind=0;kind<8;kind++) {
    const current=state(),request=command(current,action(tag));
    reduce(`unsupported-${tag}-${kind}`,current,request,observation(request,outcome(kind,current,request)),selectionFailure(2));
  }
  for(const counter of [0,23,24,255,256,65535,65536,maximumCounter-1,maximumCounter])
    for(const tag of [0,8,13,14,15])for(const kind of [0,1,2,3,7]) {
      const current=state(4,counter),request=command(current,action(tag));
      reduce(`counter-${counter}-${tag}-${kind}`,current,request,observation(request,outcome(kind,current,request)));
    }
  const current=state(4,9),selectAccount=command(current,action(15));
  for(const kind of [1,2])for(let prior=0;prior<4;prior++)for(let returned=0;returned<4;returned++) {
    const selected=state(4,9,kind===1?prior:1,prior),request=command(selected,action(kind===1?15:13));
    reduce(`returned-phase-${kind}-${prior}-${returned}`,selected,request,
      observation(request,outcome(kind,selected,request,returned,returned)));
  }
  for(const [kind,tag]of [[1,15],[2,13],[3,8]]) {
    const request=command(current,action(tag)),observed=observation(request,outcome(kind,current,request));
    const value=observed[2][1];
    for(const index of kind===1?[2,3,4]:kind===2?[1,2]:[2])value[index]=100+index;
    for(const index of kind===1?[5,6,7]:kind===2?[3,4,5]:[3])value[index]=reference(100+index,index%2);
    if(kind!==1)value[value.length-1]='Exact returned label';
    reduce(`returned-complete-snapshot-${kind}`,current,request,observed);
  }
  const accountOutcome=observation(selectAccount,[1,copy(current[3][1])]);
  const clearedAccount=copy(current);clearedAccount[2]++;clearedAccount[4]=[0];clearedAccount[5]=[0];
  reduce('same-account-clears-scope',current,selectAccount,accountOutcome,transition(1,clearedAccount));
  const selectOrg=command(current,[13,copy(current[4][1][0])]);
  const orgOutcome=observation(selectOrg,[2,copy(current[4][1])]);
  const clearedWorkspace=copy(current);clearedWorkspace[2]++;clearedWorkspace[5]=[0];
  reduce('same-organization-clears-workspace',current,selectOrg,orgOutcome,transition(1,clearedWorkspace));
  const empty=state(0,23),clear=command(empty,action(14)),advanced=copy(empty);advanced[2]++;
  reduce('clear-empty-advances',empty,clear,observation(clear,[0]),transition(1,advanced));
  for(const [name,update,error]of [
    ['account-namespace',v=>v[2][1][0]=octets(77),4],
    ['account-id',v=>v[2][1][1]=octets(77),5],
  ]) {const changed=copy(accountOutcome);update(changed);reduce(name,current,selectAccount,changed,selectionFailure(error));}
  for(const algorithm of [0,1]) {
    const request=command(current,[13,reference(45,algorithm)]),observed=observation(request,[2,organization(0,3,'Same name')]);
    observed[2][1][0]=reference(45,1-algorithm);
    reduce(`organization-algorithm-${algorithm}`,current,request,observed,selectionFailure(6));
  }
  const open=command(current,action(8)),workspaceOutcome=observation(open,outcome(3,current,open));
  for(const [name,index,error]of [['workspace-parent',0,6],['workspace-id',1,7]]) {
    const changed=copy(workspaceOutcome);changed[2][1][index]=reference(77);
    reduce(name,current,open,changed,selectionFailure(error));
  }
  for(const [name,change,error]of [
    ['current-instance',v=>v[1]=octets(99),2],
    ['current-account-namespace',v=>v[3][1][0]=octets(99),3],
    ['current-account-epoch',v=>v[3][1][3]++,3],
    ['current-organization-revision',v=>v[4][1][1]++,4],
    ['current-workspace-revision',v=>v[5][1][2]++,4],
    ['current-revision',v=>v[2]++,5],
  ]) {const changed=copy(current);change(changed);reduce(name,changed,selectAccount,accountOutcome,contextFailure(error));}
  for(const [name,change]of [
    ['request',v=>v[2]=reference(99)],['instance',v=>v[1]=octets(99)],['revision',v=>v[3]++],
    ['action',v=>v[6]=action(14)],['namespace',v=>v[4][1][0]=octets(99)],
    ['account-epoch',v=>v[4][1][3]++],['org-revision',v=>v[5][1][1]++],
    ['workspace-revision',v=>v[5][2][2]++],
  ]) {
    const changed=copy(accountOutcome);change(changed[1]);
    reduce(`observation-${name}`,current,selectAccount,changed,selectionFailure(1));
    changed[2]=[7];reduce(`cancelled-observation-${name}`,current,selectAccount,changed,selectionFailure(1));
  }
  const staleCancelled=copy(current);staleCancelled[2]++;
  reduce('cancelled-current-revision',staleCancelled,selectAccount,observation(selectAccount,[7]),contextFailure(5));
  const unsupported=command(current,action(1)),foreign=observation(unsupported,[7]);foreign[1][2]=reference(99);
  reduce('unsupported-foreign-observation',current,unsupported,foreign,selectionFailure(1));
  reduce('malformed-outcome-before-stale-context',staleCancelled,selectAccount,observation(selectAccount,[8]),cborFailure(3));
  for(const [label,text]of [['unicode','界'.repeat(85)+'x'],['markup','<script>ordinary display label</script>']]) {
    const observed=copy(orgOutcome);observed[2][1][7]=text;
    const next=copy(clearedWorkspace);next[4]=copy(observed[2]);next[4][0]=1;
    reduce(`label-${label}`,current,selectOrg,observed,transition(1,next));
  }
  for(let kind=0;kind<8;kind++) {
    const observed=observation(selectAccount,outcome(kind,current,selectAccount));
    add(`observation-roundtrip-${kind}`,'selectionObservationBytes',cbor(observed),success(observed));
    for(const [name,bad,error]of [
      ['missing',observed[2].slice(0,-1),3],['extra',[...observed[2],0],3],['unknown',[8],3],
    ]) {const changed=copy(observed);changed[2]=bad;add(`outcome-${kind}-${name}`,'selectionObservationBytes',cbor(changed),cborFailure(error));}
  }
  const invalidAccount=copy(accountOutcome);invalidAccount[2][1][8]=4;
  add('invalid-account-phase','selectionObservationBytes',cbor(invalidAccount),cborFailure(6));
  const invalidOrg=copy(orgOutcome);invalidOrg[2][1][6]=4;
  add('invalid-organization-phase','selectionObservationBytes',cbor(invalidOrg),cborFailure(6));
  const longLabel=copy(orgOutcome);longLabel[2][1][7]='x'.repeat(257);
  add('oversized-label','selectionObservationBytes',cbor(longLabel),cborFailure(6));
  const maximal=state(4,maximumCounter,1,1,'x'.repeat(256));maximal[2]--;
  for(let tag=0;tag<16;tag++) {
    const selectedAction=action(tag,maximumCounter,'x'.repeat(256));
    if(tag===1)selectedAction[2]='x'.repeat(320);
    const request=command(maximal,selectedAction),observed=observation(request,[2,organization(maximumCounter,3,'x'.repeat(256))]);
    if(tag===13)observed[2][1][0]=copy(request[6][1]);
    add(`maximum-observation-${tag}`,'selectionObservationBytes',cbor(observed),success(observed));
    reduce(`maximum-transition-${tag}`,maximal,request,observed);
    if(tag===4){
      assert.equal(cbor(observed).length,1855);assert.equal(cbor([1,maximal,request,observed]).length,4321);
      framingCases(add,'selectionObservationBytes',observed,'maximum-observation');
      framingCases(add,'applicationSelectionBytes',[1,maximal,request,observed],'maximum-transition');
    }
  }
  const maximalOpen=command(maximal,action(8)),maximalWorkspace=workspace(maximumCounter,'x'.repeat(256));
  maximalWorkspace[1]=copy(maximalOpen[6][1]);
  reduce('maximum-admitted-workspace',maximal,maximalOpen,observation(maximalOpen,[3,maximalWorkspace]));
  framingCases(add,'selectionObservationBytes',workspaceOutcome,'observation');
  framingCases(add,'applicationSelectionBytes',[1,current,open,workspaceOutcome],'transition');
  assert.equal(new Set(rows.map(row=>row.name)).size,rows.length);
  return rows;
}
