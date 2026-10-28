import {
    definirVolume,
    definirVolumeJumpscare,
    iniciarSomAmbiente,
    obterAudioContext,
    obterVolume,
    obterVolumeJumpscare,
    reproduzirBeep,
    tocarBipEntrada,
    tocarPassoMetalico,
    tocarPorta,
    tocarSomDigito,
} from './modules/audio.js';
import {
    API_URL,
    CATALOGO_CONQUISTAS,
    getEl,
    LIMITE_LINHAS,
    sleep,
} from './modules/constants.js';
import {
    aplicarEfeitoCRT,
    ativarBSOD,
    ativarJumpscare,
    play1980Slideshow,
    startFlashingSecrets,
    stopFlashingSecrets,
} from './modules/effects.js';
import { configurarLoggerStorage, SafeStorage } from './modules/storage.js';
import type {
    Conquista,
    CorTerminal,
    DadosReplay,
    EstadoJogo,
    RespostaServidor,
    RespostaShare,
    SaveExportado,
    TipoBeep,
} from './modules/types.js';

const outputDiv = getEl<HTMLDivElement>('output');
const inputField = getEl<HTMLInputElement>('comando');
const terminal = getEl<HTMLElement>('terminal');
const loadingSpinner = getEl<HTMLDivElement>('loading');
const inputLineDiv = document.querySelector<HTMLDivElement>('.input-line');
const terminalSection = document.querySelector<HTMLElement>('.terminal-section');

let historicoComandos: string[] = [];
let posicaoHistorico = -1;
let comandoDigitadoAtual = "";
let modoReplayAtivo = false;

let prefTelemetria = true;
let prefMultiplicadorVelocidade = 1.0;
let toastTimeout: ReturnType<typeof setTimeout> | undefined;

function playBip(tipo: TipoBeep = 'sucesso'): void {
    reproduzirBeep(tipo);
}

// Sanitiza HTML recebido do servidor permitindo apenas spans de cor do terminal
function sanitizarHTMLServidor(raw: string): string {
    return raw.replace(/<\/?(?!span\b)[^>]*>/gi, (tag) =>
        tag.replace(/</g, '&lt;').replace(/>/g, '&gt;')
    );
}

// 1. Sistema de conquistas

function abrirModalConquistas(): void {
    const modal = getEl<HTMLDivElement>('achievements-modal');
    const grid = getEl<HTMLDivElement>('achievements-grid');
    const title = getEl<HTMLHeadingElement>('achievements-title');
    if (!modal || !grid || !title) return;

    const salvas = SafeStorage.load<Record<string, Conquista>>('vilasBoasAchievements', {});
    grid.textContent = '';
    let quantidadeDesbloqueada = 0;

    CATALOGO_CONQUISTAS.forEach(conquista => {
        const estaDesbloqueada = Boolean(salvas[conquista.id]);
        if (estaDesbloqueada) quantidadeDesbloqueada++;

        const card = document.createElement('div');
        card.className = `achievement-card ${estaDesbloqueada ? 'unlocked' : 'locked'}`;

        const iconDiv = document.createElement('div');
        iconDiv.className = 'achieve-card-icon';
        iconDiv.textContent = conquista.icone;

        const infoDiv = document.createElement('div');
        infoDiv.className = 'achieve-card-info';

        const h3 = document.createElement('h3');
        h3.textContent = estaDesbloqueada ? conquista.nome : '???';

        const p = document.createElement('p');
        p.textContent = estaDesbloqueada ? conquista.desc : 'Continue explorando para desbloquear.';

        infoDiv.appendChild(h3);
        infoDiv.appendChild(p);
        card.appendChild(iconDiv);
        card.appendChild(infoDiv);
        grid.appendChild(card);
    });

    title.innerText = `♛ SUAS CONQUISTAS (${quantidadeDesbloqueada}/${CATALOGO_CONQUISTAS.length}) ♛`;
    modal.classList.remove('hidden');
}

