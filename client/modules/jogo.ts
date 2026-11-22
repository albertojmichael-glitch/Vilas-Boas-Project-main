import { pedirJSON } from './api.js';
import { processarConquistas } from './conquistas.js';
import { getEl } from './constants.js';
import { inputField, inputLineDiv, loadingLabel, loadingSpinner, retryBoot, shortcuts } from './dom.js';
import { mostrarTelaDePontuacao } from './placar.js';
import { prefTelemetria } from './preferencias.js';
import { adicionarLinhaTerminal, processarLinhas } from './terminal.js';
import type { RespostaServidor } from './types.js';

const indicador = getEl('save-indicator');
let ocupado = false;

function habilitarEntrada(habilitada: boolean): void {
    inputLineDiv?.classList.toggle('hidden', !habilitada);
    if (inputField) inputField.disabled = !habilitada;
    shortcuts.forEach(btn => { btn.disabled = !habilitada; });
}

async function turno(rota: '/iniciar' | '/comando', comando?: string): Promise<void> {
    if (ocupado) return;
    ocupado = true;
    let falhaNoBoot = false;
    habilitarEntrada(false);
    retryBoot?.classList.add('hidden');
    if (loadingLabel) loadingLabel.textContent = 'Processando…';
    loadingSpinner?.classList.remove('hidden');
    try {
        const opcoes: RequestInit = { method: 'POST' };
        if (comando !== undefined) {
            opcoes.headers = { 'Content-Type': 'application/json' };
            opcoes.body = JSON.stringify({ comando, telemetria: prefTelemetria });
        }
        const dados = await pedirJSON<RespostaServidor>(rota, opcoes, () => {
            if (loadingLabel) loadingLabel.textContent = 'O servidor está demorando. Aguardando resposta…';
        });
        loadingSpinner?.classList.add('hidden');
        if (!dados || !Array.isArray(dados.linhas)) throw new Error('Resposta do jogo inválida.');
        await processarLinhas(dados.linhas, dados.estado);
        processarConquistas(dados.novas_conquistas);
        const { estado_jogo, tempo_final, final_alcancado, nome_final } = dados.estado ?? {};
        if (estado_jogo === 'FIM' && tempo_final && tempo_final > 0) {
            setTimeout(() => mostrarTelaDePontuacao(tempo_final, nome_final || final_alcancado), 5000);
        }
        if (comando !== undefined && indicador) {
            indicador.classList.remove('hidden');
            setTimeout(() => indicador.classList.add('hidden'), 1500);
        }
    } catch (erro) {
        const mensagem = erro instanceof Error ? erro.message : 'Falha ao comunicar com o servidor.';
        adicionarLinhaTerminal(`[ERRO] ${mensagem}`, 'vermelho');
        falhaNoBoot = rota === '/iniciar';
    } finally {
        ocupado = false;
        loadingSpinner?.classList.add('hidden');
        habilitarEntrada(!falhaNoBoot);
        retryBoot?.classList.toggle('hidden', !falhaNoBoot);
        if (falhaNoBoot) retryBoot?.focus();
        else inputField?.focus();
    }
}

export const iniciarJogo = () => turno('/iniciar');
export const enviarComando = (cmd: string) => turno('/comando', cmd);

export function executarAtalho(cmd: string): void {
    if (!inputField || inputField.disabled) return;
    inputField.value = cmd;
    inputField.focus();
    void enviarComando(cmd);
}
