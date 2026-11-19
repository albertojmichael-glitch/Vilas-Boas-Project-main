import { expect, test } from '@playwright/test';
import { enviar, iniciarPartida, ligarTV } from './apoio';

// Comportamento em celular (projeto "mobile" = Pixel 7). No desktop são pulados.
test.beforeEach(({ isMobile }) => {
    test.skip(!isMobile, 'apenas no celular');
});

async function semRolagemHorizontal(page) {
    const { largura, conteudo } = await page.evaluate(() => ({
        largura: window.innerWidth,
        conteudo: document.documentElement.scrollWidth,
    }));
    expect(conteudo).toBeLessThanOrEqual(largura + 1);
}

test('nenhuma tela rola na horizontal', async ({ page }) => {
    await page.goto('/');
    await semRolagemHorizontal(page);
    await ligarTV(page);
    await semRolagemHorizontal(page);
    await enviar(page, 'dir');
    await semRolagemHorizontal(page);
});

test('botões de toque visíveis e com área mínima de 24px', async ({ page }) => {
    await iniciarPartida(page);
    const botoes = page.locator('.mobile-controls button');
    await expect(botoes.first()).toBeVisible();
    for (const botao of await botoes.all()) {
        const caixa = await botao.boundingBox();
        expect(caixa, await botao.getAttribute('aria-label') ?? '').not.toBeNull();
        expect(caixa!.width).toBeGreaterThanOrEqual(24);
        expect(caixa!.height).toBeGreaterThanOrEqual(24);
    }
});

test('dá para jogar só tocando nos botões', async ({ page }) => {
    await iniciarPartida(page);
    await page.getByRole('button', { name: 'Ir para a direita' }).tap();
    await expect(page.locator('#output')).toContainText('HALL DE ENTRADA', { timeout: 30_000 });
    await page.getByRole('button', { name: /INV: ver inventário/ }).tap();
    await expect(page.locator('#output')).toContainText(/lanterna/i, { timeout: 30_000 });
});

test('campo de comando legível sem zoom automático (fonte >= 16px)', async ({ page }) => {
    await ligarTV(page);
    const tamanho = await page
        .locator('#comando')
        .evaluate(el => parseFloat(getComputedStyle(el).fontSize));
    expect(tamanho).toBeGreaterThanOrEqual(16);
});
