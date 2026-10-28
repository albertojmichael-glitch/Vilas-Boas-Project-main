import type { Conquista } from './types.js';

export const API_URL = "https://vilas-boas-project-main.up.railway.app";
export const SAVE_VERSION = 1.0;
export const LIMITE_LINHAS = 50;

export const SECRET_IMAGES: readonly string[] = [
    "hidden image 19.png", "hidden image 20.png", "hidden image 21.png",
    "HiddenImage1.webp", "HiddenImage2.webp", "HiddenImage3.webp",
    "HiddenImage4.webp", "HiddenImage5.webp", "HiddenImage6.webp",
    "HiddenImage7.webp", "HiddenImage8.webp", "HiddenImage10.webp",
    "HiddenImage11.webp", "HiddenImage12.webp", "HiddenImage14.webp",
    "HiddenImage15.webp", "HiddenImage16.webp", "HiddenImage17.webp",
    "HiddenImage18.webp", "HiddenImage20.webp", "mimic.webp"
];

export const CATALOGO_CONQUISTAS: readonly Conquista[] = [
    { id: "primeira_morte", nome: "Sangue no Carpete", desc: "Bem-vindo ao Vilas Boas.", icone: "☠" },
    { id: "mente_brilhante", nome: "Mente Brilhante", desc: "Abra o cofre na primeira tentativa.", icone: "★" },
    { id: "labirinto", nome: "Labirinto", desc: "Sobreviva à Sala de Energia.", icone: "☄" },
    { id: "acumulador", nome: "Acumulador", desc: "Encha todos os 9 espaços do inventário.", icone: "☑" },
    { id: "glicose", nome: "Glicose Mortal", desc: "Coma o doce velho encontrado no chão.", icone: "✴" },
    { id: "trapaceiro", nome: "Hacker", desc: "Ative o God Mode.", icone: "☣" },
    { id: "normal_zerado", nome: "Fim do Expediente", desc: "Sobreviva à noite na dificuldade Normal.", icone: "♨" },
    { id: "final_bom", nome: "Sobrevivente", desc: "Alcance o Final Neutro.", icone: "☀" },
    { id: "final_verdadeiro", nome: "A Verdade", desc: "Alcance o Final Verdadeiro.", icone: "✦" },
    { id: "pesadelo_neutro", nome: "Pesadelo Superado", desc: "Alcance o Final Neutro no modo PESADELO.", icone: "♛" },
    { id: "pesadelo_verdadeiro", nome: "Mestre do Pesadelo", desc: "Alcance o Final Verdadeiro no modo PESADELO.", icone: "♚" }
];

export const getEl = <T extends HTMLElement>(id: string): T | null =>
    document.getElementById(id) as T | null;

export const sleep = (ms: number): Promise<void> =>
    new Promise(resolve => setTimeout(resolve, ms));

