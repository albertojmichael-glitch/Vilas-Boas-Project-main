import { pedirJSON } from './api.js';
import { sleep } from './constants.js';
import { inputField } from './dom.js';
import { processarConquistas } from './conquistas.js';
import { adicionarLinhaTerminal, processarLinhas } from './terminal.js';
import type { DadosReplay, RespostaServidor } from './types.js';

let modoReplayAtivo = false;

export async function iniciarReplay(idReplay: string): Promise<void> {
    if (modoReplayAtivo) return;

    try {
        const tokenAdmin = sessionStorage.getItem('vilasBoasAdminToken') || '';
        const headersReplay: Record<string, string> = {};
        if (tokenAdmin) {
            headersReplay['X-Admin-Token'] = tokenAdmin;
        }

        const data = await pedirJSON<DadosReplay>(`/admin/replay/${encodeURIComponent(idReplay)}`, {
            method: 'GET',
            credentials: 'include',
            headers: headersReplay
        });
        const comandos = data.log_comandos;
        if (!comandos || comandos.length === 0) {
            adicionarLinhaTerminal("[ SISTEMA ] Fita de replay vazia.", "amarelo");
            return;
        }

        modoReplayAtivo = true;
        if (inputField) {
            inputField.disabled = true;
            inputField.placeholder = "ASSISTINDO REPLAY...";
        }

        adicionarLinhaTerminal("=========================================", "amarelo");
        adicionarLinhaTerminal(`[ SISTEMA ] REPLAY INICIADO: ${data.jogador}`, "amarelo");
        adicionarLinhaTerminal(`[ SISTEMA ] DESTINO FINAL: ${data.sala_final}`, "amarelo");
        adicionarLinhaTerminal("=========================================", "amarelo");

        await sleep(2500);

        for (const comando of comandos) {
            if (!modoReplayAtivo) break;

            if (inputField) {
                inputField.value = "";
                for (const char of comando) {
                    inputField.value += char;
                    await sleep(70);
                }
            }

            await sleep(400);
            adicionarLinhaTerminal(`C:\\> ${comando}`, "branco");
            if (inputField) inputField.value = "";

            const resData = await pedirJSON<RespostaServidor>('/comando', {
                method: 'POST',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ comando, telemetria: false })
            });

            if (typeof window.processarRespostaBackend === "function") {
                window.processarRespostaBackend(resData);
            } else if (resData.linhas) {
                await processarLinhas(resData.linhas, resData.estado);
            }

            if (resData.novas_conquistas) {
                processarConquistas(resData.novas_conquistas);
            }

            await sleep(1500);
        }

        adicionarLinhaTerminal("=========================================", "amarelo");
        adicionarLinhaTerminal("[ SISTEMA ] FIM DA FITA DE REPLAY", "amarelo");
        adicionarLinhaTerminal("Recarregue a página (F5) para jogar de verdade.", "verde");
    } catch (erro) {
        console.error("Erro no replay:", erro);
        adicionarLinhaTerminal(erro instanceof Error ? erro.message : "Falha ao carregar replay.", "vermelho");
        modoReplayAtivo = false;
        if (inputField) {
            inputField.placeholder = "";
            inputField.disabled = false;
            inputField.focus();
        }
    }
}
