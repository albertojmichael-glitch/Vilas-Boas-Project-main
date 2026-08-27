import random
import copy
import logging


from villas_boas.actions import processar_comando
from villas_boas.actions.parser import normalizar

from ui import DOS_VERDE, DOS_BRANCO, DOS_AMARELO, DOS_VERMELHO, RESET
from villas_boas.utils import extrair_argumentos, atualizar_eventos_de_tempo

from data import (
    MAX_INVENTARIO, ARTE_PORCO, ARTE_ROBO, ARTE_PIANO, MAPA_ORIGINAL,
    ARTE_PASTA, ARTE_DISQUETE, ARTE_RADAR, VIDA_NORMAL, VIDA_PESADELO
)

from views import (
    imprimir_tela_boot,
    imprimir_menu_dificuldade,
    imprimir_tutorial,
    dar_dica_jon,
    falar_pianista,
    imprimir_contexto_sala,
    dar_tela_de_morte,
    rodar_final
)


from villas_boas.engine.minigames.seguranca import MinigameSeguranca
from villas_boas.engine.minigames.minotauro import MinigameMinotauro

logger = logging.getLogger(__name__)

ARTE_COFRE = r'''
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
'''


def desbloquear_conquista(jogo, id_conquista, nome_exibicao):
    if not hasattr(jogo, 'conquistas'):
        jogo.conquistas = []
        
    if id_conquista not in jogo.conquistas:
        jogo.conquistas.append(id_conquista)
        ui = jogo.ui_handler
        ui.buffer.append(f"@@TYPE@@amarelo@@0@@♔ CONQUISTA DESBLOQUEADA: {nome_exibicao} ♔")


