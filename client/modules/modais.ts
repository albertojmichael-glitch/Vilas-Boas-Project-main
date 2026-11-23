import { getEl } from './constants.js';
import { inputField } from './dom.js';

function modal(id: string) {
    const elemento = getEl(id);
    return {
        abrir: () => elemento?.classList.remove('hidden'),
        fechar: () => {
            elemento?.classList.add('hidden');
            inputField?.focus();
        },
    };
}

export const { abrir: openHelp, fechar: closeHelp } = modal('help-modal');
export const { abrir: openSaves, fechar: closeSaves } = modal('saves-modal');
export const { abrir: openSettings, fechar: closeSettings } = modal('settings-modal');
