"""Conteúdo narrativo compartilhado pelo terminal e pela edição web."""

import json

from data import (
    ARTE_FINAL_MEDIOCRE,
    ARTE_FINAL_NEUTRO,
    ARTE_FINAL_VERDADEIRO,
    ARTE_PIANO,
    ARTE_PORCO,
    ARTE_ROBO,
    BASE_DIR,
)
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET

CORES = {
    "branco": DOS_BRANCO,
    "verde": DOS_VERDE,
    "amarelo": DOS_AMARELO,
    "vermelho": DOS_VERMELHO,
    "reset": RESET,
}
ARTES = {
    "ARTE_FINAL_MEDIOCRE": ARTE_FINAL_MEDIOCRE,
    "ARTE_FINAL_NEUTRO": ARTE_FINAL_NEUTRO,
    "ARTE_FINAL_VERDADEIRO": ARTE_FINAL_VERDADEIRO,
    "ARTE_PIANO": ARTE_PIANO,
    "ARTE_PORCO": ARTE_PORCO,
    "ARTE_ROBO": ARTE_ROBO,
}


def carregar(nome):
    return json.loads((BASE_DIR / "data" / nome).read_text(encoding="utf-8"))


FINAIS = carregar("finais.pt-BR.json")
MINIGAMES = carregar("minigames.pt-BR.json")
PERGUNTAS = MINIGAMES["perguntas"]
FLIPERAMAS = {
    (maquina["sala"], comando): maquina
    for maquina in MINIGAMES["fliperamas"].values()
    for comando in maquina["comandos"]
}


def colorir(texto):
    for nome, codigo in CORES.items():
        texto = texto.replace("{" + nome + "}", codigo)
    return texto


def exibir_cena(etapas, jogo):
    ui = jogo.ui_handler
    for etapa in etapas:
        if "pausa" in etapa:
            ui.pausar(etapa["pausa"])
        elif "efeito" in etapa:
            ui.buffer.append(f"@@{etapa['efeito']}@@")
        elif "limpar" in etapa:
            ui.limpar()
        elif "arte" in etapa:
            arte = ARTES[etapa["arte"]]
            if etapa.get("espaco_fixo"):
                arte = arte.replace(" ", "\u00a0")
            ui.animar(f"{DOS_BRANCO}{arte}{RESET}", etapa["velocidade"], jogo=jogo)
        elif "velocidade" in etapa:
            ui.animar(
                colorir(etapa["texto"]),
                etapa["velocidade"],
                CORES.get(etapa.get("cor"), ""),
                jogo,
            )
        else:
            ui.exibir(colorir(etapa["texto"]))
