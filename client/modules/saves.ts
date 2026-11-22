import { pedirJSON, pedirTexto } from './api.js';
import { reproduzirBeep } from './audio.js';
import { getEl } from './constants.js';
import type { RespostaShare, SaveExportado } from './types.js';

export async function exportarSave(): Promise<void> {
    try {
        const texto = await pedirTexto('/save/export');
        if (!saveTemFormatoConhecido(JSON.parse(texto))) throw new Error("Backup inválido.");
        // O JSON assinado distingue 0 de 0.0. Preserve a representação do servidor.

        const blob = new Blob([texto], { type: 'application/json' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `vilas_boas_backup_${new Date().toISOString().slice(0, 10)}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);

        reproduzirBeep('sucesso');
    } catch (erro) {
        const msg = erro instanceof Error ? erro.message : "Erro desconhecido";
        console.error(erro);
        alert(`[ERRO] ${msg}`);
    }
}

function saveTemFormatoConhecido(dados: unknown): dados is SaveExportado {
    if (!dados || typeof dados !== 'object' || Array.isArray(dados)) return false;
    const d = dados as Record<string, unknown>;
    const v2 = d.formato === 'vilas-boas-save'
        && typeof d.assinatura === 'string'
        && !!d.estado && typeof d.estado === 'object';
    const v1 = typeof d.dados_seguros === 'string';
    return v2 || v1;
}

export async function importarSave(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = async (e: ProgressEvent<FileReader>) => {
        try {
            const conteudo = e.target?.result;
            if (typeof conteudo !== 'string') throw new Error("Falha ao ler arquivo.");

            // Editores no Windows costumam gravar um BOM no início do arquivo,
            // o que quebrava o JSON.parse ("o JS não reconhece o json").
            const texto = conteudo.replace(/^\uFEFF/, '').trim();
            let dados: unknown;
            try {
                dados = JSON.parse(texto);
            } catch {
                throw new Error("O arquivo não é um JSON válido (está truncado ou foi editado?).");
            }
            if (!saveTemFormatoConhecido(dados)) {
                throw new Error("Este arquivo não é um save do Vilas Boas OS.");
            }
            // A validação de verdade (assinatura e regras do jogo) é feita no servidor.

            await pedirJSON('/save/import', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: texto,
                credentials: 'include'
            });

            alert("Memórias injetadas com sucesso. A reiniciar o terminal...");
            window.location.reload();
        } catch (erro) {
            const msg = erro instanceof Error ? erro.message : "Arquivo inválido";
            alert(`[ERRO FATAL] O arquivo fornecido foi rejeitado.\nMotivo: ${msg}`);
        }
    };
    reader.readAsText(file);
    input.value = '';
}

export async function gerarLinkCompartilhamentoUI(): Promise<void> {
    try {
        const data = await pedirJSON<RespostaShare>('/share/generate', {
            method: 'POST',
            credentials: 'include'
        });
        const container = getEl<HTMLDivElement>('share-link-container');
        const input = getEl<HTMLInputElement>('share-link-input');
        const msg = getEl<HTMLParagraphElement>('share-link-msg');

        if (container && input && msg) {
            container.classList.remove('hidden');
            input.value = data.link;
            msg.innerText = data.mensagem;

            input.select();
            if (navigator.clipboard) {
                await navigator.clipboard.writeText(data.link);
            } else {
                document.execCommand('copy');
            }
        }
        reproduzirBeep('sucesso');
        alert("Link copiado para a área de transferência!");
    } catch (erro) {
        console.error(erro);
        alert(erro instanceof Error ? erro.message : "Falha ao compartilhar.");
    }
}
