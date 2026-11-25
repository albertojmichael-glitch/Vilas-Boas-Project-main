import { expect, test, type Page } from '@playwright/test';
import { readFile } from 'node:fs/promises';

const saida = (page: Page) => page.locator('#output');
const entrada = (page: Page) => page.getByRole('textbox', { name: 'Digite seu comando aqui' });

async function ligar(page: Page) {
    await page.locator('#power-button').click();
    await page.clock.fastForward(6100);
}

async function comando(page: Page, texto: string) {
    const campo = entrada(page);
    await campo.fill(texto);
    const resposta = page.waitForResponse(r => r.url().endsWith('/api/comando') && r.request().method() === 'POST');
    await campo.press('Enter');
    expect((await resposta).ok()).toBeTruthy();
    await expect(campo).toBeEnabled();
}

async function menu(page: Page) {
    await ligar(page);
    await expect(saida(page)).toContainText('COMMAND.COM');
    await comando(page, 'dir');
    await expect(saida(page)).toContainText('SELECIONE UMA OPÇÃO');
}

async function jogar(page: Page, cenario?: string) {
    await page.goto(cenario ? `/cenario/${cenario}` : '/');
    await menu(page);
    await comando(page, cenario ? '7' : '2');
    await expect(saida(page)).toContainText(cenario ? 'RESTAURADO COM SUCESSO' : 'MODO NORMAL SELECIONADO');
}

test.beforeEach(async ({ page }) => {
    await page.clock.install();
    await page.addInitScript(() => {
        localStorage.setItem('vilasBoasSpeed', '0');
        localStorage.setItem('vilasBoasVolume', '0');
        localStorage.setItem('vilasBoasTelemetry', 'false');
    });
});

test('inventário, autosave e restauração do arquivo exportado', async ({ page }, info) => {
    await jogar(page);
    const item = 'tabua pequena de madeira';
    await comando(page, 'pegar ' + item);
    await expect(page.locator('#inv-list')).toContainText(item);
    await expect(page.locator('#hud-inv')).toHaveText('INV (2/3):');
    await page.reload();
    await menu(page);
    await comando(page, '7');
    await expect(page.locator('#inv-list')).toContainText(item);

    await page.getByRole('button', { name: 'Gerenciar Saves' }).click();
    const download = page.waitForEvent('download');
    await page.getByRole('button', { name: /Exportar Save Atual/ }).click();
    const arquivo = info.outputPath('partida.json');
    await (await download).saveAs(arquivo);
    const save = JSON.parse(await readFile(arquivo, 'utf8'));
    expect(save.estado.inventario).toContain(item);
    expect(save.assinatura).toBeTruthy();
    await page.keyboard.press('Escape');
    await comando(page, 'largar ' + item);
    await expect(page.locator('#inv-list')).not.toContainText(item);

    await page.getByRole('button', { name: 'Gerenciar Saves' }).click();
    const seletor = page.waitForEvent('filechooser');
    await page.getByRole('button', { name: /Importar Save/ }).click();
    const dialogo = page.waitForEvent('dialog').then(async d => {
        const mensagem = d.message();
        await d.accept();
        return mensagem;
    });
    await (await seletor).setFiles(arquivo);
    expect(await dialogo).toContain('Memórias injetadas com sucesso');
    await menu(page);
    await comando(page, '7');
    await expect(page.locator('#inv-list')).toContainText(item);
    await expect(page.locator('.inv-slot.cheio')).toHaveCount(2);
});

for (const [nome, cenario, acao, final] of [
    ['Medíocre', '', 'ir atrás', 'FINAL MEDÍOCRE: ALGO TE SEGUIU'],
    ['Bons Sonhos', 'cama', 'ir frente', 'FINAL BONS SONHOS'],
    ['Neutro', 'neutro', 'ir direita', 'FINAL NEUTRO: ESCURIDÃO'],
    ['Verdadeiro', 'verdadeiro', 'ir direita', 'FINAL VERDADEIRO: LIBERTAÇÃO'],
]) {
    test(`final ${nome} chega ao placar`, async ({ page }) => {
        await jogar(page, cenario);
        await comando(page, acao);
        await expect(saida(page)).toContainText(final);
        await page.clock.fastForward(5100);
        await expect(page.locator('#arcade-score-modal')).toBeVisible();
        await expect(page.locator('#score-title')).toHaveText(['Medíocre', 'Bons Sonhos'].includes(nome) ? 'FINAL ALTERNATIVO' : 'VITÓRIA');
        await expect(page.locator('#score-ending-text')).toHaveText(nome.toUpperCase());
        await expect(page.locator('#score-time-display')).toHaveText(/^\d+:\d{2}:\d{2}$/);
    });
}

