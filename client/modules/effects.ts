import { mostrarLegendaDeAudio, tocarSusto, reproduzirBeep } from './audio.js';
import { getEl, SECRET_IMAGES, sleep } from './constants.js';

let flashInterval: ReturnType<typeof setInterval> | null = null;
let flashTimeout: ReturnType<typeof setTimeout> | null = null;

export const visualReduzido = () => document.body.classList.contains('terror-reduzido');

export function startFlashingSecrets(): void {
    const overlay = getEl<HTMLDivElement>("secret-image-overlay");
    const img = getEl<HTMLImageElement>("secret-img-display");
    if (!overlay || !img || visualReduzido()) return;

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

export function stopFlashingSecrets(): void {
    if (flashInterval) clearInterval(flashInterval);
    if (flashTimeout) clearTimeout(flashTimeout);
    flashInterval = null;
    flashTimeout = null;

    const overlay = getEl<HTMLDivElement>("secret-image-overlay");
    const img = getEl<HTMLImageElement>("secret-img-display");
    img?.classList.remove("flash-active");
    overlay?.classList.add("hidden");
}

export async function play1980Slideshow(): Promise<void> {
    const overlay = getEl<HTMLDivElement>("secret-image-overlay");
    const img = getEl<HTMLImageElement>("secret-img-display");
    if (!overlay || !img || visualReduzido()) return;

    overlay.classList.remove("hidden");
    overlay.classList.add("slideshow-mode");

    for (const src of SECRET_IMAGES) {
        if (visualReduzido()) break;
        img.src = `assets/secret images/${src}`;
        img.classList.add("slideshow-active");
        await sleep(2000);
        img.classList.remove("slideshow-active");
        await sleep(200);
    }

    overlay.classList.add("hidden");
    overlay.classList.remove("slideshow-mode");
}

export function ativarJumpscare(): void {
    const tela = getEl<HTMLDivElement>('tela-jumpscare');
    const img = getEl<HTMLImageElement>('img-jumpscare');
    if (!tela || !img) return;

    const imagens = ['./assets/static/images/JUMPSCARE1.jpg', './assets/static/images/JUMPSCARE2.jpg'];
    img.src = imagens[Math.floor(Math.random() * imagens.length)];

    tocarSusto('./assets/audio/grito.mp3');
    mostrarLegendaDeAudio('Grito repentino');
    if (visualReduzido()) return;
    tela.style.display = 'flex';
    setTimeout(() => {
        tela.style.display = 'none';
    }, 2500);
}

export function ativarBSOD(): void {
    const tela = getEl<HTMLDivElement>('tela-bsod');
    const imgCreepy = getEl<HTMLImageElement>('img-bsod-creepy');
    if (!tela || !imgCreepy || visualReduzido()) return;

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

export function aplicarEfeitoCRT(ativado: boolean): void {
    document.body.classList.toggle('crt-enabled', ativado);
}

