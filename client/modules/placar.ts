import { API_URL, getEl } from './constants.js';

export function mostrarTelaDePontuacao(segundosTotais: number, nomeDoFinal?: string, tipo?: string): void {
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

    const titulo = getEl('score-title');
    if (titulo) titulo.textContent = nomeDoFinal ? (tipo === 'alternativo' ? 'FINAL ALTERNATIVO' : 'VITÓRIA') : 'FIM DE JOGO';
    getEl('score-registration')?.classList.toggle('hidden', !nomeDoFinal);
    if (textoFinal && !nomeDoFinal) textoFinal.textContent = 'Você morreu. Retome o último autosave para tentar outra escolha.';
    if (textoFinal && nomeDoFinal) {
        textoFinal.innerText = nomeDoFinal.replace(/_/g, ' ').toUpperCase();
    }

    if (modal) {
        modal.classList.remove('hidden');
        titulo?.focus();
    }
}

export function fecharArcadeModal(): void {
    getEl<HTMLDivElement>('arcade-score-modal')?.classList.add('hidden');
    getEl('comando')?.focus();
}

export function iniciarLoginESalvar(): void {
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
