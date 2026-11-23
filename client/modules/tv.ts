import { sustosAtivos, conectarSaida, iniciarSomAmbiente, obterAudioContext, reproduzirBeep } from './audio.js';
import { getEl } from './constants.js';
import { iniciarJogo } from './jogo.js';

export function ligarTV(): void {
    const powerBtn = getEl<HTMLButtonElement>("power-button");
    const powerLed = getEl<HTMLDivElement>("power-led");
    if (powerBtn) powerBtn.disabled = true;
    powerLed?.classList.add("led-on");

    iniciarSomAmbiente();
    if (sustosAtivos()) reproduzirBeep('erro');

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
        if (ctx && sustosAtivos()) {
            window.colorBarOsc = ctx.createOscillator();
            window.colorBarOsc.type = 'sine';
            window.colorBarOsc.frequency.value = 1000;
            conectarSaida(window.colorBarOsc);
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
