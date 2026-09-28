// Independent finite protocol expectations, never an application implementation.
import assert from 'node:assert/strict';

export const UINT32_MAX = 4294967295;
export const screenNames = Object.freeze(['Welcome', 'Enrollment', 'Login', 'EmailRecovery', 'SavedCodeRecovery']);
export const phaseNames = Object.freeze(['Ready', 'Pending', 'ReplayRequired', 'Closed']);

// RFC 8949 shortest unsigned integers and definite containers, for fixtures only.
function header(major, value) {
  assert.ok(Number.isInteger(value) && value >= 0 && value <= UINT32_MAX);
  if (value < 24) return Buffer.from([(major << 5) + value]);
  const width = value <= 255 ? 1 : value <= 65535 ? 2 : 4;
  const bytes = Buffer.alloc(width + 1);
  bytes[0] = (major << 5) + {1: 24, 2: 25, 4: 26}[width];
  bytes.writeUIntBE(value, 1, width);
  return bytes;
}
export function encode(value) {
  if (Array.isArray(value))
    return Buffer.concat([header(4, value.length), ...value.map(encode)]);
  if (typeof value === 'boolean') return Buffer.from([value ? 0xf5 : 0xf4]);
  if (typeof value === 'string') {
    const bytes = Buffer.from(value, 'utf8');
    return Buffer.concat([header(3, bytes.length), bytes]);
  }
  return header(0, value);
}

export function selector(revision = 0, phase = 0, screen = 0, epoch = 0) {
  return [1, revision, phase, screen, epoch];
}

export const failure = code => encode([1, 1, code]);

export function selectorCorpus() {
  const rows = [];
  const valid = (id, value) => {
    const bytes = encode(value);
    rows.push({id, request: bytes, response: Buffer.from(bytes)});
  };
  const invalid = (id, request, code) => rows.push({id, request: Buffer.from(request), response: failure(code)});
  const boundaries = [0, 23, 24, 255, 256, 65535, 65536, UINT32_MAX];
  for (const revision of boundaries) for (const epoch of boundaries)
    for (let phase = 0; phase < 4; phase++) for (let screen = 0; screen < 5; screen++)
      valid(`Selector-${revision}-${epoch}-${phaseNames[phase]}-${screenNames[screen]}`,
        selector(revision, phase, screen, epoch));
  invalid('Version', encode([2, 0, 0, 0, 0]), 3);
  for (const phase of [4, 23, 24, 255, UINT32_MAX])
    invalid('UnknownPhase-' + phase, encode(selector(0, phase)), 6);
  for (const screen of [5, 23, 24, 255, UINT32_MAX])
    invalid('UnknownScreen-' + screen, encode(selector(0, 0, screen)), 6);
  invalid('RootType', [0x40], 3);
  invalid('ArityUnder', encode([1, 0, 0, 0]), 3);
  invalid('ArityOver', encode([1, 0, 0, 0, 0, 0]), 6);
  invalid('IndefiniteArray', [0x9f, 1, 0, 0, 0, 0, 0xff], 4);
  invalid('NoncanonicalArray', [0x98, 5, 1, 0, 0, 0, 0], 5);
  const empty = encode(selector());
  for (let index = 1; index < empty.length; index++)
    invalid('NoncanonicalField-' + index,
      [...empty.subarray(0, index), 0x18, empty[index], ...empty.subarray(index + 1)], 5);
  invalid('Trailing', [...empty, 0], 8);
  const maximum = encode(selector(UINT32_MAX, 3, 4, UINT32_MAX));
  assert.equal(maximum.length, 14);
  for (let length = 0; length < maximum.length; length++)
    invalid('TruncatedMaximum-' + length, maximum.subarray(0, length), 2);
  invalid('FrameOverflow', [...maximum, 0], 6);
  invalid('OversizedWrongType', Buffer.alloc(15), 6);
  assert.equal(rows.length, 1318);
  assert.equal(new Set(rows.map(row => row.id)).size, rows.length);
  return rows;
}

