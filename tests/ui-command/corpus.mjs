// Independent contract vectors. No product source, generated helpers or actual outputs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {cbor as primitiveCbor, head, octets, reference, state, action, copy,
  scope, maximumCounter} from '../application-context/corpus.mjs';

export function cbor(value) {
  if (typeof value === 'boolean') return Buffer.from([value ? 0xf5 : 0xf4]);
  if (Array.isArray(value)) return Buffer.concat([head(4, value.length), ...value.map(cbor)]);
  return primitiveCbor(value);
}
export const success = value => cbor([1,0,value]);
export const failure = (code, detail) => cbor([1,1,detail === undefined ? [code] : [code,detail]]);
export const encodingFailure = code => cbor([1,2,code]);
export const resource = 'foundry/command-reference/1';
export const domain = Buffer.from(resource+'\0');
export const screens = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18];
export const screenTags = [0,1,1,1,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15];
export const labels = ['Organization overview','Verify your email','Sign in','Recover your account',
  'Change email','Continue account recovery','Replace backup codes','Create organization',
  'Propose administration change','Approve administration change','Create workspace','Open workspace',
  'Apply workspace change','Send message','Sign out','Create account','Choose organization',
  'Clear organization selection','Choose account','Email address','Organization name','Workspace name',
  'Continue','Preparing request','Request ready','Request failed','Outcome unknown','Session closed'];
export function route(screen) {
  if (screen>=1 && screen<=4) return [1,screen-1,reference(12)];
  if (screen===7) return [2,reference(15),0,reference(16),reference(17)];
  if (screen===10) return [3,reference(20)];
  return [0,action(screenTags[screen])];
}
export const context = (screen=1, revision=23, epoch=24, current=state()) =>
  [copy(current),route(screen),screen,revision,epoch];
