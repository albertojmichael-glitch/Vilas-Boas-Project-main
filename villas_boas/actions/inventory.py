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

    qtd_bolsas = jogo.inventario.count("bolsa")

    limite_atual = MAX_INVENTARIO + (qtd_bolsas * 3)

    if len(jogo.inventario) >= limite_atual and not getattr(jogo, "god_mode", False):
        ui.exibir(
            f"{DOS_VERMELHO}Sua mochila está cheia! Você precisa largar algo antes.{RESET}"
        )
        return False

    jogo.inventario.append(item)

    itens_chao.remove(item)

    ui.exibir(f"{DOS_VERDE}Você pegou: {item}{RESET}")

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

def cmd_usar(comando, jogo, mapa):
    ui = jogo.ui_handler
    item = comando.replace("usar ", "").strip()
    match_item = encontrar_melhor_match(item, jogo.inventario)
    if not match_item:
        ui.exibir(f"Você não tem '{item}' no inventário.")
        return False
    item = match_item
    if item == "lanterna":
        ui.exibir(
            "Você já está usando a lanterna automaticamente (quando tem bateria)."
        )
    elif item == "chave dos fundos":
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

    elif item == "bateria nova":
        ui.exibir(
            f"{DOS_VERDE}Você abre a parte inferior da lanterna e insere a bateria nova.{RESET}"
        )
        ui.exibir(f"{DOS_AMARELO}A luz da lanterna fica forte e ofuscante!{RESET}")
        jogo.turnos_luz = TURNOS_BATERIA
        jogo.inventario.remove("bateria nova")
        return True

    elif item == "isqueiro":
        if getattr(jogo, "noite_vencida", False):
            if getattr(jogo, "fios_cortados_inventario", False):
                ui.exibir(
                    f"{DOS_VERMELHO}Você aproxima a chama do isqueiro das cortinas e da madeira podre. Em segundos, o fogo se espalha.{RESET}"
                )
                ui.exibir(
                    f"{DOS_AMARELO}O restaruante está em chamas, você sabe o que deve fazer. Vá para o Hall de entrada.{RESET}"
                )
                jogo.incendio = True
            else:
                ui.exibir(
                    f"{DOS_BRANCO}Você pensa em incendiar o lugar agora mesmo, mas precisa de algo a mais para... ela...{RESET}"
                )
        else:
            ui.exibir(
                f"{DOS_AMARELO}Você acende o isqueiro. Uma pequena chama ilumina as sombras, mas você logo a apaga para não chamar atenção.{RESET}"
            )
        return True
    elif item == "disquete":
        if jogo.sala_atual == "01":
            ui.exibir(
                f"{DOS_VERDE}Você insere o disquete sujo no drive do terminal de segurança...{RESET}"
            )
            ui.pausar(1.5)
            ui.exibir(f"{DOS_BRANCO}LENDO A:\\ ...{RESET}")
            ui.pausar(1)
            try:
                from data import ARTE_DISQUETE
                ui.animar(f"{DOS_BRANCO}{ARTE_DISQUETE}{RESET}", 0.015, jogo=jogo)
                ui.pausar(1)
            except ImportError as e:
                logger.debug(f"ARTE_DISQUETE indisponível no arquivo de dados: {e}")
                pass
            ui.animar(
                f"{DOS_AMARELO}ARQUIVO RECUPERADO: ANGELA.TXT{RESET}",
                0.05,
                DOS_AMARELO,
                jogo,
            )
            ui.animar(
                f"{DOS_BRANCO}'Hoje vim mostrar para meu esposo João, meu local de trabalho, o Vilas Boas. Talvez não tenha sido uma boa ideia.'{RESET}",
                0.06,
                DOS_BRANCO,
                jogo,
            )
            ui.animar(
                f"{DOS_BRANCO}'A gente brigou feio no meio do salão, pois aparentemente ele achava que tinha alguém me observando atrás das cortinas, sendo que não... Não tinha nada lá além de poeira e peças enferrujadas. Ele está perdendo a cabeça.'{RESET}",
                0.05,
                DOS_BRANCO,
                jogo=jogo,
            )
            ui.animar(
                f"{DOS_BRANCO}'Ele foi falar com meu chefe, o Sr. Renato, lá na salas dos fundos, enquanto eu escrevo isso.'{RESET}",
                0.08,
                DOS_BRANCO,
                jogo,
            )
            ui.animar(
                f"{DOS_VERMELHO}'Talvez... Seja loucura minha, mas eu vi alguem me chamando para a cozinha privada pela janela do escritório, vou ir lá ver.'{RESET}",
                0.05,
                DOS_VERMELHO,
                jogo,
            )
            ui.animar(
                f"{DOS_VERMELHO}'Ela foi libertada.'{RESET}", 0.10, DOS_VERMELHO, jogo
            )
            ui.pausar(2)

            ui.exibir(
                f"{DOS_VERMELHO}O drive faz um ruído horrível e ejeta o disquete arranhado. Ele está arruinado.{RESET}"
            )
            jogo.inventario.remove("disquete")
            ui.pausar(2)
        else:
            ui.exibir(
                f"{DOS_BRANCO}Você segura o velho disquete, mas não há nenhum computador neste cômodo para lê-lo. Talvez na sala de segurança?{RESET}"
            )

    elif item == "tábua pequena de madeira" or item == "tabua pequena de madeira":
        if jogo.sala_atual == "03":
            ui.exibir(
                f"{DOS_AMARELO}Você usa a tábua como alavanca e força a porta emperrada...{RESET}"
            )
            ui.exibir(
                f"{DOS_VERDE}CRASH! A porta cede e abre! A tábua quebra no processo.{RESET}"
            )
            jogo.inventario.remove(item)
            mapa["corredor"]["03"] = "sala do gerador"
            jogo.sala_atual = "sala do gerador"
            ui.pausar(2)
            return True
        else:
            ui.exibir("Não há onde usar a tábua aqui.")
    else:
        ui.exibir(f"Você não sabe como usar '{item}' aqui.")
    return True


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

    if (item1 == "tesoura quebrada" and item2 == "fita isolante") or (
        item2 == "tesoura quebrada" and item1 == "fita isolante"
    ):
        ui.exibir(
            f"{DOS_VERDE}Você enrola a fita isolante na tesoura quebrada. Ela está consertada!{RESET}"
        )
        jogo.inventario.remove("tesoura quebrada")
        jogo.inventario.append("tesoura")
        return True

    ui.exibir("Esses itens não parecem combinar.")
    return False


def cmd_inventario(jogo):
    ui = jogo.ui_handler
    if not jogo.inventario:
        ui.exibir("Sua mochila está vazia.")
    else:
        ui.exibir(f"{DOS_BRANCO}INVENTÁRIO:{RESET}")
        for item in jogo.inventario:
            ui.exibir(f" - {item}")
    return True


