import { reproduzirBeep } from './audio.js';
import { CATALOGO_CONQUISTAS, getEl } from './constants.js';
import { SafeStorage } from './storage.js';
import type { Conquista } from './types.js';

let toastTimeout: ReturnType<typeof setTimeout> | undefined;
export const modal = getEl<HTMLDivElement>('achievements-modal');
const grid = getEl<HTMLDivElement>('achievements-grid');
const title = getEl<HTMLHeadingElement>('achievements-title');

export function abrirModalConquistas(): void {
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

export function fecharModalConquistas(): void {
    modal?.classList.add('hidden');
}

export function processarConquistas(listaConquistas?: Conquista[]): void {
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
