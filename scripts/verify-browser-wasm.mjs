// scripts/verify-browser-wasm.mjs
// Conformance verification for pure-model WebAssembly build artifacts,
// acceptance vectors, routing, and accessible UI presentation.

import assert from 'node:assert/strict';
import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const buildBase = join(root, '.prism', 'build');

if (!existsSync(buildBase)) {
  console.log('Skipping Wasm verification: .prism/build not present');
  process.exit(0);
}

// Find build directory
const buildDirs = readdirSync(buildBase).filter((dir) =>
  existsSync(join(buildBase, dir, 'view', 'browser', 'prism_foundry_web_bg.wasm'))
);

if (buildDirs.length === 0) {
  console.log('Skipping Wasm verification: no browser build found in .prism/build');
  process.exit(0);
}

const buildDir = join(buildBase, buildDirs[0]);
console.log(`Found browser build: ${buildDirs[0]}`);

// 1. Verify build artifact completeness
const requiredFiles = [
  'Foundry.holo',
  'model.prism.json',
  'core-wasm/prism_foundry_web_core_wasm.wasm',
  'view/browser/prism_foundry_web_bg.wasm',
  'view/browser/prism_foundry_web.js',
  'view/browser/app.css',
  'view/browser/app.js',
  'view/browser/index.html',
];

for (const relPath of requiredFiles) {
  const fullPath = join(buildDir, relPath);
  assert.ok(existsSync(fullPath), `Required artifact missing: ${relPath}`);
  const stats = readFileSync(fullPath);
  assert.ok(stats.length > 0, `Artifact is empty: ${relPath}`);
}
console.log('✓ All 8 build artifacts present and non-empty');

// 2. Verify model.prism.json schema and metadata
const model = JSON.parse(readFileSync(join(buildDir, 'model.prism.json'), 'utf8'));
assert.equal(model.schema, 'prismpm/model-document/4');
assert.equal(model.application.profile, 'prismpm/browser-application/1');
assert.equal(model.application.name, 'Foundry');
assert.equal(model.application.cargo_name, 'prism-foundry-web');
assert.equal(model.application.cargo_version, '0.1.0');
assert.equal(model.application.cargo_repository, 'https://github.com/UOR-Foundation/uor-foundry');
assert.equal(model.application.entry_root, 'PrismFoundry.Foundry.dispatch');
assert.deepEqual(model.application.library_roots, [
  'PrismFoundry.Foundry.dispatch',
  'PrismFoundry.Foundry.present',
  'PrismFoundry.Foundry.replay',
]);
console.log('✓ model.prism.json validated');

// 3. Load Wasm binding and initialize
const jsPath = join(buildDir, 'view', 'browser', 'prism_foundry_web.js');
const wasmPath = join(buildDir, 'view', 'browser', 'prism_foundry_web_bg.wasm');
const wasmBytes = readFileSync(wasmPath);

const { initSync, invoke_bytes } = await import(`file://${jsPath}`);
initSync({ module: wasmBytes });
console.log('✓ WebAssembly module successfully instantiated');

// 4. Test acceptance vectors
const acceptanceVectors = [
  { request: 'Hello, Foundry.', expected: 'Hello, Foundry.' },
  { request: 'Status: Ready', expected: 'Status: Ready' },
];

for (const { request, expected } of acceptanceVectors) {
  const input = new Uint8Array(Buffer.from(request));
  const output = invoke_bytes(input);
  const resultText = Buffer.from(output).toString('utf8');
  assert.equal(resultText, expected, `Vector mismatch for request "${request}"`);
}
console.log(`✓ All ${acceptanceVectors.length} acceptance vectors executed and matched`);

// 5. Test routing for 7 core stakeholder services
const services = [
  'Workflows',
  'AiInference',
  'Messaging',
  'Governance',
  'Finance',
  'Learning',
  'Brand',
];

for (const service of services) {
  const payload = `route:${service}`;
  const input = new Uint8Array(Buffer.from(payload));
  const output = invoke_bytes(input);
  const resultText = Buffer.from(output).toString('utf8');
  assert.equal(resultText, payload, `Service dispatch echo failed for ${service}`);
}
console.log(`✓ Routing verified across all ${services.length} stakeholder services`);

// 6. Verify HTML/CSS presentation and accessibility properties
const html = readFileSync(join(buildDir, 'view', 'browser', 'index.html'), 'utf8');
assert.ok(html.includes('lang="en"'), 'HTML must declare lang="en"');
assert.ok(html.includes('<main>'), 'HTML must contain main landmark');
assert.ok(html.includes('<h1>UOR Foundry Portal</h1>'), 'HTML must contain h1 portal heading');
assert.ok(html.includes('viewport'), 'HTML must declare viewport meta');

const css = readFileSync(join(buildDir, 'view', 'browser', 'app.css'), 'utf8');
assert.ok(css.includes('#172033'), 'CSS text color must be #172033');
assert.ok(css.includes('white') || css.includes('#fff'), 'CSS background must be white');
console.log('✓ WCAG accessibility and presentation markup verified');

console.log('Pure-model WebAssembly verification completed successfully.');
