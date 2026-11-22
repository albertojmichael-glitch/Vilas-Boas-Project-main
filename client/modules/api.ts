import { API_URL } from './constants.js';

export async function pedirTexto(rota: string, opcoes: RequestInit = {}, aoDemorar?: () => void): Promise<string> {
    const controller = new AbortController();
    const aviso = aoDemorar && setTimeout(aoDemorar, 2500);
    const limite = setTimeout(() => controller.abort(), 15000);
    try {
        const resposta = await fetch(API_URL + rota, {
            ...opcoes, credentials: 'include', signal: controller.signal,
        });
        const texto = await resposta.text();
        if (!resposta.ok) {
            if (resposta.status >= 500) throw new Error('Servidor indisponível. Tente novamente em instantes.');
            if (resposta.status === 429) throw new Error('Muitos pedidos. Aguarde antes de tentar novamente.');
            let detalhe: string | undefined;
            try { detalhe = JSON.parse(texto)?.erro; } catch { /* Alguns proxies respondem em texto. */ }
            throw new Error(typeof detalhe === 'string' ? detalhe : `Pedido recusado pelo servidor (HTTP ${resposta.status}).`);
        }
        return texto;
    } catch (erro) {
        if (controller.signal.aborted) throw new Error('O servidor demorou demais para responder.');
        if (erro instanceof TypeError) throw new Error('Sem conexão com o servidor. Verifique sua internet.');
        throw erro;
    } finally {
        clearTimeout(aviso);
        clearTimeout(limite);
    }
}

export async function pedirJSON<T>(rota: string, opcoes: RequestInit = {}, aoDemorar?: () => void): Promise<T> {
    const texto = await pedirTexto(rota, opcoes, aoDemorar);
    try {
        return JSON.parse(texto) as T;
    } catch {
        throw new Error('O servidor enviou uma resposta inválida.');
    }
}
