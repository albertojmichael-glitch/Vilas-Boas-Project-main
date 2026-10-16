"""Modo desenvolvedor do Villas Boas.

Ativação:  digite  sudo admin 1982  no menu (começa uma partida já em modo dev)
           ou durante a partida (liga o modo dev no ponto em que você está).

Variáveis de ambiente (opcionais):
    DEV_MODE_ENABLED=0      desliga o modo dev por completo (recomendado em produção)
    DEV_MODE_CODE="..."     troca o código de ativação

Regras de segurança do modo:
    * partidas que usaram o modo dev (``dev_usado``) NÃO entram no leaderboard
    * a telemetria ignora partidas em modo dev
    * o modo é desligado ao reiniciar depois de um fim de jogo
"""

import os
from collections import Counter

from data import TURNOS_BATERIA, VIDA_NORMAL, descricoes_itens
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET
from villas_boas.utils import encontrar_melhor_match

CODIGO_PADRAO = "sudo admin 1982"
VALOR_INFINITO = 9999
DIFICULDADE_DEV = "DEV MODE"

ESTADOS_ATIVACAO = ("MENU", "JOGO", "COMBATE_ANIMATRONICO", "AGUARDANDO_DIR")
ESTADOS_DE_JOGO = ("JOGO", "COMBATE_ANIMATRONICO")

NAO_TRATADO = 0
TRATADO = 1
TRATADO_MUDOU_SALA = 2  # o chamador deve checar finais e redesenhar a sala

_SEM_ITEM_NO_TUDO = {
    "fios cortados"
}  # muda o rumo da história: só via 'gerar fios cortados'


# ---------------------------------------------------------------- ativação
def modo_dev_habilitado():
    return os.environ.get("DEV_MODE_ENABLED", "1").strip().lower() not in (
        "0",
        "false",
        "no",
        "off",
    )


def _codigo_de_ativacao():
    from villas_boas.actions.parser import normalizar

    return normalizar(os.environ.get("DEV_MODE_CODE") or CODIGO_PADRAO)


def eh_codigo_de_ativacao(comando_normalizado):
    return modo_dev_habilitado() and comando_normalizado == _codigo_de_ativacao()


def _aplicar_invulnerabilidade(jogo):
    jogo.god_mode = True  # reaproveita tudo que o God Mode já faz (inventário sem limite, tp, minigames…)
    jogo.hp = VALOR_INFINITO
    jogo.turnos_luz = VALOR_INFINITO
    jogo.turnos_enjoado = 0
    jogo.turnos_no_escuro = 0
    jogo.furia_noite = 0
    jogo.energia_min_noite = VALOR_INFINITO
    jogo.energia_max_noite = VALOR_INFINITO


def ativar(jogo, vindo_do_menu):
    ui = jogo.ui_handler
    jogo.dev_mode = True
    jogo.dev_usado = True
    jogo.dev_radar = True
    jogo.dev_ia = False
    _aplicar_invulnerabilidade(jogo)

    if vindo_do_menu:
        import time

        ui.limpar()
        jogo.dificuldade_escolhida = DIFICULDADE_DEV
        jogo.fast_mode = True
        jogo.estado_atual = "JOGO"
        jogo.tempo_inicio = time.time()

    ui.exibir(
        f"{DOS_VERDE}[sudo] acesso root concedido. MODO DESENVOLVEDOR ATIVO.{RESET}"
    )
    ui.exibir(
        f"{DOS_AMARELO}Vida, luz e itens infinitos. Perseguidor congelado (use 'dev ia on' para soltá-lo).{RESET}"
    )
    ui.exibir(f"{DOS_BRANCO}Digite 'dev' para ver todos os comandos.{RESET}")
    ui.exibir(f"{DOS_VERMELHO}Esta partida não entra no leaderboard.{RESET}")


def desativar(jogo):
    ui = jogo.ui_handler
    jogo.dev_mode = False
    jogo.dev_ia = False
    jogo.god_mode = False
    jogo.hp = VIDA_NORMAL
    jogo.turnos_luz = TURNOS_BATERIA
    jogo.furia_noite = 1
    jogo.energia_min_noite = 100
    jogo.energia_max_noite = 100
    if jogo.dificuldade_escolhida == DIFICULDADE_DEV:
        jogo.dificuldade_escolhida = "NORMAL"
    ui.exibir(
        f"{DOS_AMARELO}[DEV] Modo desenvolvedor desligado. Vida e bateria voltaram ao normal.{RESET}"
    )
    ui.exibir(f"{DOS_VERMELHO}(esta partida continua fora do leaderboard)")


