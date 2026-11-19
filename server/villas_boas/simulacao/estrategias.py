"""Estratégias de bot usadas pelo simulador de balanceamento.

* ``EstrategiaAleatoria``: passeio aleatório (a métrica original da issue #7).
* ``EstrategiaCompetente``: segue a rota do final neutro como um jogador que já
  conhece o jogo. No mapa, só usa o que o jogador vê (avisos de passos, rangido
  das grades, sala, inventário, luz). Na noite de segurança, age como um jogador
  experiente que lê corretamente os sistemas (porta, dutos, captchas).
"""

import random
import re
from collections import deque
from typing import Any

from data import SALAS_ESCONDERIJO, SALAS_ILUMINADAS
from state import CHAVES_NAO_SAIDA

_ANSI = re.compile(r"\x1b\[[0-9;]*m")

COMANDOS_ALEATORIOS = [
    "ir frente",
    "ir trás",
    "ir esquerda",
    "ir direita",
    "olhar",
    "pegar bateria nova",
    "usar bateria nova",
    "pegar chave",
    "usar isqueiro",
    "inventario",
    "correr direita",
    "correr esquerda",
    "esconder",
    "esperar",
]


def limpar_texto(texto: str) -> str:
    return _ANSI.sub("", str(texto))


class Estrategia:
    nome = "base"

    def __init__(self, rng: random.Random):
        self.rng = rng

    def proximo_comando(self, jogo: Any, mensagens: list[str]) -> str:
        raise NotImplementedError

    @property
    def desistiu(self) -> bool:
        return False


class EstrategiaAleatoria(Estrategia):
    nome = "aleatoria"

    def proximo_comando(self, jogo: Any, mensagens: list[str]) -> str:
        return self.rng.choice(COMANDOS_ALEATORIOS)


# ---------------------------------------------------------------------------
# Jogador competente
# ---------------------------------------------------------------------------

DICAS_JON = {
    "corrente de ar gelado": "f",
    "parede canhota": "e",
    "caminho destro": "d",
}
RESPOSTAS_JULGAMENTO = ["1995", "ela", "1982", "rogerio", "joao", "angela", "renato"]

_RE_PASSO_PERTO = re.compile(
    r"Passos de metal maciço na sala ao lado vindo da (.+?)\.\.\."
)
_RE_PASSO_LONGE = re.compile(r"Ao longe, passos pesados ecoam vindo da (.+?)\.\.\.")
_RE_GRADE_LADO = re.compile(r"grade de ventilação range na sala ao lado \(à (.+?)\)")
_GRADE_AQUI = "A grade da ventilação range acima de você"


def _saidas_jogador(mapa: dict, sala: str) -> list[tuple[str, str]]:
    return [
        (k, v)
        for k, v in mapa.get(sala, {}).items()
        if k not in CHAVES_NAO_SAIDA
        and isinstance(v, str)
        and v in mapa
        and v not in ("morte", "saida", "cama", "final_bom", sala)
    ]


def _caminho(mapa: dict, origem: str, destino: str, evitar=frozenset()) -> list[str]:
    if origem == destino:
        return [origem]
    anterior: dict[str, str | None] = {origem: None}
    fila = deque([origem])
    while fila:
        sala = fila.popleft()
        for _, viz in _saidas_jogador(mapa, sala):
            if viz in anterior or (viz in evitar and viz != destino):
                continue
            anterior[viz] = sala
            if viz == destino:
                caminho = [viz]
                while anterior[caminho[-1]] is not None:
                    caminho.append(anterior[caminho[-1]])  # type: ignore[arg-type]
                return caminho[::-1]
            fila.append(viz)
    return []


