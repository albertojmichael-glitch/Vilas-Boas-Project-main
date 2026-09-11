const outputDiv = document.getElementById('output');
const inputField = document.getElementById('comando');

let historicoComandos = [];
let posicaoHistorico = -1;
let comandoDigitadoAtual = "";
let pref_telemetria = true;
let modoReplayAtivo = false;

const terminal = document.getElementById('terminal');
const loadingSpinner = document.getElementById('loading');
const inputLineDiv = document.querySelector('.input-line');
const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

const hpEl = document.getElementById('hud-hp');
const luzEl = document.getElementById('hud-luz');
const invEl = document.getElementById('hud-inv');
const salaEl = document.getElementById('hud-sala');
const saidasEl = document.getElementById('hud-saidas');


let audioCtx = null;
let masterGainNode = null; // Controlador global
let ambientOsc = null;
let crtOsc = null;
let pref_volume = 1.0; // Volume padrão

function obterAudioContext() {
    if (!audioCtx) {
        const AudioContextClass = window.AudioContext || window.webkitAudioContext;
        if (AudioContextClass) {
            audioCtx = new AudioContextClass();
            
            // Cria o nó de volume mestre e o conecta na saída final (caixas de som)
            masterGainNode = audioCtx.createGain();
            masterGainNode.gain.value = pref_volume;
            masterGainNode.connect(audioCtx.destination);
        }
    }
    if (audioCtx && audioCtx.state === 'suspended') {
        audioCtx.resume();
    }
    return audioCtx;
}

// NOVO: Sistema de Legendas de Áudio
let legendaTimeout;
function mostrarLegendaDeAudio(texto) {
    const legendaDiv = document.getElementById('audio-captions');
    if (legendaDiv) {
        legendaDiv.textContent = `[ Áudio: ${texto} ]`;
        legendaDiv.classList.remove('hidden');
        
        clearTimeout(legendaTimeout);
        legendaTimeout = setTimeout(() => {
            legendaDiv.classList.add('hidden');
        }, 2000); // Some após 2 segundos
    }
}
function iniciarSomAmbiente() {
    const ctx = obterAudioContext();
    if (!ctx || ambientOsc) return;

    
    ambientOsc = ctx.createOscillator();
    const ambientGain = ctx.createGain();
    ambientOsc.type = 'triangle';
    ambientOsc.frequency.value = 55; 
    ambientGain.gain.value = 0.025; 

    ambientOsc.connect(ambientGain);
    ambientGain.connect(masterGainNode);
    ambientOsc.start();

    
    crtOsc = ctx.createOscillator();
    const crtGain = ctx.createGain();
    crtOsc.type = 'sawtooth';
    crtOsc.frequency.value = 60; 
    crtGain.gain.value = 0.005; 

    crtOsc.connect(crtGain);
    crtGain.connect(masterGainNode);
    crtOsc.start();
}

document.body.addEventListener('click', iniciarSomAmbiente, { once: true });
document.body.addEventListener('keydown', iniciarSomAmbiente, { once: true });


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
    gain.connect(masterGainNode);

    osc.start();
    osc.stop(ctx.currentTime + 0.05);
}


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
    gain.connect(masterGainNode);

    osc.start();
    osc.stop(ctx.currentTime + 0.06);
}

// O Catálogo Mestre Frontend (Deve espelhar o do backend)
const CATALOGO_CONQUISTAS = [
    { id: "primeira_morte", nome: "Sangue no Carpete", desc: "Bem-vindo ao Vilas Boas.", icone: "☠" },
    { id: "mente_brilhante", nome: "Mente Brilhante", desc: "Abra o cofre na primeira tentativa.", icone: "★" },
    { id: "labirinto", nome: "Labirinto", desc: "Sobreviva à Sala de Energia.", icone: "☄" },
    { id: "acumulador", nome: "Acumulador", desc: "Encha todos os 15 espaços do inventário.", icone: "☑" },
    { id: "glicose", nome: "Glicose Duvidosa", desc: "Coma o doce velho encontrado no chão.", icone: "✴" },
    { id: "trapaceiro", nome: "Hacker", desc: "Ative o God Mode.", icone: "☣" },
    { id: "normal_zerado", nome: "Fim do Expediente", desc: "Sobreviva à noite na dificuldade Normal.", icone: "♨" },
    { id: "final_bom", nome: "Sobrevivente", desc: "Alcance o Final Neutro.", icone: "☀" },
    { id: "final_verdadeiro", nome: "A Verdade", desc: "Alcance o Final Verdadeiro.", icone: "✦" },
    { id: "pesadelo_neutro", nome: "Pesadelo Superado", desc: "Alcance o Final Neutro no modo PESADELO.", icone: "♛" },
    { id: "pesadelo_verdadeiro", nome: "Mestre do Pesadelo", desc: "Alcance o Final Verdadeiro no modo PESADELO.", icone: "♚" }
];

