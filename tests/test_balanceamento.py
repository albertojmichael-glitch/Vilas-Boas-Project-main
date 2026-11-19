"""Balanceamento do PESADELO (issue #7) e regras novas da 'Ela'.

Os lotes são determinísticos (semente por partida), então as taxas não oscilam
entre execuções: só mudam quando as regras ou o ``balance_config.json`` mudam.
"""

import json
import random

import pytest
from data import JSON_PATH, MAPA_ORIGINAL
from state import GameState
from villas_boas import ela
from villas_boas.engine.core import processar_fluxo_jogo
from villas_boas.engine.minigames.seguranca import MinigameSeguranca
from villas_boas.simulacao.simulador import rodar_lote

META_PESADELO = (0.15, 0.30)
PARTIDAS_CI = 2000
PARTIDAS_PESADAS = 20000


@pytest.fixture(scope="module")
def relatorios():
    return {
        (dif, est): rodar_lote(dif, est, PARTIDAS_CI)
        for dif in ("NORMAL", "PESADELO")
        for est in ("competente", "aleatoria")
    }


# --------------------------------------------------------------------------
# Lotes de milhares de partidas
# --------------------------------------------------------------------------


def test_pesadelo_vencivel_mas_dificil(relatorios):
    rel = relatorios[("PESADELO", "competente")]
    assert META_PESADELO[0] <= rel.taxa_vitoria <= META_PESADELO[1], rel


def test_normal_bem_mais_facil_que_pesadelo(relatorios):
    normal = relatorios[("NORMAL", "competente")].taxa_vitoria
    pesadelo = relatorios[("PESADELO", "competente")].taxa_vitoria
    assert 0.55 <= normal <= 0.80
    assert normal >= pesadelo + 0.15


@pytest.mark.parametrize("dif", ["NORMAL", "PESADELO"])
@pytest.mark.parametrize("est", ["competente", "aleatoria"])
def test_sem_travamentos_nem_excecoes(relatorios, dif, est):
    rel = relatorios[(dif, est)]
    assert rel.erros == {}, rel.erros
    assert rel.travamentos == 0


def test_sem_morte_por_dano_aleatorio_extremo(relatorios):
    for (_dif, est), rel in relatorios.items():
        if est != "competente":
            continue
        # Tropeço no escuro e dica falsa do Jon não matam mais um jogador atento.
        assert rel.causas_morte.get("tropeco", 0) == 0
        assert rel.causas_morte.get("jon", 0) == 0
        causa, fracao = rel.causa_dominante()
        assert fracao <= 0.85, (causa, fracao)


def test_pesadelo_tem_mais_de_um_jeito_de_morrer(relatorios):
    causas = relatorios[("PESADELO", "competente")].causas_morte
    assert causas.get("ela", 0) > 0
    assert causas.get("seguranca", 0) > 0


@pytest.mark.slow
@pytest.mark.parametrize("dif", ["NORMAL", "PESADELO"])
def test_lote_pesado(dif):
    rel = rodar_lote(dif, "competente", PARTIDAS_PESADAS, seed_inicial=100_000)
    assert rel.erros == {}
    assert rel.travamentos == 0
    if dif == "PESADELO":
        assert META_PESADELO[0] <= rel.taxa_vitoria <= META_PESADELO[1]
    else:
        assert rel.taxa_vitoria >= 0.55


# --------------------------------------------------------------------------
# Regras novas
# --------------------------------------------------------------------------


def _jogo(dif="PESADELO", sala="duas salas de festas"):
    jogo = GameState()
    jogo.configurar_dificuldade(dif)
    jogo.estado_atual = "JOGO"
    jogo.sala_atual = sala
    jogo.turnos_luz = 99
    return jogo


def _textos(jogo):
    return "\n".join(jogo.ui_handler.buffer)


def test_mapa_do_jogador_nao_mudou():
    with open(JSON_PATH, encoding="utf-8") as f:
        original = json.load(f)["mapa_original"]
    assert MAPA_ORIGINAL == original
    # A sala de fliperamas continua com uma única saída (o beco faz parte do jogo).
    assert original["sala de fliperamas"]["direita"] == "sala 1"
    # Os dutos são só da 'Ela': nenhuma sala do jogador ganhou saída nova.
    for sala, dados in GameState().mapa.items():
        assert dados == original[sala]


