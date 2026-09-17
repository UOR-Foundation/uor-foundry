import assert from 'node:assert/strict';
import {resolve} from 'node:path';
import test from 'node:test';
import {browserCases, verifyBrowserReport} from './browser-report.mjs';

const root = resolve('/synthetic-foundry-report');
const start = Date.parse('2026-09-15T00:00:00.000Z');
const report = () => ({
  config: {configFile: `${root}/playwright.config.mjs`, rootDir: `${root}/tests/browser`,
    forbidOnly: true, workers: 1, projects: [{id: 'chromium', name: 'chromium',
      testDir: `${root}/tests/browser`, repeatEach: 1, retries: 0}]},
  suites: [{file: 'draft-preview.spec.mjs', specs: browserCases.map((title) => ({
    title, file: 'draft-preview.spec.mjs', ok: true, tests: [{projectId: 'chromium',
      projectName: 'chromium', expectedStatus: 'passed', status: 'expected', annotations: [],
      results: [{status: 'passed', retry: 0, errors: []}]}],
  }))}],
  errors: [], stats: {startTime: new Date(start).toISOString(), skipped: 0,
    unexpected: 0, flaky: 0, expected: browserCases.length},
});
const verify = (value) => verifyBrowserReport(value, root, start, start + 1000);

test('complete current-invocation evidence binds the exact file and Chromium project', () => {
  verify(report());
});

test('wrong-file, wrong-project, stale, skipped, retried and incomplete reports fail closed', () => {
  const mutations = [
    (value) => {value.config.configFile = '/other/playwright.config.mjs';},
    (value) => {value.config.rootDir = '/other/tests/browser';},
    (value) => {value.config.forbidOnly = false;},
    (value) => {value.config.projects[0].name = 'firefox';},
    (value) => {value.config.projects[0].testDir = '/other/tests/browser';},
    (value) => {value.config.projects[0].repeatEach = 2;},
    (value) => {value.config.projects[0].retries = 1;},
    (value) => {value.suites[0].file = 'unrelated.spec.mjs';},
    (value) => {value.suites[0].specs[0].file = 'unrelated.spec.mjs';},
    (value) => {value.suites[0].specs[0].tests[0].projectId = 'firefox';},
    (value) => {value.suites[0].specs[0].tests[0].status = 'skipped';},
    (value) => {value.suites[0].specs[0].tests[0].expectedStatus = 'failed';},
    (value) => {value.suites[0].specs[0].tests[0].annotations = [{type: 'skip'}];},
    (value) => {value.suites[0].specs[0].tests[0].results[0].retry = 1;},
    (value) => {value.suites[0].specs[0].tests[0].results[0].errors = [{message: 'failure'}];},
    (value) => {value.suites[0].specs.pop();},
    (value) => {value.suites[0].specs.push(structuredClone(value.suites[0].specs[0]));},
    (value) => {value.stats.startTime = new Date(start - 1).toISOString();},
    (value) => {value.stats.skipped = 1;},
    (value) => {value.stats.flaky = 1;},
    (value) => {value.stats.expected--;},
    (value) => {value.errors = [{message: 'worker failure'}];},
  ];
  for (const mutate of mutations) {
    const value = report();
    mutate(value);
    assert.throws(() => verify(value), mutate.toString());
  }
});