def resetar(jogo):
    """Chamado quando o jogo reinicia depois de um fim de jogo."""
    jogo.dev_mode = False
    jogo.dev_usado = False
    jogo.dev_radar = True
    jogo.dev_ia = False


# ------------------------------------------------------------------- radar
def _bfs(jogo, origem, destino):
    from villas_boas.engine.core import calcular_caminho_bfs

    return calcular_caminho_bfs(jogo.mapa, origem, destino)


def _cor_distancia(dist):
    if dist is None:
        return DOS_BRANCO
    if dist <= 1:
        return DOS_VERMELHO
    if dist == 2:
        return DOS_AMARELO
    return DOS_VERDE


def montar_radar(jogo):
    linhas = []
    caminho = _bfs(jogo, jogo.sala_atual, jogo.ai_sala)
    dist = len(caminho) - 1 if caminho else None
    cor = _cor_distancia(dist)
    ia_txt = "ATIVO" if getattr(jogo, "dev_ia", False) else "CONGELADO"

    if dist is None:
        onde = f"{jogo.ai_sala} (sem rota até você)"
    elif dist == 0:
        onde = f"{jogo.ai_sala} (NA SUA SALA)"
    else:
        onde = f"{jogo.ai_sala} ({dist} sala{'s' if dist != 1 else ''} de distância)"

    linhas.append(
        f"{cor}[RADAR] Perseguidor: {onde} | estado: {jogo.ai_estado} | IA: {ia_txt}{RESET}"
    )
    if caminho and len(caminho) > 1:
        linhas.append(f"{DOS_BRANCO}        rota: {' > '.join(caminho)}{RESET}")
    linhas.append(f"{DOS_BRANCO}        barulho: {jogo.nivel_barulho}%{RESET}")

    if jogo.dificuldade_escolhida == "PESADELO":
        c2 = _bfs(jogo, jogo.sala_atual, jogo.posicao_perseguidor)
        d2 = len(c2) - 1 if c2 else None
        d2_txt = "sem rota" if d2 is None else f"{d2} sala(s)"
        linhas.append(
            f"{_cor_distancia(d2)}[RADAR] Ela (PESADELO): {jogo.posicao_perseguidor} ({d2_txt}){RESET}"
        )
    elif jogo.dificuldade_escolhida == "NORMAL" and getattr(jogo, "dev_ia", False):
        linhas.append(
            f"{DOS_BRANCO}[RADAR] Cronômetro do corredor: {jogo.turnos_mesma_sala}/{jogo.turnos_perseguidor_morte} turnos parado{RESET}"
        )
    return linhas


def imprimir_radar(jogo):
    for linha in montar_radar(jogo):
        jogo.ui_handler.exibir(linha)


# ------------------------------------------------------- itens infinitos
def tirar_snapshot(jogo):
    return {
        "estado": jogo.estado_atual,
        "inv": Counter(jogo.inventario),
        "itens_mapa": _contar_itens_do_mapa(jogo),
    }


def _contar_itens_do_mapa(jogo):
    c = Counter()
    for sala in jogo.mapa.values():
        itens = sala.get("itens") if isinstance(sala, dict) else None
        if isinstance(itens, list):
            c.update(itens)
    return c


def _restaurar_itens(jogo, snap):
    """Devolve o que foi consumido no turno (usar, combinar, minigames…).
    Itens largados/derrubados em alguma sala não contam como consumidos."""
    depois = Counter(jogo.inventario)
    mapa_depois = _contar_itens_do_mapa(jogo)
    for item, qtd_antes in snap["inv"].items():
        faltam = qtd_antes - depois[item]
        derrubados = max(0, mapa_depois[item] - snap["itens_mapa"][item])
        for _ in range(max(0, faltam - derrubados)):
            jogo.inventario.append(item)