test('fugir de Ela custa um ponto de vida e permite voltar a explorar', async ({ page }) => {
    await jogar(page, 'combate');
    await comando(page, 'correr direita');
    await expect(saida(page)).toContainText('você escapa!');
    await expect(page.locator('#hp-val')).toHaveText('[██░]');
    await comando(page, 'olhar');
    await expect(saida(page)).toContainText('VOCÊ ESTÁ EM: HALL DE ENTRADA');
});

test('cofre aceita a senha e entrega a chave no inventário', async ({ page }) => {
    await jogar(page, 'cofre');
    await comando(page, 'abrir cofre');
    await expect(saida(page)).toContainText('Digite a senha de 4 dígitos');
    await comando(page, '1994');
    await expect(page.locator('#inv-list')).toContainText('chave dos fundos');
    await expect(page.locator('#achieve-name')).toHaveText('Mente Brilhante');
});

test('consertos consome a moeda e libera o julgamento e sua recompensa', async ({ page }) => {
    await jogar(page, 'arcade');
    await comando(page, 'jogar consertos');
    await expect(page.locator('#inv-list')).not.toContainText('moeda velha');
    await comando(page, '9');
    await expect(saida(page)).toContainText('Escolha inválida');
    for (const [pergunta, escolha] of [['Escolha a Cabeça', '1'], ['Escolha o Tronco', '2'], ['Escolha as Pernas', '1']]) {
        await expect(saida(page)).toContainText(pergunta);
        await comando(page, escolha);
    }
    await expect(page.locator('#inv-list')).toContainText('partitura rasgada');
    await comando(page, 'jogar julgamento');
    await expect(saida(page)).toContainText('PERGUNTA 1');
    for (const resposta of ['1995', 'ela', '1982', 'rogerio', 'angela', 'joao', 'renato']) {
        await comando(page, resposta);
    }
    await expect(page.locator('#inv-list')).toContainText('cartao de seguranca');
    await expect(saida(page)).toContainText('VOCÊ ESTÁ EM: SALA DE FLIPERAMAS');
});

