import random

from data import (
    BARULHO_TROPECO_NAO_LETAL,
    CHANCE_TROPECO_ESCURO_PCT,
    EVENTOS_ALEATORIOS,
    SALA_GERADOR_ABERTA,
    SALAS_ILUMINADAS,
    TROPECO_PODE_MATAR,
)
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, RESET, default_ui

from villas_boas.utils import encontrar_melhor_match, extrair_argumentos


def cmd_ir(comando, jogo, mapa):
    ui = jogo.ui_handler or default_ui
    direcao_bruta = comando.replace("ir ", "").strip()

    palavras_ignoradas = ["para", "pro", "pra", "em", "a", "o", "as", "os", "na", "no"]
    palavras_da_frase = extrair_argumentos(direcao_bruta)
    palavras_limpas = [p for p in palavras_da_frase if p not in palavras_ignoradas]
    direcao = " ".join(palavras_limpas)

    if direcao in ["tras", "atras", "fundo"]:
        direcao = "atrás"

    if jogo.sala_atual not in mapa:
        return False

    sala = mapa[jogo.sala_atual]

    if jogo.sala_atual == "03" and direcao == "frente":
        ui.exibir(
            f"{DOS_AMARELO}Você toma distância e dá um chute violento na porta emperrada!{RESET}"
        )
        ui.exibir(f"{DOS_VERDE}A madeira velha cede e a porta escancara.{RESET}")
        mapa["corredor"]["03"] = SALA_GERADOR_ABERTA
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
            if saidas_validas:
                ui.exibir(
                    f"{DOS_BRANCO}Saídas disponíveis: {', '.join(saidas_validas).title()}{RESET}"
                )
            ui.pausar(1.5)
            return False

    destino = sala.get(direcao, direcao)

    if direcao == "cadeira" and "cadeira" in sala:
        destino = sala["cadeira"]

    lugares_validos = list(mapa.keys()) + [
        "morte",
        "saida",
        "01",
        "cadeira",
        "cama",
        "final_bom",
        "hall de entrada",
    ]

    if destino not in lugares_validos:
        ui.exibir("O caminho está bloqueado ou não há saída para essa direção.")
        return False

    ui.limpar()
    jogo.turnos_mesma_sala = 0
    jogo.turnos_escondido = 0

    if (
        jogo.turnos_luz <= 0
        and jogo.sala_atual not in SALAS_ILUMINADAS
        and not getattr(jogo, "god_mode", False)
        and getattr(jogo, "dificuldade_escolhida", "") != "DESAFIO_BREU"
        and random.randint(1, 100) <= CHANCE_TROPECO_ESCURO_PCT
    ):
        ui.exibir(
            "\n No escuro, você perde a noção da direção, e acaba tropeçando no proprio pé, e cai no chão"
        )
        if jogo.hp <= 1 and not TROPECO_PODE_MATAR:
            # O tropeço nunca é o golpe final: só faz barulho e custa o turno.
            jogo.alterar_barulho(BARULHO_TROPECO_NAO_LETAL)
            jogo.ai_alvo = jogo.sala_atual
            ui.exibir(
                f"{DOS_AMARELO} Você se segura nos móveis no último segundo. O estrondo ecoa pelo prédio.{RESET}"
            )
            ui.pausar(2)
            return True
        morreu = jogo.aplicar_dano(1)
        ui.exibir(f" Você se machucou na queda. (HP: {jogo.hp})")
        ui.pausar(2)
        if morreu:
            jogo.causa_morte = "tropeco"
            ui.exibir(
                "\n Você cai no chão e quebra sua perna, você não consegue mais andar, e escuta barulhos vindo na sua direção"
            )
            ui.pausar(2)
        return True

    jogo.sala_atual = destino
    if destino == "morte":
        jogo.causa_morte = "saida_mortal"

    if EVENTOS_ALEATORIOS and random.random() < 0.25:
        evento = random.choice(EVENTOS_ALEATORIOS)
        item_drop = evento.get("item_drop")

        if item_drop == "bateria nova" and getattr(jogo, "bateria_ajuda_gerada", False):
            return True

        ui.pausar(0.5)
        ui.exibir(f"\n{DOS_AMARELO}⚠ {evento['mensagem']}{RESET}")

        if item_drop and item_drop not in jogo.inventario:
            sala_dic = mapa[jogo.sala_atual]
            sala_dic.setdefault("itens", [])
            if item_drop not in sala_dic["itens"]:
                sala_dic["itens"].append(item_drop)

                if item_drop == "bateria nova":
                    jogo.bateria_ajuda_gerada = True

    ui.pausar(0.5)
    return True