function fecharModalConquistas(): void {
    getEl<HTMLDivElement>('achievements-modal')?.classList.add('hidden');
}

function processarConquistas(listaConquistas?: Conquista[]): void {
    if (!listaConquistas || listaConquistas.length === 0) return;

    const salvas = SafeStorage.load<Record<string, Conquista>>('vilasBoasAchievements', {});

    listaConquistas.forEach(conquista => {
        if (!salvas[conquista.id]) {
            salvas[conquista.id] = conquista;
            reproduzirBeep('sucesso');
            mostrarToastConquista(conquista);
        }
    });

    SafeStorage.save('vilasBoasAchievements', salvas);
}

function mostrarToastConquista(conquista: Conquista): void {
    const toast = getEl<HTMLDivElement>('achievement-toast');
    const icon = getEl<HTMLDivElement>('achieve-icon');
    const name = getEl<HTMLDivElement>('achieve-name');
    if (!toast || !icon || !name) return;

    icon.innerText = conquista.icone;
    name.innerText = conquista.nome;

    toast.classList.remove('hidden');
    toast.classList.add('show');

    clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
        toast.classList.remove('show');
    }, 4000);
}

// 2. Gerenciamento de saves e modais

function openSaves(): void {
    getEl<HTMLDivElement>('saves-modal')?.classList.remove('hidden');
}

function closeSaves(): void {
    getEl<HTMLDivElement>('saves-modal')?.classList.add('hidden');
    inputField?.focus();
}

function openHelp(): void {
    getEl<HTMLDivElement>('help-modal')?.classList.remove('hidden');
}

function closeHelp(): void {
    getEl<HTMLDivElement>('help-modal')?.classList.add('hidden');
    inputField?.focus();
}

function openSettings(): void {
    getEl<HTMLDivElement>('settings-modal')?.classList.remove('hidden');
}

function closeSettings(): void {
    getEl<HTMLDivElement>('settings-modal')?.classList.add('hidden');
    inputField?.focus();
}

