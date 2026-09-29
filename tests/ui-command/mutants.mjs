// Each modeled defect must freshly pass the kernel and fail its independently
// specified native/std/no_std/Wasm counterexample. Source checks are not proof.
import assert from 'node:assert/strict';

export const mutants = Object.freeze([
  {name:'view-intent-admission', vector:'stale-view-1-23'},
  {name:'current-context', vector:'changed-context-draft-epoch'},
  {name:'completion-binding', vector:'wrong-completion-session'},
  {name:'captured-plan-integrity', vector:'tampered-plan-completion-payload'},
  {name:'closed-continuation', vector:'terminal-1-23-closed'},
  {name:'typed-digest-rejection', vector:'rejected-outcome-1'},
  {name:'mailbox-operation', vector:'prepare-3-23'},
  {name:'digest-domain', vector:'prepare-1-23'},
  {name:'effect-counter-exhaustion', vector:'effect-counter-exhausted'},
]);
function one(value, predicate) {
  const found = [];
  function visit(node) {
    if (!node || typeof node !== 'object') return;
    if (predicate(node)) found.push(node);
    for (const child of Object.values(node)) if (Array.isArray(child)) child.forEach(visit); else visit(child);
  }
  visit(value); assert.equal(found.length, 1, 'one exact modeled mutation'); return found[0];
}
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(key => [key, canonical(value[key])]));
  return value;
}
export function mutateSource(source, name) {
  assert.ok(mutants.some(row => row.name === name), 'closed mutant inventory');
  assert.ok(source.startsWith('\\begin{lexlean}{Foundry.UI.CommandBridge}\n'));
  const lines = source.split('\n'), index = lines.findIndex(line => line.startsWith('\\semanticdata{'));
  assert.ok(index >= 0); const model = JSON.parse(lines[index].slice(14, -1));
  const declaration = name => { const value = model.declarations.find(row => row.name === name); assert.ok(value); return value; };
  const settle = declaration('settleCommandBridge');
  if (name === 'view-intent-admission') {
    const node = one(declaration('captureCommandAction'), node => node.kind === 'if' && node.condition?.function?.name === 'intentFits');
    node.condition = {kind:'bool', value:true};
  } else if (['current-context', 'completion-binding', 'captured-plan-integrity', 'typed-digest-rejection'].includes(name)) {
    const selected = {
      'current-context': node => node.condition?.function?.name === 'contextBytesEqual',
      'completion-binding': node => node.condition?.function?.name === 'effectRequestEqual' && node.condition.arguments?.[1]?.value?.name === 'completion',
      'captured-plan-integrity': node => node.condition?.function?.name === 'effectRequestEqual' && node.condition.arguments?.[1]?.value?.name === 'reconstructed',
      'typed-digest-rejection': node => node.condition?.function?.name === 'effectResultValid',
    }[name];
    one(settle, node => node.kind === 'if' && selected(node)).condition = {kind:'bool', value:true};
  } else if (name === 'closed-continuation') {
    const pending = one(settle, node => node.constructor?.name === 'CommandContinuation.Pending' && Array.isArray(node.binders));
    const closed = one(settle, node => node.constructor?.name === 'CommandContinuation.Closed' && Array.isArray(node.binders));
    closed.body = structuredClone(pending.body);
  } else if (name === 'mailbox-operation') {
    const branch = one(declaration('commandMailboxOperation'), node => node.constructor?.name === 'MailboxOperation.Recover');
    assert.equal(branch.body.value, '2'); branch.body.value = '1';
  } else if (name === 'digest-domain') {
    const bytes = one(declaration('commandDigestMaterial'), node => node.kind === 'bytes' && node.hex?.endsWith('2f3100'));
    bytes.hex = bytes.hex.slice(0, -4) + '3200';
  } else if (name === 'effect-counter-exhaustion') {
    const condition = one(declaration('prepareCommandBridge'), node => node.kind === 'blt' && node.left?.field === 'operation');
    condition.kind = 'ble';
  } else assert.fail('unhandled mutation');
  lines[index] = '\\semanticdata{' + JSON.stringify(canonical(model)) + '}';
  const result = lines.join('\n'); assert.notEqual(result, source); return result;
}