class EstrategiaCompetente(Estrategia):
    nome = "competente"

    def __init__(self, rng: random.Random):
        super().__init__(rng)
        self.perigo: dict[str, int] = {}
        self.fila_comandos: list[str] = []
        self.coletas = {
            "balcão": ["pegar bolsa", "pegar bateria nova", "pegar doce"],
            "mesas de jantar": ["pegar remedio", "pegar bateria nova"],
            "quarto de refrigeração": ["pegar bateria nova"],
        }
        self.julgamento_feito = False
        self._desistiu = False
        self.esperas_seguidas = 0
        self.turno_checado = -1
        self.duto_vermelho = False

    @property
    def desistiu(self) -> bool:
        return self._desistiu

    # ----------------------------------------------------------- objetivos
    def _objetivo(self, jogo: Any) -> str:
        inv = jogo.inventario
        if jogo.noite_vencida:
            return "hall de entrada"
        for sala, itens in self.coletas.items():
            if itens:
                return sala
        if "cartao de seguranca" in inv:
            return "01"
        return "sala de fliperamas"

    # -------------------------------------------------------------- avisos
    def _ler_avisos(self, jogo: Any, mensagens: list[str]) -> None:
        self.perigo = {s: t - 1 for s, t in self.perigo.items() if t > 1}
        mapa = jogo.mapa.get(jogo.sala_atual, {})
        for msg in mensagens:
            if _GRADE_AQUI in msg:
                self.perigo[jogo.sala_atual] = 2
            for regex, peso in (
                (_RE_PASSO_PERTO, 2),
                (_RE_GRADE_LADO, 2),
                (_RE_PASSO_LONGE, 1),
            ):
                achou = regex.search(msg)
                if achou:
                    sala = mapa.get(achou.group(1).strip())
                    if isinstance(sala, str):
                        self.perigo[sala] = max(self.perigo.get(sala, 0), peso)

    # ------------------------------------------------------------- minigames
    def _comando_jon(self, mensagens: list[str]) -> str:
        for msg in reversed(mensagens):
            if "[SENSÓRIO" not in msg:
                continue
            confuso = "CONFUSO" in msg
            for trecho, direcao in DICAS_JON.items():
                if trecho in msg:
                    if confuso:
                        return self.rng.choice([d for d in "fed" if d != direcao])
                    return direcao
        return self.rng.choice("fed")

    def _comando_noite(self, jogo: Any, mensagens: list[str]) -> str:
        """Noite de segurança jogada como um humano experiente.

        A porta é vigiada pelo vidro (grátis). Os dutos precisam ser checados com
        'ver tubulacao' (custa energia), que só diz se há algo lá dentro; se o
        sensor acender, ilumina os dutos."""
        mg = jogo.minigame_atual
        if mg is None:
            return "esperar"
        if getattr(mg, "captcha_ativo", False):
            return str(mg.captcha_resposta)
        if mg.apagao > 0:
            return "esperar"
        custos = mg._custos_turno()

        if any("há algo nos dutos" in m for m in mensagens):
            self.duto_vermelho = True

        ameaca_porta = mg.rick_pos >= 3 or (
            mg.caroline_caminho == "porta" and mg.caroline_pos >= 5
        )
        if ameaca_porta and not mg.porta_fechada and mg.energia > custos["motor"]:
            return "fechar porta"

        if self.turno_checado != mg.turno:
            self.turno_checado = mg.turno
            self.duto_vermelho = False
            if mg.erro_deteccao:
                self.duto_vermelho = True  # sensores offline: ilumina por garantia
            elif mg.energia > custos["info_leve"] + custos["motor"]:
                return "ver tubulacao"

        if (
            self.duto_vermelho
            and mg.energia > custos["info_pesado"]
            and mg.usos_sistema_turno < 2
        ):
            self.duto_vermelho = False
            return "iluminar tubulacao"
        return "esperar"

    # ------------------------------------------------------------ navegação
    def _sala_perigosa(self, sala: str) -> bool:
        return self.perigo.get(sala, 0) > 0

    def _mover_para(self, jogo: Any, destino: str) -> str:
        direcoes = {v: k for k, v in _saidas_jogador(jogo.mapa, jogo.sala_atual)}
        return f"ir {direcoes[destino]}"

    def _nota_refugio(self, jogo: Any, sala: str, preferida: str | None) -> float:
        """Quão boa é uma sala para fugir/esperar (maior = melhor)."""
        nota = 0.0
        if sala == preferida:
            nota += 2
        if sala == "01":
            nota += 4
        if sala in SALAS_ILUMINADAS:
            nota += 1.5
        if sala in SALAS_ESCONDERIJO:
            nota += 1
        if len(_saidas_jogador(jogo.mapa, sala)) <= 1 and sala != "01":
            nota -= 2  # beco sem saída
        nota -= 3 * self.perigo.get(sala, 0)
        return nota + self.rng.random() * 0.1

    def _fugir(self, jogo: Any, preferida: str | None) -> str | None:
        opcoes = [v for _, v in _saidas_jogador(jogo.mapa, jogo.sala_atual)]
        if not opcoes:
            return None
        return max(opcoes, key=lambda v: self._nota_refugio(jogo, v, preferida))

    def _pode_esconder(self, jogo: Any) -> bool:
        return jogo.sala_atual in SALAS_ESCONDERIJO and jogo.turnos_escondido < 3

    def _esperar_ou_esconder(self, jogo: Any) -> str:
        if self._pode_esconder(jogo):
            return "esconder"
        self.esperas_seguidas += 1
        return "esperar"

    def _comando_mapa(self, jogo: Any, mensagens: list[str]) -> str:
        sala = jogo.sala_atual
        inv = jogo.inventario

        if jogo.estado_atual == "COMBATE_ANIMATRONICO":
            destino_alvo = _caminho(jogo.mapa, sala, self._objetivo(jogo))
            preferida = destino_alvo[1] if len(destino_alvo) > 1 else None
            fuga = self._fugir(jogo, preferida)
            if fuga is None:
                return "esperar"
            direcoes = {v: k for k, v in _saidas_jogador(jogo.mapa, sala)}
            return f"correr {direcoes[fuga]}"

        hp_max = jogo.obter_hp_maximo()
        if jogo.hp < hp_max:
            for cura in ("remedio", "doce"):
                if cura in inv:
                    return f"usar {cura}"

        # coletas pendentes na sala atual
        pendentes = self.coletas.get(sala)
        while pendentes:
            cmd = pendentes.pop(0)
            item = cmd.replace("pegar ", "")
            if item in jogo.mapa.get(sala, {}).get("itens", []):
                return cmd

        itens_chao = jogo.mapa.get(sala, {}).get("itens", [])
        for essencial in ("partitura rasgada", "cartao de seguranca"):
            if essencial in itens_chao and essencial not in inv:
                if jogo.tem_espaco_inventario():
                    return f"pegar {essencial}"
                for descartavel in ("chave da cozinha", "pedra", "doce", "confete"):
                    if descartavel in inv:
                        return f"largar {descartavel}"

        if sala == "sala de fliperamas":
            sem_progresso = not any(
                item in inv
                for item in (
                    "moeda velha",
                    "partitura rasgada",
                    "cartao de seguranca",
                    "chave da cozinha",
                )
            )
            if sem_progresso:
                return "jogar jon"
            if "moeda velha" in inv:
                return "jogar consertos"
            if "partitura rasgada" in inv and "cartao de seguranca" not in inv:
                self.fila_comandos = list(RESPOSTAS_JULGAMENTO)
                return "jogar julgamento"
            if "bateria nova" in jogo.mapa[sala].get("itens", []) and (
                jogo.tem_espaco_inventario()
            ):
                return "pegar bateria nova"

        if sala == "01" and "cartao de seguranca" in inv and not jogo.noite_vencida:
            # Gasta uma bateria para ganhar energia na noite, mas guarda outra
            # para a volta pelo escuro.
            if inv.count("bateria nova") >= 2 and jogo.turnos_luz < 12:
                return "usar bateria nova"
            return "cadeira"

        objetivo = self._objetivo(jogo)
        caminho = _caminho(
            jogo.mapa,
            sala,
            objetivo,
            evitar=frozenset(s for s, p in self.perigo.items() if p >= 2),
        ) or _caminho(jogo.mapa, sala, objetivo)
        if len(caminho) < 2:
            self._desistiu = sala != objetivo
            return "esperar"
        proxima = caminho[1]

        # Ela vai surgir aqui (grade rangendo) ou está na próxima sala.
        if self.perigo.get(sala, 0) >= 2:
            if self._pode_esconder(jogo):
                return "esconder"
            fuga = self._fugir(jogo, proxima)
            if fuga:
                return self._mover_para(jogo, fuga)
        if self.perigo.get(proxima, 0) >= 2:
            em_sala_segura = sala == "01" and self.esperas_seguidas < 30
            if em_sala_segura or self.esperas_seguidas < 2 or self._pode_esconder(jogo):
                return self._esperar_ou_esconder(jogo)

        # Barulho alto atrai os dois perseguidores: baixa a poeira num lugar seguro.
        refugio = sala == "01" or (
            sala in SALAS_ILUMINADAS and sala in SALAS_ESCONDERIJO
        )
        limite_espera = 8 if sala == "01" else 2
        if (
            refugio
            and jogo.nivel_barulho >= 50
            and self.esperas_seguidas < limite_espera
        ):
            return self._esperar_ou_esconder(jogo)

        if (
            proxima not in SALAS_ILUMINADAS
            and jogo.turnos_luz <= 1
            and "bateria nova" in inv
        ):
            return "usar bateria nova"

        self.esperas_seguidas = 0
        return self._mover_para(jogo, proxima)

    # ------------------------------------------------------------------ api
    def proximo_comando(self, jogo: Any, mensagens: list[str]) -> str:
        mensagens = [limpar_texto(m) for m in mensagens]
        estado = jogo.estado_atual

        if estado.startswith("MINIGAME_JULGAMENTO"):
            return self.fila_comandos.pop(0) if self.fila_comandos else "joao"
        if estado.startswith("MINIGAME_CONSERTOS"):
            return "1"
        if estado == "MINIGAME_JON":
            return self._comando_jon(mensagens)
        if estado == "MINIGAME_SEGURANCA":
            return self._comando_noite(jogo, mensagens)
        if estado.startswith("MINIGAME"):
            return "sair"

        self._ler_avisos(jogo, mensagens)
        return self._comando_mapa(jogo, mensagens)


ESTRATEGIAS: dict[str, type[Estrategia]] = {
    EstrategiaAleatoria.nome: EstrategiaAleatoria,
    EstrategiaCompetente.nome: EstrategiaCompetente,
}
