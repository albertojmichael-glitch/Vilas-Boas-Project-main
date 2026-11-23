import { configurarEntrada } from './modules/entrada.js';
import { getEl } from './modules/constants.js';
import { reproduzirBeep } from './modules/audio.js';
import { abrirModalConquistas, fecharModalConquistas } from './modules/conquistas.js';
import { iniciarJogo, executarAtalho } from './modules/jogo.js';
import { openHelp, closeHelp, openSettings, closeSettings, openSaves, closeSaves } from './modules/modais.js';
import { fecharArcadeModal, iniciarLoginESalvar } from './modules/placar.js';
import { carregarPreferencias } from './modules/preferencias.js';
import { iniciarReplay } from './modules/replay.js';
import { exportarSave, importarSave, gerarLinkCompartilhamentoUI } from './modules/saves.js';
import { configurarLoggerStorage } from './modules/storage.js';
import { adicionarLinhaTerminal } from './modules/terminal.js';
import { ligarTV } from './modules/tv.js';
import { retryBoot, shortcuts } from './modules/dom.js';

configurarLoggerStorage(adicionarLinhaTerminal);
configurarEntrada();

const importFile = getEl<HTMLInputElement>('import-file');
const initials = getEl<HTMLInputElement>('arcade-initials');
const actions = {
    openHelp, closeHelp, openSettings, closeSettings, openSaves, closeSaves,
    abrirModalConquistas, fecharModalConquistas, fecharArcadeModal, iniciarLoginESalvar,
    exportarSave, gerarLinkCompartilhamentoUI, ligarTV,
    escolherSave: () => importFile?.click(),
};
// Preserva a API usada por integrações do terminal.
Object.assign(window, { ...actions, executarAtalho, importarSave, playBip: reproduzirBeep });
for (const [name, action] of Object.entries(actions)) {
    document.querySelectorAll(`[data-acao="${name}"]`).forEach(el => el.addEventListener('click', action));
}
shortcuts.forEach(btn => btn.addEventListener('click', () => executarAtalho(btn.dataset.comando!)));
importFile?.addEventListener('change', importarSave);
initials?.addEventListener('input', () => {
    initials.value = initials.value.toUpperCase().replace(/[^A-Z]/g, '');
});
retryBoot?.addEventListener('click', iniciarJogo);

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
