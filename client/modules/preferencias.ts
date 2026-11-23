import { definirVolume, definirVolumeJumpscare, definirSustosSilenciados } from './audio.js';
import { aplicarEfeitoCRT, stopFlashingSecrets } from './effects.js';
import { getEl } from './constants.js';

export let prefTelemetria = true;
export let prefMultiplicadorVelocidade = 1;

function atualizarLabelVelocidade(val: number): void {
    const label = getEl<HTMLSpanElement>('speed-val-display');
    if (!label) return;
    label.innerText = val === 0 ? "Instantâneo (0x)" : `Atual (${val}x)`;
}

export function carregarPreferencias(): void {
    for (const tipo of ['visual', 'som']) {
        const controles = document.querySelectorAll<HTMLInputElement>('[data-pref="' + tipo + '"]');
        const chave = 'vilasBoasTerror-' + tipo;
        function aplicar(valor: boolean) {
            controles.forEach(controle => { controle.checked = valor; });
            if (tipo === 'som') definirSustosSilenciados(valor);
            else {
                document.body.classList.toggle('terror-reduzido', valor);
                if (valor) {
                    stopFlashingSecrets();
                    document.body.classList.remove('blackout-mode', 'low-power-mode', 'kernel-panic-mode', 'glitch-mode');
                }
            }
        }
        const salvo = localStorage.getItem(chave);
        const movimento = window.matchMedia('(prefers-reduced-motion: reduce)');
        aplicar(salvo === null ? tipo === 'visual' && movimento.matches : salvo === 'true');
        if (tipo === 'visual') movimento.addEventListener('change', () => {
            if (localStorage.getItem(chave) === null) aplicar(movimento.matches);
        });
        controles.forEach(controle => controle.addEventListener('change', () => {
            localStorage.setItem(chave, String(controle.checked));
            aplicar(controle.checked);
        }));
    }
    const teleCheckbox = getEl<HTMLInputElement>('telemetry-checkbox');
    const speedSlider = getEl<HTMLInputElement>('speed-slider');
    const crtCheckbox = getEl<HTMLInputElement>('crt-checkbox');
    const volSlider = getEl<HTMLInputElement>('volume-slider');
    const volJumpSlider = getEl<HTMLInputElement>('volume-jump-slider');

    const volDisplay = getEl<HTMLSpanElement>('volume-val-display');
    const displayJump = getEl<HTMLSpanElement>('volume-jump-val-display');
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
        if (volDisplay) volDisplay.innerText = `${Math.round(vol * 100)}%`;
    }
    volSlider?.addEventListener('input', (e) => {
        const vol = parseFloat((e.target as HTMLInputElement).value);
        definirVolume(vol);
        localStorage.setItem('vilasBoasVolume', String(vol));
        if (volDisplay) volDisplay.innerText = `${Math.round(vol * 100)}%`;
    });

    if (savedVolumeJump !== null && volJumpSlider) {
        const volJump = parseFloat(savedVolumeJump);
        definirVolumeJumpscare(volJump);
        volJumpSlider.value = String(volJump);
        if (displayJump) displayJump.innerText = `${Math.round(volJump * 100)}%`;
    }
    volJumpSlider?.addEventListener('input', (e) => {
        const volJump = parseFloat((e.target as HTMLInputElement).value);
        definirVolumeJumpscare(volJump);
        localStorage.setItem('vilasBoasVolumeJump', String(volJump));
        if (displayJump) displayJump.innerText = `${Math.round(volJump * 100)}%`;

    });
}
