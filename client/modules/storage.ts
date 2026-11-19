import { SAVE_VERSION } from './constants.js';
import type { CorTerminal, SavePayload } from './types.js';

export type LogTerminalFn = (texto: string, cor?: CorTerminal) => void;

let logCallback: LogTerminalFn | null = null;

export function configurarLoggerStorage(fn: LogTerminalFn): void {
    logCallback = fn;
}

function jsonValido(texto: string): boolean {
    try {
        JSON.parse(texto);
        return true;
    } catch {
        return false;
    }
}

/**
 * Persistência local de preferências e conquistas cosméticas.
 * Não há cifra nem assinatura aqui de propósito: o jogador controla o navegador.
 * O estado do jogo vive (cifrado e validado) no servidor.
 */
export const SafeStorage = {
    save<T>(key: string, data: T): void {
        try {
            const payload: SavePayload<T> = {
                __version__: SAVE_VERSION,
                timestamp: Date.now(),
                data
            };
            const jsonString = JSON.stringify(payload);
            // Confere o JSON antes de gravar: nada corrompido entra no disco.
            JSON.parse(jsonString);

            // Só promove o valor antigo a backup se ele estiver íntegro; senão o
            // último backup bom seria trocado por lixo e o progresso se perderia.
            const oldData = localStorage.getItem(key);
            if (oldData && jsonValido(oldData)) localStorage.setItem(`${key}_backup`, oldData);
            localStorage.setItem(key, jsonString);
        } catch (e) {
            console.error("[SISTEMA] Erro crítico de I/O no disco local:", e);
        }
    },

    load<T>(key: string, defaultValue: T): T {
        try {
            const raw = localStorage.getItem(key);
            if (!raw) return defaultValue;

            if (!raw.includes('__version__')) {
                const parsed = JSON.parse(raw) as T;
                this.save(key, parsed);
                return parsed;
            }

            const parsed = JSON.parse(raw) as SavePayload<T>;
            if (parsed.__version__ < SAVE_VERSION) {
                console.warn(`[SISTEMA] Atualizando save do formato ${parsed.__version__} para ${SAVE_VERSION}`);
            }
            return parsed.data;
        } catch {
            console.warn(`[SISTEMA] Setor corrompido em ${key}. Iniciando protocolo de recuperação...`);
            return this.recover(key, defaultValue);
        }
    },

    recover<T>(key: string, defaultValue: T): T {
        try {
            const rawBackup = localStorage.getItem(`${key}_backup`);
            if (!rawBackup) throw new Error("Fita de backup inexistente.");

            localStorage.setItem(key, rawBackup);
            logCallback?.("[ AVISO ] Anomalia de memória detectada. Backup de segurança restaurado.", "amarelo");

            const parsed = JSON.parse(rawBackup);
            return (parsed.data !== undefined ? parsed.data : parsed) as T;
        } catch {
            logCallback?.(`[ FATAL ] Perda de dados irreversível no setor: ${key}.`, "vermelho");
            return defaultValue;
        }
    }
};