def pos_turno(jogo, snap):
    """Roda depois de cada comando normal do jogo, com o modo dev ligado."""
    if snap is not None:
        _restaurar_itens(jogo, snap)

    jogo.hp = VALOR_INFINITO
    jogo.turnos_luz = VALOR_INFINITO
    jogo.turnos_enjoado = 0
    jogo.turnos_no_escuro = 0

    # Condição expandida: intercepta tanto se o combate acionar quanto se estiverem na mesma sala
    if jogo.estado_atual == "COMBATE_ANIMATRONICO" or jogo.ai_sala == jogo.sala_atual:
        _interceptar_perseguidor(jogo)
        from views import imprimir_contexto_sala

        imprimir_contexto_sala(jogo)

    if (
        getattr(jogo, "dev_radar", True)
        and jogo.estado_atual == "JOGO"
        and jogo.sala_atual in jogo.mapa
    ):
        imprimir_radar(jogo)


def _interceptar_perseguidor(jogo):
    """O perseguidor te alcançou: no modo dev ele recua em vez de te matar."""
    ui = jogo.ui_handler
    longe, maior = jogo.ai_sala, -1
    for sala in jogo.mapa:
        if sala in ("morte", "saida", "cama", "final_bom"):
            continue
        caminho = _bfs(jogo, jogo.sala_atual, sala)
        if caminho and len(caminho) > maior:
            longe, maior = sala, len(caminho)

    jogo.estado_atual = "JOGO"
    jogo.ai_sala = longe
    jogo.ai_estado = "PATRULHA"
    jogo.nivel_barulho = 0
    ui.exibir(
        f"{DOS_AMARELO}[DEV] O perseguidor te alcançou, mas o modo dev te protegeu. Ele recuou para: {longe}.{RESET}"
    )


# ------------------------------------------------------------- comandos
def _sala_por_nome(jogo, termo):
    termo = termo.strip()
    if termo in ("ia", "perseguidor", "monstro"):
        return jogo.ai_sala
    return encontrar_melhor_match(termo, list(jogo.mapa.keys()))


def _cmd_tp(jogo, args, estado):
    ui = jogo.ui_handler
    if not args:
        ui.exibir(
            f"{DOS_AMARELO}Uso: tp <sala>  |  tp ia  (vai até o perseguidor){RESET}"
        )
        ui.exibir(f"{DOS_BRANCO}Salas: {', '.join(jogo.mapa.keys())}{RESET}")
        return TRATADO
    destino = _sala_por_nome(jogo, " ".join(args))
    if not destino:
        ui.exibir(
            f"{DOS_VERMELHO}[DEV] Sala '{' '.join(args)}' não existe. Digite 'tp' para listar.{RESET}"
        )
        return TRATADO
    if estado.startswith("MINIGAME"):
        jogo.estado_atual = "JOGO"
        jogo.minigame_atual = None
        ui.exibir(f"{DOS_AMARELO}[DEV] Minigame abortado.{RESET}")
    jogo.sala_atual = destino
    ui.exibir(f"{DOS_AMARELO}[DEV] Teleportado para: {destino}{RESET}")
    return TRATADO_MUDOU_SALA


def _cmd_gerar(jogo, args):
    ui = jogo.ui_handler
    if not args:
        ui.exibir(f"{DOS_AMARELO}Uso: gerar <item>  |  gerar tudo{RESET}")
        ui.exibir(f"{DOS_BRANCO}Itens: {', '.join(descricoes_itens.keys())}{RESET}")
        return TRATADO
    termo = " ".join(args)

    if termo in ("tudo", "todos", "all"):
        novos = [
            i
            for i in descricoes_itens
            if i not in jogo.inventario and i not in _SEM_ITEM_NO_TUDO
        ]
        jogo.inventario.extend(novos)
        ui.exibir(
            f"{DOS_AMARELO}[DEV] {len(novos)} itens adicionados à mochila (exceto 'fios cortados').{RESET}"
        )
        return TRATADO

    if "caroline" in termo or "alma" in termo:  # easter egg original do God Mode
        ui.exibir(
            f"{DOS_VERMELHO}[DEV ERRO FATAL] VOCÊ NÃO PODE CRIAR O QUE JÁ ESTÁ MORTO.{RESET}"
        )
        ui.pausar(2)
        from views import dar_tela_kernel_panic

        dar_tela_kernel_panic(jogo)
        return TRATADO

    item = encontrar_melhor_match(termo, list(descricoes_itens.keys()))
    if not item:
        ui.exibir(
            f"{DOS_VERMELHO}[DEV] Item '{termo}' não catalogado. Digite 'gerar' para listar.{RESET}"
        )
        return TRATADO
    jogo.inventario.append(item)
    if item == "fios cortados":
        jogo.fios_cortados_inventario = True
    ui.exibir(f"{DOS_AMARELO}[DEV] '{item}' adicionado à mochila.{RESET}")
    return TRATADO