function abrirModalConquistas() {
    const modal = document.getElementById('achievements-modal');
    const grid = document.getElementById('achievements-grid');
    const title = document.getElementById('achievements-title');

    
    const salvas = JSON.parse(localStorage.getItem('vilasBoasAchievements') || '{}');
    
    grid.innerHTML = ''; 
    let quantidadeDesbloqueada = 0;

    CATALOGO_CONQUISTAS.forEach(conquista => {
        const estaDesbloqueada = !!salvas[conquista.id]; 
        if (estaDesbloqueada) quantidadeDesbloqueada++;

        
        const card = document.createElement('div');
        card.className = `achievement-card ${estaDesbloqueada ? 'unlocked' : 'locked'}`;
        
        
        const nomeExibicao = estaDesbloqueada ? conquista.nome : '???';
        const descExibicao = estaDesbloqueada ? conquista.desc : 'Continue explorando para desbloquear.';

        card.innerHTML = `
            <div class="achieve-card-icon">${conquista.icone}</div>
            <div class="achieve-card-info">
                <h3>${nomeExibicao}</h3>
                <p>${descExibicao}</p>
            </div>
        `;
        grid.appendChild(card);
    });

    
    title.innerText = `♛ SUAS CONQUISTAS (${quantidadeDesbloqueada}/${CATALOGO_CONQUISTAS.length}) ♛`;

    modal.classList.remove('hidden');
}

function fecharModalConquistas() {
    document.getElementById('achievements-modal').classList.add('hidden');
}

// Fechar o modal se o jogador clicar no fundo escuro fora da janela
window.addEventListener('click', (e) => {
    const modal = document.getElementById('achievements-modal');
    if (e.target === modal) {
        fecharModalConquistas();
    }
});


function tocarPassoMetalico() {
    const ctx = obterAudioContext();
    if (!ctx) return;

    const t = ctx.currentTime;

    
    const subOsc = ctx.createOscillator();
    const subGain = ctx.createGain();
    subOsc.type = 'sine';
    subOsc.frequency.setValueAtTime(110, t);
    subOsc.frequency.exponentialRampToValueAtTime(30, t + 0.25);

    subGain.gain.setValueAtTime(0.12, t);
    subGain.gain.exponentialRampToValueAtTime(0.001, t + 0.25);

    subOsc.connect(subGain);
    subGain.connect(masterGainNode);
    subOsc.start(t);
    subOsc.stop(t + 0.25);

    
    const metalOsc = ctx.createOscillator();
    const metalGain = ctx.createGain();
    metalOsc.type = 'sawtooth';
    metalOsc.frequency.setValueAtTime(420 + Math.random() * 80, t);
    metalOsc.frequency.exponentialRampToValueAtTime(160, t + 0.18);

    metalGain.gain.setValueAtTime(0.045, t);
    metalGain.gain.exponentialRampToValueAtTime(0.0001, t + 0.18);

    metalOsc.connect(metalGain);
    metalGain.connect(masterGainNode);
    metalOsc.start(t);
    metalOsc.stop(t + 0.18);
}


