import pytest
import copy
from state import GameState
from data import MAPA_ORIGINAL
from villas_boas.actions.movement import cmd_ir
from villas_boas.actions.inventory import cmd_pegar, cmd_usar


class DummyUI:
    def exibir(self, texto): pass
    def animar(self, texto, tempo=0, cor="", jogo=None): pass
    def pausar(self, segs): pass
    def limpar(self): pass

@pytest.fixture
def jogo_mock():
    jogo = GameState()
    jogo.ui_handler = DummyUI()
    jogo.sala_atual = "entrada"
    jogo.inventario = []
    return jogo

@pytest.fixture
def mapa_mock():
    return copy.deepcopy(MAPA_ORIGINAL)

def test_cmd_ir_valido(jogo_mock, mapa_mock):
    """Testa se o jogador consegue ir da entrada para a sala de jantar."""
    sucesso = cmd_ir("ir frente", jogo_mock, mapa_mock)
    
    assert sucesso is True
    assert jogo_mock.sala_atual == "sala de jantar"

def test_cmd_ir_invalido(jogo_mock, mapa_mock):
    """Testa o bloqueio ao tentar ir para uma direção que não existe na sala."""
    jogo_mock.sala_atual = "entrada"
    sucesso = cmd_ir("ir norte_inventado", jogo_mock, mapa_mock)
    
    
    assert sucesso is False
    assert jogo_mock.sala_atual == "entrada" 

def test_cmd_pegar_sucesso(jogo_mock, mapa_mock):
    """Testa se o jogador consegue pegar um item do chão."""
    jogo_mock.sala_atual = "entrada"
    mapa_mock["entrada"]["itens"] = ["tabua pequena de madeira"]
    
    sucesso = cmd_pegar("pegar tabua", jogo_mock, mapa_mock)
    
    assert sucesso is True
    assert "tabua pequena de madeira" in jogo_mock.inventario
    assert "tabua pequena de madeira" not in mapa_mock["entrada"]["itens"]

def test_cmd_usar_bateria(jogo_mock, mapa_mock):
    """Testa se o uso de um item altera o status do GameState corretamente."""
    jogo_mock.inventario = ["bateria nova"]
    jogo_mock.turnos_luz = 1
    
    sucesso = cmd_usar("usar bateria", jogo_mock, mapa_mock)
    
    assert sucesso is True
    assert jogo_mock.turnos_luz == 12 
    assert "bateria nova" not in jogo_mock.inventario 