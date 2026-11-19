import { defineConfig, devices } from '@playwright/test';

// Testes de acessibilidade e mobile do cliente web (tests/e2e).
// O webServer compila o front e sobe front + API Flask juntos, como na Vercel.
const PORTA = 5173;
const PYTHON = process.env.PYTHON ?? 'python3';
// Em ambientes com Chromium pré-instalado (ex.: containers), aponte o caminho aqui.
const executablePath = process.env.PW_CHROMIUM_PATH || undefined;

export default defineConfig({
    testDir: 'tests/e2e',
    timeout: 90_000,
    fullyParallel: false,
    workers: 1,
    retries: process.env.CI ? 1 : 0,
    reporter: process.env.CI ? [['list'], ['html', { open: 'never' }]] : 'list',
    use: {
        baseURL: `http://localhost:${PORTA}`,
        trace: 'off',
        launchOptions: { executablePath },
    },
    projects: [
        { name: 'desktop', use: { ...devices['Desktop Chrome'] } },
        { name: 'mobile', use: { ...devices['Pixel 7'] } },
    ],
    webServer: {
        command: `npm run build && ${PYTHON} tests/e2e/servidor_e2e.py ${PORTA}`,
        url: `http://localhost:${PORTA}/index.html`,
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
    },
});
