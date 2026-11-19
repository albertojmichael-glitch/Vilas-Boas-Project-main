import { expect, test } from '@playwright/test';
import { enviar, ligarTV } from './apoio';

// Regressão do bug relatado: as opções 5 e 6 do menu davam "OPÇÃO INVÁLIDA".
test('opção 5 abre os modos de desafio', async ({ page }) => {
    await ligarTV(page);
    await enviar(page, 'dir');
    await enviar(page, '5');
    await expect(page.locator('#output')).toContainText('MODOS DE DESAFIO');
    await expect(page.locator('#output')).not.toContainText('OPÇÃO INVÁLIDA');
});

test('opção 6 abre o modo personalizado e inicia a partida', async ({ page }) => {
    await ligarTV(page);
    await enviar(page, 'dir');
    await enviar(page, '6');
    await expect(page.locator('#output')).toContainText('MODO PERSONALIZADO');
    for (const valor of ['12', '3', '5']) await enviar(page, valor);
    await expect(page.locator('#output')).toContainText('VOCÊ ESTÁ EM', { timeout: 30_000 });
});
