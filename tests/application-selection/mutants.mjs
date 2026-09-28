import assert from 'node:assert/strict';

export const mutants=Object.freeze([
  {name:'observation-binding',vector:'observation-request'},
  {name:'cancel-before-binding',vector:'cancelled-observation-request'},
  {name:'account-namespace',vector:'account-namespace'},
  {name:'account-id',vector:'account-id'},
  {name:'workspace-parent',vector:'workspace-parent'},
  {name:'account-clearing',vector:'same-account-clears-scope'},
  {name:'organization-clearing',vector:'same-organization-clears-workspace'},
  {name:'readonly-revision',vector:'matrix-4-1-1-0-0'},
  {name:'cancelled-revision',vector:'matrix-4-2-3-8-7'},
  {name:'account-status',vector:'matrix-4-2-1-13-2'},
  {name:'organization-status',vector:'matrix-4-1-3-8-3'},
  {name:'current-revision',vector:'current-revision'},
  {name:'exact-frame-bound',vector:'transition-frame-exact-malformed'},
]);
const walk=(value,fn)=>{if(value&&typeof value==='object'){fn(value);for(const child of Object.values(value))walk(child,fn);}};
function only(value,predicate){const rows=[];walk(value,node=>{if(predicate(node))rows.push(node);});assert.equal(rows.length,1,'one exact mutation target');return rows[0];}
const variable=name=>({kind:'var',name});
const project=field=>({kind:'project',value:variable('state'),field});
export function mutate(source,name){
  assert.ok(mutants.some(row=>row.name===name));
  const lines=source.split('\n'),index=lines.findIndex(line=>line.startsWith('\\semanticdata{'));
  assert.ok(index>=0);const model=JSON.parse(lines[index].slice(14,-1));
  const declaration=label=>{const found=model.declarations.filter(row=>row.name===label);assert.equal(found.length,1);return found[0];};
  const reducer=declaration('reduceApplicationSelection');
  if(['observation-binding','account-namespace','account-id','workspace-parent'].includes(name)) {
    const predicate={
      'observation-binding':node=>node.kind==='call'&&node.function.name==='contextBytesEqual'&&node.arguments[0]?.name==='commandBytes',
      'account-namespace':node=>node.kind==='call'&&node.function.name==='contextBytesEqual'&&node.arguments[1]?.name==='namespace',
      'account-id':node=>node.kind==='call'&&node.function.name==='contextBytesEqual'&&node.arguments[1]?.name==='accountId',
      'workspace-parent':node=>node.kind==='call'&&node.function.name==='contextDigestEqual'&&node.arguments[0]?.field==='organizationRef'&&node.arguments[1]?.value?.name==='organization',
    }[name];
    const selected=only(reducer,predicate);selected.arguments[1]=structuredClone(selected.arguments[0]);
  } else if(name==='cancel-before-binding') {
    const accountAction=only(reducer,node=>node.constructor?.name==='ApplicationAction.SelectAccount'&&node.body);
    const cancelled=only(accountAction,node=>node.kind==='constructor'&&node.constructor.name==='Result.ok'
      &&node.arguments[0]?.constructor?.name==='SelectionTransition.Cancelled');
    assert.equal(reducer.body.kind,'match');
    const admitted=reducer.body.branches.find(row=>row.constructor.name==='Result.ok');
    admitted.body={kind:'if',condition:{kind:'call',function:{name:'selectionCancelled'},arguments:[
      {kind:'project',value:variable('observation'),field:'outcome'}]},
    then_value:structuredClone(cancelled),else_value:admitted.body};
  } else if(['account-clearing','organization-clearing'].includes(name)) {
    const action=only(reducer,node=>node.constructor?.name===`ApplicationAction.${name==='account-clearing'?'SelectAccount':'SelectOrganization'}`&&node.body);
    const next=only(action,node=>node.kind==='record'&&node.type.name==='ApplicationState');
    const fieldName=name==='account-clearing'?'organization':'workspace';
    next.fields.find(row=>row.field===fieldName).value=project(fieldName);
  } else if(['readonly-revision','cancelled-revision'].includes(name)) {
    const constructor=`SelectionTransition.${name==='readonly-revision'?'ReadOnly':'Cancelled'}`;
    let count=0;walk(reducer,node=>{if(node.kind==='constructor'&&node.constructor.name===constructor){
      node.arguments[0]={kind:'record',type:{module:'Foundry.Application.State',name:'ApplicationState'},fields:
        ['instance','revision','account','organization','workspace'].map(field=>({field,value:field==='revision'?{kind:'add',left:project(field),right:{kind:'nat',value:'1'}}:project(field)}))};count++;
    }});assert.equal(count,name==='readonly-revision'?1:3);
  } else if(['account-status','organization-status'].includes(name)) {
    const target=declaration(name==='account-status'?'selectionAccountEligible':'selectionOrganizationEligible');
    const branch=only(target,node=>node.constructor?.name===(name==='account-status'?'AccountPhase.Suspended':'OrganizationPhase.Retired')&&node.body);
    assert.deepEqual(branch.body,{kind:'bool',value:false});branch.body.value=true;
  } else if(name==='current-revision') {
    const call=only(reducer,node=>node.kind==='call'&&node.function.name==='checkApplicationCommand');
    call.arguments[1]={kind:'record',type:{module:'Foundry.Application.Command',name:'ApplicationCommand'},fields:
      ['instance','requestRef','expectedRevision','account','scope','action'].map(field=>({field,value:field==='expectedRevision'?project('revision'):{kind:'project',value:variable('command'),field}}))};
  } else {
    const limit=only(declaration('applicationSelectionBytes'),node=>node.kind==='ble'&&node.right?.value==='16384');
    limit.right.value='16383';
  }
  const canonical=value=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'
    ?Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])])):value;
  lines[index]='\\semanticdata{'+JSON.stringify(canonical(model))+'}';const changed=lines.join('\n');assert.notEqual(changed,source);return changed;
}
