import logging
import random
import time

from data import (
    BARULHO_ACAO_COMUM,
    BARULHO_CORRER,
    BARULHO_ERROS_CONSECUTIVOS,
    BARULHO_IR_ABRIR,
    CHANCE_PERDER_ITEM_CORRER,
    COFRE_SENHA,
    LIMITE_BARULHO_FANTASMA,
    LIMITE_ERROS_DICA,
    MAX_INVENTARIO,
    MAX_TENTATIVAS_COFRE,
    REDUCAO_OLHAR_ESPERAR,
    SALA_ENERGIA_DESTRUIDA,
    TEMPO_SPEEDRUN_NEUTRO,
    TEMPO_SPEEDRUN_VERDADEIRO,
    TURNOS_ENJOADO_CORRER,
    VIDA_GOD_MODE,
    VIDA_NORMAL,
)
from state import (  # noqa: F401 (re-exportados)
    CHAVES_NAO_SAIDA,
    SALAS_SEGURAS_PATRULHA,
    SALAS_TERMINAIS,
)
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET
from views import (
    dar_dica_jon,
    dar_tela_de_morte,
    falar_pianista,
    imprimir_contexto_sala,
    imprimir_menu_dificuldade,
    imprimir_tela_boot,
    imprimir_tutorial,
    rodar_final,
)

from villas_boas.actions import processar_comando
from villas_boas.actions.parser import ATALHOS_DIRECAO, COMANDOS_ESCONDER, normalizar
from villas_boas.conteudo import FLIPERAMAS, MINIGAMES, PERGUNTAS, colorir, exibir_cena
from villas_boas.engine import devmode
from villas_boas.engine.minigames.minotauro import MinigameMinotauro
from villas_boas.engine.minigames.seguranca import MinigameSeguranca
from villas_boas.utils import (
    CONQUISTAS_DB,
    atualizar_eventos_de_tempo,
    extrair_argumentos,
)

logger = logging.getLogger(__name__)

# Ações que gastam o turno em silêncio (o barulho cai em vez de subir).
COMANDOS_SILENCIOSOS = frozenset(
    {"olhar", "inventario", "esperar", "aguardar", "z"} | COMANDOS_ESCONDER
)


def calcular_caminho_bfs(mapa, inicio, destino):
    from state import GameState

    jogo_temp = type("Temp", (), {"mapa": mapa})()
    return GameState.calcular_caminho_bfs(jogo_temp, inicio, destino)


def processar_ia_inimigo(jogo):
    relatorio = jogo.avancar_ia_perseguidor()
    if not relatorio.get("ativa"):
        return

    distancia = relatorio["distancia"]
    if relatorio.get("passou_escondido"):
        jogo.ui_handler.buffer.append(
            "@@TYPE@@amarelo@@15@@O animatrônico atravessa a sala, fareja o ar... e segue adiante sem te ver."
        )
    elif relatorio["combate_iniciado"]:
        jogo.ui_handler.buffer.append(
            "@@TYPE@@vermelho@@15@@O animatrônico irrompe na sala! Ele encontrou você!"
        )
    elif distancia == 1:
        jogo.ui_handler.buffer.append(
            "@@PASSO@@Você ouve passos metálicos pesados na sala ao lado..."
        )
    elif distancia == 2:
        jogo.ui_handler.buffer.append(
            "@@TYPE@@amarelo@@15@@Um baque surdo ecoa pelos corredores distantes."
        )


def _fugir_da_ela(jogo):
    """Fuga bem-sucedida da 'Ela': custa HP, mas a deixa atordoada."""
    from villas_boas.ela import atordoar_apos_fuga

    ui = jogo.ui_handler
    dano = int(jogo.perfil_ela.get("dano_fuga", 1))
    if jogo.aplicar_dano(dano):
        jogo.causa_morte = "ela"
        ui.exibir(
            f"\n{DOS_VERMELHO}As garras dela rasgam suas costas enquanto você foge. Você não chega longe...{RESET}"
        )
        return
    atordoar_apos_fuga(jogo)
    ui.exibir(
        f"\n{DOS_VERMELHO}As garras dela rasgam suas costas, mas você escapa! (-{dano} HP){RESET}"
    )
    ui.exibir(
        f"{DOS_AMARELO}Ela bate contra o batente da porta e fica desorientada por alguns instantes.{RESET}"
    )


def aplicar_efeitos_pos_turno(jogo, comando, comando_correu):
    ui = jogo.ui_handler

    if comando_correu:
        jogo.nivel_barulho = BARULHO_CORRER
        jogo.ai_alvo = jogo.sala_atual
        jogo.turnos_enjoado = TURNOS_ENJOADO_CORRER

        if (
            jogo.inventario
            and len(jogo.inventario) > 0
            and random.random() <= CHANCE_PERDER_ITEM_CORRER
        ):
            item_perdido = random.choice(jogo.inventario)
            jogo.inventario.remove(item_perdido)
            sala_atual = jogo.mapa.get(jogo.sala_atual, {})
            sala_atual.setdefault("itens", []).append(item_perdido)
            ui.buffer.append(
                f"@@TYPE@@vermelho@@15@@Você corre desesperado. No pânico, você tropeça e deixa cair seu(sua) {item_perdido.upper()}!"
            )
        else:
            ui.buffer.append(
                "@@TYPE@@vermelho@@15@@Você foge correndo! Você deixa tudo para trás num piscar de olhos"
            )
        ui.buffer.append(
            "@@TYPE@@amarelo@@10@@O animatrônico te perde de vista enquanto corre."
        )

    elif comando.startswith(("ir ", "abrir ")):
        jogo.nivel_barulho = min(100, jogo.nivel_barulho + BARULHO_IR_ABRIR)
    elif comando in COMANDOS_SILENCIOSOS:
        jogo.nivel_barulho = max(0, jogo.nivel_barulho - REDUCAO_OLHAR_ESPERAR)
    else:
        jogo.nivel_barulho = min(100, jogo.nivel_barulho + BARULHO_ACAO_COMUM)

    if not comando_correu:
        processar_ia_inimigo(jogo)

    if (
        getattr(jogo, "dificuldade_escolhida", "") == "DESAFIO_FANTASMA"
        and jogo.nivel_barulho > LIMITE_BARULHO_FANTASMA
    ):
        ui.buffer.append("@@JUMPSCARE@@")
        ui.exibir(f"\n{DOS_VERMELHO}Você quebrou o silêncio. Eles ouviram.{RESET}")
        ui.pausar(1.5)
        jogo.sala_atual = "morte"
        jogo.causa_morte = "fantasma"
        jogo.estado_atual = "FIM"


