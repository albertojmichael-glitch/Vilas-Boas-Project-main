import { trechosTerminal } from './texto.js';
import { reproduzirBeep, tocarPassoMetalico, tocarPorta, tocarSomDigito } from './audio.js';
import { getEl, LIMITE_LINHAS } from './constants.js';
import { outputDiv, terminal } from './dom.js';
import { ativarBSOD, ativarJumpscare, play1980Slideshow, startFlashingSecrets, stopFlashingSecrets } from './effects.js';
import { atualizarSidebar } from './hud.js';
import { prefMultiplicadorVelocidade } from './preferencias.js';
import type { CorTerminal, EstadoJogo } from './types.js';

export function adicionarLinhaTerminal(texto: string, cor: CorTerminal = ""): void {
    if (!outputDiv) return;
    const div = document.createElement("div");
    div.className = cor ? `linha ${cor}` : "linha";
    div.textContent = texto;
    anexarLinha(div);
}

export function anexarLinha(elemento: HTMLElement): void {
    if (!outputDiv) return;
    outputDiv.appendChild(elemento);

    while (outputDiv.childElementCount > LIMITE_LINHAS) {
        if (outputDiv.firstChild) outputDiv.removeChild(outputDiv.firstChild);
    }

    if (terminal) terminal.scrollTop = terminal.scrollHeight;
}


export async function processarLinhas(linhas: string[], estado?: EstadoJogo): Promise<void> {
    for (const linha of linhas) {
        await novaLinha(linha);
    }
    atualizarSidebar(estado);
}

function novaLinha(linhaBruta: string): Promise<void> {
    return new Promise((resolve) => {
        let linha = String(linhaBruta);

        if (linha.includes("@@JUMPSCARE@@")) {
            linha = linha.replace("@@JUMPSCARE@@", "");
            ativarJumpscare();
        }
        if (linha.includes("@@START_FLASH_SECRETS@@")) {
            startFlashingSecrets();
            linha = linha.replace("@@START_FLASH_SECRETS@@", "");
        }
        if (linha.includes("@@STOP_FLASH_SECRETS@@")) {
            stopFlashingSecrets();
            linha = linha.replace("@@STOP_FLASH_SECRETS@@", "");
        }
        if (linha.includes("@@PLAY_1980_SLIDESHOW@@")) {
            play1980Slideshow();
            linha = linha.replace("@@PLAY_1980_SLIDESHOW@@", "");
        }
        if (linha.includes("@@BSOD@@")) {
            linha = linha.replace("@@BSOD@@", "");
            ativarBSOD();
        }
        if (linha.includes("@@PASSO@@")) {
            linha = linha.replace("@@PASSO@@", "");
            tocarPassoMetalico();
        }
        if (linha.includes("@@PORTA@@")) {
            linha = linha.replace("@@PORTA@@", "");
            tocarPorta();
        }

        if (linha.startsWith("@@CLEAR@@")) {
            document.body.classList.remove("glitch-mode", "low-power-mode", "blackout-mode", "kernel-panic-mode");
            if (outputDiv) outputDiv.textContent = "";
            if (terminal) terminal.scrollTop = terminal.scrollHeight;
            resolve();
        } else if (linha.includes("@@PANIC@@")) {
            document.body.classList.add("kernel-panic-mode");
            reproduzirBeep('erro');

            const imgPanic = document.createElement("img");
            imgPanic.src = "assets/images/image.png";
            imgPanic.className = "panic-img-overlay";
            imgPanic.id = "panic-img-id";

            terminal?.appendChild(imgPanic);
            resolve();
        } else if (linha.includes("@@GLITCH_LUZ@@")) {
            document.body.classList.add("glitch-mode");
            reproduzirBeep('erro');
            setTimeout(() => document.body.classList.remove("glitch-mode"), 1500);
            resolve();
        } else if (linha.includes("@@LOW_POWER@@")) {
            document.body.classList.remove("blackout-mode");
            document.body.classList.add("low-power-mode");
            resolve();
        } else if (linha.includes("@@BLACKOUT@@")) {
            document.body.classList.remove("low-power-mode");
            document.body.classList.add("blackout-mode");
            resolve();
        } else if (linha.includes("@@NORMAL_POWER@@")) {
            document.body.classList.remove("low-power-mode", "blackout-mode");
            resolve();
        } else if (linha.startsWith("@@EXIT@@")) {
            document.body.textContent = "";
            document.body.style.backgroundColor = "#000";
            resolve();
        } else if (linha.startsWith("@@RELOAD@@")) {
            document.body.classList.remove("kernel-panic-mode", "low-power-mode", "blackout-mode");
            getEl("panic-img-id")?.remove();
            window.location.reload();
            resolve();
        } else if (linha.startsWith("@@PAUSE@@")) {
            const ms = parseInt(linha.split("@@")[2], 10) / 3;
            setTimeout(resolve, ms);
        } else if (linha.startsWith("@@TYPE@@")) {
            const parts = linha.split("@@");
            const cor = parts[2] as CorTerminal;
            const ms = parseInt(parts[3], 10);
            const texto = parts.slice(4).join("@@");

            const velocidadeFinal = prefMultiplicadorVelocidade === 0 ? 0 : ms * prefMultiplicadorVelocidade;
            digitarLinha(texto, cor, velocidadeFinal, resolve);
        } else {
            const velocidadeFinal = prefMultiplicadorVelocidade === 0 ? 0 : 15 * prefMultiplicadorVelocidade;
            digitarLinha(linha, "", velocidadeFinal, resolve);
        }
    });
}

function digitarLinha(linha: string, cor: CorTerminal, velocidade: number, concluir: () => void): void {
    const p = document.createElement('p');
    p.className = cor;
    const leitor = document.createElement('span');
    leitor.className = 'sr-only';
    const visual = document.createElement('span');
    visual.setAttribute('aria-hidden', 'true');
    const trechos = trechosTerminal(linha);
    const prefixo = cor === 'vermelho' ? 'Perigo: ' : cor === 'amarelo' ? 'Atenção: ' : '';
    leitor.textContent = prefixo + trechos.map(t => t.texto).join('');
    if (cor === 'vermelho') {
        document.body.classList.add('glitch-active');
        setTimeout(() => document.body.classList.remove('glitch-active'), 250);
    }

    const textos = trechos.map(trecho => {
        const no = document.createTextNode(velocidade > 0 ? '' : trecho.texto);
        if (trecho.cor) {
            const span = document.createElement('span');
            span.className = trecho.cor;
            span.append(no);
            visual.append(span);
        } else {
            visual.append(no);
        }
        return no;
    });
    p.append(leitor, visual);
    anexarLinha(p);
    if (!(velocidade > 0) || !trechos.length) return concluir();

    let trecho = 0;
    let posicao = 0;
    let anterior = 0;
    function digitar(agora: number): void {
        if (agora - anterior >= velocidade) {
            const char = trechos[trecho].texto[posicao++];
            textos[trecho].appendData(char);
            if (char !== ' ' && char !== '\n') tocarSomDigito();
            if (posicao === trechos[trecho].texto.length) {
                trecho++;
                posicao = 0;
            }
            if (terminal) terminal.scrollTop = terminal.scrollHeight;
            if (trecho === trechos.length) return concluir();
            anterior = agora;
        }
        requestAnimationFrame(digitar);
    }
    requestAnimationFrame(digitar);
}
