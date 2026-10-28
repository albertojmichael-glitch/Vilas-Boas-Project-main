export type TipoBeep = 'sucesso' | 'erro';
export type CorTerminal = 'verde' | 'vermelho' | 'amarelo' | 'branco' | '';

export interface Conquista {
    id: string;
    nome: string;
    desc: string;
    icone: string;
}

export interface EstadoJogo {
    hp?: number | string;
    luz_restante?: number | string;
    som?: number;
    bolsas_coletadas?: number;
    inventario?: string[];
    estado_jogo?: string;
    tempo_final?: number;
    final_alcancado?: string;
    sala?: string;
}

export interface RespostaServidor {
    linhas: string[];
    estado?: EstadoJogo;
    novas_conquistas?: Conquista[];
    erro?: string;
}

export interface SavePayload<T> {
    __version__: number;
    timestamp: number;
    data: T;
}

export interface SaveExportado {
    dados_seguros: string;
    resumo: {
        sala: string;
        [key: string]: unknown;
    };
    [key: string]: unknown;
}

export interface RespostaShare {
    link: string;
    mensagem: string;
    erro?: string;
}

export interface DadosReplay {
    jogador: string;
    sala_final: string;
    log_comandos: string[];
    erro?: string;
}

declare global {
    interface Window {
        webkitAudioContext?: typeof AudioContext;
        colorBarOsc?: OscillatorNode;
        processarRespostaBackend?: (data: RespostaServidor) => void;
        openHelp: () => void;
        closeHelp: () => void;
        openSettings: () => void;
        closeSettings: () => void;
        openSaves: () => void;
        closeSaves: () => void;
        abrirModalConquistas: () => void;
        fecharModalConquistas: () => void;
        exportarSave: () => Promise<void>;
        importarSave: (event: Event) => Promise<void>;
        gerarLinkCompartilhamentoUI: () => Promise<void>;
        executarAtalho: (cmd: string) => void;
        ligarTV: () => void;
        iniciarLoginESalvar: () => void;
        fecharArcadeModal: () => void;
        playBip: (tipo?: TipoBeep) => void;
    }
}

