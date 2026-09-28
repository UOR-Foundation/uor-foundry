// Test-only semantic defects. Every candidate must freshly pass the kernel and
// then fail an independent generated native/Wasm vector, not a source-name check.
import assert from 'node:assert/strict';

export const mutants = Object.freeze([
  {name:'account-algorithm',module:'Reference',root:'referenceBytes',vector:'account-disallows-blake3'},
  {name:'hex-table',module:'Reference',root:'referenceBytes',vector:'reference-0-0-encode'},
  {name:'workspace-hierarchy',module:'State',root:'applicationStateBytes',vector:'workspace-wrong-organization'},
  {name:'account-namespace-binding',module:'Command',root:'applicationContextBytes',vector:'account-field-0'},
  {name:'workspace-revision-binding',module:'Command',root:'applicationContextBytes',vector:'workspace-field-2'},
  {name:'application-revision-binding',module:'Command',root:'applicationContextBytes',vector:'revision'},
  {name:'revision-exhaustion',module:'Command',root:'applicationContextBytes',vector:'context-4-4294967295-1'},
  {name:'missing-account-selection',module:'Command',root:'applicationContextBytes',vector:'context-0-0-1'},
  {name:'mailbox-byte-budget',module:'Command',root:'applicationCommandBytes',vector:'long-mailbox'},
]);

function visit(value,predicate) {
  const matches=[];
  function walk(node) {
    if(!node||typeof node!=='object')return;
    if(predicate(node))matches.push(node);
    for(const child of Object.values(node))if(Array.isArray(child))child.forEach(walk);else walk(child);
  }
  walk(value);return matches;
}
function one(value,predicate) {
  const matches=visit(value,predicate);assert.equal(matches.length,1,'exact semantic mutation site');return matches[0];
}
function canonical(value) {
  if(Array.isArray(value))return value.map(canonical);
  if(value&&typeof value==='object')return Object.fromEntries(Object.keys(value).sort().map(key=>[key,canonical(value[key])]));
  return value;
}

export function mutateSource(source,name) {
  const specification=mutants.find(row=>row.name===name);assert.ok(specification,'closed source mutant');
  assert.ok(source.startsWith(`\\begin{lexlean}{Foundry.Application.${specification.module}}\n`));
  const lines=source.split('\n'),index=lines.findIndex(line=>line.startsWith('\\semanticdata{'));
  assert.ok(index>=0);const model=JSON.parse(lines[index].slice(14,-1));
  const declaration=name=>{const row=model.declarations.find(row=>row.name===name);assert.ok(row);return row;};
  if(name==='account-algorithm') {
    const branch=one(declaration('accountReferenceBytes'),node=>node.constructor?.name==='HashAlgorithm.Blake3');
    branch.body={arguments:[{field:'bytes',kind:'project',value:{kind:'var',name:'digest'}}],constructor:{name:'Result.ok'},kind:'constructor',type_arguments:[{kind:'bytes'},{arguments:[],kind:'named',member:{module:'Foundation.Codec.Cbor.V1.Primitive',name:'CborError'}}]};
  } else if(name==='hex-table') {
    const literal=declaration('referenceHexTable').body;assert.equal(literal.kind,'bytes');
    // Only the canonical encoding of ff changes, preserving byte widths.
    assert.ok(literal.hex.endsWith('6666'));literal.hex=literal.hex.slice(0,-4)+'3030';
  } else if(name==='workspace-hierarchy') {
    const comparison=one(declaration('applicationSelectionValid'),node=>node.function?.name==='contextDigestEqual');
    comparison.arguments[1]=structuredClone(comparison.arguments[0]);
  } else if(name==='account-namespace-binding') {
    const comparison=one(declaration('sameAccountSnapshot'),node=>node.function?.name==='contextBytesEqual'&&node.arguments?.[0]?.field==='namespace');
    comparison.arguments[1]=structuredClone(comparison.arguments[0]);
  } else if(name==='workspace-revision-binding') {
    const comparison=one(declaration('sameWorkspaceSnapshot'),node=>node.kind==='beq'&&node.left?.field==='revision');
    comparison.right=structuredClone(comparison.left);
  } else if(name==='application-revision-binding') {
    const comparison=one(declaration('checkApplicationCommand'),node=>node.kind==='beq'&&node.right?.field==='expectedRevision');
    comparison.right=structuredClone(comparison.left);
  } else if(name==='revision-exhaustion') {
    const comparison=one(declaration('checkApplicationCommand'),node=>node.kind==='blt'&&node.right?.value==='4294967295');comparison.kind='ble';
  } else if(name==='missing-account-selection') {
    const branch=one(declaration('applicationActionHasSelection'),node=>node.constructor?.name==='ApplicationAction.BeginMailbox');branch.body={kind:'bool',value:true};
  } else if(name==='mailbox-byte-budget') {
    for(const declarationName of ['readApplicationAction','writeApplicationAction']) {
      const operation=one(declaration(declarationName),node=>['contextReadText','contextWriteText'].includes(node.function?.name)&&node.arguments?.[1]?.value==='320');
      operation.arguments[1].value='321';
    }
  } else assert.fail('unhandled source mutant');
  lines[index]='\\semanticdata{'+JSON.stringify(canonical(model))+'}';
  const output=lines.join('\n');assert.notEqual(output,source);return output;
}
