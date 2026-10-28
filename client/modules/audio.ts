import { getEl } from './constants.js';
import type { TipoBeep } from './types.js';

let prefVolume = 1.0;
let prefVolumeJumpscare = 1.0;
let legendaTimeout: ReturnType<typeof setTimeout> | undefined;
let audioAmbienteLoop: HTMLAudioElement | null = null;
let sonsRandomicosAtivos = false;
let audioCtx: AudioContext | null = null;
let masterGainNode: GainNode | null = null;
let crtOsc: OscillatorNode | null = null;

export function obterVolume(): number {
    return prefVolume;
}

export function definirVolume(val: number): void {
    prefVolume = val;
    if (masterGainNode) masterGainNode.gain.value = prefVolume;
    if (audioAmbienteLoop) audioAmbienteLoop.volume = prefVolume * 0.6;
}

export function obterVolumeJumpscare(): number {
    return prefVolumeJumpscare;
}

export function definirVolumeJumpscare(val: number): void {
    prefVolumeJumpscare = val;
}

export function obterAudioContext(): AudioContext | null {
    if (!audioCtx) {
        const AudioContextClass = window.AudioContext || window.webkitAudioContext;
        if (AudioContextClass) {
            audioCtx = new AudioContextClass();
            masterGainNode = audioCtx.createGain();
            masterGainNode.gain.value = prefVolume;
            masterGainNode.connect(audioCtx.destination);
        }
    }
    if (audioCtx && audioCtx.state === 'suspended') {
        audioCtx.resume();
    }
    return audioCtx;
}

export function mostrarLegendaDeAudio(texto: string): void {
    const legendaDiv = getEl<HTMLDivElement>('audio-captions');
    if (legendaDiv) {
        legendaDiv.textContent = `[ Áudio: ${texto} ]`;
        legendaDiv.classList.remove('hidden');

        clearTimeout(legendaTimeout);
        legendaTimeout = setTimeout(() => {
            legendaDiv.classList.add('hidden');
        }, 2000);
    }
}

export function tocarSomDigito(): void {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode) return;

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

export function iniciarSomAmbiente(): void {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode) return;

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
        audioAmbienteLoop.volume = prefVolume * 0.6;
        audioAmbienteLoop.play().catch(err => console.log("Autoplay bloqueado:", err));
    }

    if (!sonsRandomicosAtivos) {
        sonsRandomicosAtivos = true;
        tocarSonsAssustadoresAleatorios();
    }
}

function tocarSonsAssustadoresAleatorios(): void {
    const tempoAleatorio = Math.floor(Math.random() * (80000 - 30000 + 1)) + 30000;

    setTimeout(() => {
        const audios = ['./assets/audio/sonsrandomicos.mp3', './assets/audio/sonsrandomicos2.mp3'];
        const escolhido = audios[Math.floor(Math.random() * audios.length)];

        const audioSusto = new Audio(escolhido);
        audioSusto.volume = prefVolume * 0.8;
        audioSusto.play().catch(e => console.log("Erro som randomico:", e));

        tocarSonsAssustadoresAleatorios();
    }, tempoAleatorio);
}

export function tocarBipEntrada(): void {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode) return;

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

export function tocarPassoMetalico(): void {
    const passoAudio = new Audio('./assets/audio/metalrangendo.mp3');
    passoAudio.volume = prefVolume;
    passoAudio.play().catch(e => console.log("Erro ao tocar passo:", e));
    mostrarLegendaDeAudio("Passos metálicos pesados");
}

export function tocarPorta(): void {
    const portaAudio = new Audio('./assets/audio/portafechandoabrindo.mp3');
    portaAudio.volume = prefVolume;
    portaAudio.play().catch(e => console.log(e));
    mostrarLegendaDeAudio("Porta rangendo");
}

export function reproduzirBeep(tipo: TipoBeep = 'sucesso'): void {
    const ctx = obterAudioContext();
    if (!ctx || !masterGainNode) return;

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

