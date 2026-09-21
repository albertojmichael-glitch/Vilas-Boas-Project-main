import logging

from villas_boas.utils import encontrar_melhor_match
from ui import DOS_VERDE, DOS_BRANCO, DOS_AMARELO, DOS_VERMELHO, RESET
from data import MAX_INVENTARIO, TURNOS_BATERIA

logger = logging.getLogger(__name__)


def cmd_pegar(comando, jogo, mapa):
    ui = jogo.ui_handler
    item = comando.replace("pegar ", "").strip()
    sala = mapa.get(jogo.sala_atual, {})
    itens_chao = sala.get("itens", [])
    
    match_item = encontrar_melhor_match(item, itens_chao)
    if not match_item:
        ui.exibir(f"Não há nenhum '{item}' aqui para pegar.")
        return False
        
    item = match_item

    
    if item == "bolsa":
        jogo.bolsas_coletadas = getattr(jogo, "bolsas_coletadas", 0) + 1
        itens_chao.remove("bolsa")
        ui.exibir(f"{DOS_VERDE}Você equipou a bolsa! Seu limite de inventário aumentou permanentemente (+3 slots).{RESET}")
        return True

    
    qtd_bolsas = getattr(jogo, "bolsas_coletadas", 0)
    limite_atual = MAX_INVENTARIO + (qtd_bolsas * 3)

    
    if len(jogo.inventario) >= limite_atual and not getattr(jogo, "god_mode", False):
        ui.exibir(f"{DOS_VERMELHO}Sua mochila está cheia! Você precisa largar algo antes.{RESET}")
        return False

    
    jogo.inventario.append(item)
    itens_chao.remove(item)
    ui.exibir(f"{DOS_VERDE}Você pegou: {item}{RESET}")

    qtd_bolsas = getattr(jogo, "bolsas_coletadas", 0)
    limite_atual = 3 + (qtd_bolsas * 3)
    if limite_atual == 9 and len(jogo.inventario) == 9:
        from villas_boas.engine.core import desbloquear_conquista
        desbloquear_conquista(jogo, "acumulador")
        
    return True


def cmd_largar(comando, jogo, mapa):
    ui = jogo.ui_handler
    item = comando.replace("largar ", "").strip()
    match_item = encontrar_melhor_match(item, jogo.inventario)
    if not match_item:
        ui.exibir(f"Você não tem '{item}' no inventário.")
        return False
    item = match_item

    jogo.inventario.remove(item)
    sala = mapa.get(jogo.sala_atual, {})
    if "itens" not in sala:
        sala["itens"] = []
    sala["itens"].append(item)
    ui.exibir(f"{DOS_AMARELO}Você largou: {item} no chão.{RESET}")
    return True




def _usar_lanterna(jogo, mapa, item):
    ui = jogo.ui_handler
    ui.exibir(
        "Você já está usando a lanterna automaticamente (quando tem bateria)."
    )
    return True


def _usar_isqueiro(jogo, mapa, item):
    ui = jogo.ui_handler
    
    
    try:
        from data import ARTE_ISQUEIRO
        ui.animar(f"{DOS_AMARELO}{ARTE_ISQUEIRO}{RESET}", 0.015, jogo=jogo)
    except ImportError: 
        pass
        
    
    if getattr(jogo, "turnos_luz", 0) > 2:
        ui.exibir(f"{DOS_AMARELO}Você acende o isqueiro, mas a sua lanterna já ilumina o local muito bem.{RESET}")
        return False 
        
    
    ui.exibir(f"{DOS_AMARELO}Você risca a pedra do isqueiro. Uma chama tremeluzente ilumina as sombras.{RESET}")
    ui.exibir(f"{DOS_VERDE}A luz fraca permite enxergar o chão, mas o gás parece que vai durar pouco (2 turnos).{RESET}")
    
    
    jogo.turnos_luz += 2 
    
    
    jogo.nivel_barulho = min(100, getattr(jogo, "nivel_barulho", 0) + 10)
    
    
    return True