def processar_fluxo_jogo(comando_bruto, jogo, tem_save=False, callback_load_save=None):
    comando = normalizar(comando_bruto)
    ui = jogo.ui_handler

    
    if jogo.sala_atual not in jogo.mapa and jogo.sala_atual not in ["morte", "saida", "cama", "final_bom"]:
        jogo.sala_atual = "01"

    #estado fim de jogo
    if jogo.estado_atual == "FIM":
        if comando in ["f5", "reiniciar", "restart", "reset", "dir"]:
            try:
                jogo.mapa = copy.deepcopy(MAPA_ORIGINAL)
            except (ImportError, AttributeError) as e:
                logger.error(f"Erro ao restaurar MAPA_ORIGINAL: {e}")

            #reset das variaveis
            jogo.estado_atual = "AGUARDANDO_DIR"
            jogo.inventario = []
            jogo.hp = 3
            jogo.sala_atual = "entrada"
            jogo.incendio = False
            jogo.noite_vencida = False
            jogo.fios_cortados_inventario = False
            jogo.amanheceu = False
            jogo.alberto_desativado = False
            jogo.god_mode = False
            jogo.turnos_luz = 0
            jogo.fast_mode = False 
            jogo.dificuldade_escolhida = "NORMAL"
            jogo.minigame_atual = None
            
            if comando == "dir":
                ui.limpar()
                ui.exibir(f"{DOS_BRANCO} Volume in drive A is VILLASBOAS{RESET}")
                ui.exibir(f"{DOS_BRANCO} Directory of A:\\{RESET}\n")
                ui.exibir("")
                ui.exibir(f"{DOS_VERDE}COMMAND  COM        47.845  02-11-1982  6:00a{RESET}")
                ui.exibir("")
                ui.exibir(f"{DOS_VERDE}SEGURA   SYS         2.048  02-11-1982  6:00a{RESET}")
                ui.exibir("")
                ui.exibir(f"{DOS_VERDE}NOTURNO  EXE        18.204  02-11-1982  6:00a{RESET}")
                ui.exibir("")
                ui.exibir(f"{DOS_VERDE}DESKTOP  &lt;DIR&gt;              24-07-2007  4:00a{RESET}")
                ui.exibir("")
                ui.exibir(f"{DOS_VERDE}SAVES    &lt;DIR&gt;              23-07-2008  4:00a{RESET}")
                ui.exibir("")
                ui.exibir(f"{DOS_VERDE}PICTURE  &lt;DIR&gt;              05-11-1994  4:00a{RESET}")
                ui.exibir("")
                ui.exibir(f"{DOS_VERDE}VALID    &lt;DIR&gt;              24-07-2007  4:00a{RESET}")
                ui.exibir("")
                ui.exibir(f"{DOS_AMARELO}         3 file(s)       68.097 bytes{RESET}")
                ui.exibir(f"{DOS_AMARELO}         4 dir(s)       655.360 bytes free{RESET}")
                ui.exibir(f"{DOS_VERDE}=================================================={RESET}")
                jogo.estado_atual = "MENU"
                imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
                return
            else:
                ui.limpar()
                imprimir_tela_boot(ui)
                return
            
        ui.exibir(f"{DOS_VERMELHO}[SISTEMA BLOQUEADO] - Aperte a tecla F5 no teclado para jogar novamente.{RESET}")
        return

    #estado aguardando dir
    if jogo.estado_atual == "AGUARDANDO_DIR":
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            imprimir_tela_boot(ui)
        
        elif comando == "help":
            ui.exibir(f"{DOS_BRANCO}Comandos internos suportados:{RESET}")
            ui.exibir(f"{DOS_VERDE}DIR{RESET}      Exibe uma lista de arquivos e subdiretórios.")
            ui.exibir(f"{DOS_VERDE}CLS{RESET}      Limpa a tela do terminal.")
            ui.exibir(f"{DOS_VERDE}MEM{RESET}      Exibe a quantidade de memória usada e livre no sistema.")
            ui.exibir(f"{DOS_VERDE}CHKDSK{RESET}   Verifica um disco e exibe um relatório de status.")
            ui.exibir(f"{DOS_VERDE}EXIT{RESET}     Encerra o programa COMMAND.COM (Tela Preta).")
            ui.exibir(f"{DOS_VERDE}SHUTDOWN{RESET} Desliga e reinicia a máquina.")
            ui.pausar(1)
            ui.exibir(f"\n{DOS_VERMELHO}[ERRO DO SISTEMA]: O arquivo AJUDA.COM foi corrompido. Você está por conta própria aqui.{RESET}")

        
        elif comando == "mem":
            ui.exibir(f"{DOS_BRANCO}Microsoft Disk Operation System 2007{RESET}")
            ui.exibir(f"{DOS_BRANCO}VILLAS-BOAS BIOS - RELEASE 05/11/1982{RESET}\n")
            ui.exibir(f"{DOS_VERDE}    640K memória convencional total{RESET}")
            ui.exibir(f"{DOS_VERDE}    981K disponíveis para o MS-DOS{RESET}")
            ui.exibir(f"{DOS_VERDE}    598K maior programa executável{RESET}\n")
            ui.pausar(1)
            ui.exibir(f"{DOS_AMARELO}ALERTA: O módulo oculto 'CAROLINE.SYS' está consumindo memória em excesso.{RESET}")

        
        elif comando == "chkdsk":
            ui.exibir(f"{DOS_BRANCO}O tipo de sistema de arquivos é FAT16.{RESET}")
            ui.exibir(f"{DOS_BRANCO}O número de série do volume é 1982-1994{RESET}")
            ui.exibir(f"{DOS_VERDE}Verificando arquivos e pastas...{RESET}")
            ui.pausar(1.5)
            ui.exibir(f"{DOS_VERDE}Verificação de arquivo concluída.{RESET}")
            ui.pausar(1)
            ui.exibir(f"{DOS_VERMELHO}Setores defeituosos encontrados em A:\\VALID\\ROGERIO.DAT{RESET}")
            ui.exibir(f"{DOS_VERMELHO}Erro ao ler registro 44: 'Eu não consigo dormir, ela assombra meus pensamentos.'{RESET}")
            ui.exibir(f"{DOS_AMARELO}O Sistema não pôde reparar os dados corrompidos da sua mente.{RESET}")

        
        elif comando == "exit":
            ui.buffer.append("@@EXIT@@")

        elif comando == "shutdown":
            ui.buffer.append("@@RELOAD@@")

        
        elif comando == "dir":
            ui.limpar()
            ui.exibir(f"{DOS_BRANCO} Volume in drive A is VILLASBOAS{RESET}")
            ui.exibir(f"{DOS_BRANCO} Directory of A:\\{RESET}\n")
            ui.exibir("")
            ui.exibir(f"{DOS_VERDE}COMMAND  COM        47.845  02-11-1982  6:00a{RESET}")
            ui.exibir("")
            ui.exibir(f"{DOS_VERDE}SEGURA   SYS         2.048  02-11-1982  6:00a{RESET}")
            ui.exibir("")
            ui.exibir(f"{DOS_VERDE}NOTURNO  EXE        18.204  02-11-1982  6:00a{RESET}")
            ui.exibir("")
            ui.exibir(f"{DOS_VERDE}DESKTOP  &lt;DIR&gt;              24-07-2007  4:00a{RESET}")
            ui.exibir("")
            ui.exibir(f"{DOS_VERDE}SAVES    &lt;DIR&gt;              23-07-2008  4:00a{RESET}")
            ui.exibir("")
            ui.exibir(f"{DOS_VERDE}PICTURE  &lt;DIR&gt;              05-11-1994  4:00a{RESET}")
            ui.exibir("")
            ui.exibir(f"{DOS_VERDE}VALID    &lt;DIR&gt;              24-07-2007  4:00a{RESET}")
            ui.exibir("")
            ui.exibir(f"{DOS_AMARELO}         3 file(s)       68.097 bytes{RESET}")
            ui.exibir(f"{DOS_AMARELO}         4 dir(s)       655.360 bytes free{RESET}")
            ui.exibir(f"{DOS_VERDE}=================================================={RESET}")
            jogo.estado_atual = "MENU"
            imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
        else:
            ui.exibir(f"{DOS_VERMELHO}Bad command or file name{RESET}")
            ui.exibir(f"{DOS_VERDE}Digite {DOS_BRANCO}dir{DOS_VERDE} para acessar os diretórios:{RESET}")

    #estado menu

    elif jogo.estado_atual == "MENU":
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
        elif comando == "5" and tem_save:
            ui.limpar()
            if callback_load_save and callback_load_save(jogo):
                ui.animar(f"{DOS_VERDE}JOGO RESTAURADO COM SUCESSO DO SEU AUTOSAVE.{RESET}\n", 0.04, jogo=jogo)
                imprimir_contexto_sala(jogo)
            else:
                ui.animar(f"{DOS_VERMELHO}Falha ao ler o seu Autosave.{RESET}", 0.04, jogo=jogo)
                imprimir_menu_dificuldade(ui, tem_autosave=tem_save, jogo=jogo)
        elif comando in ["1", "2", "3", "4"]:
            ui.limpar()
            if comando == "1":
                jogo.dificuldade_escolhida = "NORMAL"
                jogo.fast_mode = False
                jogo.hp = VIDA_NORMAL; jogo.furia_noite = 1; jogo.energia_min_noite = 100; jogo.energia_max_noite = 100
                ui.animar(f"{DOS_VERDE}MODO NORMAL SELECIONADO. VELOCIDADE RETRÔ ATIVADA.{RESET}\n", 0.04, jogo=jogo)
            elif comando == "2":
                jogo.dificuldade_escolhida = "NORMAL"
                jogo.fast_mode = True
                jogo.hp = VIDA_NORMAL; jogo.furia_noite = 1; jogo.energia_min_noite = 100; jogo.energia_max_noite = 100
                ui.animar(f"{DOS_AMARELO}MODO NORMAL COM TEXTO RÁPIDO SELECIONADO.{RESET}\n", 0.04, jogo=jogo)
            elif comando == "3":
                jogo.dificuldade_escolhida = "PESADELO"
                jogo.fast_mode = False
                jogo.hp = VIDA_PESADELO; jogo.furia_noite = 2; jogo.energia_min_noite = 70; jogo.energia_max_noite = 82
                ui.animar(f"{DOS_VERMELHO}MODO PESADELO SELECIONADO. VELOCIDADE RETRÔ ATIVADA. BOA SORTE.{RESET}\n", 0.04, jogo=jogo)
            elif comando == "4":
                jogo.dificuldade_escolhida = "PESADELO"
                jogo.fast_mode = True
                jogo.hp = VIDA_PESADELO; jogo.furia_noite = 2; jogo.energia_min_noite = 70; jogo.energia_max_noite = 82
                ui.animar(f"{DOS_VERMELHO}MODO PESADELO COM TEXTO RÁPIDO SELECIONADO. BOA SORTE.{RESET}\n", 0.04, jogo=jogo)

            jogo.estado_atual = "JOGO"
            imprimir_tutorial(ui, jogo=jogo)
            ui.animar(f"{DOS_BRANCO}Você entra no restaurante. Sua lanterna velha dá três piscadas fracas...{RESET}", 0.04, jogo=jogo)
            ui.animar(f"{DOS_AMARELO}[AVISO DO SISTEMA]: BATERIA DA LANTERNA EM 5%. PROCURAR OUTRA FONTE DE LUZ EM ATÉ 3 TURNOS.{RESET}", 0.04, jogo=jogo)
            imprimir_contexto_sala(jogo)
            
        elif comando == "2007":
            ui.limpar()
            jogo.dificuldade_escolhida = "GOD MODE"
            jogo.god_mode = True
            jogo.hp = 9999; jogo.furia_noite = 0; jogo.energia_min_noite = 9999; jogo.energia_max_noite = 9999
            jogo.turnos_luz = 9999
            jogo.estado_atual = "JOGO"
            ui.animar(f"{DOS_AMARELO}MODO DEUS ATIVADO. ACESSO AOS BASTIDORES CONCEDIDO.{RESET}\n", 0.04, jogo=jogo)
            ui.animar(f"{DOS_BRANCO}Você entra no restaurante. Sua lanterna brilha com a força de uma estrela...{RESET}", 0.04, jogo=jogo)
            imprimir_contexto_sala(jogo)
        else:
            ui.animar(f"{DOS_VERMELHO}OPÇÃO INVÁLIDA. DIGITE UMA OPÇÃO DO MENU.{RESET}", 0.04, jogo=jogo)

    #estado jogo/combate
    elif jogo.estado_atual in ["JOGO", "COMBATE_ANIMATRONICO"]:
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            imprimir_contexto_sala(jogo)

        elif jogo.sala_atual == "sala de energia" and not getattr(jogo, 'fios_cortados_inventario', False):
            jogo.minigame_atual = MinigameMinotauro(jogo)
            jogo.estado_atual = "MINIGAME_MINOTAURO"
            jogo.minigame_atual.imprimir_status()

        elif (comando in ["cadeira", "sentar", "sentar na cadeira", "usar cadeira"] or jogo.sala_atual == "01") and not getattr(jogo, 'noite_vencida', False) and comando in ["cadeira", "sentar", "sentar na cadeira", "usar cadeira"]:
            jogo.sala_atual = "01"
            jogo.minigame_atual = MinigameSeguranca(jogo)
            jogo.estado_atual = "MINIGAME_SEGURANCA"
            jogo.minigame_atual.imprimir_status()

        elif comando == "abrir cofre" and jogo.sala_atual == "01":
            jogo.estado_atual = "MINIGAME_COFRE"
            ui.animar(f"{DOS_BRANCO}{ARTE_COFRE}{RESET}", 0.015, jogo=jogo)
            ui.exibir(f"{DOS_BRANCO}O cofre de ferro possui um teclado numérico antigo.{RESET}")
            ui.exibir(f"{DOS_VERDE}Digite a senha de 4 dígitos: {RESET}")

        elif (comando == "jogar jon" or comando == "jogar fome de jon") and jogo.sala_atual == "sala de fliperamas":
            jogo.estado_atual = "MINIGAME_JON"
            jogo.jon_passos_dados = 0
            jogo.jon_caminho_certo = [random.choice(["f", "e", "d"]) for _ in range(4)]
            ui.limpar()
            ui.animar(f"{DOS_BRANCO}{ARTE_PORCO}{RESET}", 0.015, jogo=jogo)
            ui.animar("--- A FOME DE JON ---", 0.03, DOS_VERDE, jogo)
            ui.exibir(f"{DOS_BRANCO}Guie o Porco Jon pelos dutos baseando-se nos seus sentidos.{RESET}")
            ui.exibir("Comandos: [F] Frente | [E] Esquerda | [D] Direita")
            dar_dica_jon(jogo.jon_caminho_certo[0], ui)
            ui.exibir(f"Passo 1/4 - Direção (F/E/D): ")

        elif comando == "jogar consertos" and jogo.sala_atual == "sala de fliperamas":
            if "moeda velha" not in jogo.inventario:
                ui.exibir("A máquina 'Consertos & Sorrisos' exige uma ficha (moeda velha) para iniciar.")
            else:
                jogo.inventario.remove("moeda velha")
                jogo.estado_atual = "MINIGAME_CONSERTOS_CABECA"
                jogo.web_consertos = {}
                ui.limpar()
                ui.animar(f"{DOS_BRANCO}{ARTE_ROBO}{RESET}", 0.015, jogo=jogo)
                ui.animar("--- CONSERTOS & SORRISOS ---", 0.03, DOS_VERDE, jogo)
                ui.exibir("Bem-vindo, Mecânico! Vamos montar nosso novo Festeiro!")
                ui.exibir(f"\n{DOS_AMARELO}[ FASE 1: SELEÇÃO DE PEÇAS ]{RESET}")
                ui.exibir("Escolha a Cabeça (1- Urso | 2- Coelho): ")

        elif (comando == "jogar adivinha" or comando == "jogar julgamento") and jogo.sala_atual == "sala de fliperamas":
            jogo.estado_atual = "MINIGAME_JULGAMENTO_Q1"
            jogo.web_julgamento = {"pontos": 0, "vitimas": ["angela", "joao", "renato"]}
            ui.limpar()
            ui.animar(f"{DOS_BRANCO}{ARTE_PIANO}{RESET}", 0.015, jogo=jogo)
            ui.animar("--- O JULGAMENTO DO PIANISTA ---", 0.03, DOS_VERDE, jogo)
            ui.exibir(f"{DOS_BRANCO}O animatrônico desperta. Ele detém todas as respostas.{RESET}")
            ui.exibir(f"\n{DOS_AMARELO}PERGUNTA 1: Em que ano a nossa música parou para sempre?{RESET}")

        else:
            gastou_turno = processar_comando(comando_bruto, jogo, jogo.mapa)
            if gastou_turno:
                atualizar_eventos_de_tempo(jogo)
                jogo.erros_consecutivos = 0
            
            
            if jogo.sala_atual not in jogo.mapa and jogo.sala_atual not in ["morte", "saida", "cama", "final_bom"]:
                jogo.sala_atual = "01"

            # gatilho do final verddeiro
            if jogo.estado_atual == "FIM" and getattr(jogo, 'incendio', False):
                try: 
                    from app import registrar_telemetria
                    registrar_telemetria("VITORIA", jogo.sala_atual, jogo.dificuldade_escolhida, "Final Verdadeiro")
                except (ImportError, AttributeError) as e:
                    pass
                rodar_final("verdadeiro", jogo)
                return
            
            if jogo.sala_atual == "morte":
                try: 
                    from app import registrar_telemetria
                    registrar_telemetria("MORTE", jogo.sala_atual, jogo.dificuldade_escolhida, "Morte no Mapa")
                except (ImportError, AttributeError) as e:
                    logger.warning(f"Falha ao registrar telemetria de morte: {e}")
                dar_tela_de_morte(jogo)


            elif jogo.sala_atual == "saida":
                try: 
                    from app import registrar_telemetria
                    registrar_telemetria("VITORIA", jogo.sala_atual, jogo.dificuldade_escolhida, "Final Covarde")
                except (ImportError, AttributeError) as e:
                    pass
                rodar_final("saida", jogo)

            elif jogo.sala_atual == "cama":
                try: 
                    from app import registrar_telemetria
                    registrar_telemetria("VITORIA", jogo.sala_atual, jogo.dificuldade_escolhida, "Final Dorminhoco")
                except (ImportError, AttributeError) as e:
                    pass
                rodar_final("cama", jogo)

            elif jogo.sala_atual == "hall de entrada" and getattr(jogo, 'noite_vencida', False):
                if getattr(jogo, 'incendio', False): 
                    try: 
                        from app import registrar_telemetria
                        registrar_telemetria("VITORIA", jogo.sala_atual, jogo.dificuldade_escolhida, "Final Verdadeiro")
                    except (ImportError, AttributeError) as e:
                        pass
                    rodar_final("verdadeiro", jogo)
                else: 
                    try: 
                        from app import registrar_telemetria
                        registrar_telemetria("VITORIA", jogo.sala_atual, jogo.dificuldade_escolhida, "Final Bom")
                    except (ImportError, AttributeError) as e:
                        pass
                    rodar_final("final_bom", jogo)
            elif jogo.estado_atual == "COMBATE_ANIMATRONICO":
                pass 
            else:
                if jogo.estado_atual == "JOGO":
                    imprimir_contexto_sala(jogo)

    #minigame do cofre
    elif jogo.estado_atual == "MINIGAME_COFRE":
        if comando in ["cls", "limpar", "clear", "clean"]:
            ui.limpar()
            ui.animar(f"{DOS_BRANCO}{ARTE_COFRE}{RESET}", 0.015, jogo=jogo)
            ui.exibir(f"{DOS_VERDE}Digite a senha de 4 dígitos: {RESET}")
        elif comando == "1994": 
            ui.exibir(f"{DOS_VERDE} Um som de 'click'. A pesada porta de metal se abre.{RESET}")
            sala = jogo.mapa[jogo.sala_atual]
            sala.setdefault("itens", [])
            
            if "chave dos fundos" not in jogo.inventario and "chave dos fundos" not in sala["itens"]:
                if len(jogo.inventario) < MAX_INVENTARIO or getattr(jogo, 'god_mode', False):
                    ui.exibir(f"{DOS_AMARELO}Você encontrou a 'chave dos fundos' suja de graxa lá dentro!{RESET}")
                    jogo.inventario.append("chave dos fundos")
                else:
                    ui.exibir(f"{DOS_AMARELO}Sua mochila está cheia! A 'chave dos fundos' caiu no chão da sala.{RESET}")
                    sala["itens"].append("chave dos fundos")
            else:
                ui.exibir("O cofre está vazio. Apenas poeira.")
            jogo.estado_atual = "JOGO"
            imprimir_contexto_sala(jogo)
        else:
            ui.exibir(f"{DOS_VERMELHO} ⛝ Senha incorreta. Painel pisca em vermelho.⛝{RESET}")
            jogo.estado_atual = "JOGO"
            imprimir_contexto_sala(jogo)

    # ==========================================
    # BLOCO: MINIGAME DO PORCO JON
    # ==========================================
    elif jogo.estado_atual == "MINIGAME_JON":
        passo = getattr(jogo, 'jon_passos_dados', 0)
        if comando in ["f", "e", "d", "frente", "esquerda", "direita"]:
            letra = comando[0]
            if letra == jogo.jon_caminho_certo[passo]:
                ui.exibir(f"{DOS_BRANCO}Jon rasteja em silêncio pelos dutos...{RESET}")
                jogo.jon_passos_dados += 1
                if jogo.jon_passos_dados == 4:
                    ui.exibir(f"\n{DOS_VERDE}Jon encontrou a 'comida'. A tela pinga um pixel vermelho.{RESET}")
                    ui.exibir(f"{DOS_VERMELHO}MENSAGEM: 'Eles ainda estão aqui.'{RESET}")
                    jogo.turnos_luz = max(0, jogo.turnos_luz - 1)
                    jogo.estado_atual = "JOGO"
                    imprimir_contexto_sala(jogo)
                else:
                    dar_dica_jon(jogo.jon_caminho_certo[jogo.jon_passos_dados], ui)
                    ui.exibir(f"Passo {jogo.jon_passos_dados + 1}/4 - Direção (F/E/D): ")
            else:
                ui.exibir(f"\n{DOS_VERMELHO}Jon caiu num triturador ativo! leva um choque brutal!{RESET}")
                jogo.hp -= 1
                jogo.turnos_luz = max(0, jogo.turnos_luz - 1)
                if jogo.hp <= 0: 
                    try: 
                        from app import registrar_telemetria
                        registrar_telemetria("MORTE", "MINIGAME_JON", jogo.dificuldade_escolhida, "Morto pelo Porco")
                    except (ImportError, AttributeError) as e:
                        pass
                    dar_tela_de_morte(jogo)
                else:
                    jogo.estado_atual = "JOGO"
                    imprimir_contexto_sala(jogo)
        else:
            ui.exibir("Direção inválida. Use F, E ou D.")

    
    elif jogo.estado_atual == "MINIGAME_CONSERTOS_CABECA":
        jogo.web_consertos["cabeca"] = comando
        jogo.estado_atual = "MINIGAME_CONSERTOS_TRONCO"
        ui.exibir("Escolha o Tronco (1- Fino | 2- Robusto): ")

    elif jogo.estado_atual == "MINIGAME_CONSERTOS_TRONCO":
        jogo.web_consertos["tronco"] = comando
        jogo.estado_atual = "MINIGAME_CONSERTOS_PERNAS"
        ui.exibir("Escolha as Pernas (1- Aço | 2- Pelúcia): ")

    elif jogo.estado_atual == "MINIGAME_CONSERTOS_PERNAS":
        cabeca = jogo.web_consertos.get("cabeca", "1")
        pernas = comando
        item_secreto = "remedio" if (cabeca == "2" and pernas == "2") else None
        
        ui.exibir(f"\n{DOS_VERDE}CONSERTO CONCLUÍDO. O ANIMATRÔNICO SORRI PARA VOCÊ!{RESET}")
        sala = jogo.mapa[jogo.sala_atual]
        sala.setdefault("itens", [])
        
        if "chave da cozinha" not in jogo.inventario and "chave da cozinha" not in sala["itens"]:
            if len(jogo.inventario) < MAX_INVENTARIO or getattr(jogo, 'god_mode', False):
                jogo.inventario.append("chave da cozinha")
                ui.exibir(f"{DOS_VERDE}⛋ Você obteve: CHAVE DA COZINHA!{RESET}")
            else:
                sala["itens"].append("chave da cozinha")

        if item_secreto:
            if len(jogo.inventario) < MAX_INVENTARIO or getattr(jogo, 'god_mode', False):
                jogo.inventario.append(item_secreto)
                ui.exibir(f"{DOS_VERDE}⛋ Você obteve um item extra: {item_secreto.upper()}!{RESET}")
            else:
                sala["itens"].append(item_secreto)
            
        jogo.turnos_luz = max(0, jogo.turnos_luz - 1)
        jogo.estado_atual = "JOGO"
        imprimir_contexto_sala(jogo)

    
    elif jogo.estado_atual == "MINIGAME_JULGAMENTO_Q1":
        if comando == "1994": 
            jogo.web_julgamento["pontos"] += 1
            falar_pianista(True, ui, jogo)
        else: 
            falar_pianista(False, ui, jogo)
        jogo.estado_atual = "MINIGAME_JULGAMENTO_Q2"
        ui.exibir(f"\n{DOS_AMARELO}PERGUNTA 2: Qual animatrônico está atrás de você agora?{RESET}")

    elif jogo.estado_atual == "MINIGAME_JULGAMENTO_Q2":
        if "caroline" in comando or "ela" in comando: 
            jogo.web_julgamento["pontos"] += 1
            falar_pianista(True, ui, jogo)
        else: 
            falar_pianista(False, ui, jogo)
        jogo.estado_atual = "MINIGAME_JULGAMENTO_Q3"
        ui.exibir(f"\n{DOS_AMARELO}PERGUNTA 3: Em que ano tudo isso começou?{RESET}")

    elif jogo.estado_atual == "MINIGAME_JULGAMENTO_Q3":
        if comando == "1982": 
            jogo.web_julgamento["pontos"] += 1
            falar_pianista(True, ui, jogo)
        else: 
            falar_pianista(False, ui, jogo)
        jogo.estado_atual = "MINIGAME_JULGAMENTO_Q4"
        ui.exibir(f"\n{DOS_AMARELO}PERGUNTA 4: Quem é você?{RESET}")

    elif jogo.estado_atual == "MINIGAME_JULGAMENTO_Q4":
        if "rogerio" in comando: 
            jogo.web_julgamento["pontos"] += 1
            falar_pianista(True, ui, jogo)
        else: 
            falar_pianista(False, ui, jogo)
        jogo.estado_atual = "MINIGAME_JULGAMENTO_V1"
        ui.exibir(f"\n{DOS_AMARELO}PERGUNTA 5: Quem são as três vítimas? Digite o 1º nome:{RESET}")

    elif jogo.estado_atual in ["MINIGAME_JULGAMENTO_V1", "MINIGAME_JULGAMENTO_V2", "MINIGAME_JULGAMENTO_V3"]:
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
            ui.exibir("Digite o 2º nome: ")
        elif jogo.estado_atual == "MINIGAME_JULGAMENTO_V2":
            jogo.estado_atual = "MINIGAME_JULGAMENTO_V3"
            ui.exibir("Digite o 3º nome: ")
        else:
            if len(jogo.web_julgamento["vitimas"]) == 0: 
                jogo.web_julgamento["pontos"] += 1
            if jogo.web_julgamento["pontos"] == 5:
                ui.animar("Obrigado por voltar pela gente, Rogério...", 0.08, DOS_VERDE, jogo)
                sala = jogo.mapa[jogo.sala_atual]
                sala.setdefault("itens", [])
                if "bateria nova" not in jogo.inventario and "bateria nova" not in sala["itens"]:
                    if len(jogo.inventario) < MAX_INVENTARIO or getattr(jogo, 'god_mode', False):
                        jogo.inventario.append("bateria nova")
                        ui.exibir(f"{DOS_VERDE}⛋ Você a guardou na mochila.{RESET}")
                    else:
                        ui.exibir(f"{DOS_AMARELO}⛋ Mochila cheia! A bateria nova caiu no chão.{RESET}")
                        sala["itens"].append("bateria nova")
            else:
                ui.animar("Quem é você? *A tela desliga* Você perdeu a absolvição.", 0.05, DOS_VERMELHO, jogo)
                ui.exibir("@@JUMPSCARE@@")
                
            jogo.turnos_luz = max(0, jogo.turnos_luz - 1)
            jogo.estado_atual = "JOGO"
            imprimir_contexto_sala(jogo)


    #bloco universal

    elif jogo.estado_atual.startswith("MINIGAME_") and hasattr(jogo, 'minigame_atual') and jogo.minigame_atual is not None:
        
        
        if isinstance(jogo.minigame_atual, dict):
            dados_salvos = jogo.minigame_atual
            if jogo.estado_atual == "MINIGAME_SEGURANCA":
                from villas_boas.engine.minigames.seguranca import MinigameSeguranca
                jogo.minigame_atual = MinigameSeguranca(jogo)
            elif jogo.estado_atual == "MINIGAME_MINOTAURO":
                from villas_boas.engine.minigames.minotauro import MinigameMinotauro
                jogo.minigame_atual = MinigameMinotauro(jogo)
                
            jogo.minigame_atual.__dict__.update(dados_salvos) 
            jogo.minigame_atual.jogo = jogo

        if hasattr(jogo.minigame_atual, 'ui'):
            jogo.minigame_atual.ui = jogo.ui_handler
            
        
        partes = extrair_argumentos(comando)
        verbo = partes[0] if partes else ""
        mapa_direcoes = {"f": "ir frente", "t": "ir atrás", "e": "ir esquerda", "d": "ir direita"}
        if verbo in mapa_direcoes: 
            comando = mapa_direcoes[verbo]

       
        resultado = jogo.minigame_atual.processar_turno(comando, jogo)
        
        
        if resultado == "continuar":
            jogo.minigame_atual.imprimir_status()

        elif resultado == "morte":
            jogo.estado_atual = "FIM"
            jogo.sala_atual = "morte"
            jogo.minigame_atual = None
            try: 
                from app import registrar_telemetria
                registrar_telemetria("MORTE", jogo.estado_atual, jogo.dificuldade_escolhida, "Falhou em um Minigame")
            except Exception:
                pass
            dar_tela_de_morte(jogo)

        elif resultado.startswith("vitoria_"):
            jogo.estado_atual = "JOGO"
            jogo.minigame_atual = None
            
            
            if resultado == "vitoria_seguranca":
                if jogo.sala_atual not in jogo.mapa:
                    jogo.sala_atual = "01"
                    
            elif resultado == "vitoria_minotauro":
                jogo.sala_atual = "sala dos fundos"
                if getattr(jogo, 'god_mode', False) and not getattr(jogo, 'fios_cortados_inventario', False):
                    jogo.inventario.append("fios cortados")
                    jogo.fios_cortados_inventario = True
                    ui.exibir(f"{DOS_AMARELO}[GOD MODE] Você arrancou os 'fios cortados' da parede!{RESET}")
                    
                if "sala dos fundos" in jogo.mapa:
                    jogo.mapa["sala dos fundos"]["energia"] = "A pesada porta da sala de energia está totalmente destruída."
                ui.exibir(f"{DOS_VERDE}A porta cedeu atrás de você. Você sobreviveu.{RESET}")

            imprimir_contexto_sala(jogo)

        return