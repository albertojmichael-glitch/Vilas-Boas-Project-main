from villas_boas.utils import encontrar_melhor_match, normalizar
from ui import DOS_VERDE, DOS_BRANCO, DOS_AMARELO, DOS_VERMELHO, RESET
from data import descricoes_itens


from villas_boas.actions.inspection import cmd_examinar
from villas_boas.actions.movement import cmd_ir
from villas_boas.actions.inventory import cmd_pegar, cmd_largar, cmd_usar, cmd_combinar, cmd_inventario

def processar_comando(comando, jogo, mapa):
    ui = jogo.ui_handler
    comando = comando.strip()
    if not comando: return False

    if getattr(jogo, 'estado_atual', "") == "COMBATE_ANIMATRONICO":
        if comando.lower() in ["atacar", "bater", "chutar", "lutar"] and getattr(jogo, 'god_mode', False):
            ui.exibir(f"{DOS_AMARELO}[GOD MODE] Você solta um soco devastador direto na mandíbula de metal do animatrônico!{RESET}")
            ui.exibir(f"{DOS_AMARELO}Ele solta o seu pescoço, emite um bipe de erro e foge correndo de volta pras sombras.{RESET}")
            ui.pausar(2)
            jogo.estado_atual = "JOGO"
            jogo.posicao_perseguidor = "longe"
            return True
        else:
            ui.exibir(f"{DOS_VERMELHO}Sua reação foi inútil... Ele esmaga o seu pescoço em um estalo seco.{RESET}")
            ui.pausar(2)
            jogo.sala_atual = "morte"
            ui.exibir("@@JUMPSCARE@@")
    
            jogo.estado_atual = "FIM"
            return True

    if comando.lower() == "dir" and getattr(jogo, 'estado_atual', "") == "AGUARDANDO_DIR":
        jogo.estado_atual = "JOGO"
        return "olhar"

    mapa_direcoes = {
        "f": "ir frente", "frente": "ir frente", "n": "ir frente", "norte": "ir frente",
        "t": "ir atrás", "tras": "ir atrás", "atras": "ir atrás", "atrás": "ir atrás", "s": "ir atrás", "sul": "ir atrás",
        "e": "ir esquerda", "esquerda": "ir esquerda", "w": "ir esquerda", "oeste": "ir esquerda",
        "d": "ir direita", "direita": "ir direita", "leste": "ir direita"
    }
    if comando.lower() in mapa_direcoes:
        comando = mapa_direcoes[comando.lower()]

    if comando in ["cadeira", "sentar", "sentar na cadeira", "usar cadeira"]:
    
        if jogo.sala_atual == "01":  
            if not getattr(jogo, 'noite_vencida', False):
            
                from minigames import MinigameSeguranca

                jogo.estado_atual = "MINIGAME_SEGURANCA"

                jogo.minigame_atual = MinigameSeguranca(jogo)

                jogo.minigame_atual.imprimir_status()
                return True


            else:
                jogo.ui_handler.exibir(f"{DOS_AMARELO}A mesa de controle está desligada. A noite já terminou.{RESET}")
                return True

        else:
            jogo.ui_handler.exibir(f"{DOS_BRANCO}Não há nenhuma cadeira de segurança aqui.{RESET}")
            return True

    if jogo.sala_atual in mapa:
        sala = mapa[jogo.sala_atual]
        
        saidas_validas = [
            str(k).lower()
            for k in sala
            if k not in ["descrição", "itens", "inspecionaveis", "cofre_important"]
        ]
        if normalizar(comando) in saidas_validas:
            comando = f"ir {normalizar(comando)}"

        inspecionaveis_sala = [
            normalizar(k) for k in sala.get("inspecionaveis", {})
        ]
        if normalizar(comando) in inspecionaveis_sala:
            comando = f"examinar {normalizar(comando)}"

    if comando.startswith("tp ") and getattr(jogo, 'god_mode', False):
        destino = comando.replace("tp ", "").strip()
        jogo.sala_atual = destino
        ui.exibir(f"{DOS_AMARELO}[GOD MODE] Teleportado para: {destino}{RESET}")
        return True

    if jogo.sala_atual == "sala de energia":
            
            from minigames import MinigameMinotauro
            jogo.estado_atual = "MINIGAME_MINOTAURO"
            jogo.minigame_atual = MinigameMinotauro(jogo)
            
            try:
                jogo.minigame_atual.imprimir_status()
            except AttributeError:
                pass
            return True
        
    elif comando.startswith("gerar ") and getattr(jogo, 'god_mode', False):
        item_desejado = comando.replace("gerar ", "").strip()
        match_item = encontrar_melhor_match(item_desejado, list(descricoes_itens.keys()))
        
        if match_item:
            jogo.inventario.append(match_item)
            ui.exibir(f"{DOS_AMARELO}[GOD MODE] O item '{match_item}' materializou-se na sua mochila.{RESET}")

            if match_item == "fios cortados":
                jogo.fios_cortados_inventario = True


        else:
            ui.exibir(f"{DOS_VERMELHO}[GOD MODE ERRO] Matéria não catalogada. O sistema não sabe como fabricar '{item_desejado}'.{RESET}")
        return True

    partes = comando.split(maxsplit=1)
    verbo = partes[0].lower()
    argumento = partes[1].lower() if len(partes) > 1 else ""

    aliases_verbos = {
        "p": "pegar", "l": "largar", "u": "usar", "c": "combinar", 
        "j": "jogar", "x": "examinar", "ex": "examinar", "o": "examinar", 
        "olhar": "examinar", "ver": "examinar", "investigar": "examinar",
        "i": "inventario", "inv": "inventario"
    }
    if verbo in aliases_verbos:
        verbo = aliases_verbos[verbo]

    if verbo == "ir":
        if not argumento: ui.exibir("Ir para onde?"); return False
        return cmd_ir(argumento, jogo, mapa)
        
    elif verbo == "pegar":
        if not argumento: ui.exibir("Pegar o quê?"); return False
        return cmd_pegar(argumento, jogo, mapa)
        
    elif verbo == "largar":
        if not argumento: ui.exibir("Largar o quê?"); return False
        return cmd_largar(argumento, jogo, mapa)
        
    elif verbo == "usar":
        if not argumento: ui.exibir("Usar o quê?"); return False
        return cmd_usar(argumento, jogo, mapa)
        
    elif verbo in ["combinar", "juntar"]:
        if not argumento: ui.exibir("Combinar o quê?"); return False
        return cmd_combinar(argumento, jogo, mapa)
        
    elif verbo == "examinar":
        if not argumento: return "olhar" 
        return cmd_examinar(argumento, jogo, mapa)
        
    elif verbo == "inventario":
        return cmd_inventario(jogo)
            
    elif verbo in ["limpar", "cls", "clear", "clean"]:
        ui.limpar()
        return True
        
    else:
        ui.exibir("Comando não reconhecido.")
        return False