function reproduzirBeep(tipo = 'sucesso') {
    const ctx = obterAudioContext();
    if (!ctx) return;

    const osc = ctx.createOscillator();
    const gain = ctx.createGain();

    osc.connect(gain);
    gain.connect(masterGainNode);

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
            
            
            if (!dados || typeof dados !== 'object' || !('sala_atual' in dados) || !('hp' in dados)) {
                throw new Error("Estrutura de save incompatível.");
            }
            
            
            const res = await fetch('/save/import', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(dados)
            });
            
            const result = await res.json();
            if (res.ok) {
                alert("Save corrompido... importado com sucesso! Reiniciando terminal...");
                window.location.reload();
            } else {
                alert("[ERRO DE BIOS] " + result.erro);
            }
        } catch (erro) {
            alert("[ERRO FATAL] O arquivo fornecido não é um JSON válido do sistema.");
        }
    };
    reader.readAsText(file);
    event.target.value = ''; 
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


document.addEventListener('keydown', function(event) {
    if (event.key === "Escape") {
        closeHelp();
        if(typeof closeSettings === 'function') closeSettings();
        closeSaves();
    }
});


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

    
    const savedVolume = localStorage.getItem('vilasBoasVolume');
    
    if (savedVolume !== null) {
        pref_volume = parseFloat(savedVolume);
        document.getElementById('volume-slider').value = pref_volume;
        document.getElementById('volume-val-display').innerText = `${Math.round(pref_volume * 100)}%`;
        if (masterGainNode) masterGainNode.gain.value = pref_volume;
    }

    const volSlider = document.getElementById('volume-slider');
    if (volSlider) {
        volSlider.addEventListener('input', (e) => {
            pref_volume = parseFloat(e.target.value);
            localStorage.setItem('vilasBoasVolume', pref_volume);
            document.getElementById('volume-val-display').innerText = `${Math.round(pref_volume * 100)}%`;
            
            
            if (masterGainNode) {
                masterGainNode.gain.value = pref_volume;
            }
        });
    }
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


document.addEventListener('keydown', function(event) {
    if (event.key === "Escape") {
        closeHelp();
        closeSettings();
    }
});


const onloadOriginal = window.onload;
window.onload = function() {
    const urlParams = new URLSearchParams(window.location.search);
    const idReplay = urlParams.get('replay');

    if (idReplay) {
        iniciarReplay(idReplay);
    } else {
        
        const tvRoom = document.getElementById("tv-room");
        if (!tvRoom) {
            iniciarJogo(); 
        }
    }

    carregarPreferencias();
    if(onloadOriginal) onloadOriginal();
};

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


