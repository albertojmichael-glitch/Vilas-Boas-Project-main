import { expect, test } from '@playwright/test';
import { enviar, iniciarPartida, ligarTV, violacoesWcag } from './apoio';

// Auditoria WCAG 2.2 AA (axe-core) em cada tela e modal, no desktop e no celular.

test('tela inicial sem violações WCAG', async ({ page }) => {
    await page.goto('/');
    expect(await violacoesWcag(page)).toEqual([]);
});

test('terminal e partida sem violações WCAG', async ({ page }) => {
    await ligarTV(page);
    expect(await violacoesWcag(page)).toEqual([]);
    await enviar(page, 'dir');
    await enviar(page, '2');
    expect(await violacoesWcag(page)).toEqual([]);
});

for (const [botao, modal] of [
    ['#help-btn', '#help-modal'],
    ['#settings-btn', '#settings-modal'],
    ['#saves-btn', '#saves-modal'],
    ['#achievements-btn', '#achievements-modal'],
] as const) {
    test(`modal ${modal} acessível e fecha com Esc`, async ({ page }) => {
        await ligarTV(page);
        await page.locator(botao).click();
        await expect(page.locator(modal)).toBeVisible();
        await expect(page.locator(modal)).toHaveAttribute('role', 'dialog');
        expect(await violacoesWcag(page)).toEqual([]);
        await page.keyboard.press('Escape');
        await expect(page.locator(modal)).toBeHidden();
    });
}

test('saída do jogo é anunciada por leitores de tela', async ({ page }) => {
    await page.goto('/');
    await expect(page.locator('#output')).toHaveAttribute('aria-live', 'polite');
    await expect(page.locator('#output')).toHaveAttribute('role', 'log');
});

test('zoom não é bloqueado', async ({ page }) => {
    await page.goto('/');
    const viewport = await page.locator('meta[name="viewport"]').getAttribute('content');
    expect(viewport).not.toContain('user-scalable=no');
    expect(viewport).not.toContain('maximum-scale');
});

test('partida jogável só pelo teclado', async ({ page }) => {
    await iniciarPartida(page);
    await expect(page.locator('#comando')).toBeFocused();
    await page.keyboard.type('ir direita');
    await page.keyboard.press('Enter');
    await expect(page.locator('#output')).toContainText('HALL DE ENTRADA', { timeout: 30_000 });
});
