import random
import time

from data import CAVEIRA_MORTE
from state import carregar_conquistas, registrar_final
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET
from villas_boas.conteudo import FINAIS, MINIGAMES, exibir_cena
from villas_boas.utils import corromper_texto


def imprimir_tela_boot(ui):
    ui.animar("VILLAS-BOAS INDUSTRIES (C) 1982", 0.04, DOS_BRANCO)
    ui.animar("BIOS VERSION 1.04 - RELEASE 02/11/1982", 0.04, DOS_BRANCO)
    ui.animar("RAM CHECK: 640KB OK", 0.03, DOS_VERDE)
    ui.animar("DRIVE A: READY", 0.03, DOS_VERDE)
    ui.animar("CARREGANDO 'COMMAND.COM'....... OK\n", 0.06, DOS_VERDE)
    ui.exibir(
        f"{DOS_VERDE}Digite {DOS_BRANCO}dir{DOS_VERDE} para acessar os diretórios:{RESET}"
    )


def imprimir_menu_dificuldade(ui, tem_autosave=False, jogo=None):
    ui.animar(
        "==================================================", 0.005, DOS_VERDE, jogo
    )
    ui.animar(
        "__     _____ _     _        _ ____   ___   ____ ", 0.005, DOS_VERDE, jogo
    )
    ui.animar(
        r"\ \   / /_ _| |   | |      / / ___| / _ \ / ___|", 0.005, DOS_VERDE, jogo
    )
    ui.animar(
        r" \ \ / / | || |   | |     / /\___ \| | | |\___ \\", 0.005, DOS_VERDE, jogo
    )
    ui.animar(
        r"  \ V /  | || |___| |___ / /  ___) | |_| | ___) |", 0.005, DOS_VERDE, jogo
    )
    ui.animar(
        r"   \_/  |___|_____|_____/_/  |____/ \___/ |____/", 0.005, DOS_VERDE, jogo
    )
    ui.animar(
        "==================================================", 0.005, DOS_VERDE, jogo
    )
    ui.animar(
        "        SISTEMA DE SEGURANÇA INTEGRADO v1.0       \n", 0.02, DOS_BRANCO, jogo
    )

    conquistas = carregar_conquistas()
    c_med = "[X]" if "mediocre" in conquistas else "[ ]"
    c_son = "[X]" if "bons_sonhos" in conquistas else "[ ]"
    c_bom = "[X]" if "bom" in conquistas else "[ ]"
    c_ver = "[X]" if "verdadeiro" in conquistas else "[ ]"
    qtd = len(set(conquistas) & {"mediocre", "bons_sonhos", "bom", "verdadeiro"})

    ui.animar(
        f"{DOS_AMARELO}♔ FINAIS ALCANÇADOS: {qtd}/4{RESET}", 0.01, DOS_BRANCO, jogo
    )
    ui.animar(
        f"{DOS_BRANCO}{c_med} Medíocre  {c_son} Bons Sonhos  {c_bom} neutro  {c_ver} Verdadeiro{RESET}\n",
        0.01,
        DOS_BRANCO,
        jogo,
    )

    ui.animar(
        f"{DOS_BRANCO}[1] INICIAR: MODO NORMAL (Velocidade MS-DOS Padrão){RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_AMARELO}[2] INICIAR: MODO NORMAL (Texto Rápido — Sem Delays){RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_VERMELHO}[3] INICIAR: MODO PESADELO (Velocidade MS-DOS Padrão){RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_AMARELO}[4] INICIAR: MODO PESADELO (Texto Rápido — Sem Delays){RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_AMARELO}[5] INICIAR: MODOS DE DESAFIO{RESET}", 0.01, DOS_BRANCO, jogo
    )
    ui.animar(
        f"{DOS_AMARELO}[6] INICIAR: MODO PERSONALIZADO{RESET}", 0.01, DOS_BRANCO, jogo
    )

    if tem_autosave:
        ui.animar(
            f"{DOS_VERDE}[7] CONTINUAR JOGO (Autosave Encontrado){RESET}\n",
            0.01,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            f"{DOS_VERDE}SELECIONE UMA OPÇÃO (1-7): {RESET}", 0.01, DOS_BRANCO, jogo
        )
    else:
        ui.animar(
            f"\n{DOS_VERDE}SELECIONE UMA OPÇÃO (1-6): {RESET}", 0.01, DOS_BRANCO, jogo
        )