// These expectations encode the reviewed private UI contract, not runtime output.
// They are product acceptance fixtures, not external design-oracle certification.
export const labels = Object.freeze([
  ['app_title', 'Foundry'],
  ['code_help', 'Use a saved recovery code. Never share a recovery code.'],
  ['email_help', 'Use an email address you control. Email verification is required.'],
  ['email_label', 'Email address'],
  ['enrollment_heading', 'Create your Foundry account'],
  ['enrollment_intro', 'Use one Foundry identity to participate in organizations and create your own.'],
  ['login_heading', 'Sign in to Foundry'],
  ['login_intro', 'Sign in to continue to your organizations and workspaces.'],
  ['main_label', 'Account access'],
  ['nav_enroll', 'Create an account'],
  ['nav_home', 'Welcome'],
  ['nav_login', 'Sign in'],
  ['nav_recover_code', 'Use a recovery code'],
  ['nav_recover_email', 'Recover with email'],
  ['navigation_label', 'Account navigation'],
  ['recovery_code_heading', 'Recover with a saved code'],
  ['recovery_code_intro', 'A saved recovery code restores access only after the account recovery checks succeed.'],
  ['recovery_email_heading', 'Recover with email'],
  ['recovery_email_intro', 'Verify control of your email before recovering access to your Foundry identity.'],
  ['saved_code_label', 'Saved recovery code'],
  ['skip_main', 'Skip to account access'],
  ['status_closed', 'Account view closed'],
  ['status_pending', 'Waiting for the current operation'],
  ['status_ready', 'Account navigation ready'],
  ['status_replay', 'Recovering the current operation'],
  ['unavailable', 'Account enrollment, sign-in, and recovery are not available in this development build. No account is created and no email is sent.'],
  ['welcome_heading', 'Welcome to Foundry'],
  ['welcome_intro', 'Work with your community, develop ideas, and manage your organizations in one place.'],
].map(row => Object.freeze(row)));

export function expectedFrame(value) {
  const [, revision, phase, screen, epoch] = value;
  const frame = [1, revision, phase, [24, 23, 25, 22][phase], phase === 3 ? 0 : 1, 0, []];
  if (phase === 3) return frame;
  const heading = [26, 4, 6, 17, 15][screen];
  const intro = [27, 5, 7, 18, 16][screen];
  frame[6] = [
    [0, [0, 0]], [1, [3, 1, 0]], [0, [1, 14]], [3, [2, 14]],
    ...[10, 9, 11, 13, 12].map((label, target) =>
      [4, [8, label, target + 1, phase === 0 && target !== screen, false, []]]),
    [0, [0, 8]], [10, [3, 2, heading]], [10, [4, labels[intro][1]]],
  ];
  if (screen !== 0) frame[6].push([10, [2, heading]],
    [13, screen === 4 ? [10, 19, false, true, 128, epoch] : [5, 3, false, true, 254, '', epoch]]);
  frame[6].push([10, [4, labels[25][1]]]);
  return frame;
}

export function expectedSemanticFrame(value) {
  const [, , phase, screen] = value;
  const annotations = phase === 3 ? [] : [
    [1, 0, 0, 0, 2, 1], [3, 0, 0, 0, 0, 1], [4, 0, 0, 0, 0, 1], [10, 0, 0, 0, 1, 1],
  ];
  if (phase !== 3 && screen !== 0) annotations.push([13, 0, 0, 0, 0, 1],
    [14, screen === 4 ? 8 : 4, screen === 4 ? 2 : 3, 0, 0, 0]);
  return [1, expectedFrame(value), 0, 0, phase === 3 ? 0 : 21, annotations];
}

export function presentationCorpus(semantic = false) {
  const expected = semantic ? expectedSemanticFrame : expectedFrame;
  return selectorCorpus().map(row => {
    if (!row.id.startsWith('Selector-')) return row;
    const [, revision, epoch, phase, screen] = row.id.split('-');
    return {id: row.id, request: row.request,
      response: encode(expected(selector(Number(revision), phaseNames.indexOf(phase), screenNames.indexOf(screen), Number(epoch))))};
  });
}

