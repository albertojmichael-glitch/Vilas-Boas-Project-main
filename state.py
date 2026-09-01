import copy
import json
import logging
from enum import Enum
from pathlib import Path
from typing import Any

from pydantic import BaseModel, Field, PrivateAttr

from data import MAPA_ORIGINAL

logger = logging.getLogger(__name__)


class GameException(Exception):
    pass


class QuitGameException(GameException):
    pass


class InvalidCommandError(GameException):
    pass


class GameStateEnum(str, Enum):
    MENU = "MENU"
    JOGO = "JOGO"
    AGUARDANDO_DIR = "AGUARDANDO_DIR"
    FIM = "FIM"
    COMBATE_ANIMATRONICO = "COMBATE_ANIMATRONICO"
    MINIGAME_MINOTAURO = "MINIGAME_MINOTAURO"
    MINIGAME_SEGURANCA = "MINIGAME_SEGURANCA"
    MINIGAME_COFRE = "MINIGAME_COFRE"
    MINIGAME_JON = "MINIGAME_JON"
    MINIGAME_CONSERTOS_CABECA = "MINIGAME_CONSERTOS_CABECA"
    MINIGAME_CONSERTOS_TRONCO = "MINIGAME_CONSERTOS_TRONCO"
    MINIGAME_CONSERTOS_PERNAS = "MINIGAME_CONSERTOS_PERNAS"
    MINIGAME_JULGAMENTO_Q1 = "MINIGAME_JULGAMENTO_Q1"
    MINIGAME_JULGAMENTO_Q2 = "MINIGAME_JULGAMENTO_Q2"
    MINIGAME_JULGAMENTO_Q3 = "MINIGAME_JULGAMENTO_Q3"
    MINIGAME_JULGAMENTO_Q4 = "MINIGAME_JULGAMENTO_Q4"
    MINIGAME_JULGAMENTO_V1 = "MINIGAME_JULGAMENTO_V1"
    MINIGAME_JULGAMENTO_V2 = "MINIGAME_JULGAMENTO_V2"
    MINIGAME_JULGAMENTO_V3 = "MINIGAME_JULGAMENTO_V3"


class GameState(BaseModel):
    hp: int = 3
    inventario: list[str] = Field(default_factory=lambda: ["lanterna"])
    turnos_luz: int = 3
    turnos_no_escuro: int = 0
    turnos_enjoado: int = 0
    sala_atual: str = "entrada"
    turnos_mesma_sala: int = 0
    dificuldade_escolhida: str = "NORMAL"
    chance_sprint_minotauro: int = 15
    turnos_perseguidor_aviso: int = 3
    turnos_perseguidor_morte: int = 5
    bateria_ajuda_gerada: bool = False
    energia_min_noite: int = 90
    energia_max_noite: int = 100
    furia_noite: int = 1
    fios_cortados_inventario: bool = False
    noite_vencida: bool = False
    incendio: bool = False
    turnos_fuga: int = 5
    isqueiro_usos: int = 3
    posicao_perseguidor: str = "palco"
    estado_atual: str = GameStateEnum.MENU.value
    god_mode: bool = False
    erros_consecutivos: int = Field(default=0)
    alberto_desativado: bool = False
    fast_mode: bool = False
    limite_inventario: int = 3
    amanheceu: bool = False
    jon_passos_dados: int = 0
    ai_sala: str = "palco"
    ai_estado: str = "PATRULHA"
    ai_alvo: str = ""
    nivel_barulho: int = 0
    conquistas: list[str] = Field(default_factory=list)
    jon_caminho_certo: list[str] = Field(default_factory=list)
    web_consertos: dict[str, Any] = Field(default_factory=dict)
    web_julgamento: dict[str, Any] = Field(default_factory=dict)
    mapa: dict[str, Any] = Field(default_factory=lambda: copy.deepcopy(MAPA_ORIGINAL))

    _ui_handler: Any = PrivateAttr(default=None)
    _minigame_atual: Any = PrivateAttr(default=None)

    def __init__(self, ui_handler=None, **data):
        super().__init__(**data)
        self._ui_handler = ui_handler

    def to_dict(self):
        return self.model_dump()

    @classmethod
    def from_dict(cls, data):
        return cls.model_validate(data)

    @property
    def ui_handler(self):
        return self._ui_handler

    @ui_handler.setter
    def ui_handler(self, v):
        self._ui_handler = v

    @property
    def minigame_atual(self):
        return self._minigame_atual

    @minigame_atual.setter
    def minigame_atual(self, v):
        self._minigame_atual = v


ARQUIVO_CONQUISTAS = Path("conquistas.json")
AUTOSAVE_FILE = Path("autosave.json")


def carregar_conquistas() -> list[str]:
    if ARQUIVO_CONQUISTAS.exists():
        try:
            return json.loads(ARQUIVO_CONQUISTAS.read_text(encoding="utf-8"))
        except (FileNotFoundError, json.JSONDecodeError) as e:
            print(f"Aviso: Falha ao ler conquistas - {e}")
    return []


def registrar_final(nome_final: str) -> bool:
    conquistas = carregar_conquistas()
    if nome_final not in conquistas:
        conquistas.append(nome_final)
        try:
            ARQUIVO_CONQUISTAS.write_text(
                json.dumps(conquistas, ensure_ascii=False), encoding="utf-8"
            )
        except (OSError, TypeError):
            logger.warning("Falha ao salvar conquistas")
    return all(
        f in conquistas for f in ["mediocre", "bons_sonhos", "bom", "verdadeiro"]
    )


def salvar_autosave(estado: GameState):
    if estado.estado_atual != GameStateEnum.JOGO.value:
        return
    try:
        dados = estado.to_dict()
        AUTOSAVE_FILE.write_text(
            json.dumps(dados, ensure_ascii=False, indent=4), encoding="utf-8"
        )
        return True
    except (OSError, ValueError, TypeError) as e:
        logger.warning(f"Falha ao salvar autosave: {e}")
        return False


def carregar_autosave(estado: GameState) -> bool:
    if AUTOSAVE_FILE.exists():
        try:
            dados = json.loads(AUTOSAVE_FILE.read_text(encoding="utf-8"))
            novo_estado = GameState.from_dict(dados)
            for key, value in novo_estado.to_dict().items():
                setattr(estado, key, value)
            return True
        except (FileNotFoundError, json.JSONDecodeError, ValueError) as e:
            logger.warning(f"Falha ao carregar autosave: {e}")

    return False
