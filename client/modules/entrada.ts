import { iniciarSomAmbiente, reproduzirBeep, tocarBipEntrada } from './audio.js';
import { fecharModalConquistas, modal } from './conquistas.js';
import { inputField as campoComando, outputDiv, terminal } from './dom.js';
import { getEl } from './constants.js';
import { enviarComando } from './jogo.js';
import { closeHelp, closeSaves, closeSettings, openHelp } from './modais.js';
import { adicionarLinhaTerminal, anexarLinha } from './terminal.js';

const historicoComandos: string[] = [];
let posicaoHistorico = -1;
let comandoDigitadoAtual = "";

export function configurarEntrada(): void {
    const inputField = campoComando;
    document.body.addEventListener('click', iniciarSomAmbiente, { once: true });
    document.body.addEventListener('keydown', iniciarSomAmbiente, { once: true });

    window.addEventListener('click', (e: MouseEvent) => {
        if (e.target === modal) {
            fecharModalConquistas();
        }
    });

    terminal?.addEventListener('click', (event: MouseEvent) => {
        const target = event.target as HTMLElement | null;
        const textoSelecionado = window.getSelection()?.toString() || "";
        const clicouNoBotao = target?.closest('button');
        const clicouNoInput = target?.closest('.input-line');

        if (!textoSelecionado && inputField && !clicouNoBotao) {
            const isMobile = window.matchMedia("(max-width: 768px)").matches;
            if (!isMobile || clicouNoInput) {
                inputField.focus();
            }
        }
    });

    inputField?.addEventListener("keydown", async (event: KeyboardEvent) => {
        if (event.key === "Enter") {
            const comandoBruto = inputField.value;
            const comando = comandoBruto.trim();

            if (comando !== "") {
                tocarBipEntrada();

                if (historicoComandos[historicoComandos.length - 1] !== comando) {
                    historicoComandos.push(comando);
                }
                posicaoHistorico = historicoComandos.length;

                if (outputDiv) {
                    const p = document.createElement("p");
                    p.className = "branco";
                    const spanPrompt = document.createElement("span");
                    spanPrompt.className = "prompt";
                    spanPrompt.textContent = "C:\\>";
                    p.appendChild(spanPrompt);
                    p.appendChild(document.createTextNode(` ${comandoBruto}`));
                    anexarLinha(p);
                }

                inputField.value = "";
                if (terminal) terminal.scrollTop = terminal.scrollHeight;

                if (comando.toLowerCase() === "ajuda") {
                    openHelp();
                    adicionarLinhaTerminal("[SISTEMA] Abrindo o Manual de Sobrevivência...", "verde");
                } else {
                    await enviarComando(comando);
                }
            }
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            if (posicaoHistorico === historicoComandos.length) {
                comandoDigitadoAtual = inputField.value;
            }
            if (posicaoHistorico > 0) {
                posicaoHistorico--;
                inputField.value = historicoComandos[posicaoHistorico];
            }
        } else if (event.key === "ArrowDown") {
            event.preventDefault();
            if (posicaoHistorico < historicoComandos.length - 1) {
                posicaoHistorico++;
                inputField.value = historicoComandos[posicaoHistorico];
            } else if (posicaoHistorico === historicoComandos.length - 1) {
                posicaoHistorico++;
                inputField.value = comandoDigitadoAtual;
            }
        }
    });

    document.addEventListener('keydown', (e: KeyboardEvent) => {
        if (e.key === "Escape") {
            closeHelp();
            closeSettings();
            closeSaves();
            fecharModalConquistas();
        }

        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
            e.preventDefault();
            if (outputDiv) outputDiv.textContent = '';
            reproduzirBeep('sucesso');
        }

        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
            e.preventDefault();
            adicionarLinhaTerminal(getEl("save-indicator")?.textContent || "Aguardando primeiro autosave.");
            reproduzirBeep('sucesso');
        }

        if (e.key === '?' && document.activeElement !== inputField) {
            e.preventDefault();
            openHelp();
        }
    });
}
