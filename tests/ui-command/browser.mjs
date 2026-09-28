// Requires an explicit SDK owner and genuinely generated artifacts. It is not
// invoked as a fallback for the locked repository acceptance command.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {lstatSync, readFileSync, readdirSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
import {cbor, context, intent, labels, planned, presentation, screens} from './corpus.mjs';

const expectedCases = ['all-routes-real-DOM-digest-and-pure-clone',
  'exact-context-wrong-host-operation-and-duplicate-handle',
  'current-state-change-during-real-digest', 'closed-during-real-digest',
  'typed-crypto-failure-consumed-once', 'lost-generated-completion-observation'];
const sha = value => createHash('sha256').update(value).digest('hex');
const fixture = new URL('./browser-fixture.mjs', import.meta.url);

export async function verifyBrowserComponent({sdkDirectory, bridge, wire, verifyArtifacts}) {
  assert.equal(typeof verifyArtifacts, 'function');
  const sdk = join(sdkDirectory, 'sdk/browser');
  const files = readdirSync(sdk).filter(name => name.endsWith('.mjs')).sort();
  assert.ok(files.includes('effects.mjs') && files.includes('presentation-dom.mjs'));
  const capture = () => {
    assert.deepEqual(readdirSync(sdk).filter(name => name.endsWith('.mjs')).sort(), files,
      'closed SDK browser source inventory');
    return Object.fromEntries(files.map(name => {
      const file = join(sdk, name); assert.ok(lstatSync(file).isFile());
      return [name, sha(readFileSync(file))];
    }));
  };
  const before = capture(), fixtureBytes = readFileSync(fixture), corpusBytes = readFileSync(new URL('./corpus.mjs', import.meta.url));
  const {withBrowser} = await import(pathToFileURL(join(sdk, 'browser-test-server.mjs')));
  const {decodeEffectWire: decode} = await import(pathToFileURL(join(sdk, 'effects-wire.mjs')));
  const routes = screens.map(screen => {
    const ctx = context(screen);
    return {context: cbor(ctx).toString('hex'), intent: cbor(intent(ctx)).toString('hex')};
  });
  const results = [];
  for (const engine of ['chromium', 'firefox', 'webkit']) {
    verifyArtifacts();
    const result = await withBrowser(async ({browser, baseURL}) => {
      const page = await browser.newPage(), errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await page.route('**/browser-fixture.mjs', route => route.fulfill({status: 200,
        contentType: 'text/javascript', body: fixtureBytes.toString('utf8')}));
      await page.goto(baseURL);
      let timer;
      try {
        const observed = await Promise.race([
          page.evaluate(async input => (await import('./browser-fixture.mjs')).runFixture(input),
            {wire: [...wire], bridge: [...bridge], routes, labels}),
          new Promise((_, reject) => {timer = setTimeout(() => reject(Error('UC-01 browser timeout')), 120000);}),
        ]);
        assert.deepEqual(errors, []);
        return observed;
      } finally { clearTimeout(timer); }
    }, {engine});
    verifyArtifacts();
    assert.deepEqual(result.cases, expectedCases);
    assert.deepEqual(result.routes.map(row => row.screen), screens);
    for (const row of result.routes) {
      const ctx = context(row.screen), submitted = intent(ctx);
      assert.equal(row.context, cbor(ctx).toString('hex'));
      assert.equal(row.intent, cbor(submitted).toString('hex'));
      const encodedEffect = Buffer.from(row.effect, 'hex'), decodedEffect = decode(encodedEffect);
      assert.equal(decodedEffect.length, 4);
      const effect = [...decodedEffect.slice(0, 3).map(value => Buffer.from(value)), decodedEffect[3]];
      assert.equal(cbor(effect).toString('hex'), row.effect, 'canonical actual host context');
      assert.equal(effect.length, 4); assert.equal(effect[3], 0);
      for (const reference of effect.slice(0, 3)) assert.equal(reference.length, 32);
      const expected = planned(ctx, effect, submitted);
      assert.equal(row.frame, cbor(presentation(ctx)).toString('hex'));
      assert.equal(row.pending, cbor([0, expected.plan]).toString('hex'));
      assert.equal(row.completion, cbor(expected.completion).toString('hex'), 'real SDK SHA-256 agrees with independent Node digest');
      assert.equal(row.resolved, cbor([1, expected.plan, expected.command]).toString('hex'));
    }
    assert.ok(result.calls.length > 150);
    for (const call of result.calls) {
      assert.ok(['commandBridgeBytes', 'effectWireBytes'].includes(call.kind));
      assert.match(call.request, /^(?:[0-9a-f]{2})+$/u); assert.match(call.response, /^(?:[0-9a-f]{2})+$/u);
    }
    assert.ok(result.maximum.commandBridgeBytes > 0 && result.maximum.commandBridgeBytes <= 1024 * 65536);
    assert.ok(result.maximum.effectWireBytes > 0 && result.maximum.effectWireBytes <= 16384 * 65536);
    assert.deepEqual(capture(), before, 'actual SDK source closure remains captured');
    assert.deepEqual(readFileSync(fixture), fixtureBytes);
    assert.deepEqual(readFileSync(new URL('./corpus.mjs', import.meta.url)), corpusBytes);
    results.push({engine, ...result});
  }
  return {scope: 'conditional-component-only-not-installed-SDK-acceptance',
    sdkSources: before, fixture: sha(fixtureBytes), corpus: sha(corpusBytes),
    artifacts: {wire: sha(wire), bridge: sha(bridge)}, results};
}
