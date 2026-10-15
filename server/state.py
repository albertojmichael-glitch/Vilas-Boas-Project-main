import copy
import json
import logging
import random
import time
from enum import Enum
from pathlib import Path
from typing import Any

from pydantic import BaseModel, Field, PrivateAttr

from data import (
    BALANCE_CFG,
    CHANCE_MOVER_PATRULHA,
    ISQUEIRO_BARULHO,
    ISQUEIRO_LIMITE_LUZ,
    ISQUEIRO_TURNOS_BONUS,
    LIMIAR_ALERTA,
    LIMIAR_CACA,
    MAPA_ORIGINAL,
    MAX_INVENTARIO,
    META_ACUMULADOR,
    REDUCAO_BARULHO_TURNO,
    SLOTS_POR_BOLSA,
    TURNOS_BATERIA,
    TURNOS_LUZ_INICIAL,
    VIDA_GOD_MODE,
    VIDA_NORMAL,
    VIDA_PESADELO,
)

logger = logging.getLogger(__name__)


class GameException(Exception):
    pass


class QuitGameException(GameException):
    pass


class InvalidCommandError(GameException):
    pass


class HeadlessUIHandler:
    """Handler silencioso padrão para testes unitários headless, bots e simulações."""

    def __init__(self):
        self.buffer: list[str] = []

    def limpar(self):
        self.buffer.append("@@CLEAR@@")

    def pausar(self, segs: float):
        pass

    def exibir(self, texto: str):
        self.buffer.append(str(texto))

    def animar(self, texto: str, tempo: float = 0.0, cor: str = "", jogo: Any = None):
        self.buffer.append(str(texto))

    def obter_input(self, prompt_text: str = "") -> str:
        return ""


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
    hp: int = VIDA_NORMAL
    inventario: list[str] = Field(default_factory=lambda: ["lanterna"])
    turnos_luz: int = TURNOS_LUZ_INICIAL
    turnos_no_escuro: int = 0
    turnos_enjoado: int = 0
    sala_atual: str = "entrada"
    cofre_tentativas: int = 0
    jon_erros: int = 0
    bolsas_coletadas: int = 0
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
    dev_mode: bool = False
    dev_usado: bool = False  
    dev_radar: bool = True
    dev_ia: bool = False
    erros_consecutivos: int = Field(default=0)
    alberto_desativado: bool = False
    fast_mode: bool = False
    limite_inventario: int = MAX_INVENTARIO
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
    minigame_dados: dict[str, Any] = Field(default_factory=dict)
    log_comandos: list[str] = Field(default_factory=list)
    novas_conquistas_turno: list[dict] = Field(default_factory=list)
    tempo_inicio: float = Field(default_factory=time.time)
    tempo_total_segundos: float = 0.0
    mapa: dict[str, Any] = Field(default_factory=lambda: copy.deepcopy(MAPA_ORIGINAL))

    _ui_handler: Any = PrivateAttr(default=None)
    _minigame_atual: Any = PrivateAttr(default=None)

    def __init__(self, ui_handler=None, **data):
        super().__init__(**data)
        self._ui_handler = ui_handler if ui_handler is not None else HeadlessUIHandler()

    def to_dict(self):
        return self.model_dump()

    @classmethod
    def from_dict(cls, data):
        return cls.model_validate(data)

    @property
    def ui_handler(self):
        if self._ui_handler is None:
            self._ui_handler = HeadlessUIHandler()
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

    # ==========================================
    # MÉTODOS PUROS DE REGRAS DE NEGÓCIO (CORE)
    # ==========================================

    def obter_limite_inventario(self) -> int:
        return MAX_INVENTARIO + (self.bolsas_coletadas * SLOTS_POR_BOLSA)

    def obter_hp_maximo(self) -> int:
        if self.god_mode:
            return VIDA_GOD_MODE
        return VIDA_PESADELO if self.dificuldade_escolhida == "PESADELO" else VIDA_NORMAL

    def tem_espaco_inventario(self) -> bool:
        return self.god_mode or len(self.inventario) < self.obter_limite_inventario()

    def equipar_bolsa(self) -> int:
        self.bolsas_coletadas += 1
        return self.obter_limite_inventario()

    def adicionar_item_inventario(self, item: str) -> bool:
        if not self.tem_espaco_inventario():
            return False
        self.inventario.append(item)
        if item == "fios cortados":
            self.fios_cortados_inventario = True
        return True

    def atingiu_meta_acumulador(self) -> bool:
        return len(self.inventario) >= META_ACUMULADOR

    def alterar_barulho(self, delta: int) -> int:
        self.nivel_barulho = max(0, min(100, self.nivel_barulho + delta))
        return self.nivel_barulho

    def aplicar_dano(self, quantidade: int = 1) -> bool:
        """Aplica dano ao jogador e retorna True se o HP chegou a zero."""
        if self.god_mode:
            return False
        self.hp = max(0, self.hp - quantidade)
        if self.hp <= 0:
            self.sala_atual = "morte"
            return True
        return False

    def curar(self, item: str, quantidade: int = 1) -> bool:
        """Tenta curar o jogador consumindo o item. Retorna False se já estiver com HP cheio."""
        if self.hp >= self.obter_hp_maximo():
            return False
        self.hp += quantidade
        if item in self.inventario:
            self.inventario.remove(item)
        return True

    def recarregar_lanterna(self) -> str:
        """Processa a regra pura da bateria. Retorna 'quebrada_breu' ou 'recarregada'."""
        if "bateria nova" in self.inventario:
            self.inventario.remove("bateria nova")
        if self.dificuldade_escolhida == "DESAFIO_BREU":
            return "quebrada_breu"
        self.turnos_luz = TURNOS_BATERIA
        return "recarregada"

    def acender_isqueiro(self) -> bool:
        """Tenta usar o isqueiro. Retorna False se a lanterna já estiver forte."""
        if self.turnos_luz > ISQUEIRO_LIMITE_LUZ:
            return False
        self.turnos_luz += ISQUEIRO_TURNOS_BONUS
        self.alterar_barulho(ISQUEIRO_BARULHO)
        return True

    def configurar_dificuldade(self, modo: str, fast_mode: bool = False) -> None:
        self.dificuldade_escolhida = modo
        self.fast_mode = fast_mode
        presets = BALANCE_CFG.get("dificuldades", {})

        if modo == "GOD MODE":
            self.god_mode = True
            self.hp = VIDA_GOD_MODE
            self.furia_noite = 0
            self.energia_min_noite = VIDA_GOD_MODE
            self.energia_max_noite = VIDA_GOD_MODE
            self.turnos_luz = VIDA_GOD_MODE
        elif modo == "PESADELO":
            cfg = presets.get("PESADELO", {})
            self.hp = cfg.get("hp", VIDA_PESADELO)
            self.furia_noite = cfg.get("furia_noite", 2)
            self.energia_min_noite = cfg.get("energia_min_noite", 70)
            self.energia_max_noite = cfg.get("energia_max_noite", 82)
        else:
            cfg = presets.get("NORMAL", {})
            self.hp = cfg.get("hp", VIDA_NORMAL)
            self.furia_noite = cfg.get("furia_noite", 1)
            self.energia_min_noite = cfg.get("energia_min_noite", 100)
            self.energia_max_noite = cfg.get("energia_max_noite", 100)
            if modo == "DESAFIO_BREU":
                self.turnos_luz = 0

    def resetar_para_novo_jogo(self) -> None:
        try:
            self.mapa = copy.deepcopy(MAPA_ORIGINAL)
        except Exception as e:
            logger.error(f"Erro ao restaurar MAPA_ORIGINAL: {e}")

        self.estado_atual = GameStateEnum.AGUARDANDO_DIR.value
        self.inventario = []
        self.hp = VIDA_NORMAL
        self.sala_atual = "entrada"
        self.incendio = False
        self.noite_vencida = False
        self.fios_cortados_inventario = False
        self.amanheceu = False
        self.alberto_desativado = False
        self.god_mode = False
        self.turnos_luz = 0
        self.fast_mode = False
        self.dificuldade_escolhida = "NORMAL"
        self._minigame_atual = None
        self.tempo_inicio = time.time()

    def calcular_caminho_bfs(self, inicio: str, destino: str) -> list[str]:
        if inicio not in self.mapa or destino not in self.mapa:
            return []
        fila = [[inicio]]
        visitados = {inicio}
        chaves_nao_saida = {"descrição", "itens", "inspecionaveis", "energia"}

        while fila:
            caminho = fila.pop(0)
            sala = caminho[-1]
            if sala == destino:
                return caminho

            for direcao, vizinho in self.mapa[sala].items():
                if (
                    isinstance(vizinho, str)
                    and vizinho in self.mapa
                    and vizinho not in visitados
                    and direcao not in chaves_nao_saida
                ):
                    visitados.add(vizinho)
                    fila.append([*caminho, vizinho])
        return []

    def avancar_ia_perseguidor(self) -> dict[str, Any]:
        """Executa um passo puro da IA do perseguidor e retorna o relatório de proximidade."""
        if self.god_mode or self.estado_atual != GameStateEnum.JOGO.value:
            return {"ativa": False, "distancia": 99, "combate_iniciado": False}

        if self.nivel_barulho > 0:
            self.alterar_barulho(-REDUCAO_BARULHO_TURNO)

        if self.nivel_barulho >= LIMIAR_CACA or self.ai_sala == self.sala_atual:
            self.ai_estado = "CACA"
        elif self.nivel_barulho >= LIMIAR_ALERTA and self.ai_estado != "CACA":
            self.ai_estado = "ALERTA"
        elif self.nivel_barulho < LIMIAR_ALERTA:
            self.ai_estado = "PATRULHA"

        chaves_nao_saida = {"descrição", "itens", "inspecionaveis", "energia"}

        if self.ai_estado == "PATRULHA":
            vizinhos = [
                v for k, v in self.mapa.get(self.ai_sala, {}).items()
                if isinstance(v, str) and v in self.mapa and k not in chaves_nao_saida
            ]
            if random.random() > CHANCE_MOVER_PATRULHA and vizinhos:
                self.ai_sala = random.choice(vizinhos)

        elif self.ai_estado in ["ALERTA", "CACA"]:
            destino = self.sala_atual if self.ai_estado == "CACA" else (self.ai_alvo or self.sala_atual)
            caminho = self.calcular_caminho_bfs(self.ai_sala, destino)
            if len(caminho) > 1:
                self.ai_sala = caminho[1]

        caminho_para_jogador = self.calcular_caminho_bfs(self.sala_atual, self.ai_sala)
        distancia = len(caminho_para_jogador) - 1 if caminho_para_jogador else 99

        combate = False
        if distancia == 0 and self.sala_atual not in ["01", "sala de energia"]:
            self.estado_atual = GameStateEnum.COMBATE_ANIMATRONICO.value
            combate = True

        return {
            "ativa": True,
            "distancia": distancia,
            "combate_iniciado": combate,
            "ai_estado": self.ai_estado,
            "ai_sala": self.ai_sala,
        }


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
        return False
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