inputField.addEventListener("keydown", async function(event) {
    if (event.key === "Enter") {
        const comandoBruto = inputField.value;
        const comando = comandoBruto.trim();
        
        if (comando !== "") {
            tocarBipEntrada(); 

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

let toastTimeout;

function processarConquistas(listaConquistas) {
    if (!listaConquistas || listaConquistas.length === 0) return;

    
    let salvas = JSON.parse(localStorage.getItem('vilasBoasAchievements') || '{}');
    
    listaConquistas.forEach(conquista => {
        
        if (!salvas[conquista.id]) {
            salvas[conquista.id] = conquista; 
            
            
            if (typeof reproduzirBeep === "function") reproduzirBeep('sucesso');
            
            
            mostrarToastConquista(conquista);
        }
    });

    
    localStorage.setItem('vilasBoasAchievements', JSON.stringify(salvas));
}

function mostrarToastConquista(conquista) {
    const toast = document.getElementById('achievement-toast');
    document.getElementById('achieve-icon').innerText = conquista.icone;
    document.getElementById('achieve-name').innerText = conquista.nome;
    
    toast.classList.remove('hidden');
    toast.classList.add('show');
    
    clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
        toast.classList.remove('show');
    }, 4000); // O balão some após 4 segundos
}

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
        luzVal.textContent = estado.luz_restante !== undefined ? estado.luz_restante : "??";
        luzVal.className = (estado.luz_restante === "∞" || estado.luz_restante > 3) ? "verde" : "vermelho";
    }

    const somVal = document.getElementById("som-val");
    if (somVal && estado.som !== undefined) {
        somVal.textContent = estado.som + "%";
        if (estado.som >= 80) somVal.className = "vermelho";
        else if (estado.som >= 50) somVal.className = "amarelo";
        else somVal.className = "verde";
    }

    const invList = document.getElementById("inv-list");
    const invTitulo = document.querySelector("#hud-inv");
    
    if (invList) {
        invList.innerHTML = "";
        
        
        let qtdBolsas = estado.bolsas_coletadas || 0;
        const limiteMaximo = 3 + (qtdBolsas * 3);
        const inventario = estado.inventario || [];
        const qtdAtual = inventario.length;
        
        if (invTitulo) {
            invTitulo.textContent = `INV (${qtdAtual}/${limiteMaximo}):`;
        }

        
        if (qtdAtual > 0) {
            inventario.forEach(item => {
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

        
        let slotsContainer = document.getElementById("inv-slots-container");
        if (!slotsContainer) {
            slotsContainer = document.createElement("div");
            slotsContainer.id = "inv-slots-container";
            slotsContainer.className = "inv-slots-container";

            slotsContainer.style.display = "flex";
            slotsContainer.style.flexWrap = "wrap"; 
            slotsContainer.style.gap = "5px";       
            slotsContainer.style.marginTop = "10px";

            invList.parentElement.appendChild(slotsContainer); 
        }
        
        slotsContainer.innerHTML = ""; 

        
        for (let i = 0; i < limiteMaximo; i++) {
            let slotBox = document.createElement("div");
            slotBox.className = "inv-slot";
            
            if (i < qtdAtual) {
                slotBox.classList.add("cheio"); 
            } else {
                slotBox.classList.add("vazio"); 
            }
            
            slotsContainer.appendChild(slotBox);
        }
    }
}


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
            document.body.classList.remove("glitch-mode", "low-power-mode", "blackout-mode", "kernel-panic-mode")
            outputDiv.innerHTML = "";
            if (terminalEl) terminalEl.scrollTop = terminalEl.scrollHeight;
            resolve();

        } else if (linha.includes("@@PANIC@@")) {
            linha = linha.replace("@@PANIC@@", "");
            document.body.classList.add("kernel-panic-mode");
            reproduzirBeep('erro'); 
            
            
            let imgPanic = document.createElement("img");
            imgPanic.src = "images/image.png";
            imgPanic.className = "panic-img-overlay";
            imgPanic.id = "panic-img-id"; 
            
            
            const terminalSec = document.querySelector('.terminal-section');
            if (terminalSec) terminalSec.appendChild(imgPanic);
            
            resolve();
        
        } else if (linha.includes("@@GLITCH_LUZ@@")) {
            linha = linha.replace("@@GLITCH_LUZ@@", "");
            document.body.classList.add("glitch-mode");
            reproduzirBeep('erro'); 
            
            setTimeout(() => {
                document.body.classList.remove("glitch-mode");
            }, 1500);
            resolve();

        } else if (linha.includes("@@LOW_POWER@@")) {
            linha = linha.replace("@@LOW_POWER@@", "");
            document.body.classList.remove("blackout-mode"); 
            document.body.classList.add("low-power-mode");
            resolve();

        } else if (linha.includes("@@BLACKOUT@@")) {
            linha = linha.replace("@@BLACKOUT@@", "");
            document.body.classList.remove("low-power-mode");
            document.body.classList.add("blackout-mode");
            resolve();

        } else if (linha.includes("@@NORMAL_POWER@@")) {
            linha = linha.replace("@@NORMAL_POWER@@", "");
            document.body.classList.remove("low-power-mode", "blackout-mode");
            resolve();

        } else if (linha.startsWith("@@EXIT@@")) {
            document.body.innerHTML = ""; 
            document.body.style.backgroundColor = "#000";
            resolve();
            
        } else if (linha.startsWith("@@RELOAD@@")) {
            document.body.classList.remove("kernel-panic-mode");
            document.body.classList.remove("low-power-mode", "blackout-mode");
            let img = document.getElementById("panic-img-id");
            if (img) img.remove();
            
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
            
            
            let velocidadeFinal = ms * pref_multiplicadorVelocidade;
            if (pref_multiplicadorVelocidade === 0) velocidadeFinal = 0;

            digitarTextoAnimadoHTML(texto, cor, velocidadeFinal, resolve);

        } else {
            
            let velocidadeFinal = 15 * pref_multiplicadorVelocidade;
            if (pref_multiplicadorVelocidade === 0) velocidadeFinal = 0;
            
            digitarTextoAnimadoHTML(linha, "", velocidadeFinal, resolve);
        }
    });
}

