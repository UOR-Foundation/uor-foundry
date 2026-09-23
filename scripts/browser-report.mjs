import assert from 'node:assert/strict';
import {resolve} from 'node:path';

export const browserCases = [
  'generated Wasm previews text without network effects after bootstrap',
  'byte boundaries, malformed UTF-16, empty input, and keyboard recovery',
  'initialization failures are visible without losing input',
  'unavailable scripts cannot submit drafts through native forms',
  'responsive layout and automated accessibility checks',
];

export function verifyBrowserReport(report, root, startedAt, finishedAt) {
  const directory = resolve(root, 'tests/browser');
  assert.equal(report.config.configFile, resolve(root, 'playwright.config.mjs'));
  assert.equal(report.config.rootDir, directory);
  assert.equal(report.config.forbidOnly, true);
  assert.equal(report.config.workers, 1);
  assert.equal(report.config.projects.length, 1);
  const project = report.config.projects[0];
  assert.equal(project.id, 'chromium');
  assert.equal(project.name, 'chromium');
  assert.equal(project.testDir, directory);
  assert.equal(project.repeatEach, 1);
  assert.equal(project.retries, 0);
  const start = Date.parse(report.stats.startTime);
  assert.ok(Number.isFinite(start) && start >= startedAt && start <= finishedAt,
    'browser evidence must come from this invocation');
  const cases = [];
  const visit = (suite) => {
    assert.equal(suite.file, 'draft-preview.spec.mjs');
    for (const spec of suite.specs ?? []) {
      assert.equal(spec.file, 'draft-preview.spec.mjs', 'wrong-file evidence is not acceptance');
      assert.equal(spec.ok, true, spec.title);
      assert.equal(spec.tests.length, 1, 'every case must run once');
      const test = spec.tests[0];
      assert.equal(test.projectId, project.id);
      assert.equal(test.projectName, project.name);
      assert.equal(test.expectedStatus, 'passed', spec.title);
      assert.equal(test.status, 'expected', spec.title);
      assert.deepEqual(test.annotations, []);
      assert.equal(test.results.length, 1, 'retries cannot conceal a failure');
      assert.equal(test.results[0].status, 'passed', spec.title);
      assert.equal(test.results[0].retry, 0);
      assert.deepEqual(test.results[0].errors, []);
      cases.push(spec.title);
    }
    for (const child of suite.suites ?? []) visit(child);
  };
  for (const suite of report.suites) visit(suite);
  assert.deepEqual(cases.sort(), [...browserCases].sort(),
    'missing, skipped, filtered, or unexpected browser cases are not acceptance');
  assert.deepEqual(report.errors, []);
  assert.equal(report.stats.skipped, 0);
  assert.equal(report.stats.unexpected, 0);
  assert.equal(report.stats.flaky, 0);
  assert.equal(report.stats.expected, cases.length);
}