def imprimir_tutorial(ui, jogo=None):
    ui.animar(
        f"\n{DOS_AMARELO}=================================================={RESET}",
        0.005,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_AMARELO}   [ MANUAL DE SOBREVIVÊNCIA DO SISTEMA v1.0 ]    {RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_AMARELO}=================================================={RESET}",
        0.005,
        DOS_BRANCO,
        jogo,
    )

    ui.animar(f"{DOS_BRANCO}>> NAVEGAÇÃO:{RESET}", 0.01, DOS_BRANCO, jogo)
    ui.animar(
        f"   Mova-se usando direções ({DOS_VERDE}ir frente{RESET}, {DOS_VERDE}ir esquerda{RESET})",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"   ou digite o destino exato: {DOS_VERDE}ir sala de jantar{RESET}.",
        0.01,
        DOS_BRANCO,
        jogo,
    )

    ui.animar(f"\n{DOS_BRANCO}>> INTERAÇÃO:{RESET}", 0.01, DOS_BRANCO, jogo)
    ui.animar(
        f"   Examine os detalhes ao redor com {DOS_VERDE}olhar [objeto]{RESET}.",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"   Para interagir, use {DOS_VERDE}pegar [item]{RESET} ou {DOS_VERDE}usar [item]{RESET}.",
        0.01,
        DOS_BRANCO,
        jogo,
    )

    ui.animar(f"\n{DOS_BRANCO}>> GERENCIAMENTO:{RESET}", 0.01, DOS_BRANCO, jogo)
    ui.animar(
        f"   Digite {DOS_VERDE}inventario{RESET} (ou apenas {DOS_VERDE}i{RESET}) para checar seus bolsos.",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"   Dica: Atalhos funcionam. Digite {DOS_VERDE}p chave{RESET} para pegar rápido.",
        0.01,
        DOS_BRANCO,
        jogo,
    )

    ui.animar(f"\n{DOS_VERMELHO}>> AVISO DE SEGURANÇA:{RESET}", 0.01, DOS_BRANCO, jogo)
    ui.animar(
        f"   Tropeçar no escuro ou errar comandos gera {DOS_VERMELHO}barulho{RESET}.",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"   Ela escuta. Se precisar fugir de uma sala, digite {DOS_VERDE}correr [direção]{RESET}.",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"   Ouça os passos e o ranger das grades. Para sumir, {DOS_VERDE}esconder{RESET}; para baixar o barulho, {DOS_VERDE}esperar{RESET}.",
        0.01,
        DOS_BRANCO,
        jogo,
    )

    ui.animar(
        f"\n{DOS_AMARELO}Caso precise, digite {DOS_VERDE}ajuda{DOS_AMARELO} para o guia completo novamente.{RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_AMARELO}=================================================={RESET}\n",
        0.005,
        DOS_BRANCO,
        jogo,
    )

    try:
        from data import ARTE_ENTRADA

        ui.animar(f"{DOS_BRANCO}{ARTE_ENTRADA}{RESET}", 0.005, jogo=jogo)
    except ImportError:
        pass


def dar_dica_jon(passo_certo, ui):
    """Exibe a dica do passo; retorna True se a dica exibida foi falsa."""
    dicas = MINIGAMES["fliperamas"]["jon"]["pistas"]
    if random.random() <= 0.25:
        ui.exibir(
            f"\n{DOS_VERMELHO}[SENSÓRIO CONFUSO]: {random.choice([v for k, v in dicas.items() if k != passo_certo])}{RESET}"
        )
        return True
    ui.exibir(f"\n{DOS_AMARELO}[SENSÓRIO]: {dicas[passo_certo]}{RESET}")
    return False