export function navigationCorpus() {
  const rows = [];
  const boundaries = [0, 23, 24, 255, 256, 65535, 65536, UINT32_MAX - 1, UINT32_MAX];
  const add = (id, state, intent, code = undefined) => rows.push({id,
    request: encode([1, state, intent]), response: code === undefined
      ? encode(selector(state[1] + 1, 0, intent[2] - 1, state[4] + 1)) : failure(code)});
  for (const revision of boundaries) for (const epoch of boundaries)
    for (let screen = 0; screen < 5; screen++) for (let target = 0; target < 5; target++) {
      const exhausted = revision === UINT32_MAX || epoch === UINT32_MAX;
      const code = screen === target ? 3 : exhausted ? 6 : undefined;
      add(`Navigate-${revision}-${epoch}-${screen}-${target}`, selector(revision, 0, screen, epoch),
        [1, revision, target + 1, []], code);
    }
  for (let phase = 1; phase < 4; phase++) for (let screen = 0; screen < 5; screen++)
    for (let target = 0; target < 5; target++)
      add(`Blocked-${phase}-${screen}-${target}`, selector(8, phase, screen, 9), [1, 8, target + 1, []], 3);
  for (let screen = 0; screen < 5; screen++) {
    for (const stale of [0, 7, 9, UINT32_MAX])
      add(`RevisionMismatch-${screen}-${stale}`, selector(8, 0, screen, 9), [1, stale, (screen + 1) % 5 + 1, []], 3);
    for (const action of [0, 6, 23, UINT32_MAX])
      add(`UnknownAction-${screen}-${action}`, selector(8, 0, screen, 9), [1, 8, action, []], 3);
    for (const fields of [[[14, 'user@example.test']], [[14, 1]], [[14, ''], [14, '']]])
      add(`UnexpectedFields-${screen}-${JSON.stringify(fields)}`, selector(8, 0, screen, 9),
        [1, 8, (screen + 1) % 5 + 1, fields], 3);
  }
  const exact = encode([1, selector(UINT32_MAX - 1, 0, 0, UINT32_MAX - 1), [1, UINT32_MAX - 1, 2, []]]);
  assert.equal(exact.length, 25);
  rows.push({id: 'Trailing', request: Buffer.concat([exact, Buffer.from([0])]), response: failure(8)});
  rows.push({id: 'InputOverflow', request: Buffer.alloc(65), response: failure(6)});
  rows.push({id: 'Version', request: encode([2, selector(), [1, 0, 2, []]]), response: failure(3)});
  rows.push({id: 'IntentVersion', request: encode([1, selector(), [2, 0, 2, []]]), response: failure(3)});
  rows.push({id: 'RootType', request: Buffer.from([0x40]), response: failure(3)});
  rows.push({id: 'IndefiniteRoot', request: Buffer.from([0x9f, 0xff]), response: failure(4)});
  for (let length = 0; length < exact.length; length++)
    rows.push({id: 'TruncatedMaximum-' + length, request: exact.subarray(0, length), response: failure(2)});
  const minimal = encode([1, selector(), [1, 0, 2, []]]);
  for (let index = 0; index < minimal.length; index++) if (minimal[index] < 24)
    rows.push({id: 'NoncanonicalInteger-' + index,
      request: Buffer.from([...minimal.subarray(0, index), 0x18, minimal[index], ...minimal.subarray(index + 1)]),
      response: failure(5)});
  assert.equal(new Set(rows.map(row => row.id)).size, rows.length);
  return rows;
}

export function designCorpus() {
  const tokens = colors => [...colors, 0, 1000, 1500, 1000, 500, 72, 18, 48, 44];
  const pair = [tokens(['#ffffff', '#17212f', '#526071', '#1649a2', '#ffffff', '#a31616', '#6b21a8']),
    tokens(['#111827', '#f8fafc', '#a7b4c5', '#99bfff', '#111827', '#ff9c9c', '#fcd34d'])];
  return [{id: 'Catalogue', request: Buffer.alloc(0), response: encode([1, [pair]])},
    {id: 'NonemptyRequest', request: Buffer.from([0]), response: failure(3)}];
}
