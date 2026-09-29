// Component oracle only. This coordinator is never shipped as a Foundry host.
export async function runFixture(input) {
  const {openContextualStagedEffects, EffectHostError} = await import('./effects.mjs');
  const {encodeEffectWire: encode, decodeEffectWire: decode} = await import('./effects-wire.mjs');
  const {openPresentation} = await import('./presentation-dom.mjs');
  const check = (value, message) => { if (!value) throw Error(message); };
  const hex = value => Array.from(value, byte => byte.toString(16).padStart(2, '0')).join('');
  const unhex = value => Uint8Array.from(value.match(/../g) ?? [], byte => parseInt(byte, 16));
  const copy = value => decode(encode(value));
  const same = (left, right) => hex(encode(left)) === hex(encode(right));
  const observed = value => Promise.resolve(value).then(result => ({result}), error => ({error}));
  const tick = () => new Promise(resolve => setTimeout(resolve, 0));
  const deferred = () => {
    let resolve;
    const promise = new Promise(done => { resolve = done; });
    return {promise, resolve};
  };
  async function reject(operation, code) {
    const outcome = await observed(Promise.resolve().then(operation));
    check(outcome.error instanceof EffectHostError && outcome.error.code === code,
      'expected ' + code + ', got ' + outcome.error?.code);
  }
  const cases = [], calls = [], routes = [], hosts = [], views = [], roots = [], maximum = {};
  const wire = new Uint8Array(input.wire), bridge = new Uint8Array(input.bridge);
  const originalCompile = WebAssembly.compile, OriginalInstance = WebAssembly.Instance;
  const originalDigest = crypto.subtle.digest;
  const descriptors = Object.getOwnPropertyDescriptors(crypto.subtle);
  const modules = new WeakMap();
  let faultCompletion = false, faultUsed = false;
  WebAssembly.compile = async function(bytes) {
    const captured = new Uint8Array(bytes).slice(), value = hex(captured);
    check(value === hex(wire) || value === hex(bridge), 'only exact supplied artifacts');
    const module = await Reflect.apply(originalCompile, WebAssembly, [captured]);
    modules.set(module, value === hex(wire) ? 'effectWireBytes' : 'commandBridgeBytes');
    return module;
  };
  WebAssembly.Instance = class {
    constructor(module, imports) {
      const actual = new OriginalInstance(module, imports), exports = {...actual.exports};
      const kind = modules.get(module);
      check(kind !== undefined, 'actual captured module required');
      exports.holo_run = (pointer, length) => {
        const request = new Uint8Array(exports.memory.buffer, pointer, length).slice();
        const packed = BigInt.asUintN(64, actual.exports.holo_run(pointer, length));
        const offset = Number(packed >> 32n), size = Number(packed & 0xffffffffn);
        check(offset + size <= exports.memory.buffer.byteLength, 'bounded real output');
        const response = new Uint8Array(exports.memory.buffer, offset, size).slice();
        calls.push({kind, request: hex(request), response: hex(response)});
        maximum[kind] = Math.max(maximum[kind] ?? 0, exports.memory.buffer.byteLength);
        if (kind === 'effectWireBytes' && faultCompletion && decode(request)[1] === 2) {
          faultCompletion = false; faultUsed = true;
          // Lose an observation only after the genuine generated transition ran.
          throw new WebAssembly.RuntimeError('injected missing completion observation');
        }
        return packed;
      };
      return {exports};
    }
  };
  let bridgeModule;
  function model(message) {
    const value = encode(message), instance = new WebAssembly.Instance(bridgeModule, {});
    const pointer = instance.exports.holo_alloc(value.length);
    new Uint8Array(instance.exports.memory.buffer, pointer, value.length).set(value);
    const packed = BigInt.asUintN(64, instance.exports.holo_run(pointer, value.length));
    return decode(new Uint8Array(instance.exports.memory.buffer, Number(packed >> 32n), Number(packed & 0xffffffffn)).slice());
  }
  function value(message) {
    const response = model(message);
    check(response[0] === 1 && response[1] === 0 && response.length === 3, 'real modeled success');
    return response[2];
  }
  const setup = async () => {
    const manifest = [crypto.getRandomValues(new Uint8Array(32)), crypto.getRandomValues(new Uint8Array(32)),
      [16384, 16384, 16384], [['foundry/command-reference/1', [2]]]];
    const wireDigest = new Uint8Array(await Reflect.apply(originalDigest, crypto.subtle, ['SHA-256', wire]));
    const host = await openContextualStagedEffects({wire: wire.slice(), wireDigest,
      manifest: encode(manifest), guests: [], signers: []});
    hosts.push(host);
    check(Object.keys(host).sort().join(',') === 'close,context,prepareExact,status', 'closed SDK API');
    for (const key of ['complete', 'settle', 'state', 'session']) check(host[key] === undefined, 'no caller completion API');
    return host;
  };
  function prepare(host, ctx, intent) {
    const observation = host.context();
    const effect = [observation.application, observation.manifest, observation.session, observation.nextOperation];
    const pending = value([1, 0, ctx, effect, intent]);
    check(pending[0] === 0, 'modeled pending');
    const handle = host.prepareExact(encode(pending[1][2]));
    check(same(decode(handle.request), pending[1][2]), 'actual admission retains exact source request');
    return {pending, handle, effect};
  }
  const base = () => ({ctx: decode(unhex(input.routes[1].context)), intent: decode(unhex(input.routes[1].intent))});
  function holdDigest() {
    const entered = deferred(), released = deferred();
    Object.defineProperty(crypto.subtle, 'digest', {configurable: true, value: async function(...args) {
      const actual = await Reflect.apply(originalDigest, this, args);
      entered.resolve(); await released.promise; return actual;
    }});
    return {entered, released};
  }
  const restoreDigest = () => {
    if (descriptors.digest) Object.defineProperty(crypto.subtle, 'digest', descriptors.digest);
    else delete crypto.subtle.digest;
  };
  try {
    bridgeModule = await WebAssembly.compile(bridge);
    const labels = input.labels.map((_, index) => ({id: 'label.' + String(index).padStart(2, '0'), text: value([1, 4, index])}));
    for (const route of input.routes) {
      const ctx = decode(unhex(route.context)), expectedIntent = decode(unhex(route.intent));
      const frame = value([1, 3, ctx]), root = document.createElement('div'); document.body.append(root); roots.push(root);
      let captured;
      const view = openPresentation({root, labels, maximum: 16384, requestMaximum: 16384,
        dispatch(bytes) { check(captured === undefined, 'one DOM submission'); captured = bytes.slice(); return encode(frame); }});
      views.push(view); view.render(encode(frame));
      const field = root.querySelector('input');
      if (field) { field.value = expectedIntent[3][0][1]; field.dispatchEvent(new Event('input', {bubbles: true})); }
      root.querySelector('button').click(); await tick();
      check(captured && same(decode(captured), expectedIntent), 'SDK captures exact generated form intent');
      check(!root.querySelector('[data-presentation-diagnostic]')?.textContent, 'no renderer failure');
      const host = await setup(), {pending, handle, effect} = prepare(host, ctx, decode(captured));
      const request = copy(pending[1][2]); handle.request.fill(0);
      const completion = decode(await handle.release());
      check(same(completion[0], request), 'mutable request observation cannot alter private release');
      const resolved = value([1, 1, pending, ctx, completion]);
      check(resolved[0] === 1, 'real SHA-256 resolves command');
      check(same(value([1, 1, copy(pending), copy(ctx), copy(completion)]), resolved), 'pure old clone remains replayable data');
      check(same(model([1, 1, resolved, ctx, completion]), [1, 1, [5]]), 'resolved cannot settle twice');
      await reject(() => handle.release(), 'invalid-input');
      routes.push({screen: ctx[2], context: hex(encode(ctx)), intent: hex(captured), effect: hex(encode(effect)),
        frame: hex(encode(frame)), pending: hex(encode(pending)), completion: hex(encode(completion)), resolved: hex(encode(resolved))});
      view.close(); root.remove(); host.close();
    }
    cases.push('all-routes-real-DOM-digest-and-pure-clone');

    const host = await setup(), {ctx, intent} = base();
    const first = prepare(host, ctx, intent);
    await reject(() => first.handle.release(new Uint8Array()), 'invalid-input');
    await reject(() => host.prepareExact(encode(first.pending[1][2])), 'model-rejected');
    for (const index of [0, 1, 2, 3]) {
      const current = host.context(), effect = [current.application, current.manifest, current.session, current.nextOperation];
      const wrong = value([1, 0, ctx, effect, intent])[1][2];
      if (index === 3) wrong[index]++; else wrong[index][0] ^= 1;
      await reject(() => host.prepareExact(encode(wrong)), 'model-rejected');
    }
    const secondHost = await setup();
    await reject(() => secondHost.prepareExact(encode(first.pending[1][2])), 'model-rejected');
    const firstCompletion = decode(await first.handle.release());
    check(value([1, 1, first.pending, ctx, firstCompletion])[0] === 1, 'refused substitutions preserve original handle');
    cases.push('exact-context-wrong-host-operation-and-duplicate-handle');

    const racingHost = await setup(), racing = prepare(racingHost, ctx, intent), hold = holdDigest();
    const released = observed(racing.handle.release()); await hold.entered.promise;
    const changed = copy(ctx); changed[0][2]++;
    hold.released.resolve(); const racingOutcome = await released; restoreDigest();
    check(!racingOutcome.error, 'real held SHA-256 completes');
    check(same(model([1, 1, racing.pending, changed, decode(racingOutcome.result)]), [1, 1, [4]]), 'concurrent application change rejects');
    cases.push('current-state-change-during-real-digest');

    const closingHost = await setup(), closing = prepare(closingHost, ctx, intent), closingHold = holdDigest();
    const closingOutcome = observed(closing.handle.release()); await closingHold.entered.promise;
    const closed = value([1, 2, closing.pending]); closingHost.close(); closingHold.released.resolve();
    const refused = await closingOutcome; restoreDigest();
    check(refused.error?.code === 'host-closed', 'closure refuses in-flight completion');
    const actualResult = await Reflect.apply(originalDigest, crypto.subtle, ['SHA-256', closing.pending[1][2][5][1]]);
    check(same(model([1, 1, closed, ctx, [closing.pending[1][2], [2, 'sha256:' + hex(new Uint8Array(actualResult))]]]),
      [1, 1, [5]]), 'source closed refuses even genuine late digest');
    cases.push('closed-during-real-digest');

    const failureHost = await setup(), failing = prepare(failureHost, ctx, intent);
    Object.defineProperty(crypto.subtle, 'digest', {configurable: true, value() { return Promise.reject(new DOMException('injected unavailable provider', 'OperationError')); }});
    const failure = decode(await failing.handle.release()); restoreDigest();
    check(same(failure[1], [8, [2]]), 'real SDK types unavailable cryptography');
    check(same(value([1, 1, failing.pending, ctx, failure]), [2, failing.pending[1], [2]]), 'typed definitive failure remains failed');
    await reject(() => failing.handle.release(), 'invalid-input');
    cases.push('typed-crypto-failure-consumed-once');

    const unknownHost = await setup(), unknown = prepare(unknownHost, ctx, intent);
    faultCompletion = true;
    await reject(() => unknown.handle.release(), 'effect-outcome-unknown');
    check(faultUsed, 'real generated completion observation was lost');
    check(unknownHost.status().closed, 'unknown observation closes unavailable SDK host');
    await reject(() => unknown.handle.release(), 'invalid-input');
    cases.push('lost-generated-completion-observation');
    return {cases, calls, routes, maximum};
  } finally {
    restoreDigest(); faultCompletion = false;
    for (const view of views) view.close();
    for (const root of roots) root.remove();
    for (const host of hosts) { try { host.close(); } catch { /* Preserve earlier observed fault. */ } }
    WebAssembly.compile = originalCompile; WebAssembly.Instance = OriginalInstance;
  }
}
