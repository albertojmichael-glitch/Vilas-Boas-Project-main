import difflib
import random
import shlex
import unicodedata

from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET, default_ui

CONQUISTAS_DB = {
    "primeira_morte": {
        "id": "primeira_morte",
        "nome": "Sangue no Carpete",
        "desc": "Bem-vindo ao Vilas Boas.",
        "icone": "☠",
    },
    "mente_brilhante": {
        "id": "mente_brilhante",
        "nome": "Mente Brilhante",
        "desc": "Abra o cofre na primeira tentativa.",
        "icone": "★",
    },
    "toureiro": {
        "id": "labirinto",
        "nome": "Labirinto",
        "desc": "Sobreviva à Sala de Energia.",
        "icone": "☄",
    },
    "acumulador": {
        "id": "acumulador",
        "nome": "Acumulador",
        "desc": "Encha todos os 9 espaços do inventário.",
        "icone": "☑",
    },
    "glicose": {
        "id": "glicose",
        "nome": "Glicose Duvidosa",
        "desc": "Coma o doce velho encontrado no chão.",
        "icone": "✴",
    },
    "trapaceiro": {
        "id": "trapaceiro",
        "nome": "Hacker",
        "desc": "Ative o God Mode.",
        "icone": "☣",
    },
    "speedrun": {
        "id": "speedrun",
        "nome": "Corredor Noturno",
        "desc": "Zere o jogo no Modo Desafio: Speedrun (Menos de 5 minutos reais).",
        "icone": "⏱",
    },
    "fantasma": {
        "id": "fantasma",
        "nome": "Fantasma",
        "desc": "Zere no Modo Desafio: Fantasma (Sem deixar o barulho passar de 50%).",
        "icone": "☹",
    },
    "breu_total": {
        "id": "breu_total",
        "nome": "Breu Total",
        "desc": "Zere no Modo Desafio: Breu Total (Sem luz desde o início).",
        "icone": "👁",
    },
    "normal_zerado": {
        "id": "normal_zerado",
        "nome": "Fim do Expediente",
        "desc": "Sobreviva à noite na dificuldade Normal.",
        "icone": "♨",
    },
    "final_bom": {
        "id": "final_bom",
        "nome": "Juntos Novamente",
        "desc": "Alcance o Final Neutro.",
        "icone": "☀",
    },
    "final_verdadeiro": {
        "id": "final_verdadeiro",
        "nome": "A Verdade",
        "desc": "Alcance o Final Verdadeiro.",
        "icone": "✦",
    },
    "pesadelo_neutro": {
        "id": "pesadelo_neutro",
        "nome": "Pesadelo Superado",
        "desc": "Alcance o Final Neutro no modo PESADELO.",
        "icone": "♛",
    },
    "pesadelo_verdadeiro": {
        "id": "pesadelo_verdadeiro",
        "nome": "Sonhador",
        "desc": "Alcance o Final Verdadeiro no modo PESADELO.",
        "icone": "♚",
    },
}


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


def processar_perseguidores_por_tempo(jogo, ui):
    """Perseguidores 'por tempo': o cronômetro de ficar parado e a 'Ela'.

    No NORMAL, ficar parado tempo demais faz barulho e chama a 'Ela' (em vez de
    matar sozinho). A 'Ela' existe no NORMAL e no PESADELO; o que muda entre os
    modos é o perfil de IA (``jogo.perfil_ela``)."""
    from data import LIMIAR_ALERTA, LIMIAR_CACA

    from villas_boas.ela import SALAS_SEGURAS_ELA, ela_ativa_no_modo, processar_ela

    if not ela_ativa_no_modo(jogo):
        return
    if jogo.sala_atual in SALAS_SEGURAS_ELA:
        # Ela não entra aqui e perde o rastro, mas continua andando: assim não
        # fica acampada na porta, e os passos dela seguem audíveis.
        jogo.turnos_mesma_sala = 0
        processar_ela(jogo, ui, barulho_alto=False)
        return

    if jogo.dificuldade_escolhida == "NORMAL":
        jogo.turnos_mesma_sala += 1
        if jogo.turnos_mesma_sala == jogo.turnos_perseguidor_aviso:
            ui.exibir(
                f"\n{DOS_AMARELO}Você escuta ruídos metálicos pesados ecoando no corredor próximo...{RESET}"
            )
        elif jogo.turnos_mesma_sala >= jogo.turnos_perseguidor_morte:
            ui.exibir(
                f"\n{DOS_VERMELHO}Você ficou tempo demais parado. Algo ouviu você e está vindo...{RESET}"
            )
            jogo.turnos_mesma_sala = 0
            jogo.ela_dormencia = 0
            jogo.alterar_barulho(LIMIAR_CACA)
            jogo.ai_alvo = jogo.sala_atual

    processar_ela(jogo, ui, barulho_alto=jogo.nivel_barulho >= LIMIAR_ALERTA)


