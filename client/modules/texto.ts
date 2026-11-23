import type { CorTerminal } from './types.js';

const entidades: Record<string, string> = {
    '&amp;': '&', '&lt;': '<', '&gt;': '>', '&quot;': '"', '&#x27;': "'",
};

interface Trecho {
    texto: string;
    cor: CorTerminal;
}

// O protocolo emite somente estes quatro spans e as entidades de html.escape.
export function trechosTerminal(linha: string): Trecho[] {
    const trechos: Trecho[] = [];
    let cor: CorTerminal = '';
    let inicio = 0;
    function adicionar(texto: string): void {
        if (!texto) return;
        trechos.push({ cor, texto: texto.replace(/&(?:amp|lt|gt|quot|#x27);/g, e => entidades[e]) });
    }
    for (const tag of linha.matchAll(/<span class="(verde|branco|vermelho|amarelo)">|<\/span>/g)) {
        // Um fechamento avulso também faz parte do texto recebido.
        if (!tag[1] && !cor) continue;
        adicionar(linha.slice(inicio, tag.index));
        cor = (tag[1] || '') as CorTerminal;
        inicio = tag.index! + tag[0].length;
    }
    adicionar(linha.slice(inicio));
    return trechos;
}