def test_ela_existe_no_normal_com_ia_mais_fraca():
    normal, pesadelo = _jogo("NORMAL"), _jogo("PESADELO")
    assert ela.ela_ativa_no_modo(normal) and ela.ela_ativa_no_modo(pesadelo)
    pn, pp = normal.perfil_ela, pesadelo.perfil_ela
    assert pn["turnos_dormencia"] > pp["turnos_dormencia"]
    assert pn["chance_rastrear"] < pp["chance_rastrear"]
    assert pn["chance_usar_duto"] < pp["chance_usar_duto"]


def test_ela_comeca_dormindo():
    jogo = _jogo()
    jogo.posicao_perseguidor = "sala 1"
    jogo.sala_atual = "sala 1"
    dormencia = jogo.ela_dormencia
    assert dormencia > 0
    for _ in range(dormencia - 1):
        ela.processar_ela(jogo, jogo.ui_handler, barulho_alto=False)
        assert jogo.estado_atual == "JOGO"
    ela.processar_ela(jogo, jogo.ui_handler, barulho_alto=False)
    assert "Algo despertou" in _textos(jogo)


def test_aviso_direcional_de_passos():
    jogo = _jogo(sala="duas salas de festas")
    jogo.ela_dormencia = 0
    jogo.perfil_ela = {**jogo.perfil_ela, "chance_mover": 0.0, "chance_rastrear": 0.0}
    jogo.posicao_perseguidor = "sala de jantar"
    ela.processar_ela(jogo, jogo.ui_handler, barulho_alto=False)
    assert "na sala ao lado vindo da atrás" in _textos(jogo)


def test_duto_some_por_um_turno_e_reaparece_perto_com_aviso():
    jogo = _jogo(sala="sala de jantar")
    jogo.ela_dormencia = 0
    jogo.perfil_ela = {
        **jogo.perfil_ela,
        "chance_rastrear": 1.0,
        "chance_usar_duto": 1.0,
    }
    jogo.posicao_perseguidor = "sala 1"
    random.seed(3)
    ela.processar_ela(jogo, jogo.ui_handler, barulho_alto=True)
    assert jogo.ela_no_duto
    saida = jogo.ela_saida_duto
    assert saida in ela.DUTOS_ELA
    assert 1 <= ela._distancias(jogo.mapa, "sala 1")[saida] <= 2
    assert "grade" in _textos(jogo)
    ela.processar_ela(jogo, jogo.ui_handler, barulho_alto=True)
    assert not jogo.ela_no_duto
    assert jogo.posicao_perseguidor == saida


def test_duto_nunca_sai_em_sala_sem_grade():
    jogo = _jogo()
    for origem in ela.DUTOS_ELA:
        for saida in ela.saidas_de_duto(jogo, origem):
            assert saida in ela.DUTOS_ELA
    for sem_grade in ("sala de fliperamas", "01", "04", "balcão"):
        assert sem_grade not in ela.DUTOS_ELA


def _encontro_com_ela(jogo):
    jogo.ela_dormencia = 0
    jogo.perfil_ela = {**jogo.perfil_ela, "chance_mover": 0.0, "chance_rastrear": 0.0}
    jogo.posicao_perseguidor = jogo.sala_atual
    ela.processar_ela(jogo, jogo.ui_handler, barulho_alto=False)


def test_encontro_com_ela_vira_combate_e_correr_escapa_com_dano():
    jogo = _jogo(sala="duas salas de festas")
    hp = jogo.hp
    _encontro_com_ela(jogo)
    assert jogo.estado_atual == "COMBATE_ANIMATRONICO"
    assert jogo.atacante_combate == "ela"
    processar_fluxo_jogo("correr atrás", jogo)
    assert jogo.sala_atual == "sala de jantar"
    assert jogo.hp == hp - 1
    assert jogo.ela_atordoada > 0 or jogo.estado_atual == "JOGO"


def test_hesitar_diante_dela_mata():
    jogo = _jogo(sala="duas salas de festas")
    _encontro_com_ela(jogo)
    processar_fluxo_jogo("olhar", jogo)
    assert jogo.sala_atual == "morte"
    assert jogo.causa_morte == "ela"


def test_esconder_evita_a_ela():
    jogo = _jogo(sala="balcão")
    jogo.escondido = True
    _encontro_com_ela(jogo)
    assert jogo.estado_atual == "JOGO"
    assert jogo.posicao_perseguidor != "balcão"


