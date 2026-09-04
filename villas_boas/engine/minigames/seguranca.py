import random
import logging
from ui import DOS_VERDE, DOS_BRANCO, DOS_AMARELO, DOS_VERMELHO, RESET
from data import ARTE_MESA_SEGURANCA, ARTE_INDIO
from villas_boas.engine.minigames.base import BaseMinigame

logger = logging.getLogger(__name__)



TURNO_FINAL = 24              
TURNO_INICIO_CALOR = 12       
TURNO_CRITICO = 22            

CUSTO_EXTRA_MEIO = 1          
CUSTO_EXTRA_CRITICO = 3       
CUSTO_PORTA = 2                
CUSTO_BASE_INFO_PESADO = 1     
CUSTO_BASE_MOTOR = 2           

LIMITE_USOS_SISTEMA = 2        


RICK_POS_PORTA = 3              
RICK_POS_ATAQUE = 4             
JON_POS_VISIVEL_CAMERA = 3      
JON_POS_SAINDO_DUTO = 4         
JON_POS_ATAQUE = 5              
CAROLINE_POS_PORTA = 5          
CAROLINE_POS_ATAQUE_PORTA = 6   
CAROLINE_POS_DUTO_VISIVEL = 4   
CAROLINE_POS_DUTO_FUGA = 5      
CAROLINE_POS_ATAQUE_DUTO = 6    

GERADOR_DURACAO_TURNOS = 2      

CHANCE_INTERFERENCIA_MOV = 0.10   
CHANCE_RICK_RECUAR = 0.25         
CHANCE_RICK_AVANCA_DE_3 = 0.8     
CHANCE_INDIO_JANELA = 30          
CHANCE_ALBERTO_TROLL = 20         
CHANCE_ANOMALIA_CAMERA = 1        

SISTEMAS_VALIDOS = {"camera", "relogio", "deteccao"}