ARTE_COFRE = r"""
  __________________________
 /  ______________________  \
|  |  __   __   __   __   |  |
|  | |  | |  | |  | |  |  |  |
|  | |__| |__| |__| |__|  |  |
|  |                      |  |
|  |       .------.       |  |
|  |      /        \      |  |
|  |     |   [**]   |     |  |
|  |      \        /      |  |
|  |       `------'       |  |
|  |                      |  |
|  |______________________|  |
 \__________________________/
"""


ARTE_COFRE_TENSO = r"""
  __________________________
 /  ______________/\_______  \
|  |  __   __   __\/  __   |  |
|  | |  | |  | |  |  |  |  |  |
|  | |__| |__|/|__|  |__|  |  |
|  |         /            |  |
|  |       .------.       |  |
|  |      /   !!   \      |  |
|  |     | x [**] x |     |  |
|  |      \        /      |  |
|  |       `------'       |  |
|  |                      |  |
|  |______________________|  |
 \__________________________/
"""


def gerar_charada_cofre(jogo):
    """Monta o texto de pistas embaralhando a ordem de exibição a cada
    tentativa, mas mantendo a integridade das frases."""
    pistas = MINIGAMES["cofre"]["pistas"].copy()
    random.shuffle(pistas)

    linhas = []
    for pista in pistas:
        linhas.append(f"  {pista}")

    return "\n".join(linhas)


def imprimir_painel_cofre(jogo, primeira_vez=False):
    ui = jogo.ui_handler
    tentativas = getattr(jogo, "cofre_tentativas", 0)

    if tentativas == 0:
        ui.animar(f"{DOS_BRANCO}{ARTE_COFRE}{RESET}", 0.015, jogo=jogo)
    else:
        ui.animar(f"{DOS_VERMELHO}{ARTE_COFRE_TENSO}{RESET}", 0.015, jogo=jogo)

    if primeira_vez:
        ui.exibir(
            f"{DOS_BRANCO}O cofre de ferro possui um teclado numérico antigo, gasto pelo tempo.{RESET}"
        )
        ui.exibir(
            f"{DOS_BRANCO}Gravado a canivete embaixo do teclado, quatro frases soltas:{RESET}"
        )
        ui.exibir(gerar_charada_cofre(jogo))

    restantes = MAX_TENTATIVAS_COFRE - tentativas
    if tentativas > 0:
        ui.exibir(
            f"{DOS_VERMELHO}Tentativas restantes antes do painel travar: {restantes}{RESET}"
        )
    ui.exibir(f"{DOS_VERDE}Digite a senha de 4 dígitos: {RESET}")


def registrar_telemetria_segura(evento, sala, dificuldade, motivo, jogo=None):
    try:
        from flask import has_request_context, session
        from services.telemetry_service import registrar_telemetria

        if getattr(jogo, "dev_usado", False) or not has_request_context():
            return
        registrar_telemetria(
            evento,
            sala,
            dificuldade,
            motivo,
            jogo=jogo,
            sid_sessao=session.get("sid"),
            permite_telemetria=session.get("permite_telemetria", True),
        )
    except (ImportError, AttributeError) as e:
        logger.warning(f"Falha ao registrar telemetria ({evento}/{motivo}): {e}")


def desbloquear_conquista(jogo, id_conquista):
    if id_conquista in CONQUISTAS_DB and id_conquista not in jogo.conquistas:
        jogo.conquistas.append(id_conquista)
        jogo.novas_conquistas_turno.append(CONQUISTAS_DB[id_conquista])


def verificar_final_de_jogo(jogo):
    if jogo.sala_atual == "morte":
        registrar_telemetria_segura(
            "MORTE", jogo.sala_atual, jogo.dificuldade_escolhida, "Morte no Mapa", jogo
        )
        dar_tela_de_morte(jogo)
        desbloquear_conquista(jogo, "primeira_morte")
        return True

    if jogo.sala_atual == "saida":
        registrar_telemetria_segura(
            "VITORIA",
            jogo.sala_atual,
            jogo.dificuldade_escolhida,
            "Final Covarde",
            jogo,
        )
        rodar_final("saida", jogo)
        return True

    if jogo.sala_atual == "cama":
        registrar_telemetria_segura(
            "VITORIA",
            jogo.sala_atual,
            jogo.dificuldade_escolhida,
            "Final Dorminhoco",
            jogo,
        )
        rodar_final("cama", jogo)
        return True

    if jogo.sala_atual == "hall de entrada" and getattr(jogo, "noite_vencida", False):
        if getattr(jogo, "fios_cortados_inventario", False):
            registrar_telemetria_segura(
                "VITORIA",
                jogo.sala_atual,
                jogo.dificuldade_escolhida,
                "Final Verdadeiro",
                jogo,
            )
            rodar_final("verdadeiro", jogo)
            desbloquear_conquista(jogo, "final_verdadeiro")
            if jogo.dificuldade_escolhida == "PESADELO":
                desbloquear_conquista(jogo, "pesadelo_verdadeiro")
            elif jogo.dificuldade_escolhida == "NORMAL":
                desbloquear_conquista(jogo, "normal_zerado")
            if (
                jogo.dificuldade_escolhida == "DESAFIO_SPEEDRUN"
                and jogo.tempo_total_segundos < TEMPO_SPEEDRUN_VERDADEIRO
            ):
                desbloquear_conquista(jogo, "speedrun")
        else:
            registrar_telemetria_segura(
                "VITORIA",
                jogo.sala_atual,
                jogo.dificuldade_escolhida,
                "Final Neutro",
                jogo,
            )
            rodar_final("final_bom", jogo)
            desbloquear_conquista(jogo, "final_bom")
            if jogo.dificuldade_escolhida == "PESADELO":
                desbloquear_conquista(jogo, "pesadelo_neutro")
            elif jogo.dificuldade_escolhida == "NORMAL":
                desbloquear_conquista(jogo, "normal_zerado")
            if (
                jogo.dificuldade_escolhida == "DESAFIO_SPEEDRUN"
                and jogo.tempo_total_segundos < TEMPO_SPEEDRUN_NEUTRO
            ):
                desbloquear_conquista(jogo, "speedrun")
        return True
    return False


