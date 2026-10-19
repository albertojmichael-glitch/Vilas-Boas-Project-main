import copy
import os
import sys

import pytest

# 1. FORÇA AS VARIÁVEIS DE AMBIENTE ANTES DE IMPORTAR QUALQUER COISA DO JOGO
os.environ["DATABASE_URL"] = "sqlite://"
os.environ["DEV_MODE_ENABLED"] = "1"

caminho_server = os.path.abspath(os.path.join(os.path.dirname(__file__), "../server"))
sys.path.insert(0, caminho_server)


class DummyUI:
    def __init__(self):
        self.buffer = []

    def exibir(self, msg):
        self.buffer.append(str(msg))

    def pausar(self, tempo):
        pass

    def limpar(self):
        pass


@pytest.fixture(autouse=True)
def app_ctx():
    # Importações locais: O Ruff não reclama e garante que o ambiente já está configurado
    from app import app
    from extensions import db

    with app.app_context():
        db.create_all()
        yield
        db.session.remove()
        db.drop_all()


@pytest.fixture
def cliente():
    from app import app
    from extensions import MEMORIA_SESSOES

    app.config["TESTING"] = True
    with app.test_client() as client:
        yield client
        MEMORIA_SESSOES.clear()


@pytest.fixture
def mapa_mock():
    return {
        "entrada": {
            "descrição": "Sala inicial do teste",
            "frente": "sala de jantar",
            "itens": [],
        },
        "sala de jantar": {
            "descrição": "Sala vizinha",
            "atrás": "entrada",
            "itens": [],
        },
    }


@pytest.fixture
def jogo_mock():
    from state import GameState

    jogo = GameState()
    jogo.sala_atual = "entrada"
    jogo.ui_handler = DummyUI()
    return jogo


@pytest.fixture
def jogo_base(mapa_mock):
    from state import GameState

    jogo = GameState()
    jogo.sala_atual = "entrada"
    jogo.mapa = copy.deepcopy(mapa_mock)
    jogo.ui_handler = DummyUI()
    return jogo