def _usar_chave_dos_fundos(jogo, mapa, item):
    ui = jogo.ui_handler
    if jogo.sala_atual == "porta dos fundos":
        ui.exibir(
            f"{DOS_VERDE}Você insere a chave suja de graxa na fechadura e força. Ela gira com um estalo alto.{RESET}"
        )
        ui.exibir(
            f"{DOS_AMARELO}A pesada porta de metal se escancara, revelando um corredor denso e escuro.{RESET}"
        )
        mapa["porta dos fundos"]["descrição"] = (
            "A pesada porta de metal está aberta, levando para a área de serviço."
        )
        mapa["porta dos fundos"]["frente"] = "sala dos fundos"
        jogo.inventario.remove("chave dos fundos")
    else:
        ui.exibir("Não há nenhuma fechadura por aqui que se encaixe nessa chave.")
    return True


def _usar_bateria_nova(jogo, mapa, item):
    ui = jogo.ui_handler
    ui.exibir(
        f"{DOS_VERDE}Você abre a parte inferior da lanterna e insere a bateria nova.{RESET}"
    )
    ui.exibir(f"{DOS_AMARELO}A luz da lanterna fica forte e ofuscante!{RESET}")
    jogo.turnos_luz = TURNOS_BATERIA
    jogo.inventario.remove("bateria nova")
    return True


