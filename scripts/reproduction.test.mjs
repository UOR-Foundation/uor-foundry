import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, readdirSync, rmSync, symlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {copyRegularTree, createReproductionDirectory, verifySameTree} from './reproduction.mjs';

test('clean roots use unique ignored storage and reject a symlinked target', () => {
  const directory = mkdtempSync(join(tmpdir(), 'foundry-storage-test-'));
  try {
    const first = createReproductionDirectory(directory);
    const second = createReproductionDirectory(directory);
    assert.notEqual(first, second);
    assert.ok(first.startsWith(join(directory, 'target', 'foundry-reproduction-')));
    assert.deepEqual(readdirSync(first), []);
    assert.deepEqual(readdirSync(second), []);
    const unsafe = join(directory, 'unsafe');
    mkdirSync(unsafe);
    symlinkSync(first, join(unsafe, 'target'));
    assert.throws(() => createReproductionDirectory(unsafe), /must not be a symlink/);
    assert.deepEqual(readdirSync(first), []);
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});

test('reproduction compares complete bytes and rejects extra, missing or nonregular entries', () => {
  const directory = mkdtempSync(join(tmpdir(), 'foundry-reproduction-test-'));
  try {
    const source = join(directory, 'source');
    const copy = join(directory, 'copy');
    mkdirSync(source);
    mkdirSync(join(source, 'nested'));
    writeFileSync(join(source, 'nested', 'artifact'), Buffer.from([0, 128, 255]));
    copyRegularTree(source, copy);
    verifySameTree(source, copy);
    writeFileSync(join(copy, 'nested', 'artifact'), Buffer.from([0, 128, 254]));
    assert.throws(() => verifySameTree(source, copy));
    writeFileSync(join(copy, 'nested', 'artifact'), Buffer.from([0, 128, 255]));
    writeFileSync(join(copy, 'extra'), 'unmanifested');
    assert.throws(() => verifySameTree(source, copy));
    assert.throws(() => verifySameTree(copy, source));
    rmSync(join(copy, 'extra'));
    symlinkSync(join(source, 'nested', 'artifact'), join(copy, 'nested', 'link'));
    assert.throws(() => copyRegularTree(copy, join(directory, 'unsafe-copy')));
    assert.throws(() => verifySameTree(copy, copy));
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});