def falar_pianista(acertou, ui, jogo):
    reacao = MINIGAMES["fliperamas"]["julgamento"]["acerto" if acertou else "erro"]
    ui.exibir(f"{DOS_BRANCO}{reacao['nota']}{RESET}")
    ui.animar(f'"{random.choice(reacao["falas"])}"', 0.04, DOS_AMARELO, jogo)


def dar_tela_kernel_panic(jogo):
    jogo.estado_atual = "FIM"
    ui = jogo.ui_handler

    ui.limpar()

    ui.buffer.append("@@PANIC@@")

    ARTE_PANIC = r"""
    .--.
   |0_0 |     -
   |:_/ |    | |  
  //    \\   | |
 (|      |)  | |
/'\_   _/`\  ( )
\___)=(___/  

    """

    ui.animar(ARTE_PANIC, 0, DOS_BRANCO, jogo)

    ui.animar(
        "KERNEL PANIC - NOT SYNCING: Fatal exception in interrupt", 0, DOS_BRANCO, jogo
    )
    ui.animar(
        "Hardware CPU 0: corrupted by external psychological entity.",
        0,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        "Process CAROLINE.SYS (pid: 1982, stackpage=0x00000000)", 0, DOS_BRANCO, jogo
    )
    ui.animar("Call Trace:", 0, DOS_BRANCO, jogo)
    ui.animar(" [<c010a245>] show_stack+0x7a/0x90", 0, DOS_BRANCO, jogo)
    ui.animar(" [<c010a3c4>] show_registers+0x15a/0x1dc", 0, DOS_BRANCO, jogo)
    ui.animar(" [<c010c140>] die_if_no_escape+0x12a/0x240", 0, DOS_BRANCO, jogo)
    ui.animar(" [<c0000000>] DO_NOT_LOOK_BEHIND_YOU+0x666/0x666", 0, DOS_BRANCO, jogo)

    ui.exibir("\nMemory corruption detected. Dumping physical memory to disk...")
    ui.pausar(2)
    ui.exibir("Dump failed. The soul is too heavy.")

    ui.exibir("\nSystem halted. Press F5 to attempt a manual reboot.")


