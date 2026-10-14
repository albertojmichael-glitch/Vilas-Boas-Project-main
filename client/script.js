// ==========================================
// 1. INTERFACES E TIPAGENS
// ==========================================
// ==========================================
// 2. CONSTANTES E ELEMENTOS DO DOM
// ==========================================
const API_URL = "https://vilas-boas-project-main.up.railway.app";
const SAVE_VERSION = 1.0;
const LIMITE_LINHAS = 50;
const SECRET_IMAGES = [
    "hidden image 19.png", "hidden image 20.png", "hidden image 21.png",
    "HiddenImage1.webp", "HiddenImage2.webp", "HiddenImage3.webp",
    "HiddenImage4.webp", "HiddenImage5.webp", "HiddenImage6.webp",
    "HiddenImage7.webp", "HiddenImage8.webp", "HiddenImage10.webp",
    "HiddenImage11.webp", "HiddenImage12.webp", "HiddenImage14.webp",
    "HiddenImage15.webp", "HiddenImage16.webp", "HiddenImage17.webp",
    "HiddenImage18.webp", "HiddenImage20.webp", "mimic.webp"
];
const CATALOGO_CONQUISTAS = [
    { id: "primeira_morte", nome: "Sangue no Carpete", desc: "Bem-vindo ao Vilas Boas.", icone: "☠" },
    { id: "mente_brilhante", nome: "Mente Brilhante", desc: "Abra o cofre na primeira tentativa.", icone: "★" },
    { id: "labirinto", nome: "Labirinto", desc: "Sobreviva à Sala de Energia.", icone: "☄" },
    { id: "acumulador", nome: "Acumulador", desc: "Encha todos os 9 espaços do inventário.", icone: "☑" },
    { id: "glicose", nome: "Glicose Mortal", desc: "Coma o doce velho encontrado no chão.", icone: "✴" },
    { id: "trapaceiro", nome: "Hacker", desc: "Ative o God Mode.", icone: "☣" },
    { id: "normal_zerado", nome: "Fim do Expediente", desc: "Sobreviva à noite na dificuldade Normal.", icone: "♨" },
    { id: "final_bom", nome: "Sobrevivente", desc: "Alcance o Final Neutro.", icone: "☀" },
    { id: "final_verdadeiro", nome: "A Verdade", desc: "Alcance o Final Verdadeiro.", icone: "✦" },
    { id: "pesadelo_neutro", nome: "Pesadelo Superado", desc: "Alcance o Final Neutro no modo PESADELO.", icone: "♛" },
    { id: "pesadelo_verdadeiro", nome: "Mestre do Pesadelo", desc: "Alcance o Final Verdadeiro no modo PESADELO.", icone: "♚" }
];
const getEl = (id) => document.getElementById(id);
const outputDiv = getEl('output');
const inputField = getEl('comando');
const terminal = getEl('terminal');
const loadingSpinner = getEl('loading');
const inputLineDiv = document.querySelector('.input-line');
const terminalSection = document.querySelector('.terminal-section');
const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));
// ==========================================
// 3. ESTADO GLOBAL DO CLIENTE
// ==========================================
let historicoComandos = [];
let posicaoHistorico = -1;
let comandoDigitadoAtual = "";
let modoReplayAtivo = false;
let pref_telemetria = true;
let pref_multiplicadorVelocidade = 1.0;
let pref_volume = 1.0;
let pref_volume_jumpscare = 1.0;
let flashInterval = null;
let flashTimeout = null;
let legendaTimeout;
let toastTimeout;
let audioAmbienteLoop = null;
let sonsRandomicosAtivos = false;
let audioCtx = null;
let masterGainNode = null;
let crtOsc = null;
// ==========================================
// 4. ARMAZENAMENTO SEGURO (SAFE STORAGE)
// ==========================================
const SafeStorage = {
    save(key, data) {
        try {
            const payload = {
                __version__: SAVE_VERSION,
                timestamp: Date.now(),
                data
            };
            const jsonString = JSON.stringify(payload);
            const oldData = localStorage.getItem(key);
            if (oldData)
                localStorage.setItem(`${key}_backup`, oldData);
            localStorage.setItem(key, jsonString);
        }
        catch (e) {
            console.error("[SISTEMA] Erro crítico de I/O no disco local:", e);
        }
    },
    load(key, defaultValue) {
        try {
            const raw = localStorage.getItem(key);
            if (!raw)
                return defaultValue;
            if (!raw.includes('__version__')) {
                const parsed = JSON.parse(raw);
                this.save(key, parsed);
                return parsed;
            }
            const parsed = JSON.parse(raw);
            if (parsed.__version__ < SAVE_VERSION) {
                console.warn(`[SISTEMA] Atualizando save do formato ${parsed.__version__} para ${SAVE_VERSION}`);
            }
            return parsed.data;
        }
        catch {
            console.warn(`[SISTEMA] Setor corrompido em ${key}. Iniciando protocolo de recuperação...`);
            return this.recover(key, defaultValue);
        }
    },
    recover(key, defaultValue) {
        try {
            const rawBackup = localStorage.getItem(`${key}_backup`);
            if (!rawBackup)
                throw new Error("Fita de backup inexistente.");
            localStorage.setItem(key, rawBackup);
            adicionarLinhaTerminal(`[ AVISO ] Anomalia de memória detectada. Backup de segurança restaurado.`, "amarelo");
            const parsed = JSON.parse(rawBackup);
            return (parsed.data !== undefined ? parsed.data : parsed);
        }
        catch {
            adicionarLinhaTerminal(`[ FATAL ] Perda de dados irreversível no setor: ${key}.`, "vermelho");
            return defaultValue;
        }
    }
};
// ==========================================
// 5. EFEITOS VISUAIS E IMAGENS SECRETAS
// ==========================================
function startFlashingSecrets() {
    const overlay = getEl("secret-image-overlay");
    const img = getEl("secret-img-display");
    if (!overlay || !img)
        return;
    stopFlashingSecrets();
    overlay.classList.remove("hidden", "slideshow-mode");
    flashInterval = setInterval(() => {
        const randomImg = SECRET_IMAGES[Math.floor(Math.random() * SECRET_IMAGES.length)];
        img.src = `assets/secret images/${randomImg}`;
        img.classList.add("flash-active");
        flashTimeout = setTimeout(() => {
            img.classList.remove("flash-active");
        }, 50 + Math.random() * 250);
    }, 800 + Math.random() * 1500);
}
function stopFlashingSecrets() {
    if (flashInterval)
        clearInterval(flashInterval);
    if (flashTimeout)
        clearTimeout(flashTimeout);
    flashInterval = null;
    flashTimeout = null;
    const overlay = getEl("secret-image-overlay");
    const img = getEl("secret-img-display");
    img?.classList.remove("flash-active");
    overlay?.classList.add("hidden");
}
async function play1980Slideshow() {
    const overlay = getEl("secret-image-overlay");
    const img = getEl("secret-img-display");
    if (!overlay || !img)
        return;
    overlay.classList.remove("hidden");
    overlay.classList.add("slideshow-mode");
    for (const src of SECRET_IMAGES) {
        img.src = `assets/secret images/${src}`;
        img.classList.add("slideshow-active");
        await sleep(2000);
        img.classList.remove("slideshow-active");
        await sleep(200);
    }
    overlay.classList.add("hidden");
    overlay.classList.remove("slideshow-mode");
}
function ativarJumpscare() {
    const tela = getEl('tela-jumpscare');
    const img = getEl('img-jumpscare');
    if (!tela || !img)
        return;
    const imagens = ['./assets/static/images/JUMPSCARE1.jpg', './assets/static/images/JUMPSCARE2.jpg'];
    img.src = imagens[Math.floor(Math.random() * imagens.length)];
    const audioSusto = new Audio('./assets/audio/grito.mp3');
    audioSusto.volume = pref_volume * pref_volume_jumpscare;
    audioSusto.play().catch(err => console.log("Erro de áudio do jumpscare:", err));
    tela.style.display = 'flex';
    setTimeout(() => {
        tela.style.display = 'none';
    }, 2500);
}
function ativarBSOD() {
    const tela = getEl('tela-bsod');
    const imgCreepy = getEl('img-bsod-creepy');
    if (!tela || !imgCreepy)
        return;
    const imagens = [
        './assets/static/images/JAPONESMEDO.jpg',
        './assets/static/images/SHEISHERE1.jpg',
        './assets/static/images/SHEISHERE2.jpg'
    ];
    imgCreepy.src = imagens[Math.floor(Math.random() * imagens.length)];
    tela.style.display = 'flex';
    reproduzirBeep('erro');
    setTimeout(() => { imgCreepy.style.opacity = '0.6'; }, 300);
    setTimeout(() => { imgCreepy.style.opacity = '0'; }, 400);
    setTimeout(() => { imgCreepy.style.opacity = '0.9'; }, 700);
    setTimeout(() => { imgCreepy.style.opacity = '0'; }, 800);
    setTimeout(() => {
        tela.style.display = 'none';
    }, 2500);
}
// ==========================================
// 6. SISTEMA DE ÁUDIO (WEB AUDIO API)
// ==========================================
function obterAudioContext() {
    if (!audioCtx) {
        const AudioContextClass = window.AudioContext || window.webkitAudioContext;
        if (AudioContextClass) {
            audioCtx = new AudioContextClass();
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
function mostrarLegendaDeAudio(texto) {
    const legendaDiv = getEl('audio-captions');
    if (legendaDiv) {
        legendaDiv.textContent = `[ Áudio: ${texto} ]`;
        legendaDiv.classList.remove('hidden');
        clearTimeout(legendaTimeout);
        legendaTimeout = setTimeout(() => {
            legendaDiv.classList.add('hidden');
        }, 2000);
    }
}
function tocarSomDigito() {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode)
        return;
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
function iniciarSomAmbiente() {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode)
        return;
    if (!crtOsc) {
        crtOsc = ctx.createOscillator();
        const crtGain = ctx.createGain();
        crtOsc.type = 'sawtooth';
        crtOsc.frequency.value = 60;
        crtGain.gain.value = 0.005;
        crtOsc.connect(crtGain);
        crtGain.connect(masterGainNode);
        crtOsc.start();
    }
    if (!audioAmbienteLoop) {
        audioAmbienteLoop = new Audio('./assets/audio/ambiente.mp3');
        audioAmbienteLoop.loop = true;
        audioAmbienteLoop.volume = pref_volume * 0.6;
        audioAmbienteLoop.play().catch(err => console.log("Autoplay bloqueado:", err));
    }
    if (!sonsRandomicosAtivos) {
        sonsRandomicosAtivos = true;
        tocarSonsAssustadoresAleatorios();
    }
}
function tocarSonsAssustadoresAleatorios() {
    const tempoAleatorio = Math.floor(Math.random() * (80000 - 30000 + 1)) + 30000;
    setTimeout(() => {
        const audios = ['./assets/audio/sonsrandomicos.mp3', './assets/audio/sonsrandomicos2.mp3'];
        const escolhido = audios[Math.floor(Math.random() * audios.length)];
        const audioSusto = new Audio(escolhido);
        audioSusto.volume = pref_volume * 0.8;
        audioSusto.play().catch(e => console.log("Erro som randomico:", e));
        tocarSonsAssustadoresAleatorios();
    }, tempoAleatorio);
}
function tocarBipEntrada() {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode)
        return;
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
function tocarPassoMetalico() {
    const passoAudio = new Audio('./assets/audio/metalrangendo.mp3');
    passoAudio.volume = pref_volume;
    passoAudio.play().catch(e => console.log("Erro ao tocar passo:", e));
    mostrarLegendaDeAudio("Passos metálicos pesados");
}
function tocarPorta() {
    const portaAudio = new Audio('./assets/audio/portafechandoabrindo.mp3');
    portaAudio.volume = pref_volume;
    portaAudio.play().catch(e => console.log(e));
    mostrarLegendaDeAudio("Porta rangendo");
}
function reproduzirBeep(tipo = 'sucesso') {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode)
        return;
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
    }
    else {
        osc.type = 'square';
        osc.frequency.setValueAtTime(800, ctx.currentTime);
        gain.gain.setValueAtTime(0.03, ctx.currentTime);
        osc.start();
        osc.stop(ctx.currentTime + 0.1);
    }
}
function playBip(tipo = 'sucesso') {
    reproduzirBeep(tipo);
}
// ==========================================
// 7. SISTEMA DE CONQUISTAS
// ==========================================
function abrirModalConquistas() {
    const modal = getEl('achievements-modal');
    const grid = getEl('achievements-grid');
    const title = getEl('achievements-title');
    if (!modal || !grid || !title)
        return;
    const salvas = SafeStorage.load('vilasBoasAchievements', {});
    grid.innerHTML = '';
    let quantidadeDesbloqueada = 0;
    CATALOGO_CONQUISTAS.forEach(conquista => {
        const estaDesbloqueada = Boolean(salvas[conquista.id]);
        if (estaDesbloqueada)
            quantidadeDesbloqueada++;
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
    getEl('achievements-modal')?.classList.add('hidden');
}
function processarConquistas(listaConquistas) {
    if (!listaConquistas || listaConquistas.length === 0)
        return;
    const salvas = SafeStorage.load('vilasBoasAchievements', {});
    listaConquistas.forEach(conquista => {
        if (!salvas[conquista.id]) {
            salvas[conquista.id] = conquista;
            reproduzirBeep('sucesso');
            mostrarToastConquista(conquista);
        }
    });
    SafeStorage.save('vilasBoasAchievements', salvas);
}
function mostrarToastConquista(conquista) {
    const toast = getEl('achievement-toast');
    const icon = getEl('achieve-icon');
    const name = getEl('achieve-name');
    if (!toast || !icon || !name)
        return;
    icon.innerText = conquista.icone;
    name.innerText = conquista.nome;
    toast.classList.remove('hidden');
    toast.classList.add('show');
    clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
        toast.classList.remove('show');
    }, 4000);
}
// ==========================================
// 8. GERENCIAMENTO DE SAVES E MODAIS
// ==========================================
function openSaves() {
    getEl('saves-modal')?.classList.remove('hidden');
}
function closeSaves() {
    getEl('saves-modal')?.classList.add('hidden');
    inputField?.focus();
}
function openHelp() {
    getEl('help-modal')?.classList.remove('hidden');
}
function closeHelp() {
    getEl('help-modal')?.classList.add('hidden');
    inputField?.focus();
}
function openSettings() {
    getEl('settings-modal')?.classList.remove('hidden');
}
function closeSettings() {
    getEl('settings-modal')?.classList.add('hidden');
    inputField?.focus();
}
async function exportarSave() {
    try {
        const res = await fetch(`${API_URL}/save/export`, { credentials: 'include' });
        if (!res.ok)
            throw new Error("Nenhum progresso encontrado no servidor.");
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
    }
    catch (erro) {
        const msg = erro instanceof Error ? erro.message : "Erro desconhecido";
        console.error(erro);
        alert(`[ERRO] ${msg}`);
    }
}
async function importarSave(event) {
    const input = event.target;
    const file = input.files?.[0];
    if (!file)
        return;
    const reader = new FileReader();
    reader.onload = async (e) => {
        try {
            const conteudo = e.target?.result;
            if (typeof conteudo !== 'string')
                throw new Error("Falha ao ler arquivo.");
            const dados = JSON.parse(conteudo);
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
            }
            else {
                alert(`[ERRO DE BIOS] ${result.erro || "Falha na decodificação do servidor."}`);
            }
        }
        catch (erro) {
            const msg = erro instanceof Error ? erro.message : "Arquivo inválido";
            alert(`[ERRO FATAL] O arquivo fornecido foi rejeitado.\nMotivo: ${msg}`);
        }
    };
    reader.readAsText(file);
    input.value = '';
}
async function gerarLinkCompartilhamentoUI() {
    try {
        const res = await fetch(`${API_URL}/share/generate`, { credentials: 'include' });
        const data = (await res.json());
        if (res.ok) {
            const container = getEl('share-link-container');
            const input = getEl('share-link-input');
            const msg = getEl('share-link-msg');
            if (container && input && msg) {
                container.classList.remove('hidden');
                input.value = data.link;
                msg.innerText = data.mensagem;
                input.select();
                if (navigator.clipboard) {
                    await navigator.clipboard.writeText(data.link);
                }
                else {
                    document.execCommand('copy');
                }
            }
            reproduzirBeep('sucesso');
            alert("Link copiado para a área de transferência!");
        }
        else {
            alert(`[ERRO] ${data.erro || "Falha ao gerar link."}`);
        }
    }
    catch (erro) {
        console.error(erro);
        alert("Falha de conexão com os servidores centrais.");
    }
}
// ==========================================
// 9. PREFERÊNCIAS E CONFIGURAÇÕES
// ==========================================
function atualizarLabelVelocidade(val) {
    const label = getEl('speed-val-display');
    if (!label)
        return;
    label.innerText = val === 0 ? "Instantâneo (0x)" : `Atual (${val}x)`;
}
function aplicarEfeitoCRT(ativado) {
    document.body.classList.toggle('crt-enabled', ativado);
}
function carregarPreferencias() {
    const teleCheckbox = getEl('telemetry-checkbox');
    const speedSlider = getEl('speed-slider');
    const crtCheckbox = getEl('crt-checkbox');
    const volSlider = getEl('volume-slider');
    const volJumpSlider = getEl('volume-jump-slider');
    const savedTele = localStorage.getItem('vilasBoasTelemetry');
    const savedSpeed = localStorage.getItem('vilasBoasSpeed');
    const savedCRT = localStorage.getItem('vilasBoasCRT');
    const savedVolume = localStorage.getItem('vilasBoasVolume');
    const savedVolumeJump = localStorage.getItem('vilasBoasVolumeJump');
    if (savedTele !== null && teleCheckbox) {
        pref_telemetria = savedTele === 'true';
        teleCheckbox.checked = pref_telemetria;
    }
    teleCheckbox?.addEventListener('change', (e) => {
        pref_telemetria = e.target.checked;
        localStorage.setItem('vilasBoasTelemetry', String(pref_telemetria));
    });
    if (savedSpeed !== null && speedSlider) {
        pref_multiplicadorVelocidade = parseFloat(savedSpeed);
        speedSlider.value = String(pref_multiplicadorVelocidade);
        atualizarLabelVelocidade(pref_multiplicadorVelocidade);
    }
    speedSlider?.addEventListener('input', (e) => {
        pref_multiplicadorVelocidade = parseFloat(e.target.value);
        localStorage.setItem('vilasBoasSpeed', String(pref_multiplicadorVelocidade));
        atualizarLabelVelocidade(pref_multiplicadorVelocidade);
    });
    if (crtCheckbox) {
        if (savedCRT !== null) {
            crtCheckbox.checked = savedCRT === 'true';
        }
        aplicarEfeitoCRT(crtCheckbox.checked);
        crtCheckbox.addEventListener('change', (e) => {
            const ativado = e.target.checked;
            localStorage.setItem('vilasBoasCRT', String(ativado));
            aplicarEfeitoCRT(ativado);
        });
    }
    if (savedVolume !== null && volSlider) {
        pref_volume = parseFloat(savedVolume);
        volSlider.value = String(pref_volume);
        const volDisplay = getEl('volume-val-display');
        if (volDisplay)
            volDisplay.innerText = `${Math.round(pref_volume * 100)}%`;
        if (masterGainNode)
            masterGainNode.gain.value = pref_volume;
    }
    volSlider?.addEventListener('input', (e) => {
        pref_volume = parseFloat(e.target.value);
        localStorage.setItem('vilasBoasVolume', String(pref_volume));
        const volDisplay = getEl('volume-val-display');
        if (volDisplay)
            volDisplay.innerText = `${Math.round(pref_volume * 100)}%`;
        if (masterGainNode)
            masterGainNode.gain.value = pref_volume;
        if (audioAmbienteLoop)
            audioAmbienteLoop.volume = pref_volume * 0.6;
    });
    if (savedVolumeJump !== null && volJumpSlider) {
        pref_volume_jumpscare = parseFloat(savedVolumeJump);
        volJumpSlider.value = String(pref_volume_jumpscare);
        const displayJump = getEl('volume-jump-val-display');
        if (displayJump)
            displayJump.innerText = `${Math.round(pref_volume_jumpscare * 100)}%`;
    }
    volJumpSlider?.addEventListener('input', (e) => {
        pref_volume_jumpscare = parseFloat(e.target.value);
        localStorage.setItem('vilasBoasVolumeJump', String(pref_volume_jumpscare));
        const displayJump = getEl('volume-jump-val-display');
        if (displayJump)
            displayJump.innerText = `${Math.round(pref_volume_jumpscare * 100)}%`;
        const testeSusto = new Audio('./assets/audio/grito.mp3');
        testeSusto.volume = pref_volume * pref_volume_jumpscare;
        testeSusto.play().catch(() => { });
        setTimeout(() => testeSusto.pause(), 300);
    });
}
// ==========================================
// 10. ATUALIZAÇÃO DO HUD E TERMINAL
// ==========================================
function atualizarSidebar(estado) {
    if (!estado)
        return;
    const hpVal = getEl("hp-val");
    if (hpVal) {
        if (estado.hp === "∞") {
            hpVal.textContent = "[ GOD MODE ]";
            hpVal.className = "amarelo";
            document.body.classList.remove("hp-critico");
        }
        else {
            const hpAtual = parseInt(String(estado.hp), 10) || 0;
            const maxHp = 3;
            const blocosCheios = "█".repeat(Math.max(0, hpAtual));
            const blocosVazios = "░".repeat(Math.max(0, maxHp - hpAtual));
            hpVal.textContent = `[${blocosCheios}${blocosVazios}]`;
            hpVal.className = hpAtual <= 1 ? "vermelho" : "verde";
            document.body.classList.toggle("hp-critico", hpAtual <= 1 && hpAtual > 0);
        }
    }
    const luzVal = getEl("luz-val");
    if (luzVal) {
        luzVal.textContent = estado.luz_restante !== undefined ? String(estado.luz_restante) : "??";
        const luzAlta = estado.luz_restante === "∞" || Number(estado.luz_restante) > 3;
        luzVal.className = luzAlta ? "verde" : "vermelho";
    }
    const somVal = getEl("som-val");
    if (somVal && estado.som !== undefined) {
        somVal.textContent = `${estado.som}%`;
        if (estado.som >= 80)
            somVal.className = "vermelho";
        else if (estado.som >= 50)
            somVal.className = "amarelo";
        else
            somVal.className = "verde";
    }
    const invList = getEl("inv-list");
    const invTitulo = document.querySelector("#hud-inv, .inventory-box h3");
    if (invList) {
        invList.innerHTML = "";
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
        }
        else {
            const li = document.createElement("li");
            li.textContent = "Vazio";
            li.className = "amarelo";
            invList.appendChild(li);
        }
        let slotsContainer = getEl("inv-slots-container");
        if (!slotsContainer && invList.parentElement) {
            slotsContainer = document.createElement("div");
            slotsContainer.id = "inv-slots-container";
            slotsContainer.className = "inv-slots-container";
            invList.parentElement.appendChild(slotsContainer);
        }
        if (slotsContainer) {
            slotsContainer.innerHTML = "";
            for (let i = 0; i < limiteMaximo; i++) {
                const slotBox = document.createElement("div");
                slotBox.className = `inv-slot ${i < qtdAtual ? "cheio" : "vazio"}`;
                slotsContainer.appendChild(slotBox);
            }
        }
    }
}
function adicionarLinhaTerminal(htmlContent, cor = "") {
    if (!outputDiv)
        return;
    const classeCor = cor ? ` ${cor}` : "";
    outputDiv.insertAdjacentHTML('beforeend', `<div class="linha${classeCor}">${htmlContent}</div>`);
    while (outputDiv.childElementCount > LIMITE_LINHAS) {
        if (outputDiv.firstChild)
            outputDiv.removeChild(outputDiv.firstChild);
    }
    outputDiv.scrollTop = outputDiv.scrollHeight;
}
async function processarLinhas(linhas, estado) {
    for (const linha of linhas) {
        await novaLinha(linha, terminalSection);
        if (terminalSection)
            terminalSection.scrollTop = terminalSection.scrollHeight;
    }
    atualizarSidebar(estado);
}
function novaLinha(linhaBruta, terminalEl) {
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
            if (outputDiv)
                outputDiv.innerHTML = "";
            if (terminalEl)
                terminalEl.scrollTop = terminalEl.scrollHeight;
            resolve();
        }
        else if (linha.includes("@@PANIC@@")) {
            document.body.classList.add("kernel-panic-mode");
            reproduzirBeep('erro');
            const imgPanic = document.createElement("img");
            imgPanic.src = "images/image.png";
            imgPanic.className = "panic-img-overlay";
            imgPanic.id = "panic-img-id";
            terminalSection?.appendChild(imgPanic);
            resolve();
        }
        else if (linha.includes("@@GLITCH_LUZ@@")) {
            document.body.classList.add("glitch-mode");
            reproduzirBeep('erro');
            setTimeout(() => document.body.classList.remove("glitch-mode"), 1500);
            resolve();
        }
        else if (linha.includes("@@LOW_POWER@@")) {
            document.body.classList.remove("blackout-mode");
            document.body.classList.add("low-power-mode");
            resolve();
        }
        else if (linha.includes("@@BLACKOUT@@")) {
            document.body.classList.remove("low-power-mode");
            document.body.classList.add("blackout-mode");
            resolve();
        }
        else if (linha.includes("@@NORMAL_POWER@@")) {
            document.body.classList.remove("low-power-mode", "blackout-mode");
            resolve();
        }
        else if (linha.startsWith("@@EXIT@@")) {
            document.body.innerHTML = "";
            document.body.style.backgroundColor = "#000";
            resolve();
        }
        else if (linha.startsWith("@@RELOAD@@")) {
            document.body.classList.remove("kernel-panic-mode", "low-power-mode", "blackout-mode");
            getEl("panic-img-id")?.remove();
            window.location.reload();
            resolve();
        }
        else if (linha.startsWith("@@PAUSE@@")) {
            const ms = parseInt(linha.split("@@")[2], 10) / 3;
            setTimeout(resolve, ms);
        }
        else if (linha.startsWith("@@TYPE@@")) {
            const parts = linha.split("@@");
            const cor = parts[2];
            const ms = parseInt(parts[3], 10);
            const texto = parts.slice(4).join("@@");
            const velocidadeFinal = pref_multiplicadorVelocidade === 0 ? 0 : ms * pref_multiplicadorVelocidade;
            digitarTextoAnimadoHTML(texto, cor, velocidadeFinal, resolve);
        }
        else {
            const velocidadeFinal = pref_multiplicadorVelocidade === 0 ? 0 : 15 * pref_multiplicadorVelocidade;
            digitarTextoAnimadoHTML(linha, "", velocidadeFinal, resolve);
        }
    });
}
function digitarTextoAnimadoHTML(htmlString, classeCor, velocidade, aoTerminar) {
    if (!outputDiv) {
        aoTerminar();
        return;
    }
    const p = document.createElement('p');
    if (classeCor)
        p.className = classeCor;
    const srSpan = document.createElement('span');
    srSpan.className = 'sr-only';
    let a11yPrefix = "";
    if (classeCor === 'vermelho') {
        a11yPrefix = "Perigo: ";
        document.body.classList.add('glitch-active');
        setTimeout(() => document.body.classList.remove('glitch-active'), 250);
    }
    else if (classeCor === 'amarelo') {
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
        if (terminal)
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
        if (!ultimaVez)
            ultimaVez = timestamp;
        const progresso = timestamp - ultimaVez;
        const isProcessingTag = isTag || htmlString.charAt(i) === '<';
        if (isProcessingTag || progresso >= velocidade) {
            const char = htmlString.charAt(i);
            currentHTML += char;
            visualSpan.innerHTML = currentHTML;
            i++;
            if (terminal)
                terminal.scrollTop = terminal.scrollHeight;
            if (char === '<')
                isTag = true;
            if (char === '>')
                isTag = false;
            if (isTag || (i < htmlString.length && htmlString.charAt(i) === '<')) {
                digitar(timestamp);
            }
            else {
                if (char !== ' ' && char !== '\n') {
                    tocarSomDigito();
                }
                ultimaVez = timestamp;
                requestAnimationFrame(digitar);
            }
        }
        else {
            requestAnimationFrame(digitar);
        }
    }
    requestAnimationFrame(digitar);
}
// ==========================================
// 11. COMUNICAÇÃO COM O SERVIDOR (FETCH)
// ==========================================
function mostrarSalvando() {
    const ind = getEl('save-indicator');
    if (ind) {
        ind.classList.remove('hidden');
        setTimeout(() => ind.classList.add('hidden'), 1500);
    }
}
async function fetchSeguro(url, options = {}) {
    if (inputField)
        inputField.disabled = true;
    if (inputLineDiv)
        inputLineDiv.style.display = 'none';
    if (loadingSpinner)
        loadingSpinner.style.display = 'flex';
    const startTime = Date.now();
    try {
        options.credentials = 'include';
        const res = await fetch(`${API_URL}${url}`, options);
        if (!res.ok)
            throw new Error("Servidor offline");
        const data = (await res.json());
        const tempoDecorrido = Date.now() - startTime;
        if (tempoDecorrido < 300) {
            await sleep(300 - tempoDecorrido);
        }
        if (loadingSpinner)
            loadingSpinner.style.display = 'none';
        await processarLinhas(data.linhas, data.estado);
        if (data.novas_conquistas) {
            processarConquistas(data.novas_conquistas);
        }
        if (data.estado?.estado_jogo === "FIM" && (data.estado.tempo_final ?? 0) > 0) {
            const tempoFinal = data.estado.tempo_final;
            const finalAlcancado = data.estado.final_alcancado;
            setTimeout(() => {
                mostrarTelaDePontuacao(tempoFinal, finalAlcancado);
            }, 5000);
        }
        if (url === '/comando') {
            mostrarSalvando();
        }
    }
    catch (erro) {
        console.error("Erro na comunicação:", erro);
        if (loadingSpinner)
            loadingSpinner.style.display = 'none';
        if (outputDiv) {
            const p = document.createElement('p');
            p.className = 'vermelho';
            p.innerHTML = "[ERRO DE CONEXÃO] O sinal com o servidor falhou. Verifique sua internet.";
            outputDiv.appendChild(p);
        }
        if (terminal)
            terminal.scrollTop = terminal.scrollHeight;
    }
    finally {
        if (inputLineDiv)
            inputLineDiv.style.display = 'flex';
        if (inputField) {
            inputField.disabled = false;
            inputField.focus();
        }
    }
}
function iniciarJogo() {
    fetchSeguro('/iniciar', { method: 'GET' });
}
async function enviarComando(comando) {
    await fetchSeguro('/comando', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            comando,
            telemetria: pref_telemetria
        })
    });
}
function executarAtalho(cmd) {
    if (!inputField)
        return;
    inputField.value = cmd;
    inputField.focus();
    enviarComando(cmd);
}
// ==========================================
// 12. TV RETRÔ, REPLAY E ARCADE LEADERBOARD
// ==========================================
function ligarTV() {
    const powerBtn = getEl("power-button");
    const powerLed = getEl("power-led");
    if (powerBtn)
        powerBtn.disabled = true;
    powerLed?.classList.add("led-on");
    iniciarSomAmbiente();
    reproduzirBeep('erro');
    const layerStatic = getEl("layer-static");
    const layerColorBars = getEl("layer-color-bars");
    const layerBlue = getEl("layer-blue");
    const layerGame = getEl("layer-game");
    const tvFrame = getEl("tv-frame");
    const tvOverlay = getEl("tv-overlay");
    const contentArea = getEl("tv-content-area");
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
        const tvRoom = getEl("tv-room");
        const containerJogo = document.querySelector(".container");
        if (containerJogo)
            document.body.appendChild(containerJogo);
        tvRoom?.remove();
        iniciarJogo();
    }, 6000);
}
async function iniciarReplay(idReplay) {
    if (modoReplayAtivo)
        return;
    try {
        const response = await fetch(`/api/replay/${idReplay}`);
        const data = (await response.json());
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
            if (!modoReplayAtivo)
                break;
            if (inputField) {
                inputField.value = "";
                for (const char of comando) {
                    inputField.value += char;
                    await sleep(70);
                }
            }
            await sleep(400);
            adicionarLinhaTerminal(`C:\\> ${comando}`, "branco");
            if (inputField)
                inputField.value = "";
            const resPost = await fetch('/comando', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ comando, telemetria: false })
            });
            const resData = (await resPost.json());
            if (typeof window.processarRespostaBackend === "function") {
                window.processarRespostaBackend(resData);
            }
            else if (resData.linhas) {
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
    }
    catch (erro) {
        console.error("Erro no replay:", erro);
        adicionarLinhaTerminal("[ SISTEMA ] Erro de conexão ao buscar a fita.", "vermelho");
        modoReplayAtivo = false;
    }
}
function mostrarTelaDePontuacao(segundosTotais, nomeDoFinal) {
    const modal = getEl('arcade-score-modal');
    const display = getEl('score-time-display');
    const textoFinal = getEl('score-ending-text');
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
        getEl('arcade-initials')?.focus();
    }
}
function fecharArcadeModal() {
    getEl('arcade-score-modal')?.classList.add('hidden');
    window.location.reload();
}
function iniciarLoginESalvar() {
    const iniciaisInput = getEl('arcade-initials');
    const iniciais = iniciaisInput?.value || "";
    if (iniciais.length < 3) {
        alert("Por favor, digite 3 letras para o painel do fliperama!");
        return;
    }
    const btn = getEl('btn-login-submit');
    if (btn) {
        btn.innerText = "CONECTANDO...";
        btn.disabled = true;
    }
    window.location.href = `${API_URL}/login/google?iniciais=${iniciais}`;
}
// ==========================================
// 13. EVENTOS DE TECLADO, CLIQUE E BOOT
// ==========================================
document.body.addEventListener('click', iniciarSomAmbiente, { once: true });
document.body.addEventListener('keydown', iniciarSomAmbiente, { once: true });
window.addEventListener('click', (e) => {
    const modal = getEl('achievements-modal');
    if (e.target === modal) {
        fecharModalConquistas();
    }
});
terminalSection?.addEventListener('click', (event) => {
    const target = event.target;
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
inputField?.addEventListener("keydown", async (event) => {
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
                p.innerHTML = `<span class="prompt">C:\\></span> ${comandoBruto}`;
                outputDiv.appendChild(p);
            }
            inputField.value = "";
            if (terminal)
                terminal.scrollTop = terminal.scrollHeight;
            if (comando.toLowerCase() === "ajuda") {
                openHelp();
                if (outputDiv) {
                    const pResp = document.createElement("p");
                    pResp.className = "verde";
                    pResp.innerHTML = "[SISTEMA] Abrindo o Manual de Sobrevivência...";
                    outputDiv.appendChild(pResp);
                }
                if (terminal)
                    terminal.scrollTop = terminal.scrollHeight;
            }
            else {
                await enviarComando(comando);
            }
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
        }
        else if (posicaoHistorico === historicoComandos.length - 1) {
            posicaoHistorico++;
            inputField.value = comandoDigitadoAtual;
        }
    }
});
document.addEventListener('keydown', (e) => {
    if (e.key === "Escape") {
        closeHelp();
        closeSettings();
        closeSaves();
        fecharModalConquistas();
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l') {
        e.preventDefault();
        if (outputDiv)
            outputDiv.innerHTML = '';
        reproduzirBeep('sucesso');
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 's') {
        e.preventDefault();
        if (outputDiv) {
            const p = document.createElement('p');
            p.className = 'verde';
            p.innerHTML = "[SISTEMA] O progresso do jogo é salvo automaticamente a cada turno.";
            outputDiv.appendChild(p);
        }
        if (terminal)
            terminal.scrollTop = terminal.scrollHeight;
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
    }
    else if (!getEl("tv-room")) {
        iniciarJogo();
    }
    carregarPreferencias();
    if (urlParams.get("leaderboard") === "sucesso") {
        reproduzirBeep('sucesso');
        alert("Sua pontuação foi registrada no Quadro de Líderes Global!");
        window.history.replaceState({}, document.title, "/");
    }
});
// ==========================================
// 14. EXPOSIÇÃO GLOBAL PARA O INDEX.HTML
// ==========================================
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
export {};