def mostrar_escolha_peca(estado, ui):
    etapa = MINIGAMES["montagem"][estado]
    opcoes = " | ".join(f"{tecla}- {nome}" for tecla, nome in etapa["opcoes"].items())
    ui.exibir(colorir(etapa["fase"]))
    ui.exibir(f"{etapa['pergunta']} ({opcoes}): ")


def iniciar_fliperama(maquina, jogo):
    ui = jogo.ui_handler
    item = maquina.get("requer")
    if item and item not in jogo.inventario:
        ui.exibir(colorir(maquina["bloqueado"]))
        return
    if maquina.get("consumir"):
        jogo.inventario.remove(item)
    jogo.estado_atual = maquina["estado"]
    ui.limpar()
    exibir_cena([{"arte": maquina["arte"], "velocidade": 0.015}], jogo)
    ui.animar(maquina["titulo"], 0.03, DOS_VERDE, jogo)
    for texto in maquina["introducao"]:
        ui.exibir(colorir(texto))
    if jogo.estado_atual == "MINIGAME_JON":
        jogo.jon_passos_dados = 0
        jogo.jon_caminho_certo = [random.choice(["f", "e", "d"]) for _ in range(4)]
        jogo.jon_dica_falsa = dar_dica_jon(jogo.jon_caminho_certo[0], ui)
        ui.exibir(maquina["primeiro_passo"])
    elif jogo.estado_atual in MINIGAMES["montagem"]:
        jogo.web_consertos = {}
        mostrar_escolha_peca(jogo.estado_atual, ui)
    elif jogo.estado_atual in PERGUNTAS:
        jogo.web_julgamento = {
            "pontos": 0,
            "vitimas": MINIGAMES["vitimas"]["nomes"].copy(),
        }
        ui.exibir(colorir(PERGUNTAS[jogo.estado_atual]["texto"]))


def processar_fluxo_jogo(comando_bruto, jogo, tem_save=False, callback_load_save=None):
    comando = normalizar(comando_bruto)
    estado_antes = jogo.estado_atual

    if (
        devmode.eh_codigo_de_ativacao(comando)
        and estado_antes in devmode.ESTADOS_ATIVACAO
    ):
        devmode.ativar(jogo, vindo_do_menu=(estado_antes in ["MENU", "AGUARDANDO_DIR"]))
        if estado_antes in ["MENU", "AGUARDANDO_DIR"]:
            imprimir_contexto_sala(jogo)
        return

    if getattr(jogo, "dev_mode", False):
        resultado = devmode.processar_comando_dev(comando, jogo, estado_antes)
        if resultado != devmode.NAO_TRATADO:
            if (
                resultado == devmode.TRATADO_MUDOU_SALA
                and not verificar_final_de_jogo(jogo)
                and jogo.estado_atual == "JOGO"
            ):
                imprimir_contexto_sala(jogo)
            return
        snapshot = devmode.tirar_snapshot(jogo)
        _processar_fluxo_jogo_normal(comando_bruto, jogo, tem_save, callback_load_save)
        if getattr(jogo, "dev_mode", False) and estado_antes != "FIM":
            devmode.pos_turno(jogo, snapshot)
        return

    _processar_fluxo_jogo_normal(comando_bruto, jogo, tem_save, callback_load_save)


