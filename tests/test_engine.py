import pytest
import copy
from state import GameState
from data import MAPA_ORIGINAL

from villas_boas.actions import processar_comando

class DummyUI:
    def exibir(self, texto): pass
    def animar(self, texto, tempo=0, cor="", jogo=None): pass
    def pausar(self, segs): pass
    def limpar(self): pass


def test_processar_comando_alias(jogo_mock, mapa_mock):
   
    
    processar_comando("ir frente", jogo_mock, mapa_mock)
    
    assert jogo_mock.sala_atual == "sala de jantar"

def test_processar_comando_god_mode_tp(jogo_mock, mapa_mock):
    """Testa o comando de teletransporte exclusivo do GOD MODE."""
    jogo_mock.god_mode = True
    
    processar_comando("tp 01", jogo_mock, mapa_mock)
    
    assert jogo_mock.sala_atual == "01"

def test_processar_comando_combate_morte(jogo_mock, mapa_mock):
    
    jogo_mock.estado_atual = "COMBATE_ANIMATRONICO"
    jogo_mock.god_mode = False
    
    processar_comando("atacar", jogo_mock, mapa_mock)
    
    assert jogo_mock.estado_atual == "FIM"
    assert jogo_mock.sala_atual == "morte"