def _usar_disquete(jogo, mapa, item):
    ui = jogo.ui_handler
    if jogo.sala_atual == "01":
        ui.exibir(
            f"{DOS_VERDE}Você insere o {item} sujo no drive do terminal de segurança...{RESET}"
        )
        ui.pausar(1.5)
        ui.exibir(f"{DOS_BRANCO}LENDO A:\\ ...{RESET}")
        ui.pausar(1)
        
        try:
            from data import ARTE_DISQUETE
            ui.animar(f"{DOS_BRANCO}{ARTE_DISQUETE}{RESET}", 0.015, jogo=jogo)
            ui.pausar(1)
        except ImportError as e:
            import logging
            logger = logging.getLogger(__name__)
            logger.debug(f"ARTE_DISQUETE indisponível no arquivo de dados: {e}")
            pass

        # --- LORE: DISQUETE 1 ---
        if item == "disquete1":
            ui.animar(f"{DOS_AMARELO}ARQUIVO RECUPERADO: ANGELA.TXT{RESET}", 0.05, DOS_AMARELO, jogo)
            ui.animar(f"{DOS_BRANCO}'Hoje vim mostrar para meu esposo João, meu local de trabalho, o Vilas Boas. Talvez não tenha sido uma boa ideia.'{RESET}", 0.06, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'A gente brigou feio no meio do salão, pois aparentemente ele achava que tinha alguém me observando atrás das cortinas, sendo que não... Não tinha nada lá além de poeira e peças enferrujadas. Ele está perdendo a cabeça.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Ele foi falar com meu chefe, o Sr. Renato, lá na salas dos fundos, enquanto eu escrevo isso.'{RESET}", 0.08, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_VERMELHO}'Talvez... Seja loucura minha, mas eu vi alguem me chamando para a cozinha privada pela janela do escritório, vou ir lá ver.'{RESET}", 0.05, DOS_VERMELHO, jogo)
            ui.animar(f"{DOS_VERMELHO}'Ela foi libertada.'{RESET}", 0.10, DOS_VERMELHO, jogo)
        
        # --- LORE: DISQUETE 2 ---
        elif item == "disquete2":
            ui.animar(f"{DOS_AMARELO}ARQUIVO RECUPERADO: MICHEL.TXT{RESET}", 0.05, DOS_AMARELO, jogo)
            ui.animar(f"{DOS_BRANCO}'Depois que a Caroline partiu, não encontramos ninguém para alavancar esse projeto.'{RESET}", 0.06, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Acreditamos que não há salvação mais, vamos abandonar tudo e deixar as traças. Tudo o que a gente já devia ter feito.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Perdemos tudo, e agora tem coisas no restaurante, que mesmo a gente causando isso... Nós nos arrependemos, e vimos que tudo isso não passa de uma vergonha.'{RESET}", 0.06, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Sinto culpa por tudo que fiz. Não sei como me defender, sei que sou culpado, e se isso tudo vier à tona, irei pegar prisão perpétua sem dúvidas.'{RESET}", 0.06, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Sou um covarde que levou todas essas pessoas à morte. Não vou mais deixar que isso me consuma, vou ir embora enquanto há tempo.'{RESET}", 0.06, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_VERMELHO}Ele fugiu.{RESET}", 0.10, DOS_VERMELHO, jogo)
        
        # --- LORE: DISQUETE 3 ---
        elif item == "disquete3":
            ui.animar(f"{DOS_AMARELO}ARQUIVO RECUPERADO: RENATO.TXT{RESET}", 0.05, DOS_AMARELO, jogo)
            ui.animar(f"{DOS_BRANCO}'1994, 3 de setembro. O restaurante está indo bem, mantendo uma clientela fiel. Escrevo isso como relatório.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Sou Renato Fidelis Gomes, fundador deste local. Eu mesmo fiz esses animatrônicos, juntei placas e peças. Minhas obras-primas.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Derramei muito sangue e suor. Tenho fantasias antigas no estoque: uma de lebre rosa sem nome, a do Senhor Raposa, e a de jacaré, que guardo bem.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Sou um inventor fadado a cuidar da administração sozinho. Tenho orgulho da minha primeira criação, um animatrônico de touro com 3 rostos.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Deixo ele na sala de energia. Fiz um mini programa de sonar que lança ondas nas paredes. Se algo estiver no radar, ele vai atrás.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'O porco Jon mapeia o local inteiro e procura as entradas. Se o caminho mais curto for pela tubulação, ele vai pela tubulação.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'O mosqueteiro Rick é mais direto, mas precisa de manutenção. O índio Jones segue vozes e induz pensamentos nos clientes. Não sei qual o limite.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_VERMELHO}'E o cozinheiro Alberto tem acesso ao sistema inteiro do restaurante, acho isso preocupante.'{RESET}", 0.06, DOS_VERMELHO, jogo)
            ui.animar(f"{DOS_BRANCO}'A ideia surgiu quando visitei Hurricane, em Utah. Descobri a Chuck E. Cheese e a Fazbear Pizza. Fiquei encantado com as atrações.'{RESET}", 0.05, DOS_BRANCO, jogo)
            ui.animar(f"{DOS_BRANCO}'Penso que se esse lugar crescer, farei parcerias com eles para trazer ao Brasil. Escrevi muito, hora de dar Adeus. (FIM)'{RESET}", 0.06, DOS_BRANCO, jogo)
        
        ui.pausar(2)
        ui.exibir(
            f"{DOS_VERMELHO}O drive faz um ruído horrível e ejeta o {item} arranhado. Ele está arruinado.{RESET}"
        )
        if item in jogo.inventario:
            jogo.inventario.remove(item)
        ui.pausar(2)
    else:
        ui.exibir(
            f"{DOS_BRANCO}Você segura o velho {item}, mas não há nenhum computador neste cômodo para lê-lo. Talvez na sala de segurança?{RESET}"
        )
    return True


def _usar_tabua_pequena_de_madeira(jogo, mapa, item):
    ui = jogo.ui_handler
    if jogo.sala_atual == "03":
        ui.exibir(
            f"{DOS_AMARELO}Você usa a tábua como alavanca e força a porta emperrada...{RESET}"
        )
        ui.exibir(
            f"{DOS_VERDE}CRASH! A porta cede e abre. A tábua quebra no processo.{RESET}"
        )
        jogo.inventario.remove(item)
        mapa["corredor"]["03"] = "sala do gerador"
        jogo.sala_atual = "sala do gerador"
        ui.pausar(2)
        return True
    else:
        ui.exibir("Não há onde usar a tábua aqui.")
    return True

