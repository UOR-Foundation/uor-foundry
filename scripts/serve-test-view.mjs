import assert from 'node:assert/strict';
import {readFileSync, readdirSync} from 'node:fs';
import {createServer} from 'node:http';

const root = process.env.FOUNDRY_BUILD_ROOT;
assert.match(root ?? '', /^\.prism\/build\/[0-9a-f]{64}$/);
const directory = `${root}/view/browser`;
const files = new Map([
  ['app.css', 'text/css'], ['app.js', 'text/javascript'], ['index.html', 'text/html'],
  ['prism_foundry_web.js', 'text/javascript'], ['prism_foundry_web_bg.wasm', 'application/wasm'],
  ['provenance.json', 'application/json'],
]);
assert.deepEqual(readdirSync(directory).sort(), [...files.keys()].sort());
const server = createServer((request, response) => {
  const prefix = '/foundry-web/';
  const name = request.url === prefix ? 'index.html'
    : request.url?.startsWith(prefix) ? request.url.slice(prefix.length) : undefined;
  if (request.method !== 'GET' || !files.has(name)) {
    response.writeHead(404).end();
    return;
  }
  response.writeHead(200, {'Content-Type': files.get(name), 'Cache-Control': 'no-store'});
  response.end(readFileSync(`${directory}/${name}`));
});
server.listen(4173, '127.0.0.1');
for (const signal of ['SIGTERM', 'SIGINT']) process.on(signal, () => server.close());
