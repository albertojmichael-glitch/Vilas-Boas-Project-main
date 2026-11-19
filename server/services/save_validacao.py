"""Validação de integridade de saves: nenhum estado entra no servidor sem passar aqui.

A assinatura/cifra garante que o arquivo veio do servidor; esta validação garante
que o *conteúdo* é um estado de jogo possível (defesa em profundidade contra chave
vazada, bug ou save antigo malformado). Ela vale para tudo o que é carregado ou
persistido: importação, autosave no banco e clone de link compartilhado.

Campos que são do servidor (mapa, presets da dificuldade, perfil da "Ela") nunca
são confiados: são reconstruídos a partir dos dados originais do jogo.
"""

import copy
from typing import Any

from data import (
    BALANCE_CFG,
    EVENTOS_ALEATORIOS,
    MAPA_ORIGINAL,
    MAX_ESCONDER_SEGUIDOS,
    MUTACOES_MAPA_PERMITIDAS,
    TURNOS_BATERIA,
    VIDA_GOD_MODE,
    descricoes_itens,
    perfil_ela,
)
from pydantic import ValidationError
from state import MAX_LOG_COMANDOS, GameState
from villas_boas.ela import DUTOS_ELA
from villas_boas.utils import CONQUISTAS_DB

ESTADOS_PERSISTIVEIS = frozenset({"JOGO", "COMBATE_ANIMATRONICO"})
DIFICULDADES_CONHECIDAS = frozenset(
    {
        "NORMAL",
        "PESADELO",
        "DESAFIO_SPEEDRUN",
        "DESAFIO_FANTASMA",
        "DESAFIO_BREU",
        "CUSTOM",
        "GOD MODE",
    }
)
ATACANTES = frozenset({"", "ela", "animatronico"})
LIMITE_LIVRE = VIDA_GOD_MODE  # teto dos modos personalizados (igual à edição Rust)
MAX_BOLSAS_NO_MAPA = 2
FOLGA_INVENTARIO = 2  # os Consertos podem entregar 2 itens com a mochila quase cheia


class SaveInvalido(ValueError):
    """O save não representa um estado de jogo possível."""


def catalogo_de_itens() -> frozenset[str]:
    itens = set(descricoes_itens) | {"lanterna"}
    for sala in MAPA_ORIGINAL.values():
        itens.update(sala.get("itens", []))
    itens.update(e["item_drop"] for e in EVENTOS_ALEATORIOS if e.get("item_drop"))
    return frozenset(itens)


CATALOGO_ITENS = catalogo_de_itens()


def _exigir(condicao: bool, motivo: str) -> None:
    if not condicao:
        raise SaveInvalido(motivo)


def _faixa(valor: int, minimo: int, maximo: int, campo: str) -> None:
    _exigir(minimo <= valor <= maximo, f"{campo} fora da faixa ({minimo}..{maximo})")


def reconstruir_mapa(mapa_salvo: Any) -> dict[str, Any]:
    """Mapa original + só o que o jogo de fato muda (itens no chão e portas)."""
    mapa = copy.deepcopy(MAPA_ORIGINAL)
    if not isinstance(mapa_salvo, dict):
        return mapa
    for nome, sala in mapa_salvo.items():
        if nome not in mapa or not isinstance(sala, dict):
            continue
        itens = sala.get("itens", [])
        _exigir(
            isinstance(itens, list) and all(isinstance(i, str) for i in itens),
            f"itens inválidos na sala '{nome}'",
        )
        desconhecidos = sorted(set(itens) - CATALOGO_ITENS)
        _exigir(not desconhecidos, f"item desconhecido no mapa: {desconhecidos}")
        mapa[nome]["itens"] = list(itens)
    for (nome, chave), valor in MUTACOES_MAPA_PERMITIDAS.items():
        sala = mapa_salvo.get(nome)
        if isinstance(sala, dict) and sala.get(chave) == valor:
            mapa[nome][chave] = valor
    return mapa


def _aplicar_presets_do_servidor(jogo: GameState) -> None:
    """Presets de dificuldade e perfil da 'Ela' vêm do servidor, não do save."""
    modo = jogo.dificuldade_escolhida
    jogo.perfil_ela = perfil_ela(modo)
    preset = BALANCE_CFG.get("dificuldades", {}).get(modo)
    if preset:
        jogo.furia_noite = int(preset.get("furia_noite", jogo.furia_noite))
        jogo.energia_min_noite = int(
            preset.get("energia_min_noite", jogo.energia_min_noite)
        )
        jogo.energia_max_noite = int(
            preset.get("energia_max_noite", jogo.energia_max_noite)
        )