class MinigameSeguranca(BaseMinigame):
    def __init__(self, jogo):
        self.jogo = jogo
        self.ui = jogo.ui_handler
        self.turno = 0

        god_mode = getattr(jogo, "god_mode", False)
        self.energia = 9999 if god_mode else random.randint(
            getattr(jogo, "energia_min_noite", 70),
            getattr(jogo, "energia_max_noite", 100),
        )

        bonus_luz = max(0, getattr(jogo, "turnos_luz", 0) - 3) * 5
        bonus_bateria = getattr(jogo, "inventario", []).count("bateria nova") * 10
        self.energia += bonus_luz + bonus_bateria
        if bonus_luz > 0 or bonus_bateria > 0:
            self.ui.buffer.append(
                f"@@TYPE@@verde@@15@@[SISTEMA] Baterias extras detectadas no "
                f"inventário. Energia redirecionada: +{bonus_luz + bonus_bateria}%"
            )

        self.porta_fechada = False
        self.erro_camera = False
        self.erro_relogio = False
        self.erro_deteccao = False
        self.apagao = 0
        self.rick_pos = 0
        self.jon_pos = 0
        self.caroline_pos = 0
        self.caroline_caminho = random.choice(["porta", "tubulacao"])
        self.indio_janela = False
        self.alberto_troll = False
        self.furia = getattr(jogo, "furia_noite", 1)
        self.gerador_reserva_usado = False
        self.turnos_gerador_ativo = 0
        self.usos_sistema_turno = 0

        self.ui.exibir(f"{DOS_BRANCO}{ARTE_MESA_SEGURANCA}{RESET}")
        self.ui.exibir("\n" + "=" * 50)
        self.ui.exibir("Você senta na cadeira da sala de segurança.")
        self.ui.pausar(1)

    
    def _custo_extra_por_hora(self):
        """Quanto mais tarde, mais caro fica usar qualquer sistema."""
        if self.turno >= TURNO_CRITICO:
            return CUSTO_EXTRA_CRITICO
        if self.turno >= TURNO_INICIO_CALOR:
            return CUSTO_EXTRA_MEIO
        return 0

    def _custos_turno(self):
        extra = self._custo_extra_por_hora()
        gratis = self.turnos_gerador_ativo > 0
        return {
            "info_leve": 0 if gratis else (1 + extra),
            "info_pesado": 1 + extra,
            "motor": CUSTO_BASE_MOTOR + extra,
        }

    def _sistema_sobrecarregado(self):
        return self.usos_sistema_turno >= LIMITE_USOS_SISTEMA

    def _gastar_sistema(self, custo):
        self.usos_sistema_turno += 1
        
        
        if self.usos_sistema_turno >= 5:
            self.ui.exibir(f"{DOS_VERMELHO}⚠ FALHA KERNEL: PLACA MÃE DERRETIDA POR SOBRECARGA MANUAL!{RESET}")
            self.ui.pausar(1.5)
            from views import dar_tela_kernel_panic
            dar_tela_kernel_panic(self.jogo)
            return "morte" 
            
        self.energia -= custo

    def _rick_na_porta(self):
        return self.rick_pos >= RICK_POS_PORTA

    def _caroline_na_porta(self):
        return self.caroline_caminho == "porta" and self.caroline_pos >= CAROLINE_POS_PORTA

    
    def imprimir_status(self):
        self.ui.limpar()
        self.ui.exibir("\n" + "=" * 50)
        chance_bug = self.caroline_pos * 15

        def bug(texto, chance):
            return "".join(
                c.upper() if c.isalpha() and random.randint(1, 100) <= chance else c
                for c in texto
            )

        if self.apagao > 0:
            hora_disp = "[SISTEMA DESLIGADO]"
        elif self.erro_relogio:
            hora_disp = f"0{(self.turno * 15) // 60}:??"
        else:
            hora_disp = f"0{(self.turno * 15) // 60}:{(self.turno * 15) % 60:02d}"

        texto_energia = "∞" if self.energia > 100 else f"{self.energia}%"
        self.ui.exibir(bug(f"RELOGIO: {hora_disp}", chance_bug))
        self.ui.exibir(bug(f"ENERGIA: {texto_energia}", chance_bug))
        self.ui.exibir(bug(f"PORTA CENTRAL: {'Fechada' if self.porta_fechada else 'Aberta'}", chance_bug))

        erros = []
        if self.erro_camera:
            erros.append("CÂMERAS")
        if self.erro_relogio:
            erros.append("RELÓGIO")
        if self.erro_deteccao:
            erros.append("DETECÇÃO")
        self.ui.exibir(f"ERROS ATIVOS: {', '.join(erros)}" if erros else bug("ERROS: Nenhum", chance_bug))

        if self.turnos_gerador_ativo > 0:
            self.ui.exibir(f"{DOS_VERDE}Gerador reserva: Ativo({self.turnos_gerador_ativo} turnos restantes){RESET}")
        elif not self.gerador_reserva_usado:
            self.ui.exibir(f"{DOS_AMARELO}Gerador Reserva: Disponível{RESET}")

        if self.alberto_troll:
            self.ui.exibir("\n[MENSAGEM]: ERRO CRÍTICO! FECHAR PORTA AGORA!")
        if self.indio_janela and not self.erro_deteccao:
            self.ui.exibir("\n" + bug("Você sente como se algo estivesse te olhando pelo vidro...", chance_bug))

        self.ui.exibir(
            "\nAção (ouvir | cameras | ver tubulacao | iluminar tubulacao | "
            "fechar porta | abrir porta | olhar vidro | ligar gerador | "
            "consertar [sistema] | esperar)"
        )

    
    def processar_turno(self, acao, jogo):
        ui = self.ui
        acao_norm = acao.lower().strip()
        god_mode = getattr(jogo, "god_mode", False)

        if acao_norm in ("pular noite", "pular", "set time 06:00") and god_mode:
            ui.exibir(f"{DOS_AMARELO}[GOD MODE] O tempo se contorce. O relógio salta para as 06:00.{RESET}")
            self.turno = TURNO_FINAL
            return self._checar_fim_de_noite(jogo)

        turno_passou = False
        acao_valida = True
        custos = self._custos_turno()

        if acao_norm == "fechar porta":
            self._acao_fechar_porta(ui, custos)

        elif acao_norm == "abrir porta":
            self._acao_abrir_porta(ui, custos)

        elif acao_norm == "iluminar tubulacao":
            self._acao_iluminar_tubulacao(ui, custos)

        elif acao_norm == "olhar vidro":
            self._acao_olhar_vidro(ui)

        elif acao_norm == "ligar gerador":
            turno_passou, self.turno = self._acao_ligar_gerador(ui, self.turno)

        elif acao_norm.startswith("consertar "):
            self._acao_consertar(ui, acao_norm, custos)

        elif acao_norm == "ouvir":
            self._acao_ouvir(ui, custos)

        elif acao_norm == "cameras":
            self._acao_cameras(ui, custos)

        elif acao_norm == "ver tubulacao":
            self._acao_ver_tubulacao(ui, custos)

        elif acao_norm in ("esperar", "pular noite", "pular", "set time 06:00"):
            ui.exibir("Você deixa o tempo passar...")
            turno_passou = True
            self.turno += 1
            self.alberto_troll = False

        else:
            ui.exibir("Comando inválido.")
            acao_valida = False

        if acao_valida and acao_norm not in ("esperar", "pular noite", "pular", "set time 06:00"):
            self._chance_interferencia(ui)

            if self.porta_fechada:
                self._checar_recuo_na_porta(ui)

        ui.pausar(2)

        if acao_valida:
            self._evento_ambiente_aleatorio(ui)

        ui.pausar(3)

        if turno_passou:
            resultado = self._resolver_fim_de_turno(ui, jogo, god_mode)
            if resultado is not None:
                return resultado

        if self.turno >= TURNO_FINAL:
            return self._checar_fim_de_noite(jogo)

        return "continuar"

    
    def _acao_fechar_porta(self, ui, custos):
        custo = custos["motor"]
        if self.apagao > 0 or self.energia <= custo:
            ui.exibir("Sem energia! O botão faz um clique morto.")
        elif self.porta_fechada:
            ui.exibir("A porta já está fechada.")
        else:
            self.porta_fechada = True
            self.energia -= custo
            ui.exibir(f"A pesada porta de metal desce com um estrondo. (-{custo}% Energia)")
            if self.alberto_troll:
                ui.exibir("\n Como você é tão tolo? Hahahaha")
                self.erro_camera = True
                self.erro_deteccao = True
                self.alberto_troll = False

    def _acao_abrir_porta(self, ui, custos):
        custo = custos["motor"]
        if self.apagao > 0 or self.energia <= custo:
            ui.exibir("Sem energia! A porta não responde.")
        elif not self.porta_fechada:
            ui.exibir("A porta já está aberta.")
        else:
            self.porta_fechada = False
            self.energia -= custo
            ui.exibir(f"A porta de metal se ergue lentamente. (-{custo}% Energia)")

    def _acao_iluminar_tubulacao(self, ui, custos):
        custo = custos["info_pesado"]
        if self.apagao > 0 or self.energia <= custo:
            ui.exibir("Sem força nas luzes.")
        elif self._sistema_sobrecarregado():
            ui.exibir(f"{DOS_VERMELHO} [SISTEMA SOBRECARREGADO]: Muitas requisições simultâneas. Hardware travado{RESET}")
            self.energia -= custo
        else:
            self._gastar_sistema(custo)
            ui.exibir(f"Você liga o projetor nos dutos (-{custo}% Energia)")
            if self.jon_pos >= JON_POS_SAINDO_DUTO:
                self.jon_pos = 0
                ui.exibir("Jon recua apressado pela tubulação")
            if self.caroline_caminho == "tubulacao" and self.caroline_pos >= CAROLINE_POS_DUTO_FUGA:
                self.caroline_pos = 0
                self.caroline_caminho = random.choice(["porta", "tubulacao"])
                ui.exibir("A Caroline fugiu do duto")

    def _acao_olhar_vidro(self, ui):
        if self.indio_janela:
            ui.exibir(f"{DOS_BRANCO}{ARTE_INDIO}{RESET}")
            ui.pausar(2)
            ui.exibir(
                "Você não enxerga nada, até que 2 olhos te encaram pela janela, "
                "a figura do indio jones faz você perder a cabeça"
            )
            falha = random.choice(["camera", "relogio", "deteccao"])
            if falha == "camera":
                self.erro_camera = True
            elif falha == "relogio":
                self.erro_relogio = True
            elif falha == "deteccao":
                self.erro_deteccao = True
            if self.turno < 20:
                self.indio_janela = False
            return

        rick_na_porta = self._rick_na_porta()
        carol_na_porta = self._caroline_na_porta()

        if rick_na_porta and carol_na_porta:
            ui.exibir(
                "Seu corpo treme. Você vê a carcaça maciça de Rick, o mosqueteiro, "
                "e a carcaça de coelho rosa retorcido de Caroline parados lado a lado "
                "no corredor, olhando diretamente para a câmera. O vazio nos olhos "
                "deles é aterrorizante."
            )
        elif rick_na_porta:
            ui.exibir(
                " Você olha pelo vidro e vê a silhueta gigantesca do Rick, o "
                "mosqueteiro, parado nas sombras. Os olhos de plástico sem vida "
                "dele estão focados em você."
            )
        elif carol_na_porta:
            ui.exibir(
                " Através da sujeira do vidro, você enxerga a carcaça do coelho "
                "rosa tentando se esconder nas sombras. Ela está encostada na "
                "parede do corredor"
            )
        else:
            ui.exibir(
                "Você limpa o embaçado do vidro e força a vista para o corredor "
                "escuro. Consegue distinguir as portas fechadas das outras salas, "
                "os cartazes rasgados nas paredes e o chão de linóleo imundo "
                "refletindo a pouca luz que resta. Nenhum movimento... Além das "
                "sombras, há apenas o seu reflexo devolvendo o olhar."
            )

    def _acao_ligar_gerador(self, ui, turno):
        if self.apagao > 0:
            ui.exibir("Tarde demais, o sistema principal já foi totalmente desligado")
            return False, turno
        if self.gerador_reserva_usado:
            ui.exibir("O combustivel do gerador reserva já foi queimado, ele só pode ser usado uma vez")
            return False, turno

        ui.exibir(
            f"\n{DOS_VERDE} Você aperta o botão do gerador reserva, ele cospe uma "
            f"fumaça preta, sistemas basicos operando sem custo de energia.{RESET}"
        )
        self.gerador_reserva_usado = True
        self.turnos_gerador_ativo = GERADOR_DURACAO_TURNOS
        self.alberto_troll = False
        return True, turno + 1

    def _acao_consertar(self, ui, acao_norm, custos):
        sistema = acao_norm.replace("consertar ", "", 1).strip()
        custo = custos["info_leve"]

        if self.apagao > 0:
            ui.exibir("Não há energia.")
            return
        if sistema not in SISTEMAS_VALIDOS:
            ui.exibir("Sistema não reconhecido.")
            return
        if self.energia <= custo:
            ui.exibir("Energia insuficiente para acessar o painel de manutenção.")
            return

        self.energia -= custo
        if sistema == "camera":
            self.erro_camera = False
            ui.exibir(f"Câmeras online. (-{custo}% Energia)")
        elif sistema == "relogio":
            self.erro_relogio = False
            ui.exibir(f"Relógio sincronizado. (-{custo}% Energia)")
        elif sistema == "deteccao":
            self.erro_deteccao = False
            ui.exibir(f"Sensores calibrados. (-{custo}% Energia)")

    def _acao_ouvir(self, ui, custos):
        custo = custos["info_leve"]
        if self.apagao > 0:
            ui.exibir("No apagão, você ouve sua própria respiração...")
        elif self.erro_deteccao:
            ui.exibir(
                f"{DOS_VERMELHO}⚠ ⚠ ⚠ O alarme estridente de falha nos sensores "
                f"ecoa na sala. Você não consegue ouvir nada além disso ⚠ ⚠ ⚠{RESET}"
            )
        elif self.energia <= custo:
            ui.exibir("Sistema de áudio offline (Bateria fraca).")
        elif self._sistema_sobrecarregado():
            ui.exibir(f"{DOS_VERMELHO}⚠ [SISTEMA SOBRECARREGADO]: Placa de áudio em curto. Passe o turno para resfriar!{RESET}")
            self.energia -= custo
        else:
            self._gastar_sistema(custo)
            ui.exibir(f"(-{custo}% Energia)")
            ouviu = False
            if self._rick_na_porta() or self._caroline_na_porta():
                ui.exibir("@@PASSO@@ Passos metálicos pesados são ouvidos do corredor")
                ouviu = True
            if self.jon_pos >= JON_POS_SAINDO_DUTO or (
                self.caroline_caminho == "tubulacao" and self.caroline_pos >= CAROLINE_POS_PORTA
            ):
                ui.exibir("@@PASSO@@ Você escuta arranhões e batidas vindo da tubulação")
                ouviu = True
            if not ouviu:
                ui.exibir("Apenas o zumbido dos fios elétricos e da lâmpada quase apagada.")

    def _acao_cameras(self, ui, custos):
        custo = custos["info_leve"]
        if self.apagao > 0 or self.erro_camera:
            ui.exibir("⊠ [SINAL PERDIDO]")
        elif self.energia <= custo:
            ui.exibir("Câmeras offline (Bateria fraca).")
        elif self._sistema_sobrecarregado():
            ui.exibir(f"{DOS_VERMELHO}⚠ [SISTEMA SOBRECARREGADO]: Monitor superaquecido. A tela exibe apenas estática!{RESET}")
            self.energia -= custo
        else:
            self._gastar_sistema(custo)
            ui.exibir(f"(-{custo}% Energia)")

            chance_bug_visual = self.caroline_pos * 10
            if random.randint(1, 100) <= chance_bug_visual:
                ui.exibir("⊠ [SINAL COM INTERFERÊNCIA] Imagens distorcidas...")
                ui.exibir(f"Rick: Setor {random.randint(0, 4)}/4 (???)")
                ui.exibir(f"Jon: Setor {random.randint(0, 5)}/5 (???)")
            else:
                ui.exibir(f"\n--- FEED DAS CÂMERAS ---\nRick: Setor {self.rick_pos}/4")
                ui.exibir(
                    f"Jon: Setor {self.jon_pos}/5"
                    if self.jon_pos < JON_POS_VISIVEL_CAMERA
                    else "Jon: [não é visivel nas cameras]"
                )
            ui.exibir("------------------------")

            if random.randint(1, 100) <= CHANCE_ANOMALIA_CAMERA:
                ui.exibir(
                    f"\n{DOS_VERMELHO}⊠ [ANOMALIA DETECTADA]: O feed pisca. Em uma "
                    f"das câmeras escuras, o rosto quebrado de Caroline encara "
                    f"diretamente a lente... e ela está... sorrindo?{RESET}"
                )

    def _acao_ver_tubulacao(self, ui, custos):
        custo = custos["info_leve"]
        if self.apagao > 0 or self.erro_deteccao:
            ui.exibir("◯ [SENSORES OFFLINE]")
        elif self.energia <= custo:
            ui.exibir("Sensores offline (Bateria fraca).")
        elif self._sistema_sobrecarregado():
            ui.exibir(f"{DOS_VERMELHO}⚠ [SISTEMA SOBRECARREGADO]: Painel de detecção travado!{RESET}")
            self.energia -= custo
        else:
            self._gastar_sistema(custo)
            ui.exibir(f"(-{custo}% Energia)")
            if self.jon_pos >= JON_POS_VISIVEL_CAMERA or (
                self.caroline_caminho == "tubulacao" and self.caroline_pos >= CAROLINE_POS_DUTO_VISIVEL
            ):
                ui.exibir("⭙ Sensor fica vermelho, há algo nos dutos ⭙")
            else:
                ui.exibir("◉ Sensor não detecta nada")

    
    def _chance_interferencia(self, ui):
        if random.random() > CHANCE_INTERFERENCIA_MOV:
            return
        quem = random.choice(["rick", "jon", "caroline"])
        if quem == "rick":
            self.rick_pos += 1
        elif quem == "jon":
            self.jon_pos += 1
        else:
            self.caroline_pos += 1
        ui.exibir(f"\n@@PASSO@@{DOS_VERMELHO}Você ouve um ruído metálico se aproximando enquanto mexe no sistema.{RESET}")

    def _checar_recuo_na_porta(self, ui):
        if self.rick_pos >= RICK_POS_ATAQUE:
            self.rick_pos = 0
            ui.exibir(f"\n{DOS_AMARELO} ALGO SOCA A PORTA COM VIOLÊNCIA E RECUA{RESET}")
        if self.caroline_caminho == "porta" and self.caroline_pos >= CAROLINE_POS_ATAQUE_PORTA:
            self.caroline_pos = 0
            self.caroline_caminho = random.choice(["porta", "tubulacao"])
            ui.exibir(f"\n{DOS_AMARELO} Um estrondo na porta. Ela recuou...{RESET}")

    def _evento_ambiente_aleatorio(self, ui):
        chance_evento = random.randint(1, 100)
        if chance_evento <= 3:
            ui.exibir(f"\n{DOS_AMARELO} Toc.. Toc.. Você escuta batidas fracas na janela, você não sabe se há algo ali, o vidro está muito sujo.{RESET}")
        elif chance_evento <= 7:
            ui.exibir(f"\n{DOS_AMARELO} Você escuta ruidos vindo da ventilação... Parece que algo está arranhando o aluminio. {RESET}")
        elif chance_evento <= 9:
            ui.exibir(f"\n{DOS_VERMELHO} 'Rogerio'... Você escuta algo chamar seu nome vindo do fundo do corredor.{RESET}")
        elif chance_evento <= 10:
            ui.exibir(f"\n{DOS_VERMELHO} Pelo canto do seu olho, você jura ter visto algo acenando da janela, você não sabe se é algo real ou não.{RESET}")
        elif chance_evento <= 12:
            ui.exibir(f"\n{DOS_VERMELHO} Você jura ter visto algo na ventilação... Será que é coisa da sua cabeça?{RESET}")

    def _resolver_fim_de_turno(self, ui, jogo, god_mode):
        """Processa tudo que acontece quando um turno é consumido.
        Retorna 'morte' se o jogador foi pego, ou None para continuar."""
        self.usos_sistema_turno = 0

        if self.turnos_gerador_ativo > 0:
            self.turnos_gerador_ativo -= 1
            if self.turnos_gerador_ativo == 0:
                ui.exibir(
                    f"\n{DOS_AMARELO} O gerador reserva para de soltar fumaça, e "
                    f"começa a dar gargalos, e depois deliga. A energia volta a "
                    f"ser drenada normalmente{RESET}"
                )

        if self.turno == TURNO_INICIO_CALOR:
            ui.exibir(f"\n{DOS_AMARELO} [SISTEMA] O antigo gerador está superaquecendo, cada acão custará mais energia a partir de agora.{RESET}")
        elif self.turno == TURNO_CRITICO:
            ui.exibir(f"\n {DOS_AMARELO} [SISTEMA] [AVISO CRITICO!!!] O gerador superaqueceu! Geradores reservas ligados, dreno de energia aumentou!{RESET}")

        if self.porta_fechada and self.energia > 0:
            self.energia -= CUSTO_PORTA
            ui.exibir(f" A pesada porta de metal consome energia contínua... (-{CUSTO_PORTA}% Energia)")

        if self.energia <= 0 and self.apagao == 0 and not god_mode:
            ui.exibir("\n [ ENERGIA ESGOTADA ] Tudo fica escuro. A porta abre sozinha...")
            self.porta_fechada = False
            self.apagao = 1
            ui.pausar(2)

        if self.porta_fechada:
            if self.rick_pos == RICK_POS_ATAQUE:
                self.rick_pos = 0
                ui.exibir("\n Você escuta batidas na porta, e passos para fora do corredor logo depois.")
            if self._caroline_na_porta():
                self.caroline_pos = 0
                self.caroline_caminho = random.choice(["porta", "tubulacao"])
                ui.exibir("\n Você escuta um estrondo na porta, e depois passos apressados para a sala de jantar.")

        rick_ataque = (self.rick_pos >= RICK_POS_ATAQUE) or (
            self.rick_pos == RICK_POS_PORTA and random.random() < 0.3
        )
        carol_porta_ataque = self.caroline_caminho == "porta" and (
            (self.caroline_pos >= CAROLINE_POS_ATAQUE_PORTA)
            or (self.caroline_pos == CAROLINE_POS_PORTA and random.random() < 0.3)
        )
        carol_duto_ataque = self.caroline_caminho == "tubulacao" and self.caroline_pos >= CAROLINE_POS_ATAQUE_DUTO
        jon_ataque = self.jon_pos >= JON_POS_ATAQUE

        invasao = (
            (rick_ataque and not self.porta_fechada)
            or (carol_porta_ataque and not self.porta_fechada)
            or jon_ataque
            or carol_duto_ataque
        )
        if invasao:
            if god_mode:
                ui.exibir(f"\n{DOS_AMARELO}[GOD MODE] Um animatrônico entra na sala... mas você o encara. Ele pede desculpas e sai de fininho.{RESET}")
                self.rick_pos = 0
                self.caroline_pos = 0
                self.jon_pos = 0
            else:
                ui.exibir("\n Um animatronico conseguiu entrar.")
                ui.exibir("@@JUMPSCARE@@")
                ui.pausar(2)
                return "morte"

        self._mover_animatronicos(ui)
        self._atualizar_eventos_passivos(jogo)

        ui.exibir("\n[A atualizar sistema...]")
        ui.pausar(1.2)
        return None

    def _mover_animatronicos(self, ui):
        if self.rick_pos == RICK_POS_PORTA and not self.porta_fechada and random.random() < CHANCE_RICK_RECUAR:
            self.rick_pos = 1
            ui.exibir(" Ouve passos pesados a se afastar da porta")
        else:
            furia_atual = self.furia + (self.turno // 6)
            if self.rick_pos < RICK_POS_PORTA:
                self.rick_pos = min(
                    RICK_POS_PORTA,
                    self.rick_pos + random.choice([0, 1, 1, 2]) * furia_atual,
                )
            elif self.rick_pos == RICK_POS_PORTA:
                self.rick_pos += 1 if random.random() < CHANCE_RICK_AVANCA_DE_3 else 0

        if self.erro_deteccao:
            self.jon_pos = min(JON_POS_ATAQUE, self.jon_pos + random.choice([1, 2, 3]))
        else:
            self.jon_pos = min(JON_POS_ATAQUE, self.jon_pos + random.choice([0, 1, 2]))

        self.caroline_pos = min(CAROLINE_POS_ATAQUE_PORTA, self.caroline_pos + random.choice([0, 1, 2, 3]))

    def _atualizar_eventos_passivos(self, jogo):
        if self.turno >= TURNO_INICIO_CALOR and (self.turno >= 20 or random.randint(1, 100) > 100 - CHANCE_INDIO_JANELA):
            self.indio_janela = True
        else:
            self.indio_janela = False

        if random.randint(1, 100) > 100 - CHANCE_ALBERTO_TROLL and not getattr(jogo, "alberto_desativado", False):
            self.alberto_troll = True

    def _checar_fim_de_noite(self, jogo):
        if self.turno < TURNO_FINAL:
            return "continuar"

        ui = self.ui
        ui.limpar()
        
        
        ui.animar("Você se sente aliviado quando a luz do sol começa a invadir a janela do restaurante, e o relógio marca pontualmente '06:00' ", 0.03, DOS_BRANCO, jogo)
        ui.pausar(2)
        ui.animar("O sol começa a nascer. A energia retorna aos poucos.", 0.03, DOS_BRANCO, jogo)
        ui.animar("A porta da sala destranca.", 0.03, DOS_BRANCO, jogo)
        

        jogo.mapa["sala de jantar"]["descrição"] = "A luz da manhã invade as janelas sujas."
        jogo.mapa["hall de entrada"]["descrição"] = "O hall está iluminado."
        jogo.mapa["balcão"]["descrição"] = "A claridade revela o mofo nos doces."
        jogo.mapa["entrada"]["descrição"] = "As luzes não piscam mais."
        jogo.noite_vencida = True

        if getattr(jogo, "fios_cortados_inventario", False):
            ui.pausar(2)
            try:
                from data import ARTE_RADAR
                radar = ARTE_RADAR
            except (ImportError, AttributeError):
                radar = "   .---.\n /   |   \\\n|----O----|\n \\   |   /\n   '---'"

            ui.animar("\nVocê saca o dispositivo.", 0.03, DOS_AMARELO, jogo)
            ui.exibir(f"{DOS_VERDE}{radar}{RESET}")
            ui.pausar(1)
            ui.animar("[DISPOSITIVO]: PRESENÇA ULTERIOR DETECTADA.", 0.03, DOS_VERDE, jogo)
            ui.animar("Ela ainda está aqui...\n", 0.04, DOS_AMARELO, jogo)
            ui.pausar(3)

        return "vitoria_seguranca"