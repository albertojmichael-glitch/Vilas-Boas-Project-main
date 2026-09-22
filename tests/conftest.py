import pytest
import copy
import sys
import os



from state import GameState

sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'server')))

from app import MEMORIA_SESSOES, app


# ... (seus imports iniciais)

# 1. Crie esta classe falsa para capturar os textos:
class DummyUI:
    def __init__(self):
        self.buffer = []
    
    def exibir(self, msg):
        self.buffer.append(str(msg))
        
    def pausar(self, tempo):
        pass
        
    def limpar(self):
        pass

@pytest.fixture
def jogo_base(mapa_mock):
    jogo = GameState() 
    jogo.sala_atual = "entrada"
    jogo.mapa = copy.deepcopy(mapa_mock)
    
    # 2. Substitua o MagicMock() pelo DummyUI():
    jogo.ui_handler = DummyUI()
    
    return jogo

@pytest.fixture
def jogo_mock(mapa_mock):
    jogo = GameState()
    jogo.sala_atual = "entrada"
    jogo.mapa = copy.deepcopy(mapa_mock)
    
    # 3. Substitua aqui também:
    jogo.ui_handler = DummyUI()
    
    return jogo

@pytest.fixture
def mapa_mock():
    
    return {
        "entrada": {
            "descrição": "Sala inicial do teste",
            "frente": "sala de jantar",
            "itens": []
        },
        "sala de jantar": {
            "descrição": "Sala vizinha",
            "atrás": "entrada",
            "itens": []
        }
    }


@pytest.fixture
def cliente():
    
    
    app.config['TESTING'] = True
    
    with app.test_client() as client:
       
        yield client
        
        MEMORIA_SESSOES.clear()