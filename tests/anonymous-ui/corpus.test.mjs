import assert from 'node:assert/strict';
import test from 'node:test';
import {designCorpus, encode, expectedFrame, expectedSemanticFrame, labels, navigationCorpus,
  presentationCorpus, selector, selectorCorpus, UINT32_MAX} from './corpus.mjs';

test('fixture encoder has independent exact RFC 8949 boundary bytes', () => {
  for (const [value, hex] of [[0, '00'], [23, '17'], [24, '1818'], [255, '18ff'],
    [256, '190100'], [65535, '19ffff'], [65536, '1a00010000'], [UINT32_MAX, '1affffffff']])
    assert.equal(encode(value).toString('hex'), hex);
  assert.equal(encode(selector()).toString('hex'), '850100000000');
  assert.equal(encode(selector(UINT32_MAX, 3, 4, UINT32_MAX)).toString('hex'),
    '85011affffffff03041affffffff');
  assert.equal(encode([false, true, '', 'a', 'é']).toString('hex'), '85f4f560616162c3a9');
  assert.equal(encode('a'.repeat(24)).toString('hex'), '7818' + '61'.repeat(24));
  assert.equal(encode(Array(24).fill(0)).toString('hex'), '9818' + '00'.repeat(24));
  for (const value of [-1, 0.5, UINT32_MAX + 1, NaN, Infinity, null, {}, undefined])
    assert.throws(() => encode(value));
});

test('frame expectations cover closed, pending, replay, labeled fields and disabled unavailable services', () => {
  assert.equal(labels.length, 28);
  assert.deepEqual(labels.map(row => row[0]), labels.map(row => row[0]).sort());
  assert.deepEqual(expectedFrame(selector(8, 3, 4, 9)), [1, 8, 3, 22, 0, 0, []]);
  for (let phase = 0; phase < 3; phase++) for (let screen = 0; screen < 5; screen++) {
    const frame = expectedFrame(selector(8, phase, screen, 9));
    assert.equal(frame[6].length, screen === 0 ? 13 : 15);
    assert.equal(frame[6].filter(node => node[1][0] === 8 && node[1][3]).length, phase === 0 ? 4 : 0);
    assert.deepEqual(frame[6].at(-1), [10, [4, labels[25][1]]]);
    if (screen !== 0) assert.equal(frame[6][13][1][2], false, 'no enrollment/recovery submission');
    const semantic = expectedSemanticFrame(selector(8, phase, screen, 9));
    assert.equal(semantic[4], 21);
    assert.equal(semantic[5].length, screen === 0 ? 4 : 6);
  }
  for (const semantic of [false, true]) assert.equal(presentationCorpus(semantic).length, 1318);
  assert.equal(designCorpus().length, 2);
});

test('navigation corpus binds revision, enabled screen, lifecycle and both overflow boundaries', () => {
  const rows = navigationCorpus();
  assert.equal(rows.length, 2195);
  assert.equal(rows.filter(row => row.id.startsWith('Navigate-')).length, 2025);
  assert.equal(rows.filter(row => row.id.startsWith('Blocked-')).length, 75);
  assert.equal(rows.filter(row => row.id.startsWith('UnexpectedFields-')).length, 15);
  assert.equal(rows.filter(row => row.id.startsWith('TruncatedMaximum-')).length, 25);
  assert.equal(rows.filter(row => row.id.startsWith('NoncanonicalInteger-')).length, 9);
  assert.equal(rows.find(row => row.id === 'Navigate-0-0-0-1').response.toString('hex'), '850101000101');
  assert.equal(rows.find(row => row.id === `Navigate-${UINT32_MAX}-0-0-1`).response.toString('hex'), '83010106');
  assert.equal(rows.find(row => row.id === `Navigate-${UINT32_MAX}-0-0-0`).response.toString('hex'), '83010103');
});

test('independent selector inventory spans all screens, phases and integer boundaries', () => {
  const rows = selectorCorpus();
  assert.equal(rows.filter(row => row.id.startsWith('Selector-')).length, 1280);
  assert.equal(rows.filter(row => row.id.startsWith('TruncatedMaximum-')).length, 14);
  assert.equal(rows.filter(row => row.id.startsWith('NoncanonicalField-')).length, 5);
  assert.equal(rows.find(row => row.id === 'Trailing').response.toString('hex'), '83010108');
  assert.equal(rows.find(row => row.id === 'FrameOverflow').response.toString('hex'), '83010106');
});