function digitarTextoAnimadoHTML(htmlString, classeCor, velocidade, aoTerminar) {
    const p = document.createElement('p');
    if (classeCor) p.className = classeCor;
    
    const srSpan = document.createElement('span');
    srSpan.className = 'sr-only';
    
    let a11yPrefix = "";
    if (classeCor === 'vermelho') {
        a11yPrefix = "Perigo: ";
        document.body.classList.add('glitch-active');
        setTimeout(() => document.body.classList.remove('glitch-active'), 250);
    }
    if (classeCor === 'amarelo') a11yPrefix = "Atenção: ";
    
    const textoLimpo = htmlString.replace(/<[^>]*>?/gm, '');
    srSpan.innerText = a11yPrefix + textoLimpo;

    const visualSpan = document.createElement('span');
    visualSpan.setAttribute('aria-hidden', 'true');

    p.appendChild(srSpan);
    p.appendChild(visualSpan);
    outputDiv.appendChild(p);
    
    if (velocidade === 0) {
        visualSpan.innerHTML = htmlString;
        terminal.scrollTop = terminal.scrollHeight;
        aoTerminar();
        return;
    }
    
    let i = 0;
    let isTag = false;
    let currentHTML = "";
    let ultimaVez = 0;
    
    
    function digitar(timestamp) {
        if (i >= htmlString.length) {
            aoTerminar(); 
            return;
        }

        if (!ultimaVez) ultimaVez = timestamp;
        const progresso = timestamp - ultimaVez;

        
        let isProcessingTag = (isTag || (i < htmlString.length && htmlString.charAt(i) === '<'));

        if (isProcessingTag || progresso >= velocidade) {
            let char = htmlString.charAt(i);
            currentHTML += char;
            visualSpan.innerHTML = currentHTML;
            i++;
            terminal.scrollTop = terminal.scrollHeight;
            
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
        
        
        if (data.novas_conquistas) {
            processarConquistas(data.novas_conquistas);
        }

        
        if (data.estado) {
            if (data.estado.estado_jogo === "FIM" && data.estado.tempo_final > 0) {
                setTimeout(() => {
                    mostrarTelaDePontuacao(data.estado.tempo_final);
                }, 5000); 
            }
        }

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
        body: JSON.stringify({ 
            comando: comando, 
            telemetria: pref_telemetria 
        })
    });
}




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
        scareGain.connect(masterGainNode);
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

function ligarTV() {
    document.getElementById("power-button").disabled = true;
    document.getElementById("power-led").classList.add("led-on"); 
    
    iniciarSomAmbiente();
    reproduzirBeep('erro'); 

    const layerStatic = document.getElementById("layer-static");
    const layerColorBars = document.getElementById("layer-color-bars");
    const layerBlue = document.getElementById("layer-blue");
    const layerGame = document.getElementById("layer-game");
    
    const tvFrame = document.getElementById("tv-frame");
    const tvOverlay = document.getElementById("tv-overlay");
    const contentArea = document.getElementById("tv-content-area");

    
    layerStatic.classList.add("static-active");

    
    setTimeout(() => {
        layerStatic.classList.remove("static-active");
        layerStatic.classList.add("hidden");
        layerColorBars.classList.remove("hidden");
        
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
        layerColorBars.classList.add("hidden");
        layerBlue.classList.remove("hidden");
        if (window.colorBarOsc) window.colorBarOsc.stop();
    }, 3000);

    
    setTimeout(() => {
        layerBlue.classList.add("hidden");
        layerGame.classList.remove("hidden"); 
        
        
        tvFrame.classList.add("zoom-into-tv");       
        tvOverlay.classList.add("fade-out-tv");      
        contentArea.classList.add("expand-screen");  
    }, 4500);

    
    setTimeout(() => {
        const tvRoom = document.getElementById("tv-room");
        const containerJogo = document.querySelector(".container");
        
        document.body.appendChild(containerJogo); 
        tvRoom.remove();                         
        
        iniciarJogo(); 
    }, 6000);
}

const output = document.getElementById("output");
const LIMITE_LINHAS = 50; 