def _usar_cura(jogo, mapa, item):
    ui = jogo.ui_handler
    
    
    hp_maximo = 2 if jogo.dificuldade_escolhida == "PESADELO" else 3
    if getattr(jogo, "god_mode", False):
        hp_maximo = 9999

    
    if jogo.hp >= hp_maximo:
        ui.exibir(f"Você já está se sentindo bem. Melhor guardar isso para depois.")
        return False

    
    jogo.hp += 1
    jogo.inventario.remove(item)

    
    if item == "remedio":
        ui.exibir(f"{DOS_VERDE}Você engole o relaxante muscular vencido. Seus músculos destravam e a dor diminui (+1 HP).{RESET}")
    elif item == "doce":
        ui.exibir(f"{DOS_VERDE}Você mastiga o doce velho. O pico de açúcar te dá um pouco mais de resistência (+1 HP).{RESET}")
    
    return True



_USAR_HANDLERS = {
    "lanterna": _usar_lanterna,
    "chave dos fundos": _usar_chave_dos_fundos,
    "bateria nova": _usar_bateria_nova,
    "isqueiro": _usar_isqueiro, 
    "disquete1": _usar_disquete,
    "disquete2": _usar_disquete,
    "disquete3": _usar_disquete,
    "tábua pequena de madeira": _usar_tabua_pequena_de_madeira,
    "tabua pequena de madeira": _usar_tabua_pequena_de_madeira,
    "remedio": _usar_cura,  
    "doce": _usar_cura      
}


def cmd_usar(comando, jogo, mapa):
    ui = jogo.ui_handler
    item = comando.replace("usar ", "").strip()
    match_item = encontrar_melhor_match(item, jogo.inventario)
    if not match_item:
        ui.exibir(f"Você não tem '{item}' no inventário.")
        return False
    item = match_item

    handler = _USAR_HANDLERS.get(item)
    if handler is None:
        ui.exibir(f"Você não sabe como usar '{item}' aqui.")
        return True

    return handler(jogo, mapa, item)




def _combinar_tesoura_fita(jogo, mapa, ui):
    ui.exibir(f"{DOS_VERDE}Você enrola a fita isolante na tesoura quebrada. Ela está consertada.{RESET}")
    jogo.inventario.remove("tesoura quebrada")
    jogo.inventario.remove("fita isolante") 
    jogo.inventario.append("tesoura")
    return True


_COMBINAR_HANDLERS = {
    frozenset(["tesoura quebrada", "fita isolante"]): _combinar_tesoura_fita,
    
}

def cmd_combinar(comando, jogo, mapa):
    ui = jogo.ui_handler
    partes = comando.replace("combinar ", "").replace("juntar ", "").split(" com ")
    
    if len(partes) != 2:
        ui.exibir("Use o formato: combinar [item1] com [item2]")
        return False

    item1 = encontrar_melhor_match(partes[0].strip(), jogo.inventario)
    item2 = encontrar_melhor_match(partes[1].strip(), jogo.inventario)
    
    if not item1 or not item2:
        ui.exibir("Você precisa ter os dois itens no inventário.")
        return False

    chave_combinacao = frozenset([item1, item2])
    handler = _COMBINAR_HANDLERS.get(chave_combinacao)

    if handler is None:
        ui.exibir("Esses itens não parecem combinar.")
        return False

    return handler(jogo, mapa, ui)


def cmd_inventario(jogo):
    ui = jogo.ui_handler
    if not jogo.inventario:
        ui.exibir("Sua mochila está vazia.")
    else:
        ui.exibir(f"{DOS_BRANCO}INVENTÁRIO:{RESET}")
        for item in jogo.inventario:
            ui.exibir(f" - {item}")
    return True