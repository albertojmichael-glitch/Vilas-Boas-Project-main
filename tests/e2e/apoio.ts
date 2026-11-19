import { expect, type Page } from '@playwright/test';
import AxeBuilder from '@axe-core/playwright';

export const TAGS_WCAG = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

/** Texto instantâneo (a mesma preferência "velocidade 0" das configurações). */
export async function textoInstantaneo(page: Page): Promise<void> {
    await page.addInitScript(() => localStorage.setItem('vilasBoasSpeed', '0'));
}

/** Liga a TV e espera o terminal liberar o campo de comando. */
export async function ligarTV(page: Page): Promise<void> {
    await textoInstantaneo(page);
    await page.goto('/');
    await page.getByRole('button', { name: /ligar a tv/i }).click();
    await expect(page.locator('#comando')).toBeEditable({ timeout: 30_000 });
}

/** Digita um comando como o jogador e espera a resposta terminar de aparecer. */
export async function enviar(page: Page, comando: string): Promise<void> {
    const campo = page.locator('#comando');
    await expect(campo).toBeEditable({ timeout: 30_000 });
    await campo.fill(comando);
    await campo.press('Enter');
    await page.waitForTimeout(200);
    await expect(campo).toBeEditable({ timeout: 30_000 });
}

export async function textoDoTerminal(page: Page): Promise<string> {
    return page.locator('#output').innerText();
}

/** Inicia uma partida no NORMAL com texto rápido. */
export async function iniciarPartida(page: Page): Promise<void> {
    await ligarTV(page);
    await enviar(page, 'dir');
    await enviar(page, '2');
    await expect(page.locator('#output')).toContainText('VOCÊ ESTÁ EM', { timeout: 30_000 });
}

export async function violacoesWcag(page: Page): Promise<string[]> {
    const resultado = await new AxeBuilder({ page }).withTags(TAGS_WCAG).analyze();
    return resultado.violations.map(
        v => `${v.id} (${v.impact}): ${v.nodes.map(n => n.target.join(' ')).join(', ')}`,
    );
}
