import { getEl } from './constants.js';

export const outputDiv = getEl<HTMLDivElement>('output');
export const inputField = getEl<HTMLInputElement>('comando');
export const terminal = getEl<HTMLElement>('terminal');
export const loadingSpinner = getEl<HTMLDivElement>('loading');
export const loadingLabel = getEl<HTMLSpanElement>('loading-label');
export const inputLineDiv = document.querySelector<HTMLDivElement>('.input-line');
export const retryBoot = getEl<HTMLButtonElement>('retry-boot');
export const shortcuts = document.querySelectorAll<HTMLButtonElement>('[data-comando]');
