import { defineConfig } from '@playwright/test';

const baseURL = process.env.E2E_BASE_URL;

export default defineConfig({
    testDir: './e2e',
    timeout: 60_000,
    expect: { timeout: 10_000 },
    workers: 1,
    forbidOnly: !!process.env.CI,
    reporter: [['list'], ['html', { open: 'never' }]],
    use: { baseURL, trace: 'retain-on-failure' },
});
