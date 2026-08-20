import random

from data import CAVEIRA_MORTE
from state import carregar_conquistas, registrar_final
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET
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
        f"{DOS_BRANCO}{c_med} Medíocre  {c_son} Bons Sonhos  {c_bom} Bom  {c_ver} Verdadeiro{RESET}\n",
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

    if tem_autosave:
        ui.animar(
            f"{DOS_VERDE}[5] CONTINUAR JOGO (Autosave Encontrado){RESET}\n",
            0.01,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            f"{DOS_VERDE}SELECIONE UMA OPÇÃO (1-5): {RESET}", 0.01, DOS_BRANCO, jogo
        )
    else:
        ui.animar(
            f"\n{DOS_VERDE}SELECIONE UMA OPÇÃO (1-4): {RESET}", 0.01, DOS_BRANCO, jogo
        )


def imprimir_tutorial(ui, jogo=None):
    ui.animar(
        f"\n{DOS_AMARELO}--- DICAS DE SOBREVIVÊNCIA (TUTORIAL) ---{RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_BRANCO}1. Mova-se digitando {DOS_VERDE}ir frente{DOS_BRANCO}, ou apenas o nome da sala.{RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f'{DOS_BRANCO}2. Use aspas para nomes compostos: {DOS_VERDE}pegar "tabua pequena de madeira"{DOS_BRANCO}.{RESET}',
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_BRANCO}3. Você pode usar atalhos como {DOS_VERDE}p chave{DOS_BRANCO} em vez de {DOS_VERDE}pegar chave{DOS_BRANCO}.{RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_BRANCO}4. Digite {DOS_VERDE}ajuda{DOS_BRANCO} a qualquer momento para ver o manual do sistema.{RESET}",
        0.01,
        DOS_BRANCO,
        jogo,
    )
    ui.animar(
        f"{DOS_AMARELO}-----------------------------------------{RESET}\n",
        0.01,
        DOS_BRANCO,
        jogo,
    )


def dar_dica_jon(passo_certo, ui):
    dicas = {
        "f": "Uma corrente de ar gelado bate direto no seu rosto.",
        "e": "Um som agudo de metal arranhando reverbera pela parede canhota do duto.",
        "d": "O cheiro podre de carne estragada fica mais forte no caminho destro.",
    }
    if random.random() <= 0.25:
        ui.exibir(
            f"\n{DOS_VERMELHO}[SENSÓRIO CONFUSO]: {random.choice([v for k, v in dicas.items() if k != passo_certo])}{RESET}"
        )
    else:
        ui.exibir(f"\n{DOS_AMARELO}[SENSÓRIO]: {dicas[passo_certo]}{RESET}")