def atualizar_eventos_de_tempo(jogo):
    ui = jogo.ui_handler or default_ui

    if getattr(jogo, "god_mode", False):
        jogo.hp = 9999
        jogo.turnos_luz = 9999
        if getattr(jogo, "dev_ia", False):
            processar_perseguidores_por_tempo(jogo, ui)
        return

    from data import SALAS_ILUMINADAS as salas_iluminadas

    if getattr(jogo, "amanheceu", False) or jogo.sala_atual in salas_iluminadas:
        jogo.turnos_no_escuro = 0
    else:
        if jogo.turnos_luz > 0:
            jogo.turnos_luz -= 1
            jogo.turnos_no_escuro = 0
            if jogo.turnos_luz == 2:
                ui.buffer.append("@@GLITCH_LUZ@@")
                ui.exibir(
                    f"\n{DOS_AMARELO}Sua lanterna dá uma leve piscada. A luz está enfraquecendo...{RESET}"
                )
            elif jogo.turnos_luz == 1:
                ui.buffer.append("@@GLITCH_LUZ@@")
                ui.exibir(
                    f"\n{DOS_VERMELHO}A luz da lanterna falha quase que totalmente. Vai apagar a qualquer momento.{RESET}"
                )
            elif jogo.turnos_luz == 0:
                ui.exibir(
                    f"\n{DOS_VERMELHO}A escuridão volta a dominar... Sua fonte de luz se apagou{RESET}"
                )
                ui.pausar(1.5)
        else:
            jogo.turnos_no_escuro += 1
            if jogo.turnos_no_escuro == 1:
                ui.exibir(
                    f"\n{DOS_AMARELO}As sombras parecem se mexer nos cantos da sua visão...{RESET}"
                )
            elif jogo.turnos_no_escuro == 3:
                jogo.nivel_barulho = 100
                jogo.ai_alvo = jogo.sala_atual
                ui.exibir(
                    f"\n{DOS_VERMELHO}No escuro absoluto, você entra em pânico e esbarra nos móveis. O barulho ecoa pelo corredor!{RESET}"
                )
            elif jogo.turnos_no_escuro >= 5:
                jogo.turnos_no_escuro = 0
                if not jogo.aplicar_dano(1):
                    ui.exibir(
                        f"\n{DOS_VERMELHO}Você tenta andar no escuro, tropeça violentamente e se machuca! (-1 HP){RESET}"
                    )
                else:
                    ui.exibir(
                        f"\n{DOS_VERMELHO}Você cai de mau jeito no escuro e bate a cabeça. Você não consegue mais levantar...{RESET}"
                    )
                    jogo.causa_morte = "escuro"

    if getattr(jogo, "incendio", False):
        jogo.turnos_fuga -= 1
        ui.exibir(
            f"\n{DOS_VERMELHO}O RESTAURANTE ESTÁ DESMORONANDO ({jogo.turnos_fuga} turnos para fugir){RESET}"
        )
        if jogo.turnos_fuga <= 0:
            ui.exibir(
                f"\n{DOS_VERMELHO}O teto desaba sobre você. O fogo consome o que restou.{RESET}"
            )
            jogo.sala_atual = "morte"
            jogo.causa_morte = "incendio"

    if jogo.turnos_enjoado > 0:
        ui.exibir(
            f"\n{DOS_AMARELO}Você está enjoado e com tontura... Seus olhos embaçam.{RESET}"
        )
        if (
            jogo.turnos_luz > 0
            and not getattr(jogo, "amanheceu", False)
            and jogo.sala_atual not in salas_iluminadas
        ):
            jogo.turnos_luz -= 1
        jogo.turnos_enjoado -= 1

    processar_perseguidores_por_tempo(jogo, ui)


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


def texto_cintilante(texto, paleta="perigo"):
    if paleta == "perigo":
        cores = [DOS_VERMELHO, DOS_AMARELO]
    elif paleta == "sistema":
        cores = [DOS_VERDE, DOS_BRANCO]
    else:
        cores = [DOS_BRANCO, RESET]

    resultado = ""
    for i, char in enumerate(texto):
        if char.strip():
            resultado += f"{cores[i % len(cores)]}{char}"
        else:
            resultado += char
    return resultado + RESET
