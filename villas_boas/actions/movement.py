import random
from villas_boas.utils import extrair_argumentos, encontrar_melhor_match
from ui import DOS_VERDE, DOS_BRANCO, DOS_AMARELO, DOS_VERMELHO, RESET, default_ui

def cmd_ir(comando, jogo, mapa):
    ui = jogo.ui_handler or default_ui
    direcao_bruta = comando.replace("ir ", "").strip()
    
    palavras_ignoradas = ["para", "pro", "pra", "em", "a", "o", "as", "os", "na", "no"]
    palavras_da_frase = extrair_argumentos(direcao_bruta)
    palavras_limpas = [p for p in palavras_da_frase if p not in palavras_ignoradas]
    direcao = " ".join(palavras_limpas)
    
    if direcao in ["tras", "atras", "fundo"]: direcao = "atrás"

    if jogo.sala_atual not in mapa:
        return False
    
    sala = mapa[jogo.sala_atual]

    if jogo.sala_atual == "03" and direcao == "frente":
        ui.exibir(f"{DOS_AMARELO}Você toma distância e dá um chute violento na porta emperrada!{RESET}")
        ui.exibir(f"{DOS_VERDE}CRASH! A madeira velha cede e a porta escancara.{RESET}")
        mapa["corredor"]["03"] = "sala do gerador"
        jogo.sala_atual = "sala do gerador"
        ui.pausar(2)
        return True

    saidas_validas = [
        k
        for k in sala
        if k not in ["descrição", "itens", "inspecionaveis", "cofre_important"]
        and isinstance(sala[k], str)
    ]
    
    match_direcao = encontrar_melhor_match(direcao, saidas_validas)
    if match_direcao:
        direcao = match_direcao
    else:
        if direcao == "cadeira" and "cadeira" in sala:
            pass 
        else:
            ui.exibir(f"Você não pode ir para '{direcao_bruta}'.")
            if saidas_validas: ui.exibir(f"{DOS_BRANCO}Saídas disponíveis: {', '.join(saidas_validas).title()}{RESET}")
            ui.pausar(1.5)
            return False

    destino = sala.get(direcao, direcao) 
    
    if direcao == "cadeira" and "cadeira" in sala:
        destino = sala["cadeira"]

    lugares_validos = list(mapa.keys()) + ["morte", "saida", "01", "cadeira"]

    if destino in lugares_validos:
        ui.limpar()
        jogo.turnos_mesma_sala = 0

        if jogo.turnos_luz <= 0 and not getattr(jogo, 'god_mode', False) and random.randint(1, 100) <= 10:
            ui.exibir("\n No escuro, você perde a noção da direção, e acaba tropeçando no proprio pé, e cai no chão")
            jogo.hp -= 1
            ui.exibir(f" Você se machucou na queda. (HP: {jogo.hp})")
            ui.pausar(2)
            if jogo.hp <= 0:
                ui.exibir("\n Você cai no chão e quebra sua perna, você não consegue mais andar, e escuta barulhos vindo na sua direção")
                ui.pausar(2)
                jogo.sala_atual = "morte"
            return True

        jogo.sala_atual = destino

        if getattr(jogo, 'dificuldade_escolhida', 'NORMAL') == "PESADELO" and jogo.sala_atual == getattr(jogo, 'posicao_perseguidor', ''):
            ui.limpar()
            ui.exibir("\n" + "="*50)
            ui.exibir(f"{DOS_VERMELHO}Quando voce entra na sala, passos pesados e cheiro de fuligem invadem o ar.{RESET}")
            ui.exibir(f"{DOS_VERMELHO}Uma mão robótica gigante segura o seu pescoço e te levanta do chão.{RESET}")
            ui.exibir(f"{DOS_AMARELO}Você tem UMA ação para reagir antes que ele quebre o seu pescoço.{RESET}")
            jogo.estado_atual = "COMBATE_ANIMATRONICO" 
            ui.pausar(2)
            return True
        
        if jogo.sala_atual == "saida" and (
        getattr(jogo, "noite_vencida", False)
        and getattr(jogo, "fios_cortados_inventario", False)
        and not getattr(jogo, "incendio", False)
        ):
            ui.exibir(f"\n{DOS_VERDE}[DISPOSITIVO]: NÍVEL 2 - PRESENÇA PRÓXIMA.{RESET}")
            ui.exibir(f"{DOS_AMARELO}'Eu preciso terminar isso antes...', você murmura para si mesmo.{RESET}")
            ui.exibir(f"{DOS_AMARELO}Você vira as costas para a saída.{RESET}")
            jogo.sala_atual = "entrada"
            ui.pausar(3)
        else:
            ui.pausar(1.5)
            
        return True