async function exportarSave(): Promise<void> {
    try {
        const res = await fetch(`${API_URL}/save/export`, { credentials: 'include' });
        if (!res.ok) throw new Error("Nenhum progresso encontrado no servidor.");
        const data = await res.json();

        const blob = new Blob([JSON.stringify(data, null, 4)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `vilas_boas_backup_${new Date().toISOString().slice(0, 10)}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        reproduzirBeep('sucesso');
    } catch (erro) {
        const msg = erro instanceof Error ? erro.message : "Erro desconhecido";
        console.error(erro);
        alert(`[ERRO] ${msg}`);
    }
}

async function importarSave(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e: ProgressEvent<FileReader>) => {
        try {
            const conteudo = e.target?.result;
            if (typeof conteudo !== 'string') throw new Error("Falha ao ler arquivo.");

            const dados = JSON.parse(conteudo) as SaveExportado;

            if (!dados || typeof dados !== 'object' || Array.isArray(dados)) {
                throw new Error("Estrutura do arquivo incompatível com o sistema host.");
            }
            if (!dados.dados_seguros || typeof dados.dados_seguros !== 'string') {
                throw new Error("Assinatura de segurança (dados_seguros) ausente ou corrompida.");
            }
            if (!dados.resumo || typeof dados.resumo !== 'object' || !dados.resumo.sala) {
                throw new Error("Metadados (header) de ambiente ausentes.");
            }

            const res = await fetch(`${API_URL}/save/import`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(dados),
                credentials: 'include'
            });

            const result = await res.json();
            if (res.ok) {
                alert("Memórias injetadas com sucesso. A reiniciar o terminal...");
                window.location.reload();
            } else {
                alert(`[ERRO DE BIOS] ${result.erro || "Falha na decodificação do servidor."}`);
            }
        } catch (erro) {
            const msg = erro instanceof Error ? erro.message : "Arquivo inválido";
            alert(`[ERRO FATAL] O arquivo fornecido foi rejeitado.\nMotivo: ${msg}`);
        }
    };
    reader.readAsText(file);
    input.value = '';
}

async function gerarLinkCompartilhamentoUI(): Promise<void> {
    try {
        const res = await fetch(`${API_URL}/share/generate`, {
            method: 'POST',
            credentials: 'include'
        });
        const data = (await res.json()) as RespostaShare;

        if (res.ok) {
            const container = getEl<HTMLDivElement>('share-link-container');
            const input = getEl<HTMLInputElement>('share-link-input');
            const msg = getEl<HTMLParagraphElement>('share-link-msg');

            if (container && input && msg) {
                container.classList.remove('hidden');
                input.value = data.link;
                msg.innerText = data.mensagem;

                input.select();
                if (navigator.clipboard) {
                    await navigator.clipboard.writeText(data.link);
                } else {
                    document.execCommand('copy');
                }
            }
            reproduzirBeep('sucesso');
            alert("Link copiado para a área de transferência!");
        } else {
            alert(`[ERRO] ${data.erro || "Falha ao gerar link."}`);
        }
    } catch (erro) {
        console.error(erro);
        alert("Falha de conexão com os servidores centrais.");
    }
}

// 3. Preferências e configurações

function atualizarLabelVelocidade(val: number): void {
    const label = getEl<HTMLSpanElement>('speed-val-display');
    if (!label) return;
    label.innerText = val === 0 ? "Instantâneo (0x)" : `Atual (${val}x)`;
}

function carregarPreferencias(): void {
    const teleCheckbox = getEl<HTMLInputElement>('telemetry-checkbox');
    const speedSlider = getEl<HTMLInputElement>('speed-slider');
    const crtCheckbox = getEl<HTMLInputElement>('crt-checkbox');
    const volSlider = getEl<HTMLInputElement>('volume-slider');
    const volJumpSlider = getEl<HTMLInputElement>('volume-jump-slider');

    const savedTele = localStorage.getItem('vilasBoasTelemetry');
    const savedSpeed = localStorage.getItem('vilasBoasSpeed');
    const savedCRT = localStorage.getItem('vilasBoasCRT');
    const savedVolume = localStorage.getItem('vilasBoasVolume');
    const savedVolumeJump = localStorage.getItem('vilasBoasVolumeJump');

    if (savedTele !== null && teleCheckbox) {
        prefTelemetria = savedTele === 'true';
        teleCheckbox.checked = prefTelemetria;
    }
    teleCheckbox?.addEventListener('change', (e) => {
        prefTelemetria = (e.target as HTMLInputElement).checked;
        localStorage.setItem('vilasBoasTelemetry', String(prefTelemetria));
    });

    if (savedSpeed !== null && speedSlider) {
        prefMultiplicadorVelocidade = parseFloat(savedSpeed);
        speedSlider.value = String(prefMultiplicadorVelocidade);
        atualizarLabelVelocidade(prefMultiplicadorVelocidade);
    }
    speedSlider?.addEventListener('input', (e) => {
        prefMultiplicadorVelocidade = parseFloat((e.target as HTMLInputElement).value);
        localStorage.setItem('vilasBoasSpeed', String(prefMultiplicadorVelocidade));
        atualizarLabelVelocidade(prefMultiplicadorVelocidade);
    });

    if (crtCheckbox) {
        if (savedCRT !== null) {
            crtCheckbox.checked = savedCRT === 'true';
        }
        aplicarEfeitoCRT(crtCheckbox.checked);
        crtCheckbox.addEventListener('change', (e) => {
            const ativado = (e.target as HTMLInputElement).checked;
            localStorage.setItem('vilasBoasCRT', String(ativado));
            aplicarEfeitoCRT(ativado);
        });
    }

    if (savedVolume !== null && volSlider) {
        const vol = parseFloat(savedVolume);
        definirVolume(vol);
        volSlider.value = String(vol);
        const volDisplay = getEl<HTMLSpanElement>('volume-val-display');
        if (volDisplay) volDisplay.innerText = `${Math.round(vol * 100)}%`;
    }
    volSlider?.addEventListener('input', (e) => {
        const vol = parseFloat((e.target as HTMLInputElement).value);
        definirVolume(vol);
        localStorage.setItem('vilasBoasVolume', String(vol));
        const volDisplay = getEl<HTMLSpanElement>('volume-val-display');
        if (volDisplay) volDisplay.innerText = `${Math.round(vol * 100)}%`;
    });

    if (savedVolumeJump !== null && volJumpSlider) {
        const volJump = parseFloat(savedVolumeJump);
        definirVolumeJumpscare(volJump);
        volJumpSlider.value = String(volJump);
        const displayJump = getEl<HTMLSpanElement>('volume-jump-val-display');
        if (displayJump) displayJump.innerText = `${Math.round(volJump * 100)}%`;
    }
    volJumpSlider?.addEventListener('input', (e) => {
        const volJump = parseFloat((e.target as HTMLInputElement).value);
        definirVolumeJumpscare(volJump);
        localStorage.setItem('vilasBoasVolumeJump', String(volJump));
        const displayJump = getEl<HTMLSpanElement>('volume-jump-val-display');
        if (displayJump) displayJump.innerText = `${Math.round(volJump * 100)}%`;

        const testeSusto = new Audio('./assets/audio/grito.mp3');
        testeSusto.volume = obterVolume() * obterVolumeJumpscare();
        testeSusto.play().catch(() => {});
        setTimeout(() => testeSusto.pause(), 300);
    });
}

// 4. Atualização de HUD e terminal

function atualizarSidebar(estado?: EstadoJogo): void {
    if (!estado) return;

    const hpVal = getEl<HTMLSpanElement>("hp-val");
    if (hpVal) {
        if (estado.hp === "∞") {
            hpVal.textContent = "[ GOD MODE ]";
            hpVal.className = "amarelo";
            document.body.classList.remove("hp-critico");
        } else {
            const hpAtual = parseInt(String(estado.hp), 10) || 0;
            const maxHp = 3;
            const blocosCheios = "█".repeat(Math.max(0, hpAtual));
            const blocosVazios = "░".repeat(Math.max(0, maxHp - hpAtual));

            hpVal.textContent = `[${blocosCheios}${blocosVazios}]`;
            hpVal.className = hpAtual <= 1 ? "vermelho" : "verde";
            document.body.classList.toggle("hp-critico", hpAtual <= 1 && hpAtual > 0);
        }
    }

    const luzVal = getEl<HTMLSpanElement>("luz-val");
    if (luzVal) {
        luzVal.textContent = estado.luz_restante !== undefined ? String(estado.luz_restante) : "??";
        const luzAlta = estado.luz_restante === "∞" || Number(estado.luz_restante) > 3;
        luzVal.className = luzAlta ? "verde" : "vermelho";
    }

    const somVal = getEl<HTMLSpanElement>("som-val");
    if (somVal && estado.som !== undefined) {
        somVal.textContent = `${estado.som}%`;
        if (estado.som >= 80) somVal.className = "vermelho";
        else if (estado.som >= 50) somVal.className = "amarelo";
        else somVal.className = "verde";
    }

    const invList = getEl<HTMLUListElement>("inv-list");
    const invTitulo = document.querySelector<HTMLElement>("#hud-inv, .inventory-box h3");

    if (invList) {
        invList.textContent = "";

        const qtdBolsas = estado.bolsas_coletadas || 0;
        const limiteMaximo = 3 + (qtdBolsas * 3);
        const inventario = estado.inventario || [];
        const qtdAtual = inventario.length;

        if (invTitulo) {
            invTitulo.textContent = `INV (${qtdAtual}/${limiteMaximo}):`;
        }

        if (qtdAtual > 0) {
            inventario.forEach(item => {
                const li = document.createElement("li");
                li.textContent = `- ${item}`;
                li.className = "branco";
                invList.appendChild(li);
            });
        } else {
            const li = document.createElement("li");
            li.textContent = "Vazio";
            li.className = "amarelo";
            invList.appendChild(li);
        }

        let slotsContainer = getEl<HTMLDivElement>("inv-slots-container");
        if (!slotsContainer && invList.parentElement) {
            slotsContainer = document.createElement("div");
            slotsContainer.id = "inv-slots-container";
            slotsContainer.className = "inv-slots-container";
            invList.parentElement.appendChild(slotsContainer);
        }

        if (slotsContainer) {
            slotsContainer.textContent = "";
            for (let i = 0; i < limiteMaximo; i++) {
                const slotBox = document.createElement("div");
                slotBox.className = `inv-slot ${i < qtdAtual ? "cheio" : "vazio"}`;
                slotsContainer.appendChild(slotBox);
            }
        }
    }
}

function adicionarLinhaTerminal(texto: string, cor: CorTerminal = ""): void {
    if (!outputDiv) return;
    const div = document.createElement("div");
    div.className = cor ? `linha ${cor}` : "linha";
    div.textContent = texto;
    outputDiv.appendChild(div);

    while (outputDiv.childElementCount > LIMITE_LINHAS) {
        if (outputDiv.firstChild) outputDiv.removeChild(outputDiv.firstChild);
    }

    outputDiv.scrollTop = outputDiv.scrollHeight;
}

configurarLoggerStorage(adicionarLinhaTerminal);

async function processarLinhas(linhas: string[], estado?: EstadoJogo): Promise<void> {
    for (const linha of linhas) {
        await novaLinha(linha, terminalSection);
        if (terminalSection) terminalSection.scrollTop = terminalSection.scrollHeight;
    }
    atualizarSidebar(estado);
}

function novaLinha(linhaBruta: string, terminalEl: HTMLElement | null): Promise<void> {
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
            if (terminalEl) terminalEl.scrollTop = terminalEl.scrollHeight;
            resolve();
        } else if (linha.includes("@@PANIC@@")) {
            document.body.classList.add("kernel-panic-mode");
            reproduzirBeep('erro');

            const imgPanic = document.createElement("img");
            imgPanic.src = "images/image.png";
            imgPanic.className = "panic-img-overlay";
            imgPanic.id = "panic-img-id";

            terminalSection?.appendChild(imgPanic);
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
            digitarTextoAnimadoHTML(texto, cor, velocidadeFinal, resolve);
        } else {
            const velocidadeFinal = prefMultiplicadorVelocidade === 0 ? 0 : 15 * prefMultiplicadorVelocidade;
            digitarTextoAnimadoHTML(linha, "", velocidadeFinal, resolve);
        }
    });
}

function digitarTextoAnimadoHTML(
    htmlStringBruto: string,
    classeCor: CorTerminal,
    velocidade: number,
    aoTerminar: () => void
): void {
    if (!outputDiv) {
        aoTerminar();
        return;
    }

    const htmlString = sanitizarHTMLServidor(htmlStringBruto);

    const p = document.createElement('p');
    if (classeCor) p.className = classeCor;

    const srSpan = document.createElement('span');
    srSpan.className = 'sr-only';

    let a11yPrefix = "";
    if (classeCor === 'vermelho') {
        a11yPrefix = "Perigo: ";
        document.body.classList.add('glitch-active');
        setTimeout(() => document.body.classList.remove('glitch-active'), 250);
    } else if (classeCor === 'amarelo') {
        a11yPrefix = "Atenção: ";
    }

    const textoLimpo = htmlString.replace(/<[^>]*>?/gm, '');
    srSpan.innerText = a11yPrefix + textoLimpo;

    const visualSpan = document.createElement('span');
    visualSpan.setAttribute('aria-hidden', 'true');

    p.appendChild(srSpan);
    p.appendChild(visualSpan);
    outputDiv.appendChild(p);

    if (velocidade === 0) {
        visualSpan.innerHTML = htmlString;
        if (terminal) terminal.scrollTop = terminal.scrollHeight;
        aoTerminar();
        return;
    }

    let i = 0;
    let isTag = false;
    let currentHTML = "";
    let ultimaVez = 0;

    function digitar(timestamp: number): void {
        if (i >= htmlString.length) {
            aoTerminar();
            return;
        }

        if (!ultimaVez) ultimaVez = timestamp;
        const progresso = timestamp - ultimaVez;
        const isProcessingTag = isTag || htmlString.charAt(i) === '<';

        if (isProcessingTag || progresso >= velocidade) {
            const char = htmlString.charAt(i);
            currentHTML += char;
            visualSpan.innerHTML = currentHTML;
            i++;
            if (terminal) terminal.scrollTop = terminal.scrollHeight;

            if (char === '<') isTag = true;
            if (char === '>') isTag = false;

            if (isTag || (i < htmlString.length && htmlString.charAt(i) === '<')) {
                digitar(timestamp);
            } else {
                if (char !== ' ' && char !== '\n') {
                    tocarSomDigito();
                }
                ultimaVez = timestamp;
                requestAnimationFrame(digitar);
            }
        } else {
            requestAnimationFrame(digitar);
        }
    }

    requestAnimationFrame(digitar);
}

// 5. Comunicação com o servidor (fetch)

function mostrarSalvando(): void {
    const ind = getEl<HTMLDivElement>('save-indicator');
    if (ind) {
        ind.classList.remove('hidden');
        setTimeout(() => ind.classList.add('hidden'), 1500);
    }
}

async function fetchSeguro(url: string, options: RequestInit = {}): Promise<void> {
    if (inputField) inputField.disabled = true;
    if (inputLineDiv) inputLineDiv.style.display = 'none';
    if (loadingSpinner) loadingSpinner.style.display = 'flex';

    const startTime = Date.now();

    try {
        options.credentials = 'include';
        const res = await fetch(`${API_URL}${url}`, options);
        if (!res.ok) throw new Error("Servidor offline");
        const data = (await res.json()) as RespostaServidor;

        const tempoDecorrido = Date.now() - startTime;
        if (tempoDecorrido < 300) {
            await sleep(300 - tempoDecorrido);
        }

        if (loadingSpinner) loadingSpinner.style.display = 'none';

        await processarLinhas(data.linhas, data.estado);

        if (data.novas_conquistas) {
            processarConquistas(data.novas_conquistas);
        }

        if (data.estado?.estado_jogo === "FIM" && (data.estado.tempo_final ?? 0) > 0) {
            const tempoFinal = data.estado.tempo_final!;
            const finalAlcancado = data.estado.final_alcancado;
            setTimeout(() => {
                mostrarTelaDePontuacao(tempoFinal, finalAlcancado);
            }, 5000);
        }

        if (url === '/comando') {
            mostrarSalvando();
        }
    } catch (erro) {
        console.error("Erro na comunicação:", erro);
        if (loadingSpinner) loadingSpinner.style.display = 'none';
        if (outputDiv) {
            const p = document.createElement('p');
            p.className = 'vermelho';
            p.textContent = "[ERRO DE CONEXÃO] O sinal com o servidor falhou. Verifique sua internet.";
            outputDiv.appendChild(p);
        }
        if (terminal) terminal.scrollTop = terminal.scrollHeight;
    } finally {
        if (inputLineDiv) inputLineDiv.style.display = 'flex';
        if (inputField) {
            inputField.disabled = false;
            inputField.focus();
        }
    }
}

function iniciarJogo(): void {
    fetchSeguro('/iniciar', { method: 'POST' });
}

async function enviarComando(comando: string): Promise<void> {
    await fetchSeguro('/comando', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            comando,
            telemetria: prefTelemetria
        })
    });
}

function executarAtalho(cmd: string): void {
    if (!inputField) return;
    inputField.value = cmd;
    inputField.focus();
    enviarComando(cmd);
}

// 6. TV Retro, Replay e Leaderboard

function ligarTV(): void {
    const powerBtn = getEl<HTMLButtonElement>("power-button");
    const powerLed = getEl<HTMLDivElement>("power-led");
    if (powerBtn) powerBtn.disabled = true;
    powerLed?.classList.add("led-on");

    iniciarSomAmbiente();
    reproduzirBeep('erro');

    const layerStatic = getEl<HTMLDivElement>("layer-static");
    const layerColorBars = getEl<HTMLDivElement>("layer-color-bars");
    const layerBlue = getEl<HTMLDivElement>("layer-blue");
    const layerGame = getEl<HTMLDivElement>("layer-game");
    const tvFrame = getEl<HTMLDivElement>("tv-frame");
    const tvOverlay = getEl<HTMLImageElement>("tv-overlay");
    const contentArea = getEl<HTMLDivElement>("tv-content-area");

    layerStatic?.classList.add("static-active");

    setTimeout(() => {
        layerStatic?.classList.remove("static-active");
        layerStatic?.classList.add("hidden");
        layerColorBars?.classList.remove("hidden");

        const ctx = obterAudioContext();
        if (ctx) {
            window.colorBarOsc = ctx.createOscillator();
            window.colorBarOsc.type = 'sine';
            window.colorBarOsc.frequency.value = 1000;
            window.colorBarOsc.connect(ctx.destination);
            window.colorBarOsc.start();
        }
    }, 1500);

    setTimeout(() => {
        layerColorBars?.classList.add("hidden");
        layerBlue?.classList.remove("hidden");
        window.colorBarOsc?.stop();
    }, 3000);

    setTimeout(() => {
        layerBlue?.classList.add("hidden");
        layerGame?.classList.remove("hidden");

        tvFrame?.classList.add("zoom-into-tv");
        tvOverlay?.classList.add("fade-out-tv");
        contentArea?.classList.add("expand-screen");
    }, 4500);

    setTimeout(() => {
        const tvRoom = getEl<HTMLDivElement>("tv-room");
        const containerJogo = document.querySelector<HTMLDivElement>(".container");

        if (containerJogo) document.body.appendChild(containerJogo);
        tvRoom?.remove();

        iniciarJogo();
    }, 6000);
}

async function iniciarReplay(idReplay: string): Promise<void> {
    if (modoReplayAtivo) return;

    try {
        const tokenAdmin = sessionStorage.getItem('vilasBoasAdminToken') || '';
        const headersReplay: Record<string, string> = {};
        if (tokenAdmin) {
            headersReplay['X-Admin-Token'] = tokenAdmin;
        }

        const response = await fetch(`${API_URL}/admin/replay/${encodeURIComponent(idReplay)}`, {
            method: 'GET',
            credentials: 'include',
            headers: headersReplay
        });
        const data = (await response.json()) as DadosReplay;

        if (!response.ok) {
            adicionarLinhaTerminal(`[ SISTEMA ] ${data.erro || 'Acesso negado ao replay.'}`, "vermelho");
            return;
        }

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

            const resPost = await fetch(`${API_URL}/comando`, {
                method: 'POST',
                credentials: 'include',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ comando, telemetria: false })
            });

            const resData = (await resPost.json()) as RespostaServidor;

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
        adicionarLinhaTerminal("[ SISTEMA ] Erro de conexão ao buscar a fita.", "vermelho");
        modoReplayAtivo = false;
    }
}

function mostrarTelaDePontuacao(segundosTotais: number, nomeDoFinal?: string): void {
    const modal = getEl<HTMLDivElement>('arcade-score-modal');
    const display = getEl<HTMLHeadingElement>('score-time-display');
    const textoFinal = getEl<HTMLParagraphElement>('score-ending-text');

    const mins = Math.floor(segundosTotais / 60);
    const secs = Math.floor(segundosTotais % 60);
    const ms = Math.floor((segundosTotais % 1) * 100);

    if (display) {
        display.innerText =
            `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}:${String(ms).padStart(2, '0')}`;
    }

    if (textoFinal && nomeDoFinal) {
        textoFinal.innerText = nomeDoFinal.replace(/_/g, ' ').toUpperCase();
    }

    if (modal) {
        modal.classList.remove('hidden');
        getEl<HTMLInputElement>('arcade-initials')?.focus();
    }
}

function fecharArcadeModal(): void {
    getEl<HTMLDivElement>('arcade-score-modal')?.classList.add('hidden');
    window.location.reload();
}

function iniciarLoginESalvar(): void {
    const iniciaisInput = getEl<HTMLInputElement>('arcade-initials');
    const iniciais = iniciaisInput?.value || "";
    if (iniciais.length < 3) {
        alert("Por favor, digite 3 letras para o painel do fliperama!");
        return;
    }

    const btn = getEl<HTMLButtonElement>('btn-login-submit');
    if (btn) {
        btn.innerText = "CONECTANDO...";
        btn.disabled = true;
    }

    window.location.href = `${API_URL}/login/google?iniciais=${encodeURIComponent(iniciais)}`;
}

// 7. Eventos de teclado, clique e boot

document.body.addEventListener('click', iniciarSomAmbiente, { once: true });
document.body.addEventListener('keydown', iniciarSomAmbiente, { once: true });

window.addEventListener('click', (e: MouseEvent) => {
    const modal = getEl<HTMLDivElement>('achievements-modal');
    if (e.target === modal) {
        fecharModalConquistas();
    }
});

terminalSection?.addEventListener('click', (event: MouseEvent) => {
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
                outputDiv.appendChild(p);
            }

            inputField.value = "";
            if (terminal) terminal.scrollTop = terminal.scrollHeight;

            if (comando.toLowerCase() === "ajuda") {
                openHelp();
                if (outputDiv) {
                    const pResp = document.createElement("p");
                    pResp.className = "verde";
                    pResp.textContent = "[SISTEMA] Abrindo o Manual de Sobrevivência...";
                    outputDiv.appendChild(pResp);
                }
                if (terminal) terminal.scrollTop = terminal.scrollHeight;
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
        if (outputDiv) {
            const p = document.createElement('p');
            p.className = 'verde';
            p.textContent = "[SISTEMA] O progresso do jogo é salvo automaticamente quando o estado muda.";
            outputDiv.appendChild(p);
        }
        if (terminal) terminal.scrollTop = terminal.scrollHeight;
        reproduzirBeep('sucesso');
    }

    if (e.key === '?' && document.activeElement !== inputField) {
        e.preventDefault();
        openHelp();
    }
});

window.addEventListener('load', () => {
    const urlParams = new URLSearchParams(window.location.search);
    const idReplay = urlParams.get('replay');

    if (idReplay) {
        iniciarReplay(idReplay);
    } else if (!getEl("tv-room")) {
        iniciarJogo();
    }

    carregarPreferencias();

    if (urlParams.get("leaderboard") === "sucesso") {
        reproduzirBeep('sucesso');
        alert("Sua pontuação foi registrada no Quadro de Líderes Global!");
        window.history.replaceState({}, document.title, "/");
    }
});

window.openHelp = openHelp;
window.closeHelp = closeHelp;
window.openSettings = openSettings;
window.closeSettings = closeSettings;
window.openSaves = openSaves;
window.closeSaves = closeSaves;
window.abrirModalConquistas = abrirModalConquistas;
window.fecharModalConquistas = fecharModalConquistas;
window.exportarSave = exportarSave;
window.importarSave = importarSave;
window.gerarLinkCompartilhamentoUI = gerarLinkCompartilhamentoUI;
window.executarAtalho = executarAtalho;
window.ligarTV = ligarTV;
window.iniciarLoginESalvar = iniciarLoginESalvar;
window.fecharArcadeModal = fecharArcadeModal;
window.playBip = playBip;
