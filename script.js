const outputDiv = document.getElementById('output');
const inputField = document.getElementById('comando');

let historicoComandos = [];
let posicaoHistorico = -1;
let comandoDigitadoAtual = "";
let pref_telemetria = true;

const terminal = document.getElementById('terminal');
const loadingSpinner = document.getElementById('loading');
const inputLineDiv = document.querySelector('.input-line'); 

const hpEl = document.getElementById('hud-hp');
const luzEl = document.getElementById('hud-luz');
const invEl = document.getElementById('hud-inv');
const salaEl = document.getElementById('hud-sala');
const saidasEl = document.getElementById('hud-saidas');

// ==========================================
// GERENCIADOR DE ÁUDIO E ZUMBIDO CRT
// ==========================================
let audioCtx = null;
let ambientOsc = null;
let crtOsc = null;

function obterAudioContext() {
    if (!audioCtx) {
        const AudioContextClass = window.AudioContext || window.webkitAudioContext;
        if (AudioContextClass) {
            audioCtx = new AudioContextClass();
        }
    }
    if (audioCtx && audioCtx.state === 'suspended') {
        audioCtx.resume();
    }
    return audioCtx;
}

function iniciarSomAmbiente() {
    const ctx = obterAudioContext();
    if (!ctx || ambientOsc) return;

    // 1. Drone Grave de Fundo (Ambiente abafado de $55\text{ Hz}$)
    ambientOsc = ctx.createOscillator();
    const ambientGain = ctx.createGain();
    ambientOsc.type = 'triangle';
    ambientOsc.frequency.value = 55; 
    ambientGain.gain.value = 0.025; 

    ambientOsc.connect(ambientGain);
    ambientGain.connect(ctx.destination);
    ambientOsc.start();

    // 2. Zumbido de Monitor CRT (Chiado elétrico de $60\text{ Hz}$)
    crtOsc = ctx.createOscillator();
    const crtGain = ctx.createGain();
    crtOsc.type = 'sawtooth';
    crtOsc.frequency.value = 60; 
    crtGain.gain.value = 0.005; 

    crtOsc.connect(crtGain);
    crtGain.connect(ctx.destination);
    crtOsc.start();
}

document.body.addEventListener('click', iniciarSomAmbiente, { once: true });
document.body.addEventListener('keydown', iniciarSomAmbiente, { once: true });

// ==========================================
// SINTETIZADORES DE EFEITOS SONOROS
// ==========================================

// 1. Som de caractere sendo impresso na tela
function tocarSomDigito() {
    const ctx = obterAudioContext();
    if (!ctx) return;

    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'square'; 
    osc.frequency.setValueAtTime(400 + Math.random() * 150, ctx.currentTime);

    gain.gain.setValueAtTime(0.015, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.04);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.05);
}

// 2. Bip de Confirmação de Comando (Enter)
function tocarBipEntrada() {
    const ctx = obterAudioContext();
    if (!ctx) return;

    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.type = 'square'; 
    osc.frequency.setValueAtTime(550, ctx.currentTime);
    osc.frequency.exponentialRampToValueAtTime(1100, ctx.currentTime + 0.05);

    gain.gain.setValueAtTime(0.035, ctx.currentTime);
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.06);

    osc.connect(gain);
    gain.connect(ctx.destination);

    osc.start();
    osc.stop(ctx.currentTime + 0.06);
}

// 3. Passos Metálicos Pesados
function tocarPassoMetalico() {
    const ctx = obterAudioContext();
    if (!ctx) return;

    const t = ctx.currentTime;

    // Impacto Sub-grave do peso
    const subOsc = ctx.createOscillator();
    const subGain = ctx.createGain();
    subOsc.type = 'sine';
    subOsc.frequency.setValueAtTime(110, t);
    subOsc.frequency.exponentialRampToValueAtTime(30, t + 0.25);

    subGain.gain.setValueAtTime(0.12, t);
    subGain.gain.exponentialRampToValueAtTime(0.001, t + 0.25);

    subOsc.connect(subGain);
    subGain.connect(ctx.destination);
    subOsc.start(t);
    subOsc.stop(t + 0.25);

    // Rangido e fricção do metal
    const metalOsc = ctx.createOscillator();
    const metalGain = ctx.createGain();
    metalOsc.type = 'sawtooth';
    metalOsc.frequency.setValueAtTime(420 + Math.random() * 80, t);
    metalOsc.frequency.exponentialRampToValueAtTime(160, t + 0.18);

    metalGain.gain.setValueAtTime(0.045, t);
    metalGain.gain.exponentialRampToValueAtTime(0.0001, t + 0.18);

    metalOsc.connect(metalGain);
    metalGain.connect(ctx.destination);
    metalOsc.start(t);
    metalOsc.stop(t + 0.18);
}

