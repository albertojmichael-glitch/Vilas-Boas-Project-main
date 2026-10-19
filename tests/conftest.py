import copy
import os
import sys

import pytest

os.environ["DATABASE_URL"] = "sqlite://"
os.environ["DEV_MODE_ENABLED"] = "1"

caminho_server = os.path.abspath(os.path.join(os.path.dirname(__file__), "../server"))
sys.path.insert(0, caminho_server)

from app import app  # noqa: E402
from extensions import MEMORIA_SESSOES, db  # noqa: E402
from state import GameState  # noqa: E402


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
    with app.app_context():
        db.create_all()
        yield
        db.session.remove()
        db.drop_all()


@pytest.fixture
def cliente():
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
    jogo = GameState()
    jogo.sala_atual = "entrada"
    jogo.ui_handler = DummyUI()
    return jogo


@pytest.fixture
def jogo_base(mapa_mock):
    jogo = GameState()
    jogo.sala_atual = "entrada"
    jogo.mapa = copy.deepcopy(mapa_mock)
    jogo.ui_handler = DummyUI()
    return jogo
