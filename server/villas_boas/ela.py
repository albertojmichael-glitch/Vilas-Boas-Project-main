"""IA da "Ela", a perseguidora presente no NORMAL e no PESADELO.

Regras (perfil por dificuldade em ``balance_config.json`` -> ``dificuldades.<MODO>.ela``):

* Começa adormecida no palco e só desperta depois de ``turnos_dormencia`` turnos.
* Patrulha ao acaso; com barulho alto, rastreia o jogador por BFS.
* Pode usar os dutos de ventilação (atalho exclusivo dela, o mapa do jogador não
  muda): entra num duto e, no turno seguinte, sai numa sala com grade a até
  ``alcance_duto`` salas de distância. O rangido da grade avisa o jogador.
* Os passos são ouvidos com a direção de onde vêm, até ``aviso_distancia`` salas.
* Encontrá-la não mata na hora: começa um combate do qual dá para fugir correndo
  (custa ``dano_fuga`` de HP e a deixa atordoada por ``turnos_atordoada`` turnos).
* Escondido num esconderijo, o jogador não é encontrado.
"""

import random
from collections import deque

from data import DUTOS_ELA, SALA_INICIAL_ELA
from ui import DOS_AMARELO, DOS_VERMELHO, RESET

# Salas onde ela nunca entra (e onde o jogador está fora do alcance dela).
SALAS_SEGURAS_ELA = frozenset(
    {"saida", "cama", "final_bom", "morte", "tubo de ventilação", "01"}
)
MODOS_COM_ELA = frozenset({"NORMAL", "PESADELO"})
_CHAVES_NAO_SAIDA = frozenset(
    {"descrição", "itens", "inspecionaveis", "energia", "cofre_important", "cadeira"}
)


def ela_ativa_no_modo(jogo) -> bool:
    return jogo.dificuldade_escolhida in MODOS_COM_ELA


def _saidas(mapa, sala):
    """Pares (direção, sala vizinha) que ela consegue percorrer a pé."""
    return [
        (k, v)
        for k, v in mapa.get(sala, {}).items()
        if k not in _CHAVES_NAO_SAIDA
        and isinstance(v, str)
        and v in mapa
        and v not in SALAS_SEGURAS_ELA
        and v != sala
    ]


def _distancias(mapa, origem):
    """BFS pelas salas que ela percorre; devolve {sala: distância}."""
    if origem not in mapa:
        return {}
    dist = {origem: 0}
    fila = deque([origem])
    while fila:
        sala = fila.popleft()
        for _, viz in _saidas(mapa, sala):
            if viz not in dist:
                dist[viz] = dist[sala] + 1
                fila.append(viz)
    return dist


def _proximo_passo(mapa, origem, destino):
    """Primeira sala do menor caminho de origem até destino (ou None)."""
    if origem == destino:
        return None
    anterior = {origem: None}
    fila = deque([origem])
    while fila:
        sala = fila.popleft()
        if sala == destino:
            break
        for _, viz in _saidas(mapa, sala):
            if viz not in anterior:
                anterior[viz] = sala
                fila.append(viz)
    if destino not in anterior:
        return None
    passo = destino
    while anterior[passo] != origem:
        passo = anterior[passo]
    return passo


def rastro_ate_jogador(jogo, origem):
    """(distância, direção) de quem está em ``origem`` até a sala do jogador.

    A direção é a saída da sala do jogador por onde essa pessoa chegaria. Funciona
    mesmo com o jogador numa sala segura (onde ela não entra)."""
    if origem == jogo.sala_atual:
        return 0, None
    dist = _distancias(jogo.mapa, origem)
    melhor = None
    for direcao, viz in jogo.mapa.get(jogo.sala_atual, {}).items():
        if direcao in _CHAVES_NAO_SAIDA or not isinstance(viz, str) or viz not in dist:
            continue
        if melhor is None or dist[viz] < melhor[0]:
            melhor = (dist[viz], direcao)
    if melhor is None:
        return None, None
    return melhor[0] + 1, melhor[1]


def saidas_de_duto(jogo, origem):
    """Salas com grade alcançáveis pelo duto a partir de ``origem``."""
    alcance = int(jogo.perfil_ela.get("alcance_duto", 2))
    dist = _distancias(jogo.mapa, origem)
    return sorted(
        sala
        for sala, d in dist.items()
        if 1 <= d <= alcance and sala in DUTOS_ELA and sala != origem
    )


def distancia_ate_jogador(jogo):
    if jogo.ela_no_duto:
        return None
    return rastro_ate_jogador(jogo, jogo.posicao_perseguidor)[0]


def _entrar_no_duto(jogo, destinos, alvo=None):
    if alvo is not None:
        dist_alvo = _distancias(jogo.mapa, alvo)
        destinos = sorted(destinos, key=lambda s: (dist_alvo.get(s, 99), s))
        saida = destinos[0]
    else:
        saida = random.choice(destinos)
    jogo.ela_no_duto = True
    jogo.ela_saida_duto = saida