def test_comando_esconder_tem_limite_e_exige_esconderijo():
    jogo = _jogo(sala="balcão")
    jogo.ela_dormencia = 99
    for _ in range(3):
        processar_fluxo_jogo("esconder", jogo)
    assert jogo.turnos_escondido == 3
    processar_fluxo_jogo("esconder", jogo)
    assert "não aguenta mais" in _textos(jogo)
    jogo2 = _jogo(sala="sala de jantar")
    processar_fluxo_jogo("esconder", jogo2)
    assert "Não há onde se esconder" in _textos(jogo2)


def test_ela_nao_acampa_na_porta_da_sala_segura():
    jogo = _jogo(sala="01")
    jogo.ela_dormencia = 0
    jogo.perfil_ela = {**jogo.perfil_ela, "chance_mover": 1.0, "chance_usar_duto": 0}
    jogo.posicao_perseguidor = "corredor"
    random.seed(1)
    posicoes = set()
    for _ in range(10):
        processar_fluxo_jogo("esperar", jogo)
        posicoes.add(jogo.posicao_perseguidor)
    assert jogo.estado_atual == "JOGO"
    assert len(posicoes) > 1


def test_animatronico_nao_entra_na_sala_segura():
    jogo = _jogo(sala="01")
    jogo.ai_sala = "corredor"
    jogo.nivel_barulho = 100
    for _ in range(5):
        jogo.avancar_ia_perseguidor()
        assert jogo.ai_sala != "01"


def test_tropeco_no_escuro_nao_mata(monkeypatch):
    from villas_boas.actions import movement

    monkeypatch.setattr(movement.random, "randint", lambda a, b: 1)
    jogo = _jogo(sala="sala de jantar")
    jogo.turnos_luz = 0
    jogo.hp = 1
    jogo.ela_dormencia = 99
    processar_fluxo_jogo("ir frente", jogo)
    assert jogo.hp == 1
    assert jogo.sala_atual != "morte"


def test_tropeco_nunca_acontece_em_sala_iluminada(monkeypatch):
    from villas_boas.actions import movement

    monkeypatch.setattr(movement.random, "randint", lambda a, b: 1)
    jogo = _jogo(sala="hall de entrada")
    jogo.turnos_luz = 0
    jogo.ela_dormencia = 99
    processar_fluxo_jogo("ir esquerda", jogo)
    assert jogo.sala_atual == "mesas de jantar"


def test_dica_falsa_do_jon_nao_tira_vida():
    jogo = _jogo(sala="sala de fliperamas")
    processar_fluxo_jogo("jogar jon", jogo)
    jogo.jon_dica_falsa = True
    errada = next(d for d in "fed" if d != jogo.jon_caminho_certo[0])
    hp = jogo.hp
    processar_fluxo_jogo(errada, jogo)
    assert jogo.hp == hp
    assert jogo.estado_atual == "JOGO"


def test_morte_no_jon_nao_quebra_mais_a_telemetria():
    jogo = _jogo(sala="sala de fliperamas")
    processar_fluxo_jogo("jogar jon", jogo)
    jogo.jon_dica_falsa = False
    jogo.hp = 1
    errada = next(d for d in "fed" if d != jogo.jon_caminho_certo[0])
    processar_fluxo_jogo(errada, jogo)  # antes: TypeError
    assert jogo.estado_atual == "FIM"
    assert jogo.causa_morte == "jon"


def test_esperar_ate_as_seis_vence_a_noite():
    jogo = _jogo(sala="01")
    jogo.god_mode = True
    mg = MinigameSeguranca(jogo)
    mg.turno = 23
    assert mg.processar_turno("esperar", jogo) == "vitoria_seguranca"
    assert jogo.noite_vencida


def test_reiniciar_restaura_lanterna_luz_e_perseguidores():
    jogo = _jogo()
    jogo.inventario = []
    jogo.turnos_luz = 0
    jogo.posicao_perseguidor = "corredor"
    jogo.ai_sala = "corredor"
    jogo.nivel_barulho = 100
    jogo.conquistas = ["primeira_morte"]
    jogo.estado_atual = "FIM"
    processar_fluxo_jogo("f5", jogo)
    novo = GameState()
    assert jogo.inventario == ["lanterna"]
    assert jogo.turnos_luz == novo.turnos_luz
    assert jogo.posicao_perseguidor == novo.posicao_perseguidor
    assert jogo.ai_sala == novo.ai_sala
    assert jogo.nivel_barulho == 0
    assert jogo.conquistas == ["primeira_morte"]
