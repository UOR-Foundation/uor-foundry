import AxeBuilder from '@axe-core/playwright';
import {expect} from '@playwright/test';

/**
 * Standard WCAG 2.2 Level AA tags enforced across all UOR Foundry views.
 */
export const WCAG_22_AA_TAGS = [
  'wcag2a',
  'wcag2aa',
  'wcag21a',
  'wcag21aa',
  'wcag22aa',
];

/**
 * Run strict WCAG 2.2 Level AA accessibility audit on a Playwright page.
 *
 * @param {import('@playwright/test').Page} page - The active page to audit
 * @param {object} [options] - Additional AxeBuilder options
 * @param {string[]} [options.tags] - Override or augment accessibility tags
 * @param {string[]} [options.disableRules] - Optional rules to disable if explicitly scoped
 * @param {string} [options.include] - CSS selector to restrict audit scope
 * @param {string} [options.exclude] - CSS selector to exclude from audit
 * @returns {Promise<import('axe-core').AxeResults>}
 */
export async function runAxeCheck(page, options = {}) {
  const tags = options.tags || WCAG_22_AA_TAGS;
  let builder = new AxeBuilder({page}).withTags(tags);

  if (options.disableRules && options.disableRules.length > 0) {
    builder = builder.disableRules(options.disableRules);
  }
  if (options.include) {
    builder = builder.include(options.include);
  }
  if (options.exclude) {
    builder = builder.exclude(options.exclude);
  }

  const results = await builder.analyze();

  if (results.violations.length > 0) {
    const details = results.violations.map((violation) => {
      const nodes = violation.nodes.map((n) => `  - Target: ${n.target.join(' ')}\n    Failure: ${n.failureSummary}`).join('\n');
      return `[${violation.impact.toUpperCase()}] ${violation.id}: ${violation.description}\nHelp: ${violation.helpUrl}\nNodes:\n${nodes}`;
    }).join('\n\n');

    expect(results.violations, `WCAG 2.2 AA accessibility violations detected:\n${details}`).toEqual([]);
  }

  return results;
}

export default runAxeCheck;