// Beep genérico de erro/sucesso
function reproduzirBeep(tipo = 'sucesso') {
    const ctx = obterAudioContext();
    if (!ctx) return;

    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.connect(gain);
    gain.connect(ctx.destination);

    if (tipo === 'erro') {
        osc.type = 'sawtooth'; 
        osc.frequency.setValueAtTime(150, ctx.currentTime); 
        gain.gain.setValueAtTime(0.08, ctx.currentTime);
        osc.start();
        osc.stop(ctx.currentTime + 0.3);
    } else {
        osc.type = 'square'; 
        osc.frequency.setValueAtTime(800, ctx.currentTime); 
        gain.gain.setValueAtTime(0.03, ctx.currentTime);
        osc.start();
        osc.stop(ctx.currentTime + 0.1);
    }
}

function playBip(tipo) {
    reproduzirBeep(tipo);
}

// ==========================================
// GERENCIAMENTO DE SAVES (UI)
// ==========================================
function openSaves() {
    document.getElementById('saves-modal').classList.remove('hidden');
}

function closeSaves() {
    document.getElementById('saves-modal').classList.add('hidden');
    document.getElementById('comando').focus();
}

async function exportarSave() {
    try {
        const res = await fetch('/save/export');
        if (!res.ok) throw new Error("Nenhum progresso encontrado no servidor.");
        const data = await res.json();
        
        // Cria um Blob JSON e força o Download no navegador do usuário
        const blob = new Blob([JSON.stringify(data, null, 4)], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `vilas_boas_backup_${new Date().toISOString().slice(0,10)}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
        
        reproduzirBeep('sucesso');
    } catch (erro) {
        console.error(erro);
        alert("[ERRO] " + erro.message);
    }
}

async function importarSave(event) {
    const file = event.target.files[0];
    if (!file) return;
    
    const reader = new FileReader();
    reader.onload = async function(e) {
        try {
            const dados = JSON.parse(e.target.result);
            
            // Envia o JSON do usuário para o endpoint que acabamos de criar
            const res = await fetch('/save/import', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(dados)
            });
            
            const result = await res.json();
            if (res.ok) {
                alert("Save corrompido... Digo, importado com sucesso! Reiniciando terminal...");
                window.location.reload();
            } else {
                alert("[ERRO DE BIOS] " + result.erro);
            }
        } catch (erro) {
            alert("[ERRO FATAL] O arquivo fornecido não é um JSON válido do sistema Vilas Boas.");
        }
    };
    reader.readAsText(file);
    event.target.value = ''; // Limpa o input para permitir re-upload do mesmo arquivo se necessário
}

async function gerarLinkCompartilhamentoUI() {
    try {
        const res = await fetch('/share/generate');
        const data = await res.json();
        
        if (res.ok) {
            const container = document.getElementById('share-link-container');
            const input = document.getElementById('share-link-input');
            const msg = document.getElementById('share-link-msg');
            
            container.classList.remove('hidden');
            input.value = data.link;
            msg.innerText = data.mensagem;
            
            // Copia automaticamente para a área de transferência
            input.select();
            document.execCommand('copy');
            reproduzirBeep('sucesso');
            alert("Link copiado para a área de transferência!");
        } else {
            alert("[ERRO] " + (data.erro || "Falha ao gerar link."));
        }
    } catch (erro) {
        console.error(erro);
        alert("Falha de conexão com os servidores centrais.");
    }
}

// Lembre-se de adicionar closeSaves() na sua escuta da tecla "Escape" já existente:
document.addEventListener('keydown', function(event) {
    if (event.key === "Escape") {
        closeHelp();
        if(typeof closeSettings === 'function') closeSettings();
        closeSaves();
    }
});

// ==========================================
// PREFERÊNCIAS DO JOGADOR
// ==========================================
let pref_multiplicadorVelocidade = 1.0;

function carregarPreferencias() {

    const teleCheckbox = document.getElementById('telemetry-checkbox');
    const savedTele = localStorage.getItem('vilasBoasTelemetry');
    const savedSpeed = localStorage.getItem('vilasBoasSpeed');
    const savedCRT = localStorage.getItem('vilasBoasCRT');

    if (savedTele !== null) {
    pref_telemetria = (savedTele === 'true');
    teleCheckbox.checked = pref_telemetria;
    }

    teleCheckbox.addEventListener('change', (e) => {
        pref_telemetria = e.target.checked;
        localStorage.setItem('vilasBoasTelemetry', pref_telemetria);
    });
    
    if (savedSpeed !== null) {
        pref_multiplicadorVelocidade = parseFloat(savedSpeed);
        document.getElementById('speed-slider').value = pref_multiplicadorVelocidade;
        atualizarLabelVelocidade(pref_multiplicadorVelocidade);
    }
    
    const slider = document.getElementById('speed-slider');
    slider.addEventListener('input', (e) => {
        pref_multiplicadorVelocidade = parseFloat(e.target.value);
        localStorage.setItem('vilasBoasSpeed', pref_multiplicadorVelocidade);
        atualizarLabelVelocidade(pref_multiplicadorVelocidade);
    });

    const crtCheckbox = document.getElementById('crt-checkbox');
    if (savedCRT !== null) {
        crtCheckbox.checked = (savedCRT === 'true');
    }
    aplicarEfeitoCRT(crtCheckbox.checked);
    
    crtCheckbox.addEventListener('change', (e) => {
        const ativado = e.target.checked;
        localStorage.setItem('vilasBoasCRT', ativado);
        aplicarEfeitoCRT(ativado);
    });
}

function atualizarLabelVelocidade(val) {
    const label = document.getElementById('speed-val-display');
    if(val === 0) label.innerText = "Instantâneo (0x)";
    else label.innerText = `Atual (${val}x)`;
}

function aplicarEfeitoCRT(ativado) {
    if (ativado) document.body.classList.add('crt-enabled');
    else document.body.classList.remove('crt-enabled');
}

function openSettings() {
    document.getElementById('settings-modal').classList.remove('hidden');
}

function closeSettings() {
    document.getElementById('settings-modal').classList.add('hidden');
    document.getElementById('comando').focus();
}

// Escutar "Escape" para fechar também as configurações
document.addEventListener('keydown', function(event) {
    if (event.key === "Escape") {
        closeHelp();
        closeSettings();
    }
});

// Inicializar as preferências ao carregar a janela
const onloadOriginal = window.onload;
window.onload = function() {
    carregarPreferencias();
    if(onloadOriginal) onloadOriginal();
};

// ==========================================
// FOCO INTELIGENTE E CONTROLES
// ==========================================
const terminalSection = document.querySelector('.terminal-section');


terminalSection.addEventListener('click', (event) => {
    const inputTerminal = document.getElementById('comando');
    const textoSelecionado = window.getSelection().toString();
    const clicouNoBotao = event.target.closest('button');
    const clicouNoInput = event.target.closest('.input-line');
    
    
    if (!textoSelecionado && inputTerminal && !clicouNoBotao) {
        
        const isMobile = window.matchMedia("(max-width: 768px)").matches;
        if (!isMobile || clicouNoInput) {
            inputTerminal.focus();
        }
    }
});

// ==========================================
// EVENTOS DE TECLADO (INPUT)
// ==========================================
inputField.addEventListener("keydown", async function(event) {
    if (event.key === "Enter") {
        const comandoBruto = inputField.value;
        const comando = comandoBruto.trim();
        
        if (comando !== "") {
            tocarBipEntrada(); // Toca o Bip ao apertar Enter

            if (historicoComandos[historicoComandos.length - 1] !== comando) {
                historicoComandos.push(comando);
            }
            posicaoHistorico = historicoComandos.length; 
            
            const p = document.createElement("p");
            p.className = "branco";
            p.innerHTML = `<span class="prompt">C:\\></span> ${comandoBruto}`;
            outputDiv.appendChild(p);
            
            inputField.value = "";
            terminal.scrollTop = terminal.scrollHeight;
            
            await enviarComando(comando);
        }
    } 
    else if (event.key === "ArrowUp") {
        event.preventDefault(); 
        if (posicaoHistorico === historicoComandos.length) {
            comandoDigitadoAtual = inputField.value; 
        }
        if (posicaoHistorico > 0) {
            posicaoHistorico--;
            inputField.value = historicoComandos[posicaoHistorico];
        }
    } 
    else if (event.key === "ArrowDown") {
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

// ==========================================
// ATUALIZAÇÃO DO HUD VISUAL
// ==========================================
function atualizarSidebar(estado) {
    if (!estado) return;

    const hpVal = document.getElementById("hp-val");
    if (hpVal) {

        if (estado.hp === "∞") {
            hpVal.textContent = "[ GOD MODE ]";
            hpVal.className = "amarelo";
            document.body.classList.remove("hp-critico");

        } else {
            const hpAtual = parseInt(estado.hp) || 0;
            const maxHp = 3;
            const blocosCheios = "█".repeat(hpAtual);
            const blocosVazios = "░".repeat(Math.max(0, maxHp - hpAtual));
            
            hpVal.textContent = `[${blocosCheios}${blocosVazios}]`;
            hpVal.className = (hpAtual <= 1) ? "vermelho" : "verde";


            if (hpAtual <= 1 && hpAtual > 0) {
                document.body.classList.add("hp-critico");
            } else {
                document.body.classList.remove("hp-critico");
            }
        }
    }

    const luzVal = document.getElementById("luz-val");
    if (luzVal) {
        luzVal.textContent = estado.luz !== undefined ? estado.luz : "??";
        luzVal.className = (estado.luz === "∞" || estado.luz > 3) ? "verde" : "vermelho";
    }

    const invList = document.getElementById("inv-list");
    const invTitulo = document.querySelector("#hud-inv");
    
    if (invList) {
        invList.innerHTML = "";
        
        let qtdBolsas = 0;
        if (estado.inventario) {
            qtdBolsas = estado.inventario.filter(item => item === "bolsa").length;
        }
        
        const limiteMaximo = 3 + (qtdBolsas * 3);
        const qtdAtual = estado.inventario ? estado.inventario.length : 0;
        
        if (invTitulo) {
            invTitulo.textContent = `INV (${qtdAtual}/${limiteMaximo}):`;
        }

        if (qtdAtual > 0) {
            estado.inventario.forEach(item => {
                let li = document.createElement("li");
                li.textContent = `- ${item}`;
                li.className = "branco";
                invList.appendChild(li);
            });
        } else {
            let li = document.createElement("li");
            li.textContent = "Vazio";
            li.className = "amarelo";
            invList.appendChild(li);
        }
    }
}

// ==========================================
// PROCESSAMENTO E ANIMAÇÃO DE TEXTO
// ==========================================
async function processarLinhas(linhas, estado) {
    const terminalEl = document.querySelector('.terminal-section'); 

    for (let linha of linhas) {
        await novaLinha(linha, terminalEl); 
        if (terminalEl) terminalEl.scrollTop = terminalEl.scrollHeight;
    }
    atualizarSidebar(estado);
}

function novaLinha(linha, terminalEl) {
    return new Promise((resolve) => {
        if (typeof linha === 'string') {
            if (linha.includes("@@JUMPSCARE@@")) {
                linha = linha.replace("@@JUMPSCARE@@", ""); 
                triggerJumpscare(); 
            }
            if (linha.includes("@@PASSO@@")) {
                linha = linha.replace("@@PASSO@@", "");
                tocarPassoMetalico();
            }
        }

        if (linha.startsWith("@@CLEAR@@")) {
            outputDiv.innerHTML = "";
            if (terminalEl) terminalEl.scrollTop = terminalEl.scrollHeight;
            resolve();

        } else if (linha.startsWith("@@EXIT@@")) {
            document.body.innerHTML = ""; 
            document.body.style.backgroundColor = "#000";
            resolve();
            
        } else if (linha.startsWith("@@RELOAD@@")) {
            // força o f5
            window.location.reload();
            resolve();

        } else if (linha.startsWith("@@PAUSE@@")) {
            let ms = parseInt(linha.split("@@")[2]) / 3;
            setTimeout(resolve, ms);

        } else if (linha.startsWith("@@TYPE@@")) {
            let parts = linha.split("@@");
            let cor = parts[2];
            let ms = parseInt(parts[3]);
            let texto = parts.slice(4).join("@@"); 
            
            // APLICA O MULTIPLICADOR DO USUÁRIO
            let velocidadeFinal = ms * pref_multiplicadorVelocidade;
            if (pref_multiplicadorVelocidade === 0) velocidadeFinal = 0;

            digitarTextoAnimadoHTML(texto, cor, velocidadeFinal, resolve);

        } else {
            // FALLBACK: APLICA MULTIPLICADOR NO TEXTO PADRÃO
            let velocidadeFinal = 15 * pref_multiplicadorVelocidade;
            if (pref_multiplicadorVelocidade === 0) velocidadeFinal = 0;
            
            digitarTextoAnimadoHTML(linha, "", velocidadeFinal, resolve);
        }
    });
}

function digitarTextoAnimadoHTML(htmlString, classeCor, velocidade, aoTerminar) {
    const p = document.createElement('p');
    if (classeCor) p.className = classeCor;
    
    // --- LÓGICA DE ACESSIBILIDADE (Leitor de Tela) ---
    // 1. Cria um span invisível onde o texto entra de uma vez só
    const srSpan = document.createElement('span');
    srSpan.className = 'sr-only';
    
    let a11yPrefix = "";
    if (classeCor === 'vermelho') {
        a11yPrefix = "Perigo: ";
        document.body.classList.add('glitch-active');
        setTimeout(() => document.body.classList.remove('glitch-active'), 250);
    }
    if (classeCor === 'amarelo') a11yPrefix = "Atenção: ";
    
    // Removemos tags HTML para o leitor ler apenas texto puro
    const textoLimpo = htmlString.replace(/<[^>]*>?/gm, '');
    srSpan.innerText = a11yPrefix + textoLimpo;

    // 2. Cria o span visual onde a animação vai acontecer (invisível pro Leitor de Tela)
    const visualSpan = document.createElement('span');
    visualSpan.setAttribute('aria-hidden', 'true');

    // Injeta os dois spans no parágrafo
    p.appendChild(srSpan);
    p.appendChild(visualSpan);
    outputDiv.appendChild(p);
    
    // Se a velocidade for 0 (Fast Mode), preenche e sai
    if (velocidade === 0) {
        visualSpan.innerHTML = htmlString;
        terminal.scrollTop = terminal.scrollHeight;
        aoTerminar();
        return;
    }
    
    // --- LÓGICA DE ANIMAÇÃO VISUAL ---
    let i = 0;
    let isTag = false;
    let currentHTML = "";
    
    function digitar() {
        if (i < htmlString.length) {
            let char = htmlString.charAt(i);
            currentHTML += char;
            
            // Injeta a animação APENAS no span visual
            visualSpan.innerHTML = currentHTML;
            i++;
            
            terminal.scrollTop = terminal.scrollHeight;
            
            if (char === '<') isTag = true;
            if (char === '>') isTag = false;
            
            if (isTag || (i < htmlString.length && htmlString.charAt(i) === '<')) {
                digitar(); 
            } else {
                if (char !== ' ' && char !== '\n') {
                    tocarSomDigito(); 
                }
                setTimeout(digitar, velocidade);
            }
        } else {
            aoTerminar(); 
        }
    }
    digitar();
}

// ==========================================
// COMUNICAÇÃO COM O SERVIDOR (API)
// ==========================================
async function fetchSeguro(url, options) {
    inputField.disabled = true;
    inputLineDiv.style.display = 'none'; 
    loadingSpinner.style.display = 'flex';
    
    const startTime = Date.now(); 
    
    try {
        const res = await fetch(url, options);
        if (!res.ok) throw new Error("Servidor offline");
        const data = await res.json();

        const tempoDecorrido = Date.now() - startTime;
        if (tempoDecorrido < 300) {
            await new Promise(resolve => setTimeout(resolve, 300 - tempoDecorrido));
        }
        
        loadingSpinner.style.display = 'none';
        await processarLinhas(data.linhas, data.estado);

        if (url === '/comando') { 
            mostrarSalvando(); 
        }

    } catch (erro) {
        console.error("Erro na comunicação:", erro);
        loadingSpinner.style.display = 'none';
        let p = document.createElement('p');
        p.className = 'vermelho';
        p.innerHTML = "[ERRO DE CONEXÃO] O sinal com o servidor falhou. Verifique sua internet.";
        outputDiv.appendChild(p);
        terminal.scrollTop = terminal.scrollHeight;
    } finally {
        inputLineDiv.style.display = 'flex'; 
        inputField.disabled = false;
        inputField.focus();
    }
}

function iniciarJogo() {
    fetchSeguro('/iniciar', { method: 'GET' });
}

async function enviarComando(comando) {
    fetchSeguro('/comando', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        // NOVO: Adicionamos o campo telemetria no payload
        body: JSON.stringify({ 
            comando: comando, 
            telemetria: pref_telemetria 
        })
    });
}

window.onload = iniciarJogo;

// ==========================================
// UTILITÁRIOS E ATALHOS
// ==========================================
function openHelp() {
    document.getElementById('help-modal').classList.remove('hidden');
}

function closeHelp() {
    document.getElementById('help-modal').classList.add('hidden');
    document.getElementById('comando').focus(); 
}

document.addEventListener('keydown', function(event) {
    if (event.key === "Escape") {
        closeHelp();
    }
});

document.addEventListener('keydown', function (e) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
        e.preventDefault();
        outputDiv.innerHTML = '';
        reproduzirBeep('sucesso');
    }
    
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
        e.preventDefault();
        let p = document.createElement('p');
        p.className = 'verde';
        p.innerHTML = "[SISTEMA] O progresso do jogo é salvo automaticamente a cada turno.";
        outputDiv.appendChild(p);
        terminal.scrollTop = terminal.scrollHeight;
        reproduzirBeep('sucesso');
    }

    if (e.key === '?' && document.activeElement !== inputField) {
        e.preventDefault();
        openHelp(); 
    }
});

function executarAtalho(cmd) {
    const input = document.getElementById('comando');
    input.value = cmd;
    input.focus();
    if (typeof enviarComando === "function") enviarComando(cmd);
}

function triggerJumpscare() {
    const overlay = document.getElementById('jumpscare-overlay');
    if (overlay) overlay.classList.remove('hidden');

    document.body.classList.add('glitch-active');
    setTimeout(() => document.body.classList.remove('glitch-active'), 300);
    
    const ctx = obterAudioContext();
    if (ctx) {
        const scareOsc = ctx.createOscillator();
        const scareGain = ctx.createGain();
        scareOsc.type = 'sawtooth';
        scareOsc.frequency.value = 130;
        scareGain.gain.value = 0.6; 
        scareOsc.connect(scareGain);
        scareGain.connect(ctx.destination);
        scareOsc.start();
        scareOsc.stop(ctx.currentTime + 0.15); 
    }
    
    if (overlay) setTimeout(() => overlay.classList.add('hidden'), 150); 
}

function mostrarSalvando() {
    const ind = document.getElementById('save-indicator');
    if (ind) {
        ind.classList.remove('hidden');
        setTimeout(() => ind.classList.add('hidden'), 1500);
    }
}

function fazerNada() {
    
}

