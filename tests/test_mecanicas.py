import pytest
from state import GameState
from villas_boas.actions.parser import normalizar
from villas_boas.engine.minigames.minotauro import MinigameMinotauro
from villas_boas.engine.minigames.seguranca import MinigameSeguranca


@pytest.fixture
def jogo_mock():
    jogo = GameState()
    jogo.estado_atual = "JOGO"
    return jogo


def test_bfs_caminho_livre(jogo_mock):
    mapa_teste = {
        "A": {"frente": "B"},
        "B": {"atrás": "A", "direita": "C"},
        "C": {"esquerda": "B"},
    }
    jogo_mock.mapa = mapa_teste
    caminho = jogo_mock.calcular_caminho_bfs("A", "C")
    assert caminho == ["A", "B", "C"]


def test_bfs_bloqueio_rota(jogo_mock):
    mapa_teste = {"A": {"frente": "B"}, "B": {"atrás": "A"}, "C": {}}
    jogo_mock.mapa = mapa_teste
    caminho = jogo_mock.calcular_caminho_bfs("A", "C")
    assert caminho == []


def test_bfs_ignora_salas_terminais(jogo_mock):
    mapa_teste = {
        "A": {"frente": "B"},
        "B": {"direita": "morte", "frente": "C"},
        "C": {"atrás": "B"},
    }
    jogo_mock.mapa = mapa_teste
    caminho = jogo_mock.calcular_caminho_bfs("A", "morte")

    assert caminho == []


def test_normalizador_texto():
    assert normalizar("   PeGaR    ChAvE  ") == "pegar chave"
    assert normalizar("olhar!") == "olhar"
    assert normalizar("ir...   frente") == "ir frente"
    assert normalizar("1994") == "1994"


def test_minotauro_fluxo_vitoria(jogo_mock):
    minigame = MinigameMinotauro(jogo_mock)

    minigame.px, minigame.py = 0, 4
    jogo_mock.inventario.append("tesoura")

    minigame.processar_turno("cortar fios", jogo_mock)
    assert minigame.fios_cortados is True

    minigame.px, minigame.py = 0, 0
    resultado = minigame.processar_turno("sair", jogo_mock)
    assert resultado == "vitoria_minotauro"


def test_seguranca_ataque_rick(jogo_mock):
    minigame = MinigameSeguranca(jogo_mock)

    minigame.rick_pos = 4
    minigame.porta_fechada = False

    resultado = minigame.processar_turno("esperar", jogo_mock)
    assert resultado == "morte"


def test_seguranca_bloqueio_salva_jogador(jogo_mock):
    minigame = MinigameSeguranca(jogo_mock)

    minigame.rick_pos = 4
    minigame.porta_fechada = True

    resultado = minigame.processar_turno("esperar", jogo_mock)
    assert resultado == "continuar"
    assert minigame.rick_pos == 0