def _mover(jogo, perfil, barulho_alto):
    """Decide o movimento do turno (a pé ou pelo duto)."""
    pos = jogo.posicao_perseguidor
    no_duto_possivel = pos in DUTOS_ELA and saidas_de_duto(jogo, pos)

    rastreando = barulho_alto and random.random() < float(perfil["chance_rastrear"])
    if rastreando:
        alvo = jogo.sala_atual
        dist_a_pe = _distancias(jogo.mapa, pos).get(alvo, 99)
        if (
            no_duto_possivel
            and dist_a_pe > 1
            and random.random() < float(perfil["chance_usar_duto"])
        ):
            _entrar_no_duto(jogo, saidas_de_duto(jogo, pos), alvo=alvo)
            return
        passo = _proximo_passo(jogo.mapa, pos, alvo)
        if passo:
            jogo.posicao_perseguidor = passo
        return

    if random.random() >= float(perfil["chance_mover"]):
        return
    if no_duto_possivel and random.random() < float(perfil["chance_usar_duto"]):
        _entrar_no_duto(jogo, saidas_de_duto(jogo, pos))
        return
    vizinhos = [v for _, v in _saidas(jogo.mapa, pos)]
    if vizinhos:
        jogo.posicao_perseguidor = random.choice(vizinhos)


def _avisar(jogo, ui, perfil):
    if jogo.ela_no_duto:
        saida = jogo.ela_saida_duto
        if saida == jogo.sala_atual:
            ui.exibir(
                f"\n{DOS_VERMELHO}A grade da ventilação range acima de você... algo se arrasta lá dentro.{RESET}"
            )
        else:
            dist, direcao = rastro_ate_jogador(jogo, saida)
            if dist != 1:
                return
            onde = f" (à {direcao})" if direcao else ""
            ui.exibir(
                f"\n{DOS_AMARELO}Uma grade de ventilação range na sala ao lado{onde}.{RESET}"
            )
        return

    dist, direcao = rastro_ate_jogador(jogo, jogo.posicao_perseguidor)
    if dist is None or dist < 1 or dist > int(perfil["aviso_distancia"]):
        return
    vindo = f" vindo da {direcao}" if direcao else ""
    if dist == 1:
        ui.exibir(
            f"\n{DOS_AMARELO}O chão vibra. Passos de metal maciço na sala ao lado{vindo}...{RESET}"
        )
    else:
        ui.exibir(f"\n{DOS_AMARELO}Ao longe, passos pesados ecoam{vindo}...{RESET}")


def _encontro(jogo, ui):
    if getattr(jogo, "dev_mode", False):
        ui.exibir(
            f"{DOS_AMARELO}[DEV] 'Ela' te encontrou em {jogo.sala_atual}: você seria atacado. Ela recuou.{RESET}"
        )
        jogo.posicao_perseguidor = SALA_INICIAL_ELA
        return
    if jogo.escondido:
        ui.exibir(
            f"\n{DOS_AMARELO}Ela entra na sala. Você prende a respiração no esconderijo... "
            f"os passos param, giram, e se afastam.{RESET}"
        )
        vizinhos = [v for _, v in _saidas(jogo.mapa, jogo.sala_atual)]
        if vizinhos:
            jogo.posicao_perseguidor = random.choice(vizinhos)
        return
    ui.exibir("\n" + "=" * 50)
    ui.exibir(
        f"{DOS_VERMELHO}A porta quebra. Ela te encontrou! CORRA! (correr [direção]){RESET}"
    )
    ui.pausar(2)
    jogo.estado_atual = "COMBATE_ANIMATRONICO"
    jogo.atacante_combate = "ela"


def processar_ela(jogo, ui, barulho_alto: bool):
    """Um turno da IA da 'Ela'. Chamado só quando o jogador gastou um turno
    fora das salas seguras."""
    perfil = jogo.perfil_ela

    if jogo.ela_dormencia > 0:
        jogo.ela_dormencia -= 1
        if jogo.ela_dormencia == 0:
            ui.exibir(
                f"\n{DOS_AMARELO}Um rangido metálico longo vem do palco. Algo despertou.{RESET}"
            )
        return

    if jogo.ela_atordoada > 0:
        jogo.ela_atordoada -= 1
        return

    if jogo.ela_no_duto:
        jogo.posicao_perseguidor = jogo.ela_saida_duto or jogo.posicao_perseguidor
        jogo.ela_no_duto = False
        jogo.ela_saida_duto = ""
        if jogo.posicao_perseguidor == jogo.sala_atual:
            ui.exibir(f"\n{DOS_VERMELHO}A grade da ventilação cai no chão.{RESET}")
    else:
        _mover(jogo, perfil, barulho_alto)

    if not jogo.ela_no_duto and jogo.posicao_perseguidor == jogo.sala_atual:
        _encontro(jogo, ui)
    else:
        _avisar(jogo, ui, perfil)


def atordoar_apos_fuga(jogo):
    """Fuga bem-sucedida: ela fica atordoada e perde o rastro por alguns turnos."""
    jogo.ela_atordoada = int(jogo.perfil_ela.get("turnos_atordoada", 3))
    jogo.ela_no_duto = False
    jogo.ela_saida_duto = ""