def imprimir_contexto_sala(jogo):

    # trava 1
    if getattr(jogo, "estado_atual", "JOGO") != "JOGO":
        return

    # trava 2
    if jogo.minigame_atual or jogo.sala_atual not in jogo.mapa:
        return

    ui = jogo.ui_handler
    sala = jogo.mapa[jogo.sala_atual]

    ui.exibir("\n" + "=" * 50)
    ui.animar(f"⚇ VOCÊ ESTÁ EM: {jogo.sala_atual.upper()}", 0.01, DOS_VERDE, jogo)

    if getattr(jogo, "amanheceu", False):
        ui.animar(
            f"{DOS_BRANCO} ☀ A luz pálida da manhã ilumina a sala através das frestas.{RESET}",
            0.01,
            DOS_BRANCO,
            jogo,
        )

    descricao_colorida = sala.get("descrição", "")

    intensidade_glitch = 0.0

    if jogo.hp <= 1 and not getattr(jogo, "god_mode", False):
        intensidade_glitch += 0.3
        ui.animar(
            f"{DOS_VERMELHO}[SISTEMA NEUROLÓGICO COMPROMETIDO]{RESET}",
            0.01,
            DOS_VERMELHO,
            jogo,
        )

    barulho = getattr(jogo, "nivel_barulho", 0)
    if barulho > 50:
        intensidade_glitch += (barulho - 50) / 125.0

    if jogo.turnos_luz <= 0 and not getattr(jogo, "amanheceu", False):
        intensidade_glitch += 0.5
        ui.animar(
            f"{DOS_VERMELHO}Sua visão falha na escuridão absoluta...{RESET}",
            0.01,
            DOS_BRANCO,
            jogo,
        )

    if intensidade_glitch > 0 and not getattr(jogo, "god_mode", False):
        descricao_colorida = corromper_texto(
            descricao_colorida, intensidade=min(0.9, intensidade_glitch)
        )
    for inspecionavel in sala.get("inspecionaveis", {}):
        descricao_colorida = descricao_colorida.replace(
            inspecionavel, f"{DOS_AMARELO}{inspecionavel}{RESET}"
        )
    for item in sala.get("itens", []):
        descricao_colorida = descricao_colorida.replace(
            item, f"{DOS_VERDE}{item}{RESET}"
        )

    ui.animar(f"⏿ Visão: {descricao_colorida}", 0.01, DOS_BRANCO, jogo)

    inspecionaveis = list(sala.get("inspecionaveis", {}).keys())
    if inspecionaveis:
        ui.animar(
            f"☞ Investigar: {DOS_AMARELO}{', '.join(inspecionaveis)}{RESET}",
            0.01,
            DOS_BRANCO,
            jogo,
        )

    if len(sala.get("itens", [])) > 0:
        if jogo.turnos_luz > 0 or getattr(jogo, "amanheceu", False):
            itens_formatados = [f"{DOS_VERDE}{item}{RESET}" for item in sala["itens"]]
            ui.animar(
                f"[i] Itens no chão: {', '.join(itens_formatados)}",
                0.01,
                DOS_BRANCO,
                jogo,
            )
        else:
            ui.animar(
                f" {DOS_BRANCO}Deve ter algo no chão, mas escuro demais para ver o quê.{RESET}",
                0.01,
                DOS_BRANCO,
                jogo,
            )

    chaves_ignoradas = [
        "descrição",
        "itens",
        "inspecionaveis",
        "cofre_important",
        "cadeira",
    ]
    saidas = [k for k in sala if k not in chaves_ignoradas and isinstance(sala[k], str)]
    if saidas:
        ui.animar(
            f"⏱ Saídas: {DOS_AMARELO}{', '.join(saidas).title()}{RESET}",
            0.01,
            DOS_BRANCO,
            jogo,
        )
    else:
        ui.animar(
            f"⏱ Saídas: {DOS_VERMELHO}Nenhuma saída aparente...{RESET}",
            0.01,
            DOS_BRANCO,
            jogo,
        )


def dar_tela_de_morte(jogo):
    jogo.estado_atual = "FIM"
    ui = jogo.ui_handler
    ui.buffer.append("@@JUMPSCARE@@")
    ui.pausar(2)
    ui.animar(f"{DOS_VERMELHO}{CAVEIRA_MORTE}{RESET}", 0.002, jogo=jogo)
    ui.animar("☠ GAME OVER. A NOITE ENGOLIU VOCÊ.", 0.05, DOS_VERMELHO, jogo)
    ui.animar(
        "=== SISTEMA CORROMPIDO. APERTE F5 PARA REINICIAR ===", 0.05, DOS_AMARELO, jogo
    )


def animar_frames_ascii(ui, frames, delay=0.4, jogo=None):

    for frame in frames:
        ui.limpar()
        ui.exibir(f"{DOS_BRANCO}{frame}{RESET}")
        ui.pausar(delay)


def rodar_final(tipo_final, jogo):
    jogo.estado_atual = "FIM"
    ui = jogo.ui_handler
    ui.limpar()
    final = FINAIS["finais"].get(tipo_final)
    if final:
        jogo.tempo_total_segundos = max(0.0, time.time() - jogo.tempo_inicio)
        jogo.nome_final = final["nome"]
        jogo.tipo_final = final["tipo"]
        exibir_cena(final["cena"], jogo)
        if registrar_final(final["conquista"]):
            exibir_cena(FINAIS["desbloqueio"], jogo)
    exibir_cena(FINAIS["encerramento"], jogo)