def _cmd_radar(jogo, args):
    ui = jogo.ui_handler
    if args and args[0] in ("on", "ligar", "off", "desligar"):
        jogo.dev_radar = args[0] in ("on", "ligar")
        ui.exibir(
            f"{DOS_AMARELO}[DEV] Radar automático {'LIGADO' if jogo.dev_radar else 'DESLIGADO'}.{RESET}"
        )
    if not args or jogo.dev_radar:
        imprimir_radar(jogo)
    return TRATADO


def _cmd_ia(jogo, args):
    ui = jogo.ui_handler
    if not args:
        ui.exibir(
            f"{DOS_AMARELO}[DEV] IA do perseguidor: {'ATIVA' if jogo.dev_ia else 'CONGELADA'} | em: {jogo.ai_sala}{RESET}"
        )
        ui.exibir(
            f"{DOS_BRANCO}Uso: dev ia on | off | <sala> (move o perseguidor){RESET}"
        )
    elif args[0] in ("on", "ligar"):
        jogo.dev_ia = True
        ui.exibir(
            f"{DOS_AMARELO}[DEV] Perseguidor SOLTO. Se te alcançar, ele recua (você não morre).{RESET}"
        )
    elif args[0] in ("off", "desligar"):
        jogo.dev_ia = False
        ui.exibir(f"{DOS_AMARELO}[DEV] Perseguidor CONGELADO.{RESET}")
    else:
        sala = encontrar_melhor_match(" ".join(args), list(jogo.mapa.keys()))
        if not sala or sala in ("morte", "saida", "cama", "final_bom"):
            ui.exibir(f"{DOS_VERMELHO}[DEV] Sala inválida para o perseguidor.{RESET}")
        else:
            jogo.ai_sala = sala
            ui.exibir(f"{DOS_AMARELO}[DEV] Perseguidor movido para: {sala}{RESET}")
    return TRATADO


def _cmd_dificuldade(jogo, args):
    ui = jogo.ui_handler
    opcoes = {"normal": "NORMAL", "pesadelo": "PESADELO", "dev": DIFICULDADE_DEV}
    if not args or args[0] not in opcoes:
        ui.exibir(
            f"{DOS_AMARELO}Uso: dev dificuldade normal | pesadelo | dev  (atual: {jogo.dificuldade_escolhida}){RESET}"
        )
        return TRATADO
    jogo.dificuldade_escolhida = opcoes[args[0]]
    jogo.turnos_mesma_sala = 0
    ui.exibir(
        f"{DOS_AMARELO}[DEV] Dificuldade: {jogo.dificuldade_escolhida}. "
        f"(os perseguidores 'por tempo' só agem com 'dev ia on'){RESET}"
    )
    return TRATADO


def _cmd_status(jogo):
    ui = jogo.ui_handler
    ui.exibir(f"{DOS_VERDE}=== STATUS DEV ==={RESET}")
    linhas = [
        f"sala: {jogo.sala_atual} | estado: {jogo.estado_atual} | dificuldade: {jogo.dificuldade_escolhida}",
        f"hp: ∞ | luz: ∞ | barulho: {jogo.nivel_barulho}% | turnos na mesma sala: {jogo.turnos_mesma_sala}",
        f"perseguidor: {jogo.ai_sala} ({jogo.ai_estado}, alvo: {jogo.ai_alvo or '-'}) | IA: {'ativa' if jogo.dev_ia else 'congelada'}",
        f"perseguidora (pesadelo): {jogo.posicao_perseguidor}",
        f"inventário ({len(jogo.inventario)}): {', '.join(jogo.inventario) or '-'}",
        (
            f"flags: noite_vencida={jogo.noite_vencida} amanheceu={jogo.amanheceu} incendio={jogo.incendio} "
            f"fios_cortados={jogo.fios_cortados_inventario} alberto_desativado={jogo.alberto_desativado}"
        ),
        f"conquistas: {len(jogo.conquistas)} | comandos digitados: {len(jogo.log_comandos)}",
    ]
    for linha in linhas:
        ui.exibir(f"{DOS_BRANCO}{linha}{RESET}")
    return TRATADO