export const effectContext = (operation=0) => [octets(31),octets(32),octets(33),operation];
export const fieldMaximum = screen => screen>=1&&screen<=4?320:[7,10].includes(screen)?256:0;
export function intent(ctx, text='citizen@example.test') {
  return [1,ctx[3],100+screenTags[ctx[2]],fieldMaximum(ctx[2])?[[5,text]]:[]];
}
export function capturedAction(ctx, submitted) {
  const screen=ctx[2], target=ctx[1], text=submitted[3][0]?.[1];
  if(screen>=1&&screen<=4)return [1,screen-1,text,copy(target[2])];
  if(screen===7)return [4,...copy(target.slice(1)),text];
  if(screen===10)return [7,copy(target[1]),text];
  return copy(target[1]);
}
export function planned(ctx=context(), effect=effectContext(), submitted=intent(ctx)) {
  const captured=capturedAction(ctx,submitted);
  const material=Buffer.concat([domain,cbor([1,...copy(effect),copy(ctx[0]),ctx[2],ctx[3],ctx[4],captured])]);
  const request=[...copy(effect),resource,[2,material]];
  const plan=[copy(ctx),copy(submitted),request];
  const digest=createHash('sha256').update(material).digest();
  const command=[1,copy(ctx[0][1]),[1,digest],ctx[0][2],copy(ctx[0][3]),scope(ctx[0]),captured];
  return {plan,request,material,digest,command,completion:[copy(request),[2,'sha256:'+digest.toString('hex')]]};
}
export function presentation(ctx) {
  const screen=ctx[2], maximum=fieldMaximum(screen);
  const nodes=[[0,[0,screen]],[1,[3,1,screen]],
    [1,[4,'Your selection determines the request scope. Access is checked before any change.']],
    [1,[2,screen]]];
  if(maximum)nodes.push([4,[5,screen<=4?19:screen===7?20:21,true,true,maximum,'',ctx[4]]]);
  nodes.push([4,[8,screen,100+screenTags[screen],true,true,maximum?[5]:[]]]);
  return [1,ctx[3],0,0,0,0,nodes];
}
export function allCases() {
  const rows=[];
  const add=(name,input,output)=>rows.push({name,root:'commandBridgeBytes',input,output});
  for(const screen of screens)for(const revision of [0,23,24,255,256,65535,65536,maximumCounter]) {
    const ctx=context(screen,revision,revision), effect=effectContext(revision===maximumCounter?revision-1:revision);
    const submitted=intent(ctx), p=planned(ctx,effect,submitted);
    add(`prepare-${screen}-${revision}`,cbor([1,0,ctx,effect,submitted]),success([0,p.plan]));
    add(`resolve-${screen}-${revision}`,cbor([1,1,[0,p.plan],ctx,p.completion]),success([1,p.plan,p.command]));
    add(`view-${screen}-${revision}`,cbor([1,3,ctx]),success(presentation(ctx)));
    // Explicit counterexample: a second cloned old Pending value resolves identically.
    add(`pure-cloned-pending-${screen}-${revision}`,cbor([1,1,[0,copy(p.plan)],copy(ctx),copy(p.completion)]),success([1,p.plan,p.command]));
    const closed=[4,p.plan];
    for(const [name,continuation]of [['pending',[0,p.plan]],['resolved',[1,p.plan,p.command]],
      ['failed',[2,p.plan,[1]]],['unknown',[3,p.plan]],['closed',closed]]) {
      add(`close-${screen}-${revision}-${name}`,cbor([1,2,continuation]),success(closed));
      if(name!=='pending')add(`terminal-${screen}-${revision}-${name}`,cbor([1,1,continuation,ctx,p.completion]),failure(5));
    }
    add(`unknown-${screen}-${revision}`,cbor([1,1,[0,p.plan],ctx,[p.request,[9]]]),success([3,p.plan]));
    add(`stale-view-${screen}-${revision}`,cbor([1,0,ctx,effect,[1,revision===0?1:0,submitted[2],submitted[3]]]),failure(1));
    add(`wrong-action-${screen}-${revision}`,cbor([1,0,ctx,effect,[1,revision,200,submitted[3]]]),failure(1));
  }
  for(const screen of [1,2,3,4,7,10]) {
    const ctx=context(screen), effect=effectContext();
    for(const text of ['x','é','\ufeffa',' A@example.test ', 'x'.repeat(fieldMaximum(screen)), 'é'.repeat(fieldMaximum(screen)/2)]) {
      const submitted=intent(ctx,text), p=planned(ctx,effect,submitted);
      add(`text-${screen}-${Buffer.from(text).toString('hex')}`,cbor([1,0,ctx,effect,submitted]),success([0,p.plan]));
    }
    for(const [name,fields]of [['empty',[[5,'']]],['missing',[]],['wrong-node',[[14,'x']]],['duplicate',[[5,'x'],[5,'x']]],
      ['extra',[[5,'x'],[6,'y']]],['choice',[[5,1]]],['one-over',[[5,'x'.repeat(fieldMaximum(screen)+1)]]]])
      add(`bad-fields-${screen}-${name}`,cbor([1,0,ctx,effect,[1,ctx[3],100+screenTags[screen],fields]]),failure(1));
  }
  const ctx=context(), effect=effectContext(), submitted=intent(ctx), p=planned(ctx,effect,submitted);
  for(const index of screens) {
    if(index===1)continue;
    const wrong=copy(ctx);wrong[2]=index;
    add(`wrong-screen-${index}`,cbor([1,0,wrong,effect,submitted]),failure(0));
  }
  for(const index of [0,1,2])for(const size of [0,31,33]) {
    const wrong=copy(effect);wrong[index]=Buffer.alloc(size);
    add(`effect-width-${index}-${size}`,cbor([1,0,ctx,wrong,submitted]),encodingFailure(6));
  }
  add('effect-counter-exhausted',cbor([1,0,ctx,effectContext(maximumCounter),submitted]),failure(2));
  const withoutAccount=context(1,23,24,state(0));
  add('mailbox-needs-account',cbor([1,0,withoutAccount,effect,intent(withoutAccount)]),failure(8,7));
  const missingAndExhausted=context(1,23,24,state(0,maximumCounter));
  add('selection-before-exhaustion',cbor([1,0,missingAndExhausted,effect,intent(missingAndExhausted)]),failure(8,7));
  for(const screen of [0,15,18]) {
    const empty=context(screen,0,0,state(0)), submitted=intent(empty), candidate=planned(empty,effect,submitted);
    add(`empty-prepare-${screen}`,cbor([1,0,empty,effect,submitted]),success([0,candidate.plan]));
    add(`empty-resolve-${screen}`,cbor([1,1,[0,candidate.plan],empty,candidate.completion]),success([1,candidate.plan,candidate.command]));
  }
  for(const screen of [0,1,15,18]) {
    const exhausted=context(screen,23,24,state(4,maximumCounter));
    add(`application-counter-${screen}`,cbor([1,0,exhausted,effect,intent(exhausted)]),screen===0?success([0,planned(exhausted,effect).plan]):failure(8,6));
  }
  for(const [name,path,replacement]of [
    ['instance',[0,1],octets(55)],['application-revision',[0,2],1],['account-namespace',[0,3,1,0],octets(55)],
    ['account-id',[0,3,1,1],octets(55)],['account-revision',[0,3,1,2],1],['credential-epoch',[0,3,1,3],1],
    ['recovery-epoch',[0,3,1,4],1],['credential-ref',[0,3,1,5],reference(55)],['authorization-ref',[0,3,1,6],reference(55)],
    ['account-state',[0,3,1,7],reference(55)],['account-phase',[0,3,1,8],3],
    ['organization-revision',[0,4,1,1],1],['authority-revision',[0,4,1,2],1],['policy',[0,4,1,3],reference(55)],
    ['organization-state',[0,4,1,4],reference(55)],['workspace-index',[0,4,1,5],reference(55)],['organization-phase',[0,4,1,6],3],
    ['organization-label',[0,4,1,7],'Other'],['workspace-reference',[0,5,1,1],reference(55)],['workspace-revision',[0,5,1,2],1],
    ['workspace-state',[0,5,1,3],reference(55)],['workspace-label',[0,5,1,4],'Other'],
    ['route-reference',[1,2],reference(55)],['view-revision',[3],99],['draft-epoch',[4],99],
  ]) {
    const current=copy(ctx);let parent=current;for(const key of path.slice(0,-1))parent=parent[key];parent[path.at(-1)]=copy(replacement);
    add(`changed-context-${name}`,cbor([1,1,[0,p.plan],current,p.completion]),failure(4));
  }
  for(const [name,index,value]of [['application',0,octets(60)],['manifest',1,octets(60)],['session',2,octets(60)],['operation',3,1],['resource',4,'other'],['payload',5,[2,Buffer.from('other')]]]) {
    const completion=copy(p.completion);completion[0][index]=copy(value);
    add(`wrong-completion-${name}`,cbor([1,1,[0,p.plan],ctx,completion]),failure(3));
    const plan=copy(p.plan);plan[2][index]=copy(value);
    // A valid different contextual request is data, not an authentic handle.
    if(index<4) {
      const freshEffect=copy(effect);freshEffect[index]=copy(value);
      const other=planned(ctx,freshEffect,submitted);
      add(`substituted-plan-${name}`,cbor([1,1,[0,plan],ctx,other.completion]),failure(3));
    } else add(`substituted-plan-${name}`,cbor([1,1,[0,plan],ctx,p.completion]),failure(3));
    const matched=copy(p.completion);matched[0]=copy(plan[2]);
    add(`tampered-plan-completion-${name}`,cbor([1,1,[0,plan],ctx,matched]),failure(3));
  }
  for(const digest of ['','sha256:'+'a'.repeat(63),'sha256:'+'a'.repeat(65),'sha256:'+'A'.repeat(64),'blake3:'+'a'.repeat(64)])
    add(`bad-digest-${digest}`,cbor([1,1,[0,p.plan],ctx,[p.request,[2,digest]]]),failure(6));
  for(const [name,result]of [['guest',[0,Buffer.alloc(0)]],['random',[1,octets(1)]],['signature',[3,Buffer.alloc(64)]],
    ['verification',[4,true]],['object',[5,[0]]],['head',[6,[0]]],['committed',[7,'sha256:'+'a'.repeat(64)]]])
    add(`wrong-outcome-${name}`,cbor([1,1,[0,p.plan],ctx,[p.request,result]]),failure(3));
  for(let cause=0;cause<13;cause++)
    add(`rejected-outcome-${cause}`,cbor([1,1,[0,p.plan],ctx,[p.request,[8,[cause]]]]),
      [0,2].includes(cause)?success([2,p.plan,[cause]]):failure(3));
  for(let i=0;i<labels.length;i++)add(`label-${i}`,cbor([1,4,i]),success(labels[i]));
  add('unknown-label',cbor([1,4,labels.length]),encodingFailure(6));
  const complete=cbor([1,0,ctx,effect,submitted]);
  for(let size=0;size<complete.length;size++)add(`truncated-prepare-${size}`,complete.subarray(0,size),encodingFailure(2));
  add('trailing',Buffer.concat([complete,Buffer.from([0])]),encodingFailure(8));
  add('unknown-version',cbor([2,0,ctx,effect,submitted]),encodingFailure(3));
  add('unknown-operation',cbor([1,5,ctx]),encodingFailure(3));
  add('wrong-root-arity',cbor([1,0,ctx,effect]),encodingFailure(3));
  add('noncanonical-version',Buffer.concat([Buffer.from([0x85,0x18,1]),complete.subarray(2)]),encodingFailure(5));
  add('exact-malformed-frame',Buffer.alloc(16384),encodingFailure(3));
  add('over-frame',Buffer.alloc(16385),encodingFailure(6));
  assert.equal(new Set(rows.map(row=>row.name)).size,rows.length);
  return rows;
}
