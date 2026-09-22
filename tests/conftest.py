import pytest
import copy
import sys
import os


sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'server')))

from app import MEMORIA_SESSOES, app


@pytest.fixture
def jogo_base(mapa_mock):
    jogo = GameState() # Ou como estiver a sua inicialização
    jogo.sala_atual = "entrada"
    
    # ADICIONE ESTA LINHA PARA INJETAR O MAPA NO ESTADO DO JOGO:
    jogo.mapa = copy.deepcopy(mapa_mock)
    
    return jogo

@pytest.fixture
def jogo_mock(mapa_mock):
    # Faça exatamente a mesma coisa aqui, se houver uma fixture separada
    jogo = GameState()
    jogo.sala_atual = "entrada"
    jogo.mapa = copy.deepcopy(mapa_mock)
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