def _cmd_mapa(jogo):
    ui = jogo.ui_handler
    ignorar = {"descrição", "itens", "inspecionaveis", "energia", "cofre_important"}
    ui.exibir(f"{DOS_VERDE}=== MAPA (@ você | X perseguidor) ==={RESET}")
    for nome, sala in jogo.mapa.items():
        if not isinstance(sala, dict):
            continue
        marca = ("@" if nome == jogo.sala_atual else " ") + (
            "X" if nome == jogo.ai_sala else " "
        )
        saidas = ", ".join(
            f"{d}→{v}"
            for d, v in sala.items()
            if d not in ignorar and isinstance(v, str)
        )
        itens = sala.get("itens") or []
        extra = f"  [itens: {', '.join(itens)}]" if itens else ""
        ui.exibir(f"{DOS_BRANCO}{marca} {nome}: {saidas}{extra}{RESET}")
    return TRATADO


def _cmd_ajuda(jogo):
    ui = jogo.ui_handler
    ui.exibir(f"{DOS_VERDE}=== COMANDOS DO MODO DEV ==={RESET}")
    ajuda = [
        (
            "tp <sala> | tp ia | tp",
            "teleporta (ia = até o perseguidor; sem argumento lista as salas)",
        ),
        ("gerar <item> | gerar tudo", "cria itens na mochila"),
        ("radar | dev radar on/off", "mostra / liga / desliga o radar automático"),
        ("dev ia on/off/<sala>", "solta, congela ou move o perseguidor"),
        ("dev mapa", "mapa completo com você (@) e o perseguidor (X)"),
        ("dev status", "estado interno da partida"),
        ("dev dificuldade <normal|pesadelo|dev>", "troca a dificuldade em jogo"),
        ("dev morrer", "força a tela de morte"),
        ("dev off", "desliga o modo dev"),
    ]
    for cmd, desc in ajuda:
        ui.exibir(f"{DOS_AMARELO}{cmd:<42}{DOS_BRANCO}{desc}{RESET}")
    ui.exibir(
        f"{DOS_BRANCO}Sempre ligado: vida, luz e itens infinitos (itens usados voltam sozinhos).{RESET}"
    )
    return TRATADO


def processar_comando_dev(comando, jogo, estado):
    """Trata comandos do modo dev. Devolve NAO_TRATADO / TRATADO / TRATADO_MUDOU_SALA."""
    partes = comando.split()
    if not partes:
        return NAO_TRATADO
    verbo, args = partes[0], partes[1:]
    em_minigame = estado.startswith("MINIGAME")
    em_jogo = estado in ESTADOS_DE_JOGO

    if not (em_jogo or em_minigame):
        return NAO_TRATADO

    if verbo == "dev":
        sub = args[0] if args else "ajuda"
        resto = args[1:]
        if sub == "off":
            desativar(jogo)
            return TRATADO
        if sub in ("ajuda", "help"):
            return _cmd_ajuda(jogo)
        if sub == "status":
            return _cmd_status(jogo)
        if sub == "mapa":
            return _cmd_mapa(jogo)
        if sub == "radar":
            return _cmd_radar(jogo, resto)
        if sub == "ia":
            return _cmd_ia(jogo, resto)
        if sub == "dificuldade":
            return _cmd_dificuldade(jogo, resto)
        if sub == "tp":
            return _cmd_tp(jogo, resto, estado)
        if sub == "gerar":
            return _cmd_gerar(jogo, resto)
        if sub == "morrer":
            jogo.estado_atual = "JOGO" if em_minigame else jogo.estado_atual
            jogo.minigame_atual = None
            jogo.sala_atual = "morte"
            return TRATADO_MUDOU_SALA
        jogo.ui_handler.exibir(
            f"{DOS_VERMELHO}[DEV] Subcomando '{sub}' desconhecido. Digite 'dev' para ajuda.{RESET}"
        )
        return TRATADO

    if verbo == "tp":
        return _cmd_tp(jogo, args, estado)
    if em_jogo and verbo == "radar":
        return _cmd_radar(jogo, args)
    if em_jogo and verbo == "gerar":
        return _cmd_gerar(jogo, args)
    return NAO_TRATADO
