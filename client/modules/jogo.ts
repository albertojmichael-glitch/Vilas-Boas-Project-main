import { pedirJSON } from './api.js';
import { processarConquistas } from './conquistas.js';
import { getEl } from './constants.js';
import { inputField, inputLineDiv, loadingLabel, loadingSpinner, retryBoot, shortcuts } from './dom.js';
import { mostrarTelaDePontuacao } from './placar.js';
import { prefTelemetria } from './preferencias.js';
import { adicionarLinhaTerminal, processarLinhas } from './terminal.js';
import type { RespostaServidor } from './types.js';

const indicador = getEl('save-indicator');
const capitulo = getEl<HTMLSelectElement>('capitulo');
const navegacao = document.querySelectorAll<HTMLButtonElement>('[data-nav]');
let ocupado = false;

function atualizarProgresso(dados: RespostaServidor): void {
    const progresso = dados.progresso;
    if (!progresso) return;
    if (capitulo) {
        const anterior = capitulo.value;
        capitulo.replaceChildren(...progresso.capitulos.map(c => new Option(c.nome, c.id)));
        if (progresso.capitulos.some(c => c.id === anterior)) capitulo.value = anterior;
        capitulo.disabled = !progresso.capitulos.length;
    }
    for (const botao of navegacao) {
        switch (botao.dataset.nav) {
            case 'continuar': botao.disabled = !progresso.tem_save; break;
            case 'voltar': botao.disabled = !progresso.pode_voltar; break;
            case 'capitulo': botao.disabled = !progresso.capitulos.length; break;
        }
    }
    if (!indicador) return;
    if (dados.autosave === 'falhou') indicador.textContent = 'Falha ao salvar este turno.';
    else if (dados.autosave === 'salvo') indicador.textContent = 'Autosave confirmado às ' + new Date().toLocaleTimeString('pt-BR');
    else if (!indicador.textContent) indicador.textContent = progresso.tem_save ? 'Autosave disponível.' : 'Aguardando primeiro autosave.';
}

export function configurarNavegacao(): void {
    for (const botao of navegacao) botao.addEventListener('click', () => {
        const acao = botao.dataset.nav;
        if (!acao || ocupado) return;
        void turno('/navegar', acao === 'capitulo' ? {acao, capitulo: capitulo?.value} : {acao});
    });
}

function habilitarEntrada(habilitada: boolean): void {
    inputLineDiv?.classList.toggle('hidden', !habilitada);
    if (inputField) inputField.disabled = !habilitada;
    shortcuts.forEach(btn => { btn.disabled = !habilitada; });
}

async function turno(rota: '/iniciar' | '/comando' | '/navegar', corpo?: object): Promise<void> {
    if (ocupado) return;
    ocupado = true;
    let falhaNoBoot = false;
    habilitarEntrada(false);
    retryBoot?.classList.add('hidden');
    if (loadingLabel) loadingLabel.textContent = 'Processando…';
    loadingSpinner?.classList.remove('hidden');
    try {
        const opcoes: RequestInit = { method: 'POST' };
        if (corpo !== undefined) {
            opcoes.headers = { 'Content-Type': 'application/json' };
            opcoes.body = JSON.stringify(corpo);
        }
        const dados = await pedirJSON<RespostaServidor>(rota, opcoes, () => {
            if (loadingLabel) loadingLabel.textContent = 'O servidor está demorando. Aguardando resposta…';
        });
        loadingSpinner?.classList.add('hidden');
        if (!dados || !Array.isArray(dados.linhas)) throw new Error('Resposta do jogo inválida.');
        if (rota === '/navegar') {
            getEl('saves-modal')?.classList.add('hidden');
            getEl('arcade-score-modal')?.classList.add('hidden');
        }
        atualizarProgresso(dados);
        await processarLinhas(dados.linhas, dados.estado);
        processarConquistas(dados.novas_conquistas);
        const estado = dados.estado;
        if (estado?.estado_jogo === 'FIM') {
            mostrarTelaDePontuacao(estado.tempo_final ?? 0, estado.nome_final, estado.tipo_final);
        }
    } catch (erro) {
        const mensagem = erro instanceof Error ? erro.message : 'Falha ao comunicar com o servidor.';
        adicionarLinhaTerminal(`[ERRO] ${mensagem}`, 'vermelho');
        if (indicador && rota !== '/iniciar') indicador.textContent = 'Sem confirmação de novo autosave.';
        if (rota === '/navegar') {
            getEl('saves-modal')?.classList.add('hidden');
            getEl('arcade-score-modal')?.classList.add('hidden');
        }
        falhaNoBoot = rota === '/iniciar';
    } finally {
        ocupado = false;
        loadingSpinner?.classList.add('hidden');
        habilitarEntrada(!falhaNoBoot);
        retryBoot?.classList.toggle('hidden', !falhaNoBoot);
        if (falhaNoBoot) retryBoot?.focus();
        else if (getEl('arcade-score-modal')?.classList.contains('hidden')) inputField?.focus();
    }
}

export const iniciarJogo = () => turno('/iniciar');
export const enviarComando = (cmd: string) => turno('/comando', { comando: cmd, telemetria: prefTelemetria });

export function executarAtalho(cmd: string): void {
    if (!inputField || inputField.disabled) return;
    inputField.value = cmd;
    inputField.focus();
    void enviarComando(cmd);
}
