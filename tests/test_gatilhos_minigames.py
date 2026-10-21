"""Regressão: gatilhos de minigame ficaram num bloco 'elif' duplicado e inalcançável do core."""

import pytest
from state import GameState
from test_devmode import UIGravador
from villas_boas.engine import core


def novo_jogo(sala, inventario=()):
    j = GameState(ui_handler=UIGravador())
    j.estado_atual, j.sala_atual = "JOGO", sala
    j.inventario = list(inventario)
    return j


def rodar(jogo, cmd):
    core.processar_fluxo_jogo(cmd, jogo)
    return jogo


@pytest.mark.parametrize("cmd", ["jogar jon", "jogar fome de jon"])
def test_jogar_jon_inicia_minigame(cmd):
    assert rodar(novo_jogo("sala de fliperamas"), cmd).estado_atual == "MINIGAME_JON"


def test_jogar_jon_so_funciona_no_fliperama():
    assert rodar(novo_jogo("entrada"), "jogar jon").estado_atual == "JOGO"


def test_consertos_exige_moeda_e_consome_a_ficha():
    sem = rodar(novo_jogo("sala de fliperamas"), "jogar consertos")
    assert sem.estado_atual == "JOGO" and "ficha" in sem.ui_handler.texto
    com = rodar(novo_jogo("sala de fliperamas", ["moeda velha"]), "jogar consertos")
    assert (
        com.estado_atual == "MINIGAME_CONSERTOS_CABECA"
        and "moeda velha" not in com.inventario
    )


def test_julgamento_exige_partitura():
    sem = rodar(novo_jogo("sala de fliperamas"), "jogar adivinha")
    assert sem.estado_atual == "JOGO"
    com = rodar(
        novo_jogo("sala de fliperamas", ["partitura rasgada"]), "jogar adivinha"
    )
    assert com.estado_atual == "MINIGAME_JULGAMENTO_Q1"


def test_cadeira_exige_cartao_de_seguranca():
    sem = rodar(novo_jogo("01"), "cadeira")
    assert sem.estado_atual == "JOGO" and "Cartão de Segurança" in sem.ui_handler.texto
    com = rodar(novo_jogo("01", ["cartao de seguranca"]), "cadeira")
    assert com.estado_atual == "MINIGAME_SEGURANCA"


def test_sala_de_energia_abre_o_minotauro():
    assert (
        rodar(novo_jogo("sala de energia"), "olhar").estado_atual
        == "MINIGAME_MINOTAURO"
    )


def test_cofre_abre_painel_com_as_pistas():
    j = rodar(novo_jogo("01"), "abrir cofre")
    assert j.estado_atual == "MINIGAME_COFRE"
    t = j.ui_handler.texto
    assert "[1º dígito]" in t and "senha de 4 dígitos" in t


def test_cofre_errar_mostra_tentativas_restantes():
    j = rodar(novo_jogo("01"), "abrir cofre")
    j.ui_handler.saida.clear()
    rodar(j, "0000")
    assert (
        "Tentativas restantes" in j.ui_handler.texto
        or j.estado_atual != "MINIGAME_COFRE"
    )
