"""Simulador de balanceamento: roda partidas completas e agrega estatísticas."""

import math
import random
from collections import Counter
from dataclasses import asdict, dataclass, field
from typing import Any

from state import GameState

from villas_boas.engine.core import processar_fluxo_jogo
from villas_boas.simulacao.ambiente import ambiente_simulacao
from villas_boas.simulacao.estrategias import ESTRATEGIAS

FINAIS_VITORIA = frozenset({"neutro", "verdadeiro"})
MAX_COMANDOS_PADRAO = {"aleatoria": 100, "competente": 800}


@dataclass
class ResultadoPartida:
    seed: int
    dificuldade: str
    estrategia: str
    final: str  # neutro | verdadeiro | covarde | dorminhoco | morte | sem_final
    causa_morte: str = ""
    comandos: int = 0
    travou: bool = False
    erro: str = ""

    @property
    def vitoria(self) -> bool:
        return self.final in FINAIS_VITORIA


@dataclass
class RelatorioLote:
    dificuldade: str
    estrategia: str
    partidas: int
    vitorias: int
    taxa_vitoria: float
    intervalo_95: tuple[float, float]
    finais: dict[str, int]
    causas_morte: dict[str, int]
    travamentos: int
    erros: dict[str, int]
    comandos_medios: float
    resultados: list[ResultadoPartida] = field(default_factory=list, repr=False)

    def como_dict(self) -> dict[str, Any]:
        dados = asdict(self)
        dados.pop("resultados", None)
        return dados

    def causa_dominante(self) -> tuple[str, float]:
        mortes = sum(self.causas_morte.values())
        if not mortes:
            return ("", 0.0)
        causa, qtd = max(self.causas_morte.items(), key=lambda kv: kv[1])
        return causa, qtd / mortes


def intervalo_wilson(sucessos: int, n: int, z: float = 1.96) -> tuple[float, float]:
    if n == 0:
        return (0.0, 0.0)
    p = sucessos / n
    denom = 1 + z * z / n
    centro = (p + z * z / (2 * n)) / denom
    margem = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denom
    return (max(0.0, centro - margem), min(1.0, centro + margem))


def _classificar_final(jogo: GameState) -> str:
    if jogo.sala_atual == "morte" or jogo.causa_morte:
        return "morte"
    if jogo.estado_atual != "FIM":
        return "sem_final"
    if jogo.sala_atual == "saida":
        return "covarde"
    if jogo.sala_atual == "cama":
        return "dorminhoco"
    if jogo.noite_vencida and jogo.sala_atual == "hall de entrada":
        return "verdadeiro" if jogo.fios_cortados_inventario else "neutro"
    return "morte"


def _novo_jogo(dificuldade: str) -> GameState:
    jogo = GameState()
    jogo.configurar_dificuldade(dificuldade)
    jogo.estado_atual = "JOGO"
    jogo.sala_atual = "entrada"
    return jogo


def simular_partida(
    dificuldade: str,
    estrategia: str,
    seed: int,
    max_comandos: int | None = None,
) -> ResultadoPartida:
    """Roda uma partida completa e determinística (mesma semente, mesmo resultado).

    Deve ser chamada dentro de ``ambiente_simulacao()``."""
    limite = max_comandos or MAX_COMANDOS_PADRAO.get(estrategia, 400)
    random.seed(seed)
    bot = ESTRATEGIAS[estrategia](random.Random(seed * 7919 + 1))
    jogo = _novo_jogo(dificuldade)
    buffer = jogo.ui_handler.buffer
    lido = 0
    comandos = 0
    resultado = ResultadoPartida(seed, dificuldade, estrategia, "sem_final")

    try:
        while comandos < limite:
            if jogo.estado_atual == "FIM" or jogo.sala_atual == "morte":
                break
            mensagens = buffer[lido:]
            lido = len(buffer)
            comando = bot.proximo_comando(jogo, mensagens)
            if bot.desistiu:
                break
            processar_fluxo_jogo(comando, jogo)
            comandos += 1
    except Exception as exc:  # noqa: BLE001 - qualquer exceção é um bug a reportar
        resultado.erro = f"{type(exc).__name__}: {exc}"

    resultado.comandos = comandos
    resultado.final = _classificar_final(jogo)
    if resultado.final == "morte":
        resultado.causa_morte = jogo.causa_morte or "desconhecida"
    if estrategia != "aleatoria" and resultado.final == "sem_final":
        resultado.travou = True
    return resultado


def rodar_lote(
    dificuldade: str,
    estrategia: str,
    partidas: int,
    seed_inicial: int = 0,
    max_comandos: int | None = None,
) -> RelatorioLote:
    resultados = []
    with ambiente_simulacao():
        for i in range(partidas):
            resultados.append(
                simular_partida(dificuldade, estrategia, seed_inicial + i, max_comandos)
            )

    vitorias = sum(r.vitoria for r in resultados)
    finais = Counter(r.final for r in resultados)
    causas = Counter(r.causa_morte for r in resultados if r.final == "morte")
    erros = Counter(r.erro.split(":")[0] for r in resultados if r.erro)
    return RelatorioLote(
        dificuldade=dificuldade,
        estrategia=estrategia,
        partidas=partidas,
        vitorias=vitorias,
        taxa_vitoria=vitorias / partidas if partidas else 0.0,
        intervalo_95=intervalo_wilson(vitorias, partidas),
        finais=dict(finais.most_common()),
        causas_morte=dict(causas.most_common()),
        travamentos=sum(r.travou for r in resultados),
        erros=dict(erros),
        comandos_medios=(
            sum(r.comandos for r in resultados) / partidas if partidas else 0.0
        ),
        resultados=resultados,
    )


def formatar_relatorio(rel: RelatorioLote) -> str:
    baixo, alto = rel.intervalo_95
    linhas = [
        f"=== {rel.dificuldade} | estratégia {rel.estrategia} | {rel.partidas} partidas ===",
        (
            f"Vitórias (final neutro/verdadeiro): {rel.vitorias} "
            f"({rel.taxa_vitoria:.1%}, IC95% {baixo:.1%}–{alto:.1%})"
        ),
        f"Finais: {rel.finais}",
        f"Causas de morte: {rel.causas_morte}",
        f"Travamentos (softlock): {rel.travamentos} | Erros: {rel.erros or 0}",
        f"Comandos por partida (média): {rel.comandos_medios:.1f}",
    ]
    causa, fracao = rel.causa_dominante()
    if causa:
        linhas.append(f"Causa dominante: {causa} ({fracao:.0%} das mortes)")
    return "\n".join(linhas)
