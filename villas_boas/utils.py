import difflib
import random
import shlex
import unicodedata

from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERMELHO, RESET, default_ui


def normalizar(texto):
    texto_sem_acento = (
        unicodedata.normalize("NFKD", texto).encode("ASCII", "ignore").decode("utf-8")
    )
    return texto_sem_acento.strip().lower()


def extrair_argumentos(comando_bruto):
    try:
        args = shlex.split(comando_bruto)
        return [normalizar(a) for a in args]
    except ValueError:
        return [normalizar(a) for a in comando_bruto.split()]


def encontrar_melhor_match(termo, lista_opcoes, cutoff=0.70):
    if not termo or not lista_opcoes:
        return None
    for op in lista_opcoes:
        if termo == op:
            return op
    for op in lista_opcoes:
        if op.startswith(termo) or termo.startswith(op):
            return op
    for op in lista_opcoes:
        if termo in op or op in termo:
            return op
    sugestoes = difflib.get_close_matches(termo, lista_opcoes, n=1, cutoff=cutoff)
    return sugestoes[0] if sugestoes else None

def atualizar_eventos_de_tempo(jogo):
    ui = jogo.ui_handler or default_ui

    if getattr(jogo, "god_mode", False):
        jogo.hp = 9999
        jogo.turnos_luz = 9999
        return

    # 1. DEFINIÇÃO DAS SALAS ILUMINADAS (ZONAS SEGURAS DE LUZ)
    salas_iluminadas = ["sala de fliperamas", "hall de entrada", "sala de refrigeração", "01"]

    if getattr(jogo, "amanheceu", False) or jogo.sala_atual in salas_iluminadas:
        jogo.turnos_no_escuro = 0
        # A luz da lanterna NÃO é drenada aqui, e o escuro é resetado.
    else:
        if jogo.turnos_luz > 0:
            jogo.turnos_luz -= 1
            jogo.turnos_no_escuro = 0
            
            # Avisos prévios de bateria fraca
            if jogo.turnos_luz == 2:
                ui.buffer.append("@@GLITCH_LUZ@@")
                ui.exibir(f"\n{DOS_AMARELO}Sua lanterna dá uma leve piscada. A luz está enfraquecendo...{RESET}")

            elif jogo.turnos_luz == 1:
                ui.buffer.append("@@GLITCH_LUZ@@")
                ui.exibir(f"\n{DOS_VERMELHO}A luz da lanterna falha quase que totalmente. Vai apagar a qualquer momento.{RESET}")

            elif jogo.turnos_luz == 0:
                ui.exibir(f"\n{DOS_VERMELHO}A escuridão volta a dominar... Sua fonte de luz se apagou{RESET}")
                ui.pausar(1.5)
        else:
            jogo.turnos_no_escuro += 1
            
            if jogo.turnos_no_escuro == 1:
                ui.exibir(f"\n{DOS_AMARELO}As sombras parecem se mexer nos cantos da sua visão...{RESET}")
            elif jogo.turnos_no_escuro == 3:
                jogo.nivel_barulho = 100
                jogo.ai_alvo = jogo.sala_atual
                ui.exibir(f"\n{DOS_VERMELHO}No escuro absoluto, você entra em pânico e esbarra nos móveis. O barulho ecoa pelo corredor!{RESET}")
            elif jogo.turnos_no_escuro >= 5:
                jogo.hp -= 1
                jogo.turnos_no_escuro = 0 
                if jogo.hp > 0:
                    ui.exibir(f"\n{DOS_VERMELHO}Você tenta andar no escuro, tropeça violentamente e se machuca! (-1 HP){RESET}")
                else:
                    ui.exibir(f"\n{DOS_VERMELHO}Você cai de mau jeito no escuro e bate a cabeça. Você não consegue mais levantar...{RESET}")
                    jogo.sala_atual = "morte"

    # Restante dos eventos (Fuga do incêndio e enjoo)
    if getattr(jogo, "incendio", False):
        jogo.turnos_fuga -= 1
        ui.exibir(f"\n{DOS_VERMELHO}O RESTAURANTE ESTÁ DESMORONANDO ({jogo.turnos_fuga} turnos para fugir){RESET}")
        if jogo.turnos_fuga <= 0:
            ui.exibir(f"\n{DOS_VERMELHO}O teto desaba sobre você. O fogo consome o que restou.{RESET}")
            jogo.sala_atual = "morte"

    if jogo.turnos_enjoado > 0:
        ui.exibir(f"\n{DOS_AMARELO}Você está enjoado e com tontura... Seus olhos embaçam.{RESET}")
        if jogo.turnos_luz > 0 and not getattr(jogo, "amanheceu", False) and jogo.sala_atual not in salas_iluminadas:
            jogo.turnos_luz -= 1
        jogo.turnos_enjoado -= 1

    # 2. LÓGICA DA SAFEROOM (SALA 01 E OUTRAS)
    salas_seguras_ia = ["saida", "cama", "final_bom", "morte", "tubo de ventilação", "01"]

    if jogo.dificuldade_escolhida == "NORMAL":
        # No modo normal, o perseguidor só conta turnos se você NÃO estiver na saferoom
        if jogo.sala_atual not in salas_seguras_ia:
            jogo.turnos_mesma_sala += 1
            if jogo.turnos_mesma_sala == jogo.turnos_perseguidor_aviso:
                ui.exibir(f"\n{DOS_AMARELO}Você escuta ruídos metálicos pesados ecoando no corredor próximo...{RESET}")
            elif jogo.turnos_mesma_sala == jogo.turnos_perseguidor_morte:
                ui.exibir("\n" + "=" * 50 + f"\n{DOS_VERMELHO}Você ficou muito tempo parado. Algo entrou na sua sala...\n{RESET}" + "=" * 50)
                ui.pausar(4)
                jogo.sala_atual = "morte"
        else:
            # Reseta os turnos se ele entrou na Saferoom
            jogo.turnos_mesma_sala = 0

    elif jogo.dificuldade_escolhida == "PESADELO":
        if jogo.posicao_perseguidor != "morte" and jogo.sala_atual not in salas_seguras_ia:
            sala_monstro = jogo.mapa.get(jogo.posicao_perseguidor, {})
            conexoes = [
                v for k, v in sala_monstro.items()
                if k not in ["descrição", "itens", "inspecionaveis"]
                and v in jogo.mapa
                and v not in salas_seguras_ia # O monstro não anda para salas seguras
            ]
            if conexoes and random.random() < 0.40:
                jogo.posicao_perseguidor = random.choice(conexoes)

            if jogo.posicao_perseguidor == jogo.sala_atual:
                ui.exibir("\n" + "=" * 50)
                ui.exibir(f"{DOS_VERMELHO}A porta quebra. Ela te encontrou{RESET}")
                ui.pausar(3)
                jogo.sala_atual = "morte"
            else:
                conexoes_jogador = [
                    v for k, v in jogo.mapa[jogo.sala_atual].items()
                    if k not in ["descrição", "itens", "inspecionaveis"] and isinstance(v, str)
                ]
                if jogo.posicao_perseguidor in conexoes_jogador:
                    ui.exibir(f"\n{DOS_AMARELO}O chão vibra. Você ouve passos de metal maciço na sala ao lado...{RESET}")


def corromper_texto(texto, intensidade=0.5):
    """Adiciona caracteres Zalgo (ruído visual) simulando perda de sanidade."""
    if intensidade <= 0:
        return texto

    zalgo_chars = [chr(i) for i in range(0x0300, 0x036F)]
    resultado = []

    for char in texto:
        resultado.append(char)

        if char.isalpha() and random.random() < intensidade:
            num_zalgos = random.randint(1, 3)
            for _ in range(num_zalgos):
                resultado.append(random.choice(zalgo_chars))

    return "".join(resultado)