test('Jon reage à pista falsa ou a uma direção errada e devolve o controle ao jogador', async ({ page }) => {
    await jogar(page, 'arcade');
    await comando(page, 'jogar jon');
    const pista = await saida(page).locator('p').filter({ hasText: /\[SENSÓRIO/ }).last().innerText();
    const sugerida = pista.includes('ar gelado') ? 'f' : pista.includes('canhota') ? 'e' : 'd';
    const errada = pista.includes('CONFUSO') ? sugerida : (sugerida === 'f' ? 'e' : 'f');
    await comando(page, errada);
    await expect(saida(page)).toContainText(/te enganou|triturador ativo/);
    await expect(saida(page)).toContainText('VOCÊ ESTÁ EM: SALA DE FLIPERAMAS');
});

test('mesa de segurança responde à abertura e ao fechamento da porta', async ({ page }) => {
    await jogar(page, 'seguranca');
    await comando(page, 'sentar');
    await expect(saida(page)).toContainText(/PORTA CENTRAL: Aberta/i);
    await comando(page, 'fechar porta');
    await expect(saida(page)).toContainText('porta de metal desce');
    await comando(page, 'fechar porta');
    await expect(saida(page)).toContainText('A porta já está fechada');
    await comando(page, 'abrir porta');
    await expect(saida(page)).toContainText('porta de metal se ergue');
});

test('labirinto do Minotauro atualiza o radar e desconta a bateria', async ({ page }) => {
    await jogar(page, 'energia');
    await comando(page, 'olhar');
    await expect(saida(page)).toContainText('Bateria da Lanterna: 30 turnos restantes');
    await comando(page, 'esperar');
    await expect(saida(page)).toContainText('Bateria da Lanterna: 29 turnos restantes');
    await expect(saida(page)).not.toContainText('30 turnos restantes');
});


test('API indisponível permite tentar o boot novamente', async ({ page }) => {
    await page.route('**/api/iniciar', r => r.fulfill({ status: 503, body: '' }), { times: 1 });
    await page.goto('/');
    await ligar(page);
    await expect(saida(page)).toContainText('Servidor indisponível');
    await page.getByRole('button', { name: 'Tentar conectar novamente' }).click();
    await expect(saida(page)).toContainText('COMMAND.COM');
    await comando(page, 'dir');
    await expect(saida(page)).toContainText('SELECIONE UMA OPÇÃO');
});

test('comando lento avisa, expira e libera o terminal', async ({ page }) => {
    await jogar(page);
    let soltar!: () => void;
    const liberado = new Promise<void>(resolve => { soltar = resolve; });
    await page.route('**/api/comando', async r => { await liberado; await r.abort(); }, { times: 1 });
    const pedido = page.waitForRequest('**/api/comando');
    await entrada(page).fill('olhar');
    await entrada(page).press('Enter');
    await pedido;
    try {
        await page.clock.fastForward(2600);
        await expect(page.getByRole('status').filter({ hasText: 'está demorando' })).toBeVisible();
        await page.clock.fastForward(15000);
        await expect(saida(page)).toContainText('demorou demais');
        await expect(entrada(page)).toBeEnabled();
    } finally {
        soltar();
    }
    await comando(page, 'olhar');
    await expect(saida(page)).toContainText('VOCÊ ESTÁ EM: ENTRADA');
});

for (const modo of ['instantâneo', 'animado']) {
    test(`texto externo no modo ${modo} preserva apenas as cores do terminal`, async ({ page }) => {
        await jogar(page);
        if (modo === 'animado') {
            await page.getByRole('button', { name: 'Abrir configurações' }).click();
            await page.locator('#speed-slider').press('ArrowRight');
            await page.keyboard.press('Escape');
        }
        await page.route('**/api/comando', r => r.fulfill({ json: {
            linhas: ['@@TYPE@@verde@@5@@<span class="amarelo">A &amp; B</span> <span title="nota">C</span>'],
        } }));
        await comando(page, 'olhar');
        const visual = saida(page).locator('p').last().locator('[aria-hidden]');
        await expect(visual).toHaveText('A & B <span title="nota">C</span>');
        await expect(visual.locator('span.amarelo')).toHaveText('A & B');
        await expect(visual.locator('[title]')).toHaveCount(0);
    });
}

test('autosave confirmado permite desfazer, reiniciar e carregar o capítulo alcançado', async ({ page }) => {
    await jogar(page);
    const status = page.locator('#save-indicator');
    await expect(status).toContainText('Autosave confirmado');
    const saves = page.getByRole('button', {name: 'Gerenciar Saves'});
    await comando(page, 'pegar tabua pequena de madeira');
    await saves.click();
    await page.getByRole('button', {name: 'Voltar uma ação salva'}).click();
    await expect(page.locator('#inv-list')).not.toContainText('tabua pequena de madeira');
    await saves.click();
    await expect(page.getByRole('button', {name: 'Voltar uma ação salva'})).toBeDisabled();
    await page.getByRole('button', {name: 'Reiniciar no menu de dificuldade'}).click();
    await expect(saida(page)).toContainText('SELECIONE UMA OPÇÃO');
    await saves.click();
    await page.getByLabel('Capítulo alcançado').selectOption('entrada');
    await page.getByRole('button', {name: 'Carregar capítulo'}).click();
    await expect(saida(page)).toContainText('VOCÊ ESTÁ EM: ENTRADA');
});

test('preferências antes do boot e retomada após a morte', async ({ page }) => {
    await page.emulateMedia({reducedMotion: 'reduce'});
    await page.goto('/cenario/morte');
    await expect(page.getByLabel('Reduzir efeitos visuais').filter({visible:true})).toBeChecked();
    await page.getByLabel('Silenciar sons de susto').filter({visible:true}).check();
    await page.reload();
    await expect(page.getByLabel('Silenciar sons de susto').filter({visible:true})).toBeChecked();
    await menu(page);
    await comando(page, '7');
    await comando(page, 'correr direita');
    await expect(page.locator('#score-title')).toHaveText('FIM DE JOGO');
    await expect(page.locator('#score-ending-text')).toContainText('Você morreu');
    await page.getByRole('button', {name:'Retomar último autosave'}).click();
    await expect(page.locator('#arcade-score-modal')).toBeHidden();
    await expect(page.locator('#hp-val')).toHaveText('[█░░]');
});
