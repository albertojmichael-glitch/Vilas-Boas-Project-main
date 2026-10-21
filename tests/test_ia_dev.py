import random

import pytest
from state import SALAS_SEGURAS_PATRULHA, SALAS_TERMINAIS, GameState
from test_devmode import CODIGO, UIGravador
from villas_boas.engine import core


@pytest.fixture
def jogo_dev():
    j = GameState(ui_handler=UIGravador())
    core.processar_fluxo_jogo(CODIGO, j)
    j.sala_atual = "entrada"
    return j


def test_perseguidor_congelado_por_padrao_no_modo_dev(jogo_dev):
    jogo_dev.ai_sala, jogo_dev.nivel_barulho = "palco", 100
    for cmd in ["ir frente", "ir atrás"] * 3:
        core.processar_fluxo_jogo(cmd, jogo_dev)
    assert jogo_dev.ai_sala == "palco"


def test_dev_ia_on_solta_o_perseguidor(jogo_dev):
    """Regressão: com 'dev ia on' ele continuava parado no palco (god_mode congelava a IA)."""
    core.processar_fluxo_jogo("dev ia on", jogo_dev)
    jogo_dev.ai_sala, jogo_dev.nivel_barulho = "palco", 100
    random.seed(0)
    visitadas = set()
    for cmd in ["ir frente", "ir atrás"] * 8:
        core.processar_fluxo_jogo(cmd, jogo_dev)
        visitadas.add(jogo_dev.ai_sala)
    assert jogo_dev.ai_sala != "palco" or len(visitadas) > 1
    assert jogo_dev.nivel_barulho < 100 or len(visitadas) > 1  # o ruído volta a decair


def test_ia_roda_uma_unica_vez_por_comando_no_modo_dev(jogo_dev, monkeypatch):
    core.processar_fluxo_jogo("dev ia on", jogo_dev)
    chamadas = []
    original = GameState.avancar_ia_perseguidor
    monkeypatch.setattr(
        GameState,
        "avancar_ia_perseguidor",
        lambda self: (chamadas.append(1), original(self))[1],
    )
    core.processar_fluxo_jogo("ir frente", jogo_dev)
    assert len(chamadas) == 1


def test_dev_ia_off_volta_a_congelar(jogo_dev):
    core.processar_fluxo_jogo("dev ia on", jogo_dev)
    core.processar_fluxo_jogo("dev ia off", jogo_dev)
    jogo_dev.ai_sala, jogo_dev.nivel_barulho = "palco", 100
    core.processar_fluxo_jogo("ir frente", jogo_dev)
    assert jogo_dev.ai_sala == "palco"


def test_perseguidor_solto_te_alcanca_mas_o_modo_dev_protege(jogo_dev):
    core.processar_fluxo_jogo("dev ia on", jogo_dev)
    jogo_dev.ai_sala, jogo_dev.nivel_barulho = "sala de jantar", 100
    core.processar_fluxo_jogo("ir frente", jogo_dev)  # entra na sala dele
    assert jogo_dev.estado_atual == "JOGO" and jogo_dev.sala_atual != "morte"
    assert jogo_dev.ai_sala != jogo_dev.sala_atual


def test_patrulha_nunca_entra_em_sala_terminal_ou_segura():
    j = GameState(ui_handler=UIGravador())
    j.estado_atual, j.sala_atual = "JOGO", "entrada"
    origens = [
        nome
        for nome, sala in j.mapa.items()
        if isinstance(sala, dict)
        and any(
            v in SALAS_SEGURAS_PATRULHA for v in sala.values() if isinstance(v, str)
        )
    ]
    assert origens, "o mapa deveria ter salas que levam a salas terminais/seguras"
    random.seed(11)
    for _ in range(300):
        for origem in origens:
            j.ai_sala, j.nivel_barulho, j.estado_atual = origem, 0, "JOGO"
            j.avancar_ia_perseguidor()
            assert j.ai_sala not in SALAS_SEGURAS_PATRULHA or j.ai_sala == origem


def test_perseguidor_nao_fica_preso_em_partida_longa():
    j = GameState(ui_handler=UIGravador())
    j.estado_atual, j.sala_atual = "JOGO", "entrada"
    random.seed(5)
    visitadas = set()
    for _ in range(400):
        j.nivel_barulho, j.estado_atual = 0, "JOGO"
        j.avancar_ia_perseguidor()
        visitadas.add(j.ai_sala)
    assert len(visitadas) > 5 and not (visitadas & SALAS_TERMINAIS)


def test_recuo_do_perseguidor_no_modo_dev_nao_vai_para_sala_segura(jogo_dev):
    core.processar_fluxo_jogo("dev ia on", jogo_dev)
    for _ in range(30):
        jogo_dev.sala_atual, jogo_dev.ai_sala, jogo_dev.nivel_barulho = (
            "entrada",
            "sala de jantar",
            100,
        )
        jogo_dev.estado_atual = "COMBATE_ANIMATRONICO"
        from villas_boas.engine import devmode

        devmode._interceptar_perseguidor(jogo_dev)
        assert jogo_dev.ai_sala not in SALAS_SEGURAS_PATRULHA
        assert jogo_dev.estado_atual == "JOGO"