def falar_pianista(acertou, ui, jogo):
    if acertou:
        ui.exibir(f"{DOS_BRANCO}A máquina toca uma nota suave e agradável.{RESET}")
        ui.animar(
            f'"{random.choice(["Você lembra bem,", "O ritmo continua. Você ainda tem ouvido para isso.", "Correto. Nós sempre soubemos que você voltaria.", "Sim... exatamente como aconteceu."])}"',
            0.04,
            DOS_AMARELO,
            jogo,
        )
    else:
        ui.exibir(f"{DOS_VERMELHO}Acorde dissonante.{RESET}")
        ui.animar(
            f'"{random.choice(["Errado. As teclas pretas não perdoam mentiras.", "Você deveria lembrar melhor do que isso, Rogério.", "Uma nota fora do lugar... como você, aquela noite.", "Isso não é o que consta no registro do restaurante."])}"',
            0.04,
            DOS_AMARELO,
            jogo,
        )


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

    if jogo.hp <= 1 and not getattr(jogo, "god_mode", False):
        descricao_colorida = corromper_texto(descricao_colorida, intensidade=0.4)
        ui.animar(
            f"{DOS_VERMELHO}[SISTEMA NEUROLÓGICO COMPROMETIDO]{RESET}",
            0.01,
            DOS_VERMELHO,
            jogo,
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
    ui.animar(f"{DOS_VERMELHO}{CAVEIRA_MORTE}{RESET}", 0.002, jogo=jogo)
    ui.animar("☠ GAME OVER. A NOITE ENGOLIU VOCÊ.", 0.05, DOS_VERMELHO, jogo)
    ui.animar(
        "=== SISTEMA CORROMPIDO. APERTE F5 PARA REINICIAR ===", 0.05, DOS_AMARELO, jogo
    )


def rodar_final(tipo_final, jogo):
    jogo.estado_atual = "FIM"
    ui = jogo.ui_handler
    ui.limpar()
    liberou_deus = False

    if tipo_final == "saida":
        ui.animar("Você sai pela porta do restaurante...", 0.08, DOS_AMARELO, jogo)
        ui.animar(
            "Você acredita que vai conseguir superar tudo isso, e continuar uma nova vida...",
            0.09,
            DOS_VERMELHO,
            jogo,
        )
        ui.animar(
            "Você olha para o céu, ve as estrelas, e uma estrela cadente passa...",
            0.09,
            DOS_VERMELHO,
            jogo,
        )
        ui.animar(
            "Você deseja esquecer ela, esquecer tudo isso...", 0.09, DOS_VERMELHO, jogo
        )
        ui.animar(
            "Você quer esquecer dela... Mesmo ela suplicando por ajuda...",
            0.10,
            DOS_VERMELHO,
            jogo,
        )
        ui.animar(
            "Entra no seu carro, e parte de volta para casa.", 0.09, DOS_VERMELHO, jogo
        )
        ui.animar("Alguma coisa te seguiu...", 0.08, DOS_AMARELO, jogo)
        ui.animar("[ FINAL MEDÍOCRE ]", 0.05, DOS_VERDE, jogo)
        liberou_deus = registrar_final("mediocre")

    elif tipo_final == "cama":
        ui.animar("Você deita na cama...", 0.05, DOS_AMARELO, jogo)
        ui.animar("Você sente algo abrir a porta...", 0.09, DOS_VERMELHO, jogo)
        ui.animar(
            "Você finalmente sente ela, o seu cheiro, sua vibração... Seu dispositivo apita...",
            0.05,
            DOS_VERDE,
            jogo,
        )
        ui.animar("PRESENÇA ULTERIOR PROXIMA...", 0.10, DOS_VERMELHO, jogo)
        ui.animar("Você dorme... Pensando em não acordar mais.", 0.10, DOS_VERDE, jogo)
        ui.animar("[ FINAL BONS SONHOS ]", 0.05, DOS_BRANCO, jogo)
        liberou_deus = registrar_final("bons_sonhos")

    elif tipo_final == "final_bom":
        ui.limpar()

        ui.animar(
            f"{DOS_VERDE} ======================================================================{RESET}",
            0.03,
            jogo=jogo,
        )
        ui.animar(
            f"{DOS_VERDE}                             [FINAL BOM]                               {RESET}",
            0.04,
            jogo=jogo,
        )
        ui.animar(
            f"{DOS_VERDE} ======================================================================{RESET}",
            0.03,
            jogo=jogo,
        )
        ui.animar(
            "Você acende o isqueiro e ilumina o local. A luz do fogo traz calma...",
            0.04,
            DOS_VERDE,
            jogo,
        )
        ui.animar(
            "- Por que não deu certo? O que eu fiz de errado?", 0.05, DOS_AMARELO, jogo
        )
        ui.animar("- 'Ainda estou aqui...'", 0.09, DOS_VERMELHO, jogo)
        ui.animar("- Amor? É voce? Mesmo???", 0.05, DOS_AMARELO, jogo)
        ui.animar("- 'Eu espero que ainda seja eu...'", 0.09, DOS_VERMELHO, jogo)
        ui.animar(
            "- Caroline... desista desse corpo que não lhe pertence. Siga o rumo das estrelas.",
            0.05,
            DOS_AMARELO,
            jogo,
        )
        ui.animar("- ... *Caroline abraça Rogério*", 0.09, DOS_VERMELHO, jogo)
        ui.animar("- 'Vamos nos encontrar no céu, meu bem.'", 0.09, DOS_VERMELHO, jogo)
        ui.exibir(f"\n{DOS_BRANCO}[ FINAL BOM ]{RESET}")
        liberou_deus = registrar_final("bom")

    elif tipo_final == "verdadeiro":
        ui.limpar()
        ui.animar(
            f"{DOS_VERMELHO}=================================================================={RESET}",
            0.02,
            jogo=jogo,
        )
        ui.animar(
            f"{DOS_VERMELHO}                         [ FINAL VERDADEIRO ]                     {RESET}",
            0.03,
            jogo=jogo,
        )
        ui.animar(
            f"{DOS_VERMELHO}=================================================================={RESET}\n",
            0.02,
            jogo=jogo,
        )

        ui.animar(
            "O estalo das chamas consome as cortinas velhas e a madeira podre da sala.",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "A fumaça preta e espessa começa a subir, cobrindo o teto do restaurante.",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "O calor é sufocante, mas você não corre para a saída ainda. Você caminha até o centro do hall de entrada.\n",
            0.04,
            DOS_BRANCO,
            jogo,
        )

        ui.animar(
            "Lá está a carcaça de pelúcia rosa, com o metal rangendo e o plástico derretendo.",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "Você se aproxima do animatrônico... dela. E encaixa os fios na sua fiação exposta...",
            0.05,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "Você acende o isqueiro. Os olhos de plástico parecem te encarar.",
            0.05,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "Os LEDs vermelhos piscam fracos. Ela tenta se mover... mas o mecanismo emperra.\n",
            0.05,
            DOS_BRANCO,
            jogo,
        )

        ui.animar(
            "- Por que não deu certo? O que eu fiz de errado...?",
            0.05,
            DOS_AMARELO,
            jogo,
        )
        ui.animar(
            "Sua voz treme. As lágrimas cortam a fuligem no seu rosto.",
            0.04,
            DOS_BRANCO,
            jogo,
        )

        ui.animar(
            "\nUm zumbido eletrônico arranhado sai do alto-falante no peito da carcaça:",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar("- '... v-você... fez... dar... certo...'", 0.08, DOS_VERMELHO, jogo)
        ui.animar("- '... R-Rogério...?'", 0.08, DOS_VERMELHO, jogo)

        ui.animar("\n- Caro... Caroline? É você?", 0.05, DOS_AMARELO, jogo)
        ui.animar("*(Você abraça a carcaça de pelagem rosa)*", 0.04, DOS_BRANCO, jogo)
        ui.animar(
            "Mesmo com o calor das chamas se espalhando, o metal que te toca é frio e rígido.",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "Mas você não solta. Você a segura com toda a força que te resta.",
            0.04,
            DOS_BRANCO,
            jogo,
        )

        ui.animar(
            "- Meu corpo ficou em silêncio... O barulho da minha cabeça, simplesmente parou.",
            0.07,
            DOS_VERDE,
            jogo,
        )
        ui.animar(
            "- A dor nas minhas costas... o ferro rasgando minha pele... sumiu.",
            0.07,
            DOS_VERDE,
            jogo,
        )
        ui.animar("- Eu não sinto mais raiva.", 0.07, DOS_VERDE, jogo)

        ui.animar(
            "\n- Eu sinto muito, Caroline... Me desculpa por não ter chegado a tempo...",
            0.05,
            DOS_AMARELO,
            jogo,
        )
        ui.animar(
            "- Eu... Eu li seus e-mails. Eu vim assim que veio as noticias, eu tenho te procurando em todo o lugar...",
            0.05,
            DOS_AMARELO,
            jogo,
        )

        ui.animar(
            "\n*(O fogo se alastra pelo restaurante, a fumaça chega no hall)*",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "O crepitar do incêndio abafa os ruídos metálicos dos outros robôs, gritando, tentando sobreviver, mesmo já mortos.",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "O filtro de luz vermelha dos olhos de plástico do coelho se apaga para sempre.",
            0.04,
            DOS_BRANCO,
            jogo,
        )

        ui.animar(
            "\nNa penumbra dourada pelas chamas, a voz não vem mais do alto-falante. Ela ecoa suave na sua mente:",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar("- Não peça desculpas, meu amor... Você veio.", 0.07, DOS_VERDE, jogo)
        ui.animar(
            "- Durante semanas, no escuro dessa máquina, eu não tinha consciencia...",
            0.07,
            DOS_VERDE,
            jogo,
        )
        ui.animar(
            "- Eu queria fazer eles pagarem pelo que 'eles' me fizeram passar nesse lugar...",
            0.07,
            DOS_VERDE,
            jogo,
        )
        ui.animar(
            "- Mas quando você me abraçou... a escuridão simplesmente evaporou.",
            0.07,
            DOS_VERDE,
            jogo,
        )

        ui.animar(
            "\n- Não podia te deixar presa aqui, Caroline. Esse lugar precisa queimar. Eu procurei por arquivos, descobri os horrores que aconteceram... vim para te salvar, e os outros também. Hoje acaba.",
            0.06,
            DOS_AMARELO,
            jogo,
        )
        ui.animar(
            "- A gente ainda vai viver juntos... Pena que seja em outra vida.",
            0.05,
            DOS_AMARELO,
            jogo,
        )

        ui.animar(
            "\nUma brisa suave e fresca corta o ar quente do incêndio, envolvendo seu pescoço.",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar("- Me sinta pela última vez.", 0.07, DOS_VERDE, jogo)

        ui.animar(
            "\n*(Você sente mãos invisíveis e macias em seus ombros, um alívio inunda sua mente)*",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "Toda a dor, a paranoia e o pavor que pesavam sobre seu peito desde que entrou no Vilas Boas desaparecem.",
            0.04,
            DOS_BRANCO,
            jogo,
        )

        ui.animar(
            "\n- Obrigado por não desistir de mim... Obrigado por me deixar assim pela última vez.",
            0.07,
            DOS_VERDE,
            jogo,
        )
        ui.animar(
            "- Vai embora agora, viva por nós dois. Eu sempre estarei contigo.",
            0.07,
            DOS_VERDE,
            jogo,
        )
        ui.animar("- Eu te amo.", 0.06, DOS_AMARELO, jogo)
        ui.animar("- Eu também te amo, amor. Pra sempre.", 0.06, DOS_AMARELO, jogo)

        ui.animar(
            "\n*(O animatrônico cai no chão, o fogo cobre o metal e o plástico rosa)*",
            0.05,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "A carcaça é consumida pelas chamas enquanto a estrutura do palco cede.",
            0.04,
            DOS_BRANCO,
            jogo,
        )

        ui.animar(
            f"\n{DOS_VERDE}[DISPOSITIVO]: NENHUMA PRESENÇA DETECTADA.{RESET}",
            0.05,
            jogo=jogo,
        )

        ui.animar(
            "\nVocê se levanta e caminha para a saída antes que o teto desabe.",
            0.05,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "Você empurra as portas dos fundos e sai para o ar frio da madrugada de Curitiba.",
            0.04,
            DOS_BRANCO,
            jogo,
        )
        ui.animar(
            "Pela calçada, você vê a fumaça subindo ao amanhecer. O restaurante Vilas Boas virou cinzas.",
            0.04,
            DOS_BRANCO,
            jogo,
        )

        ui.exibir(f"\n{DOS_BRANCO}[ FINAL VERDADEIRO: LIBERTAÇÃO ]{RESET}")
        liberou_deus = registrar_final("verdadeiro")

    if liberou_deus:
        ui.exibir(
            f"\n{DOS_AMARELO}=================================================={RESET}"
        )
        ui.animar(
            "VOCÊ DESVENDOU TODAS AS VERDADES DESTA NOITE.", 0.05, DOS_VERMELHO, jogo
        )
        ui.exibir(
            f"{DOS_AMARELO}DIGITE O ANO EM QUE TUDO ACABOU NA TELA DE MENU: {DOS_BRANCO}2007{RESET}"
        )
        ui.exibir(
            f"{DOS_AMARELO}=================================================={RESET}"
        )

    ui.animar("\n=== APERTE F5 PARA REINICIAR ===", 0.05, DOS_AMARELO, jogo)
