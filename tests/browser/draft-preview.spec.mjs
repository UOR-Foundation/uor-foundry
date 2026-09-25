import {test, expect} from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';
import {runAxeCheck} from './helpers/axe-check.mjs';

const prefix = 'Local draft — not saved, published, or approved.\n\n';
const invalid = 'Enter non-empty UTF-8 text within 4,096 bytes.';
const input = (page) => page.getByRole('textbox', {name: 'Draft text (maximum 4,096 UTF-8 bytes)'});
const submit = (page) => page.getByRole('button', {name: 'Preview draft'});
const result = (page) => page.getByRole('status', {name: 'Unpublished local preview'});
const expectResult = async (page, value) => {
  await expect.poll(() => result(page).evaluate((element) => element.textContent)).toBe(value);
};

test.beforeAll(async ({browser, browserName}) => {
  expect(browserName).toBe('chromium');
  expect(browser.browserType().name()).toBe('chromium');
});

test('generated Wasm previews text without network effects after bootstrap', async ({page, context}) => {
  const requests = [];
  page.on('request', (request) => requests.push({url: request.url(), method: request.method()}));
  const errors = [];
  page.on('pageerror', (error) => errors.push(error.message));
  await Promise.all([
    page.waitForResponse((response) => response.url().endsWith('/prism_foundry_web_bg.wasm')),
    page.goto('./'),
  ]);
  await page.waitForLoadState('networkidle');
  await expect(page).toHaveTitle('Foundry — draft preview');
  await expect(page.getByRole('heading', {name: 'Local draft preview', exact: true})).toBeVisible();
  const bootstrapPaths = new Set(['/foundry-web/', '/foundry-web/app.css',
    '/foundry-web/app.js', '/foundry-web/prism_foundry_web.js',
    '/foundry-web/prism_foundry_web_bg.wasm', '/favicon.ico']);
  expect(requests.every(({url, method}) => method === 'GET'
    && new URL(url).origin === 'http://127.0.0.1:4173'
    && bootstrapPaths.has(new URL(url).pathname))).toBe(true);
  const requestCount = requests.length;
  await input(page).fill('Hello, Foundry.');
  await submit(page).click();
  await expectResult(page, prefix + 'Hello, Foundry.');
  expect(requests).toHaveLength(requestCount);
  await context.setOffline(true);
  for (const text of ['Citizen Gardens — ideas 🌱', '<script>alert("draft")</script>', '\uFEFFdata', '  draft\n\ntext  ']) {
    await input(page).fill(text);
    await input(page).press('Control+Enter');
    await expectResult(page, prefix + text);
    await expect(result(page).locator('script')).toHaveCount(0);
  }
  expect(requests).toHaveLength(requestCount);
  expect(errors).toEqual([]);
});

test('byte boundaries, malformed UTF-16, empty input, and keyboard recovery', async ({page}) => {
  await page.goto('./');
  await input(page).fill('🌱'.repeat(1024));
  await submit(page).click();
  await expectResult(page, prefix + '🌱'.repeat(1024));
  await input(page).fill('🌱'.repeat(1024) + 'x');
  await submit(page).click();
  await expectResult(page, invalid);
  await expect(input(page)).toHaveAttribute('aria-invalid', 'true');
  await expect(input(page)).toBeFocused();
  await input(page).evaluate((element) => {element.value = '\ud800';});
  await submit(page).click();
  await expectResult(page, invalid);
  await input(page).fill('');
  await submit(page).click();
  await expectResult(page, invalid);
  await input(page).fill('Recovered draft');
  await input(page).press('Tab');
  await expect(submit(page)).toBeFocused();
  await page.keyboard.press('Enter');
  await expectResult(page, prefix + 'Recovered draft');
  await expect(input(page)).not.toHaveAttribute('aria-invalid');
  await expect(submit(page)).toBeEnabled();
});

test('initialization failures are visible without losing input', async ({page}) => {
  const requests = [];
  page.on('request', (request) => requests.push(request.url()));
  for (const resource of ['**/*.wasm', '**/prism_foundry_web.js']) {
    await page.route(resource, (route) => route.abort());
    await page.goto('./');
    await page.waitForLoadState('networkidle');
    const requestCount = requests.length;
    await input(page).fill('Retain this draft');
    await submit(page).click();
    await expectResult(page,
      'The draft preview could not be produced. Your input has not been published.');
    await expect(input(page)).toHaveValue('Retain this draft');
    await expect(submit(page)).toBeEnabled();
    await expect(page).toHaveURL('http://127.0.0.1:4173/foundry-web/');
    expect(requests).toHaveLength(requestCount);
    await page.unroute(resource);
  }
});

test('unavailable scripts cannot submit drafts through native forms', async ({browser}) => {
  for (const javaScriptEnabled of [false, true]) {
    const context = await browser.newContext({javaScriptEnabled, baseURL: 'http://127.0.0.1:4173/foundry-web/'});
    try {
      const requests = [];
      context.on('request', (request) => requests.push(request.url()));
      const page = await context.newPage();
      await page.route('**/app.js', (route) => route.abort());
      await page.goto('./');
      await page.waitForLoadState('networkidle');
      const requestCount = requests.length;
      await input(page).fill('Synthetic private draft — do not send');
      await expect(submit(page)).toBeDisabled();
      await expectResult(page,
        'The draft preview could not be produced. Your input has not been published.');
      // Bypass the disabled button and any submit handler: HTML policy must still block navigation.
      const directive = await page.evaluate(() => new Promise((resolve) => {
        document.addEventListener('securitypolicyviolation', (event) => resolve(event.violatedDirective), {once: true});
        HTMLFormElement.prototype.submit.call(document.getElementById('application-form'));
      }));
      expect(directive).toBe('form-action');
      expect(requests).toHaveLength(requestCount);
      await expect(page).toHaveURL('http://127.0.0.1:4173/foundry-web/');
      await expect(input(page)).toHaveValue('Synthetic private draft — do not send');
    } finally {
      await context.close();
    }
  }
});

test('responsive layout and automated accessibility checks', async ({page}) => {
  test.setTimeout(60_000);
  for (const width of [320, 1280]) {
    await page.setViewportSize({width, height: 800});
    await page.goto('./');
    await expect(input(page)).toBeVisible();
    await expect(submit(page)).toBeVisible();
    const auditState = async () => {
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
      await runAxeCheck(page);
    };
    await auditState();
    await input(page).fill('x'.repeat(4096));
    await submit(page).click();
    await expectResult(page, prefix + 'x'.repeat(4096));
    await auditState();
    await input(page).fill('x'.repeat(4097));
    await submit(page).click();
    await expectResult(page, invalid);
    await expect(input(page)).toHaveAttribute('aria-invalid', 'true');
    await auditState();
  }
});