def validar_estado_save(dados: Any, permitir_dev: bool = False) -> GameState:
    """Devolve o estado normalizado ou levanta ``SaveInvalido`` com o motivo."""
    _exigir(isinstance(dados, dict), "o save não é um objeto JSON")
    try:
        jogo = GameState.from_dict(dados)
    except ValidationError as exc:
        campos = sorted({".".join(map(str, e["loc"])) for e in exc.errors()})
        raise SaveInvalido(f"campos com tipo inválido: {campos}") from exc

    modo = jogo.dificuldade_escolhida
    _exigir(jogo.estado_atual in ESTADOS_PERSISTIVEIS, "estado de jogo não salvável")
    _exigir(modo in DIFICULDADES_CONHECIDAS, f"dificuldade desconhecida: {modo}")
    if not permitir_dev:
        _exigir(
            not (jogo.god_mode or jogo.dev_mode or modo == "GOD MODE"),
            "save com modo deus/desenvolvedor",
        )
    livre = modo in ("CUSTOM", "GOD MODE") or jogo.god_mode

    # Vida, luz, barulho e mochila
    hp_max = LIMITE_LIVRE if livre else jogo.obter_hp_maximo()
    _faixa(jogo.hp, 1, hp_max, "hp")
    _faixa(jogo.turnos_luz, 0, LIMITE_LIVRE if livre else TURNOS_BATERIA, "turnos_luz")
    _faixa(jogo.nivel_barulho, 0, 100, "nivel_barulho")
    _faixa(
        jogo.bolsas_coletadas,
        0,
        1000 if livre else MAX_BOLSAS_NO_MAPA,
        "bolsas_coletadas",
    )
    for campo in ("turnos_no_escuro", "turnos_enjoado", "turnos_mesma_sala"):
        _faixa(getattr(jogo, campo), 0, 100, campo)

    desconhecidos = sorted(set(jogo.inventario) - CATALOGO_ITENS)
    _exigir(not desconhecidos, f"item desconhecido no inventário: {desconhecidos}")
    if not jogo.god_mode:
        limite = jogo.obter_limite_inventario() + FOLGA_INVENTARIO
        _exigir(len(jogo.inventario) <= limite, "inventário acima do limite")

    # Mapa e posições
    jogo.mapa = reconstruir_mapa(dados.get("mapa"))
    _exigir(jogo.sala_atual in jogo.mapa, f"sala inexistente: {jogo.sala_atual}")
    _exigir(jogo.ai_sala in jogo.mapa, "posição do animatrônico inválida")
    _exigir(
        jogo.posicao_perseguidor in jogo.mapa or jogo.posicao_perseguidor == "longe",
        "posição da 'Ela' inválida",
    )
    _exigir(
        jogo.ela_saida_duto == "" or jogo.ela_saida_duto in DUTOS_ELA,
        "saída de duto inválida",
    )
    _exigir(jogo.atacante_combate in ATACANTES, "atacante de combate inválido")
    _faixa(jogo.ela_dormencia, 0, 1000, "ela_dormencia")
    _faixa(jogo.ela_atordoada, 0, 100, "ela_atordoada")
    _faixa(jogo.turnos_escondido, 0, MAX_ESCONDER_SEGUIDOS, "turnos_escondido")
    _exigir(
        len(jogo.jon_caminho_certo) <= 4
        and all(p in ("f", "e", "d") for p in jogo.jon_caminho_certo),
        "caminho do Jon inválido",
    )

    conquistas_invalidas = sorted(set(jogo.conquistas) - set(CONQUISTAS_DB))
    _exigir(not conquistas_invalidas, f"conquista desconhecida: {conquistas_invalidas}")
    _exigir(jogo.tempo_total_segundos >= 0, "tempo de jogo negativo")

    # Normalização: nada de minigame pela metade nem configuração vinda do cliente.
    jogo.minigame_dados = {}
    jogo.web_consertos = {}
    jogo.web_julgamento = {}
    jogo.log_comandos = jogo.log_comandos[-MAX_LOG_COMANDOS:]
    jogo.novas_conquistas_turno = []
    _aplicar_presets_do_servidor(jogo)
    return jogo
