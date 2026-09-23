import {defineConfig} from '@playwright/test';

export default defineConfig({
  testDir: './tests/browser',
  fullyParallel: false,
  workers: 1,
  forbidOnly: true,
  retries: 0,
  reporter: 'line',
  use: {baseURL: 'http://127.0.0.1:4173/foundry-web/', headless: true},
  projects: [{name: 'chromium', use: {browserName: 'chromium'}}],
  webServer: {
    command: 'node scripts/serve-test-view.mjs',
    url: 'http://127.0.0.1:4173/foundry-web/',
    reuseExistingServer: false,
  },
});