function adicionarLinhaTerminal(htmlContent) {
    
    output.insertAdjacentHTML('beforeend', `<div class="linha">${htmlContent}</div>`);
    
    
    while (output.childElementCount > LIMITE_LINHAS) {
        output.removeChild(output.firstChild);
    }
    

    output.scrollTop = output.scrollHeight;
}

function fazerNada() {
    
}

async function iniciarReplay(idReplay) {
    if (modoReplayAtivo) return;

    try {
        
        const response = await fetch(`/api/replay/${idReplay}`);
        const data = await response.json();

        if (!response.ok) {
            adicionarLinhaTerminal(`[ SISTEMA ] ${data.erro}`, "vermelho");
            return;
        }

        const comandos = data.log_comandos;
        if (!comandos || comandos.length === 0) {
            adicionarLinhaTerminal("[ SISTEMA ] Fita de replay vazia.", "amarelo");
            return;
        }

        
        modoReplayAtivo = true;
        const inputCmd = document.getElementById('cmd');
        if (inputCmd) {
            inputCmd.disabled = true;
            inputCmd.placeholder = "ASSISTINDO REPLAY...";
        }

        adicionarLinhaTerminal("=========================================", "amarelo");
        adicionarLinhaTerminal(`[ SISTEMA ] REPLAY INICIADO: ${data.jogador}`, "amarelo");
        adicionarLinhaTerminal(`[ SISTEMA ] DESTINO FINAL: ${data.sala_final}`, "amarelo");
        adicionarLinhaTerminal("=========================================", "amarelo");

        await sleep(2500); 

        
        for (let i = 0; i < comandos.length; i++) {
            if (!modoReplayAtivo) break;

            const comando = comandos[i];

            
            if (inputCmd) {
                inputCmd.value = "";
                for (let char of comando) {
                    inputCmd.value += char;
                    await sleep(70); 
                }
            }

            await sleep(400); 

            
            adicionarLinhaTerminal(`C:\\> ${comando}`, "branco");
            if (inputCmd) inputCmd.value = ""; 

            
            const resPost = await fetch('/comando', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ comando: comando, telemetria: false }) 
            });

            const resData = await resPost.json();
            
            
            if (typeof processarRespostaBackend === "function") {
                processarRespostaBackend(resData); 
            } else if (resData.linhas) {
                
                resData.linhas.forEach(linha => adicionarLinhaTerminal(linha));
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



function mostrarTelaDePontuacao(segundosTotais) {
    const modal = document.getElementById('arcade-score-modal');
    const display = document.getElementById('score-time-display');
    
    
    const mins = Math.floor(segundosTotais / 60);
    const secs = Math.floor(segundosTotais % 60);
    const ms = Math.floor((segundosTotais % 1) * 100); 
    
    display.innerText = 
        String(mins).padStart(2, '0') + ':' + 
        String(secs).padStart(2, '0') + ':' + 
        String(ms).padStart(2, '0');

    if (modal) {
        modal.classList.remove('hidden');
        document.getElementById('arcade-initials').focus();
    }
}

function fecharArcadeModal() {
    const modal = document.getElementById('arcade-score-modal');
    if (modal) modal.classList.add('hidden');
}

function iniciarLoginESalvar() {
    const iniciais = document.getElementById('arcade-initials').value;
    if (iniciais.length < 3) {
        alert("Por favor, digite 3 letras para o painel do fliperama!");
        return;
    }
    
    // Trava o botão para evitar duplos cliques
    const btn = document.getElementById('btn-login-submit');
    btn.innerText = "CONECTANDO...";
    btn.disabled = true;

    // Redireciona o navegador para o Google, levando as 3 letras na mochila (URL)
    window.location.href = `/login/google?iniciais=${iniciais}`;
}

// NOVO: Lê a URL quando o jogador volta do Google
const originalOnload = window.onload;
window.onload = function() {
    if (originalOnload) originalOnload(); // Roda o seu onload existente (TV, replay, etc)
    
    const params = new URLSearchParams(window.location.search);
    if (params.get("leaderboard") === "sucesso") {
        reproduzirBeep('sucesso');
        alert("Sua pontuação foi registrada no Quadro de Líderes Global!");
        // Limpa a URL para não ficar repetindo o alerta se ele der F5
        window.history.replaceState({}, document.title, "/"); 
    }
};