def _processar_fluxo_jogo_normal(
    comando_bruto, jogo, tem_save=False, callback_load_save=None
):
    comando = normalizar(comando_bruto)
    ui = jogo.ui_handler
    estado_antes = jogo.estado_atual

    if (
        comando.startswith("tp ")
        and getattr(jogo, "god_mode", False)
        and "MINIGAME" in getattr(jogo, "estado_atual", "")
    ):
        jogo.estado_atual = "JOGO"
        jogo.minigame_atual = None
        ui.exibir(
            f"{DOS_AMARELO}[GOD MODE] Sequência do minigame abortada via distorção espacial.{RESET}"
        )

    if jogo.sala_atual not in jogo.mapa and jogo.sala_atual not in SALAS_TERMINAIS:
        jogo.sala_atual = "01"

    if jogo.estado_atual == "FIM":
        if comando in ["f5", "reiniciar", "restart", "reset", "dir"]:
            jogo.resetar_para_novo_jogo()
            devmode.resetar(jogo)
            if comando == "dir":
                ui.limpar()
                ui.exibir(f"{DOS_BRANCO} Volume in drive A is VILLASBOAS{RESET}")
                ui.exibir(f"{DOS_BRANCO} Directory of A:\\{RESET}\n\n")
                ui.exibir(
                    f"{DOS_VERDE}COMMAND  COM        47.845  02-11-1982  6:00a{RESET}\n"
                )
                ui.exibir(
                    f"{DOS_VERDE}SEGURA   SYS         2.048  02-11-1982  6:00a{RESET}\n"
                )
                ui.exibir(
                    f"{DOS_VERDE}NOTURNO  EXE        18.204  02-11-1982  6:00a{RESET}\n"
                )
                ui.exibir(f"{DOS_AMARELO}         3 file(s)       68.097 bytes{RESET}")
                ui.exibir(
                    f"{DOS_VERDE}=================================================={RESET}"
                )
                jogo.estado_atual = "MENU"
                imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
            else:
                ui.limpar()
                imprimir_tela_boot(ui)
            return
        ui.exibir(
            f"{DOS_VERMELHO}[SISTEMA BLOQUEADO] - Aperte a tecla F5 no teclado para jogar novamente.{RESET}"
        )
        return

    if jogo.estado_atual == "AGUARDANDO_DIR":
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            imprimir_tela_boot(ui)
        elif comando == "help":
            ui.exibir(
                f"{DOS_VERMELHO}[ERRO DO SISTEMA]: O arquivo AJUDA.COM foi corrompido.{RESET}"
            )
        elif comando in [
            "del caroline.sys",
            "delete caroline.sys",
            "erase caroline.sys",
            "format a:",
        ]:
            ui.exibir(
                f"{DOS_VERMELHO}Acesso Negado. O arquivo CAROLINE.SYS está enraizado na sua mente.{RESET}"
            )
            ui.pausar(2)
            from views import dar_tela_kernel_panic

            dar_tela_kernel_panic(jogo)
            return
        elif comando == "exit":
            ui.buffer.append("@@EXIT@@")
        elif comando == "shutdown":
            ui.buffer.append("@@RELOAD@@")
        elif comando == "dir":
            ui.limpar()
            ui.exibir(f"{DOS_BRANCO} Volume in drive A is VILLASBOAS{RESET}")
            ui.exibir(f"{DOS_BRANCO} Directory of A:\\{RESET}\n\n")
            ui.exibir(
                f"{DOS_VERDE}COMMAND  COM        47.845  02-11-1982  6:00a{RESET}\n"
            )
            ui.exibir(
                f"{DOS_VERDE}SEGURA   SYS         2.048  02-11-1982  6:00a{RESET}\n"
            )
            ui.exibir(
                f"{DOS_VERDE}NOTURNO  EXE        18.204  02-11-1982  6:00a{RESET}\n"
            )
            ui.exibir(f"{DOS_AMARELO}         3 file(s)       68.097 bytes{RESET}")
            ui.exibir(
                f"{DOS_VERDE}=================================================={RESET}"
            )
            jogo.estado_atual = "MENU"
            imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
        else:
            ui.exibir(f"{DOS_VERMELHO}Bad command or file name{RESET}")
            ui.exibir(
                f"{DOS_VERDE}Digite {DOS_BRANCO}dir{DOS_VERDE} para acessar os diretórios:{RESET}"
            )

    elif jogo.estado_atual == "MENU":
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
        elif comando == "7" and tem_save:
            ui.limpar()
            if callback_load_save and callback_load_save(jogo):
                ui.animar(
                    f"{DOS_VERDE}JOGO RESTAURADO COM SUCESSO DO SEU AUTOSAVE.{RESET}\n",
                    0.04,
                    jogo=jogo,
                )
                imprimir_contexto_sala(jogo)
            else:
                ui.animar(
                    f"{DOS_VERMELHO}Falha ao ler o seu Autosave.{RESET}",
                    0.04,
                    jogo=jogo,
                )
                imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
        elif comando in ["1", "2", "3", "4"]:
            ui.limpar()
            is_fast = comando in ["2", "4"]
            diff = "PESADELO" if comando in ["3", "4"] else "NORMAL"
            jogo.configurar_dificuldade(diff, fast_mode=is_fast)
            ui.animar(
                f"{DOS_VERDE}MODO {diff} SELECIONADO. BOA SORTE.{RESET}\n",
                0.04,
                jogo=jogo,
            )
            jogo.estado_atual = "JOGO"
            jogo.tempo_inicio = time.time()
            imprimir_tutorial(ui, jogo=jogo)
            imprimir_contexto_sala(jogo)

        elif comando == "2007":
            if not devmode.modo_dev_habilitado():
                ui.animar(
                    f"{DOS_VERMELHO}OPÇÃO INVÁLIDA. DIGITE UM NÚMERO DO MENU (1 A {7 if tem_save else 6}).{RESET}",
                    0.04,
                    jogo=jogo,
                )
                return

            ui.limpar()
            jogo.configurar_dificuldade("GOD MODE", fast_mode=False)
            jogo.estado_atual = "JOGO"
            jogo.tempo_inicio = time.time()
            ui.animar(
                f"{DOS_AMARELO}MODO DEUS ATIVADO. ACESSO AOS BASTIDORES CONCEDIDO.{RESET}\n",
                0.04,
                jogo=jogo,
            )
            imprimir_contexto_sala(jogo)
            desbloquear_conquista(jogo, "trapaceiro")

        elif comando == "1980":
            ui.exibir("@@PLAY_1980_SLIDESHOW@@")
            return
        else:
            ui.animar(
                f"{DOS_VERMELHO}OPÇÃO INVÁLIDA. DIGITE UM NÚMERO DO MENU (1 A {7 if tem_save else 6}).{RESET}",
                0.04,
                jogo=jogo,
            )

    elif jogo.estado_atual in ["JOGO", "COMBATE_ANIMATRONICO"]:
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            imprimir_contexto_sala(jogo)

        elif jogo.sala_atual == MINIGAMES["minotauro"]["sala"] and not getattr(
            jogo, "fios_cortados_inventario", False
        ):
            jogo.minigame_atual = MinigameMinotauro(jogo)
            jogo.estado_atual = "MINIGAME_MINOTAURO"
            jogo.minigame_atual.imprimir_status()

        elif comando in MINIGAMES["seguranca"]["comandos"]:
            cadeira = MINIGAMES["seguranca"]
            if jogo.sala_atual != cadeira["sala"]:
                ui.exibir(cadeira["fora_da_sala"])
            elif getattr(jogo, "noite_vencida", False):
                ui.exibir(colorir(cadeira["noite_vencida"]))
            elif cadeira["requer"] not in jogo.inventario and not getattr(
                jogo, "god_mode", False
            ):
                ui.exibir(colorir(cadeira["bloqueado"]))
            else:
                jogo.minigame_atual = MinigameSeguranca(jogo)
                jogo.estado_atual = "MINIGAME_SEGURANCA"
                jogo.minigame_atual.imprimir_status()

        elif (
            comando in MINIGAMES["cofre"]["comandos"]
            and jogo.sala_atual == MINIGAMES["cofre"]["sala"]
        ):
            jogo.estado_atual = "MINIGAME_COFRE"
            jogo.cofre_tentativas = 0
            imprimir_painel_cofre(jogo, primeira_vez=True)

        elif maquina := FLIPERAMAS.get((jogo.sala_atual, comando)):
            iniciar_fliperama(maquina, jogo)

        else:
            jogador_correu = False
            fugindo_de = ""
            comando_para_executar = comando_bruto
            jogo.escondido = False

            if comando.startswith("correr "):
                direcao = comando.replace("correr ", "").strip()
                mapa_direcoes = {
                    "f": "frente",
                    "t": "atrás",
                    "e": "esquerda",
                    "d": "direita",
                }
                direcao = mapa_direcoes.get(direcao, direcao)
                comando_para_executar = f"ir {direcao}"
                comando = comando_para_executar
                jogador_correu = True
                if jogo.estado_atual == "COMBATE_ANIMATRONICO":
                    jogo.estado_atual = "JOGO"
                    fugindo_de = jogo.atacante_combate or "animatronico"

            gastou_turno = processar_comando(comando_para_executar, jogo, jogo.mapa)

            if gastou_turno and fugindo_de:
                jogo.atacante_combate = ""
                if fugindo_de == "ela":
                    _fugir_da_ela(jogo)

            if gastou_turno:
                if jogo.sala_atual != "morte":
                    atualizar_eventos_de_tempo(jogo)
                jogo.erros_consecutivos = 0
                if jogo.estado_atual == "JOGO" and gastou_turno != "olhar":
                    aplicar_efeitos_pos_turno(
                        jogo, ATALHOS_DIRECAO.get(comando, comando), jogador_correu
                    )
            else:
                if jogador_correu and estado_antes == "COMBATE_ANIMATRONICO":
                    jogo.estado_atual = "COMBATE_ANIMATRONICO"

                jogo.erros_consecutivos += 1
                if jogo.erros_consecutivos >= LIMITE_ERROS_DICA:
                    jogo.alterar_barulho(BARULHO_ERROS_CONSECUTIVOS)
                    jogo.ai_alvo = jogo.sala_atual
                    ui.animar(
                        "Seus tropeços no escuro estão chamando muita atenção...",
                        0.03,
                        DOS_AMARELO,
                        jogo,
                    )
                    jogo.erros_consecutivos = 0

            if (
                jogo.sala_atual not in jogo.mapa
                and jogo.sala_atual not in SALAS_TERMINAIS
            ):
                jogo.sala_atual = "01"

            if jogo.estado_atual == "FIM" and getattr(jogo, "incendio", False):
                registrar_telemetria_segura(
                    "VITORIA",
                    jogo.sala_atual,
                    jogo.dificuldade_escolhida,
                    "Final Verdadeiro",
                    jogo,
                )
                desbloquear_conquista(jogo, "final_verdadeiro")
                rodar_final("verdadeiro", jogo)
                return

            if jogo.atingiu_meta_acumulador():
                desbloquear_conquista(jogo, "acumulador")

            if not verificar_final_de_jogo(jogo) and jogo.estado_atual == "JOGO":
                imprimir_contexto_sala(jogo)
    elif jogo.estado_atual == "MENU_DESAFIOS":
        if comando == "voltar":
            jogo.estado_atual = "MENU"
            imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
            return

        jogo.fast_mode = False
        jogo.hp = VIDA_NORMAL
        jogo.furia_noite = 1
        jogo.energia_min_noite = 100
        jogo.energia_max_noite = 100

        if comando == "1":
            jogo.dificuldade_escolhida = "DESAFIO_SPEEDRUN"
            jogo.fast_mode = True
            ui.animar(
                f"{DOS_AMARELO}MODO SPEEDRUN ATIVADO. O CRONÔMETRO ESTÁ RODANDO.{RESET}\n",
                0.04,
                jogo=jogo,
            )
        elif comando == "2":
            jogo.dificuldade_escolhida = "DESAFIO_FANTASMA"
            ui.animar(
                f"{DOS_AMARELO}MODO FANTASMA ATIVADO. FAÇA SILÊNCIO ABSOLUTO.{RESET}\n",
                0.04,
                jogo=jogo,
            )
        elif comando == "3":
            jogo.dificuldade_escolhida = "DESAFIO_BREU"
            jogo.turnos_luz = 0
            ui.animar(
                f"{DOS_AMARELO}MODO BREU TOTAL ATIVADO. VOCÊ ESTÁ NO ESCURO.{RESET}\n",
                0.04,
                jogo=jogo,
            )
        else:
            ui.exibir(f"{DOS_VERMELHO}Opção inválida.{RESET}")
            return

        jogo.estado_atual = "JOGO"
        jogo.tempo_inicio = time.time()
        imprimir_tutorial(ui, jogo=jogo)
        imprimir_contexto_sala(jogo)

    elif jogo.estado_atual == "MENU_CUSTOM_BAT":
        try:
            valor = int(comando)
            if not 0 <= valor <= VIDA_GOD_MODE:
                ui.exibir(f"Escolha inválida. Use um inteiro de 0 a {VIDA_GOD_MODE}.")
                return
            jogo.turnos_luz = valor
            jogo.estado_atual = "MENU_CUSTOM_INV"
            ui.exibir(
                "\n2. Bônus de Espaço na Mochila Inicial (Digite um número, ex: 0, 3, 6):"
            )
        except ValueError:
            ui.exibir(
                f"{DOS_VERMELHO}Por favor, digite apenas números inteiros.{RESET}"
            )

    elif jogo.estado_atual == "MENU_CUSTOM_INV":
        try:
            valor = int(comando)
            if not 0 <= valor <= 3000 or valor % 3 != 0:
                ui.exibir(
                    f"Escolha inválida. Use um inteiro de 0 a {3000} em múltiplos de 3."
                )
                return
            jogo.bolsas_coletadas = valor // 3
            jogo.estado_atual = "MENU_CUSTOM_HP"
            ui.exibir("\n3. Pontos de Vida (HP) Iniciais (ex: 1, 3, 10):")
        except ValueError:
            ui.exibir(
                f"{DOS_VERMELHO}Por favor, digite apenas números inteiros.{RESET}"
            )

    elif jogo.estado_atual == "MENU_CUSTOM_HP":
        try:
            valor = int(comando)
            if not 1 <= valor <= VIDA_GOD_MODE:
                ui.exibir(f"Escolha inválida. Use um inteiro de 1 a {VIDA_GOD_MODE}.")
                return
            jogo.hp = valor
            jogo.dificuldade_escolhida = "CUSTOM"
            jogo.estado_atual = "JOGO"
            jogo.tempo_inicio = time.time()
            ui.limpar()
            ui.animar(
                f"{DOS_VERDE}PARÂMETROS ACEITOS. INICIANDO SIMULAÇÃO PERSONALIZADA.{RESET}\n",
                0.04,
                jogo=jogo,
            )
            imprimir_tutorial(ui, jogo=jogo)
            imprimir_contexto_sala(jogo)
        except ValueError:
            ui.exibir(
                f"{DOS_VERMELHO}Por favor, digite apenas números inteiros.{RESET}"
            )

    # minigame do cofre
    elif jogo.estado_atual == "MINIGAME_COFRE":
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            imprimir_painel_cofre(jogo, primeira_vez=True)

        elif comando == COFRE_SENHA:
            ui.exibir(
                f"{DOS_VERDE} Um som de 'click'. A pesada porta de metal se abre.{RESET}"
            )

            if getattr(jogo, "cofre_tentativas", 0) == 0:
                desbloquear_conquista(jogo, "mente_brilhante")

            sala = jogo.mapa[jogo.sala_atual]
            sala.setdefault("itens", [])

            if (
                "chave dos fundos" not in jogo.inventario
                and "chave dos fundos" not in sala["itens"]
            ):
                if len(jogo.inventario) < (
                    MAX_INVENTARIO + (getattr(jogo, "bolsas_coletadas", 0) * 3)
                ) or getattr(jogo, "god_mode", False):
                    ui.exibir(
                        f"{DOS_AMARELO}Você encontrou a 'chave dos fundos' suja de graxa lá dentro!{RESET}"
                    )
                    jogo.inventario.append("chave dos fundos")
                else:
                    ui.exibir(
                        f"{DOS_AMARELO}Sua mochila está cheia! A 'chave dos fundos' caiu no chão da sala.{RESET}"
                    )
                    sala["itens"].append("chave dos fundos")
            else:
                ui.exibir("O cofre está vazio. Apenas poeira.")

            jogo.cofre_tentativas = 0
            jogo.estado_atual = "JOGO"
            imprimir_contexto_sala(jogo)

        else:
            jogo.cofre_tentativas = getattr(jogo, "cofre_tentativas", 0) + 1

            if jogo.cofre_tentativas >= MAX_TENTATIVAS_COFRE:
                registrar_telemetria_segura(
                    "FALHA_MINIGAME",
                    "01",
                    jogo.dificuldade_escolhida,
                    "Cofre Travado",
                    jogo,
                )

                ui.exibir(
                    f"{DOS_VERMELHO} ⛝ Senha incorreta. O painel solta uma fagulha e trava com um estalo seco.⛝{RESET}"
                )
                ui.exibir(
                    f"{DOS_AMARELO}O teclado esfria, morto. Talvez ele aceite tentar de novo mais tarde.{RESET}"
                )

                jogo.cofre_tentativas = 0
                jogo.estado_atual = "JOGO"
                imprimir_contexto_sala(jogo)
            else:
                mensagens_erro = [
                    f"{DOS_VERMELHO} ⛝ Senha incorreta. Painel pisca em vermelho.⛝{RESET}",
                    f"{DOS_VERMELHO} ⛝ Errado. Um zumbido baixo sai de dentro do cofre.⛝{RESET}",
                ]
                ui.exibir(
                    mensagens_erro[
                        min(jogo.cofre_tentativas - 1, len(mensagens_erro) - 1)
                    ]
                )
                imprimir_painel_cofre(jogo, primeira_vez=False)

    elif jogo.estado_atual == "MINIGAME_JON":
        passo = getattr(jogo, "jon_passos_dados", 0)

        frases_progresso = MINIGAMES["fliperamas"]["jon"]["frases_progresso"]

        if comando in ["f", "e", "d", "frente", "esquerda", "direita"]:
            letra = comando[0]
            if letra == jogo.jon_caminho_certo[passo]:
                ui.exibir(
                    f"{DOS_BRANCO}{frases_progresso[min(passo, len(frases_progresso) - 1)]}{RESET}"
                )
                jogo.jon_passos_dados += 1
                if jogo.jon_passos_dados == 4:
                    ui.exibir(
                        f"\n{DOS_VERDE}Jon encontrou a 'comida'. A tela pinga um pixel vermelho.{RESET}"
                    )
                    ui.exibir(
                        f"{DOS_VERMELHO}MENSAGEM: 'Eles ainda estão aqui.'{RESET}"
                    )
                    jogo.inventario.append("moeda velha")
                    ui.exibir(
                        f"{DOS_AMARELO}A maquina cospe uma ficha velha, de 1982, você guarda na sua bolsa."
                    )

                    jogo.turnos_luz = max(0, jogo.turnos_luz - 1)
                    jogo.estado_atual = "JOGO"
                    imprimir_contexto_sala(jogo)
                else:
                    jogo.jon_dica_falsa = dar_dica_jon(
                        jogo.jon_caminho_certo[jogo.jon_passos_dados], ui
                    )
                    ui.exibir(
                        f"Passo {jogo.jon_passos_dados + 1}/4 - Direção (F/E/D): "
                    )
            else:
                frases_erro = MINIGAMES["fliperamas"]["jon"]["frases_erro"]
                erro_idx = min(getattr(jogo, "jon_erros", 0), len(frases_erro) - 1)
                jogo.jon_erros = getattr(jogo, "jon_erros", 0) + 1
                jogo.turnos_luz = max(0, jogo.turnos_luz - 1)
                morreu = False
                if jogo.jon_dica_falsa:
                    # O sensório mentiu: o jogador não tinha como acertar, então
                    # o erro custa luz e barulho, mas não vida.
                    ui.exibir(
                        f"\n{DOS_AMARELO}O sensório te enganou. Jon bate a cabeça no duto e a máquina solta um zumbido alto.{RESET}"
                    )
                    jogo.alterar_barulho(BARULHO_ERROS_CONSECUTIVOS)
                else:
                    ui.exibir(f"\n{DOS_VERMELHO}{frases_erro[erro_idx]}{RESET}")
                    morreu = jogo.aplicar_dano(1)
                jogo.jon_dica_falsa = False
                if morreu:
                    jogo.causa_morte = "jon"
                    registrar_telemetria_segura(
                        "FALHA_MINIGAME",
                        "MINIGAME_JON",
                        jogo.dificuldade_escolhida,
                        "Morto pelo Porco (errou a direção)",
                        jogo,
                    )
                    dar_tela_de_morte(jogo)
                    desbloquear_conquista(jogo, "primeira_morte")
                else:
                    jogo.estado_atual = "JOGO"
                    imprimir_contexto_sala(jogo)
        else:
            ui.exibir("Direção inválida. Use F, E ou D.")

    elif jogo.estado_atual in MINIGAMES["montagem"]:
        etapa = MINIGAMES["montagem"][jogo.estado_atual]
        opcoes = etapa["opcoes"]
        if comando not in opcoes:
            ui.exibir("Escolha inválida. Digite uma das opções: " + ", ".join(opcoes))
            mostrar_escolha_peca(jogo.estado_atual, ui)
            return
        jogo.web_consertos[etapa["peca"]] = comando
        nome_peca = opcoes[comando]
        ui.exibir(DOS_VERDE + etapa["resultado"].replace("{peca}", nome_peca) + RESET)
        if "proxima" in etapa:
            jogo.estado_atual = etapa["proxima"]
            mostrar_escolha_peca(jogo.estado_atual, ui)
            return

        cabeca = jogo.web_consertos.get("cabeca", "1")
        item_secreto = "remedio" if (cabeca == "2" and comando == "2") else None
        ui.exibir(
            f"\n{DOS_VERDE}CONSERTO CONCLUÍDO. O ANIMATRÔNICO SORRI PARA VOCÊ!{RESET}"
        )
        ui.exibir(
            f"\n{DOS_VERMELHO}ele começa a sangrar, e a gritar no seu ouvido.{RESET}"
        )
        ui.pausar(2)
        ui.exibir("@@JUMPSCARE@@")
        ui.pausar(1)
        sala = jogo.mapa[jogo.sala_atual]
        sala.setdefault("itens", [])

        if (
            "chave da cozinha" not in jogo.inventario
            and "chave da cozinha" not in sala["itens"]
        ):
            if len(jogo.inventario) < (
                MAX_INVENTARIO + (getattr(jogo, "bolsas_coletadas", 0) * 3)
            ) or getattr(jogo, "god_mode", False):
                jogo.inventario.append("chave da cozinha")
                ui.exibir(f"{DOS_VERDE}⛋ Você obteve: CHAVE DA COZINHA{RESET}")
                jogo.inventario.append("partitura rasgada")
                ui.exibir(f"{DOS_VERDE}⛋ Você obteve: Partitura Rasgada.")
            else:
                sala["itens"].append("chave da cozinha")
                sala["itens"].append("partitura rasgada")

        if item_secreto:
            if len(jogo.inventario) < MAX_INVENTARIO or getattr(
                jogo, "god_mode", False
            ):
                jogo.inventario.append(item_secreto)
                ui.exibir(
                    f"{DOS_VERDE}⛋ Você obteve um item extra: {item_secreto.upper()}!{RESET}"
                )
            else:
                sala["itens"].append(item_secreto)

        jogo.turnos_luz = max(0, jogo.turnos_luz - 1)
        jogo.estado_atual = "JOGO"
        imprimir_contexto_sala(jogo)

    elif jogo.estado_atual in PERGUNTAS:
        pergunta = PERGUNTAS[jogo.estado_atual]
        respostas = pergunta["respostas"]
        acertou = (
            comando in respostas
            if pergunta["comparacao"] == "exata"
            else any(resposta in comando for resposta in respostas)
        )
        if acertou:
            jogo.web_julgamento["pontos"] += 1
        falar_pianista(acertou, ui, jogo)
        jogo.estado_atual = pergunta["proxima"]
        proxima = PERGUNTAS.get(jogo.estado_atual)
        ui.exibir(
            colorir(
                proxima["texto"] if proxima else MINIGAMES["vitimas"]["perguntas"][0]
            )
        )

    elif jogo.estado_atual in [
        "MINIGAME_JULGAMENTO_V1",
        "MINIGAME_JULGAMENTO_V2",
        "MINIGAME_JULGAMENTO_V3",
    ]:
        acertou = False
        for v in jogo.web_julgamento["vitimas"][:]:
            if v in comando:
                jogo.web_julgamento["vitimas"].remove(v)
                acertou = True

        if acertou:
            falar_pianista(True, ui, jogo)
        else:
            falar_pianista(False, ui, jogo)

        if jogo.estado_atual == "MINIGAME_JULGAMENTO_V1":
            jogo.estado_atual = "MINIGAME_JULGAMENTO_V2"
            ui.exibir(MINIGAMES["vitimas"]["perguntas"][1])

        elif jogo.estado_atual == "MINIGAME_JULGAMENTO_V2":
            jogo.estado_atual = "MINIGAME_JULGAMENTO_V3"
            ui.exibir(MINIGAMES["vitimas"]["perguntas"][2])

        else:
            if len(jogo.web_julgamento["vitimas"]) == 0:
                jogo.web_julgamento["pontos"] += 1

            if jogo.web_julgamento["pontos"] == len(PERGUNTAS) + 1:
                ui.animar(
                    "Obrigado por voltar pela gente, Rogério...", 0.08, DOS_VERDE, jogo
                )
                ui.exibir(
                    f"{DOS_VERDE}⛋ Um compartimento do piano se abre. Você obteve o CARTÃO DE SEGURANÇA{RESET}"
                )
                jogo.inventario.append("cartao de seguranca")
                sala = jogo.mapa[jogo.sala_atual]
                sala.setdefault("itens", [])

                if (
                    "bateria nova" not in jogo.inventario
                    and "bateria nova" not in sala["itens"]
                ):
                    if len(jogo.inventario) < MAX_INVENTARIO or getattr(
                        jogo, "god_mode", False
                    ):
                        jogo.inventario.append("bateria nova")
                        ui.exibir(f"{DOS_VERDE}⛋ Você a guardou na mochila.{RESET}")
                    else:
                        ui.exibir(
                            f"{DOS_AMARELO}⛋ Mochila cheia! A bateria nova caiu no chão.{RESET}"
                        )
                        sala["itens"].append("bateria nova")

                jogo.estado_atual = "JOGO"
                imprimir_contexto_sala(jogo)

            else:
                ui.exibir("@@JUMPSCARE@@")
                ui.pausar(2)
                ui.animar(
                    "Quem é você? *A tela desliga* Você não merece nossa ajuda.",
                    0.05,
                    DOS_VERMELHO,
                    jogo,
                )
                ui.pausar(1)

                registrar_telemetria_segura(
                    "MORTE",
                    "MINIGAME_PIANISTA",
                    jogo.dificuldade_escolhida,
                    "Kernel Panic - Falhou no Julgamento",
                    jogo,
                )

                from views import dar_tela_kernel_panic

                jogo.causa_morte = "julgamento"
                dar_tela_kernel_panic(jogo)

                return

    # bloco universal (minigames orientados a objeto: Segurança e Minotauro)
    elif jogo.estado_atual in ["MINIGAME_SEGURANCA", "MINIGAME_MINOTAURO"]:
        if getattr(jogo, "minigame_atual", None) is None:
            if jogo.estado_atual == "MINIGAME_SEGURANCA":
                jogo.minigame_atual = MinigameSeguranca(jogo)
            elif jogo.estado_atual == "MINIGAME_MINOTAURO":
                jogo.minigame_atual = MinigameMinotauro(jogo)

            if getattr(jogo, "minigame_dados", {}):
                jogo.minigame_atual.__dict__.update(jogo.minigame_dados)

        jogo.minigame_atual.jogo = jogo
        jogo.minigame_atual.ui = jogo.ui_handler

        partes = extrair_argumentos(comando)
        verbo = partes[0] if partes else ""
        mapa_direcoes = {
            "f": "ir frente",
            "t": "ir atrás",
            "e": "ir esquerda",
            "d": "ir direita",
        }
        if verbo in mapa_direcoes:
            comando = mapa_direcoes[verbo]

        resultado = jogo.minigame_atual.processar_turno(comando, jogo)

        if resultado == "continuar":
            jogo.minigame_atual.imprimir_status()

        elif resultado == "morte":
            minigame = (
                "seguranca"
                if jogo.estado_atual == "MINIGAME_SEGURANCA"
                else "minotauro"
            )
            jogo.estado_atual = "FIM"
            jogo.sala_atual = "morte"
            jogo.causa_morte = minigame
            jogo.minigame_atual = None
            registrar_telemetria_segura(
                "MORTE",
                f"MINIGAME_{minigame.upper()}",
                jogo.dificuldade_escolhida,
                "Falhou em um Minigame",
                jogo,
            )
            dar_tela_de_morte(jogo)
            desbloquear_conquista(jogo, "primeira_morte")

        elif isinstance(resultado, str) and resultado.startswith("vitoria_"):
            jogo.estado_atual = "JOGO"
            jogo.minigame_atual = None

            if resultado == "vitoria_seguranca":
                if jogo.sala_atual not in jogo.mapa:
                    jogo.sala_atual = "01"

            elif resultado == "vitoria_minotauro":
                desbloquear_conquista(jogo, "toureiro")
                jogo.sala_atual = "sala dos fundos"
                if getattr(jogo, "god_mode", False) and not getattr(
                    jogo, "fios_cortados_inventario", False
                ):
                    jogo.inventario.append("fios cortados")
                    jogo.fios_cortados_inventario = True
                    ui.exibir(
                        f"{DOS_AMARELO}[GOD MODE] Você arrancou os 'fios cortados' da parede!{RESET}"
                    )

                if "sala dos fundos" in jogo.mapa:
                    jogo.mapa["sala dos fundos"]["energia"] = SALA_ENERGIA_DESTRUIDA
                ui.exibir(
                    f"{DOS_VERDE}A porta cedeu atrás de você. Você sobreviveu.{RESET}"
                )

            imprimir_contexto_sala(jogo)

        if getattr(jogo, "minigame_atual", None) is not None:
            jogo.minigame_dados = {
                k: v
                for k, v in jogo.minigame_atual.__dict__.items()
                if k not in ["jogo", "ui"]
            }
        else:
            jogo.minigame_dados = {}

        return
