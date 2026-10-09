#olamundo
import pytest
import copy
import sys
import os
os.environ.setdefault("DATABASE_URL", "sqlite://")





caminho_server = os.path.abspath(os.path.join(os.path.dirname(__file__), '../server'))
sys.path.insert(0, caminho_server)

from state import GameState

from app import app
from extensions import MEMORIA_SESSOES



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
    
    
    jogo.ui_handler = DummyUI()
    
    return jogo

@pytest.fixture
def jogo_mock(mapa_mock):
    jogo = GameState()
    jogo.sala_atual = "entrada"
    jogo.mapa = copy.deepcopy(mapa_mock)
    
   
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

@pytest.fixture
def app_ctx():
    from app import app
    from extensions import db
    with app.app_context():
        db.create_all()
        yield
        db.session.remove()
        db.drop_all()