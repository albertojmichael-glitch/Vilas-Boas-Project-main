import random
import time
import logging
from ui import DOS_VERDE, DOS_BRANCO, DOS_AMARELO, DOS_VERMELHO, RESET
from villas_boas.engine.minigames.base import BaseMinigame

logger = logging.getLogger(__name__)

def pausar(segundos=0):
    try:
        time.sleep(segundos)
    except (OSError, ValueError) as e:
        logger.debug(f"Função de pausa interrompida: {e}")

CAVEIRA_ASCII = r'''
                     .ed"""" """$$$$be.
                   -"           ^""**$$$e.
                 ."                   '$$$c
                /                      "4$$b
               d  3                     $$$$
               $  * .$$$$$$
              .$  ^c           $$$$$e$$$$$$$$.
              d$L  4.         4$$$$$$$$$$$$$$b
              $$$$b ^ceeeee.  4$$ECL.F*$$$$$$$
  e$""=.      $$$$P d$$$$F $ $$$$$$$$$- $$$$$$
 z$$b. ^c     3$$$F "$$$$b   $"$$$$$$$  $$$$*"      .=""$c
4$$$$L   \     $$P"  "$$b   .$ $$$$$...e$$        .=  e$$$.
^*$$$$$c  %..   *c    ..    $$ 3$$$$$$$$$$eF     zP  d$$$$$
  "**$$$ec   "\   %ce""    $$$  $$$$$$$$$$* .r" =$$$$P""
        "*$b.  "c  *$e.    *** d$$$$$"L$$    .d"  e$$***"
          ^*$$c ^$c $$$      4J$$$$$% $$$ .e*".eeP"
             "$$$$$$"'$=e....$*$$**$cz$$" "..d$*"
               "*$$$  *=%4.$ L L$ P3$$$F $$$P"
                  "$   "%*ebJLzb$e$$$$$b $P"
                    %..      4$$$$$$$$$$ "
                     $$$e   z$$$$$$$$$$%
                      "*$c  "$$$$$$$P"
                       ."""*$$$$$$$$bc
                    .-"    .e$"     "*$c  ^*b.
          .=*""""    .e$*"          "*bc  "*$e..
        .$"        .z*"               ^*$e.   "*****e.
        $$ee$c   .d"                     "*$.        3.
        ^*$E")$..$"                         * .ee==d%
           $.d$$$* * J$$$e*
            """""                             "$$$"    
'''



#minigame minotauro


BATERIA_INICIAL = 18
CHANCE_PADRAO_SPRINT = 60         
DISTANCIA_PERIGO = 1              


GRID_MIN_X = -1
GRID_MAX_X = 1
GRID_MIN_Y = 0
GRID_MAX_Y = 3


POS_FUSIVEL_X = 0
POS_FUSIVEL_Y = 3
POS_SAIDA_X = 0
POS_SAIDA_Y = 0


# Variações de texto para não repetir sempre a mesma frase nas mesmas situações
MSGS_PERIGO_ESQUERDA = [
    "⚠ Você sente um ar pesado em sua esquerda.",
    "⚠ Algo se arrasta lentamente à sua esquerda.",
    "⚠ Um cheiro de metal queimado vem da esquerda.",
]
MSGS_PERIGO_DIREITA = [
    "⚠ Você enxerga um vulto a sua direita.",
    "⚠ Um brilho metálico reflete rapidamente à direita.",
    "⚠ Você ouve unhas riscando a parede à direita.",
]
MSGS_PERIGO_FRENTE = [
    "⚠ Uma mancha negra bloqueia sua frente.",
    "⚠ O ar fica mais frio à sua frente.",
    "⚠ Uma respiração pesada ecoa logo à frente.",
]
MSGS_PERIGO_TRAS = [
    "⚠ Passos pesados são ouvidos atrás de você.",
    "⚠ Algo pesado se arrasta bem atrás de você.",
    "⚠ Uma respiração fria bate na sua nuca.",
]
MSGS_ESTATICA = [
    "[!] O radar está falhando... ruídos de estática por todo lado.",
    "[!] Estática. O radar pisca e não mostra nada por um instante.",
    "[!] Um zumbido elétrico cobre qualquer outro som ao redor.",
]
MSGS_COLISAO_JOGADOR = [
    "⚠ Você dá um passo e bate o rosto em uma carcaça de metal fria. Você recua pelo susto.⚠",
    "⚠ Sua mão encontra algo úmido e frio na escuridão. Você recua na hora.⚠",
    "⚠ Você tropeça em algo pesado no chão e quase cai. Recua assustado.⚠",
]
MSGS_COLISAO_MONSTRO = [
    "⚠ VOCÊ TROMBA COM ALGO METÁLICO NO ESCURO. ELE ESTÁ NA SUA FRENTE⚠",
    "⚠ ALGO ENORME PASSA BEM AO SEU LADO NO ESCURO⚠",
    "⚠ VOCÊ SENTE O CHÃO TREMER PERTO DEMAIS DE VOCÊ⚠",
]

def _sortear(lista):
    return random.choice(lista)

def desenhar_radar_ascii(px, py, mx, my, fios_cortados):
    """Gera o grid 3x4 do radar com as posições dinâmicas."""
    
    distancia = abs(px - mx) + abs(py - my)
    monstro_visivel = (distancia <= 1)
    
    borda = DOS_VERDE
    reset = RESET
    
    linhas = []
    linhas.append(f"{borda}  +=======+=======+=======+{reset}")
    
    
    for y in range(GRID_MAX_Y, GRID_MIN_Y - 1, -1):
        miolo1 = f"{borda}  /{reset}"
        miolo2 = f"{borda}  /{reset}"
        
        
        for x in range(GRID_MIN_X, GRID_MAX_X + 1):
            char = " "
            
            
            if x == px and y == py:
                char = f"{DOS_BRANCO}●{reset}"         
            elif monstro_visivel and x == mx and y == my:
                char = f"{DOS_VERMELHO}●{reset}"      
            elif x == POS_FUSIVEL_X and y == POS_FUSIVEL_Y and not fios_cortados:
                char = f"{DOS_AMARELO}F{reset}"       
            elif x == POS_SAIDA_X and y == POS_SAIDA_Y:
                char = f"{DOS_VERDE}S{reset}"         
                
            miolo1 += f"   {char}   {borda}/{reset}"
            miolo2 += f"       {borda}/{reset}"
            
        linhas.append(miolo1)
        linhas.append(miolo2)
        linhas.append(f"{borda}  +=======+=======+=======+{reset}")
        
    return "\n".join(linhas)


class MinigameMinotauro(BaseMinigame):
    def __init__(self, jogo):
        self.px, self.py = POS_SAIDA_X, POS_SAIDA_Y 
        self.mx, self.my = random.choice([GRID_MIN_X, 0, GRID_MAX_X]), random.choice([2, 3]) 
        self.tesoura_chao = True
        self.fios_cortados = False
        
        
        chance_raw = getattr(jogo, 'chance_sprint_minotauro', CHANCE_PADRAO_SPRINT)
        self.chance_sprint = chance_raw / 100.0
        
        self.bateria = 9999 if getattr(jogo, 'god_mode', False) else BATERIA_INICIAL

        
        self.ui = getattr(jogo, 'ui_handler', getattr(jogo, 'ui', None))
        if not self.ui or not hasattr(self.ui, 'exibir'):
            class DummyUI:
                def exibir(self, *args, **kwargs): pass
                def animar(self, *args, **kwargs): pass
                def pausar(self, *args, **kwargs): pass
            self.ui = DummyUI()
        
        self.ui.exibir("\n" + "="*50)
        self.ui.exibir("Você entra na Sala de Energia... e a pesada porta de metal bate atrás de você.")
        pausar(2)
        self.ui.exibir("Você escuta uma respiração pesada.")
        self.ui.exibir("Ele está aqui.")
        self.ui.pausar(2)

    def imprimir_status(self):
        
        self.ui.limpar()
        
        
        radar_visual = desenhar_radar_ascii(self.px, self.py, self.mx, self.my, self.fios_cortados)
        self.ui.exibir(radar_visual)
        
        
        self.ui.exibir("\n" + "-"*30)
        texto_bat = "∞" if self.bateria > 100 else str(self.bateria)
        self.ui.exibir(f" Bateria da Lanterna: {texto_bat} turnos restantes")
        
        distancia = abs(self.px - self.mx) + abs(self.py - self.my)
        
        if distancia > DISTANCIA_PERIGO: 
            self.ui.animar("[v] O radar não detecta nada próximo. Silêncio.")
        elif distancia == DISTANCIA_PERIGO:
            if random.random() < 0.2:
                self.ui.exibir(f"{DOS_VERMELHO}{_sortear(MSGS_ESTATICA)}{RESET}")
            else:
                if self.mx < self.px: self.ui.animar(f"{DOS_VERMELHO}{_sortear(MSGS_PERIGO_ESQUERDA)}{RESET}")
                elif self.mx > self.px: self.ui.animar(f"{DOS_VERMELHO}{_sortear(MSGS_PERIGO_DIREITA)}{RESET}")
                elif self.my > self.py: self.ui.animar(f"{DOS_VERMELHO}{_sortear(MSGS_PERIGO_FRENTE)}{RESET}")
                elif self.my < self.py: self.ui.animar(f"{DOS_VERMELHO}{_sortear(MSGS_PERIGO_TRAS)}{RESET}")

        
        opcoes = "ir frente | ir trás | ir esquerda | ir direita | esperar"
        
        if self.px == POS_FUSIVEL_X and self.py == POS_FUSIVEL_Y and not self.fios_cortados:
            self.ui.exibir(f"\n{DOS_AMARELO} ↯ Você está de frente para a caixa de fusíveis!{RESET}")
            if self.tesoura_chao:
                self.ui.animar(" ✂ Há uma tesoura caída no chão.")
                opcoes += " | pegar tesoura"
            opcoes += " | cortar fios"
            
        if self.fios_cortados:
            self.ui.exibir(f"\n{DOS_VERMELHO} ↯ FIOS CORTADOS! A SALA ESTÁ DESMORONANDO! FUJA PARA A SAÍDA (S)!{RESET}")
            if self.px == POS_SAIDA_X and self.py == POS_SAIDA_Y:
                self.ui.exibir(f"{DOS_VERDE} ⍍ Você chegou na porta de entrada!{RESET}")
                opcoes += " | sair"
                
        self.ui.exibir(f"\n[{opcoes}]")

    def mover_minotauro(self):
        """Move o minotauro. Em modo sprint, persegue o eixo em que ainda há distância,
        evitando desperdiçar o turno de perseguição escolhendo um eixo já alinhado."""
        if random.random() < self.chance_sprint:
            dif_x = self.px - self.mx
            dif_y = self.py - self.my

            pode_mover_x = dif_x != 0
            pode_mover_y = dif_y != 0

            if pode_mover_x and pode_mover_y:
                mover_em_x = random.random() < 0.5
            elif pode_mover_x:
                mover_em_x = True
            elif pode_mover_y:
                mover_em_x = False
            else:
                
                direcao = random.choice([(-1, 0), (1, 0), (0, -1), (0, 1)])
                self.mx += direcao[0]
                self.my += direcao[1]
                mover_em_x = None

            if mover_em_x is True:
                self.mx += 1 if dif_x > 0 else -1
            elif mover_em_x is False:
                self.my += 1 if dif_y > 0 else -1
        else:
            direcao = random.choice([(-1, 0), (1, 0), (0, -1), (0, 1)])
            self.mx += direcao[0]
            self.my += direcao[1]
            
        self.mx = max(GRID_MIN_X, min(GRID_MAX_X, self.mx)) 
        self.my = max(GRID_MIN_Y, min(GRID_MAX_Y, self.my))

    def _resolver_colisao(self, jogo, dist_ui, px_old, py_old, mx_old, my_old, quem_moveu="jogador"):
        """Centraliza a lógica de colisão para evitar duplicação de código"""
        if self.px == self.mx and self.py == self.my:
            if dist_ui > DISTANCIA_PERIGO:
                
                if quem_moveu == "jogador":
                    self.ui.exibir(f"\n{DOS_VERMELHO}{_sortear(MSGS_COLISAO_JOGADOR)}{RESET}")
                    self.px, self.py = px_old, py_old
                else:
                    self.ui.exibir(f"\n{DOS_VERMELHO}{_sortear(MSGS_COLISAO_MONSTRO)}{RESET}")
                    self.mx, self.my = mx_old, my_old
                self.ui.pausar(2)
                return "continuar"
            else:
                
                if getattr(jogo, 'god_mode', False):
                    self.ui.exibir(f"\n{DOS_AMARELO}[GOD MODE] O Minotauro tenta te atacar, mas é repelido por um escudo de energia! Ele desiste e foge.{RESET}")
                    self.ui.pausar(1.5)
                    return "vitoria_minotauro"
                else:
                    if quem_moveu == "jogador":
                        self.ui.exibir("\n Você andou direto para as mãos do monstro no escuro...")
                    else:
                        self.ui.exibir(f"\n{DOS_VERMELHO} ☠ O Minotauro te encontrou no escuro. Mãos frias de metal te rasgam por inteiro ☠{RESET}")
                    self.ui.exibir("@@JUMPSCARE@@")
                    self.ui.pausar(1.5)
                    if quem_moveu == "jogador":
                        self.ui.exibir("\n No vazio, você morre sozinho, sem poder salvar ninguém. ")
                    return "morte"
        return None

    def _processar_investigacao(self, acao, jogo):
        """Trata o comando de examinar o celular quebrado. Retorna string de resultado ou None se a ação não for essa."""
        if acao not in ["celular quebrado", "ver celular quebrado", "olhar celular quebrado", "examinar celular quebrado", "investigar celular quebrado", "celular"]:
            return None

        if self.px == POS_FUSIVEL_X and self.py == POS_FUSIVEL_Y:
            desc = jogo.mapa.get("sala de energia", {}).get("inspecionaveis", {}).get("celular quebrado", "Parece ser dela...")
            self.ui.exibir(f"\n{DOS_AMARELO}🔎 {desc}{RESET}")
        else:
            self.ui.exibir("O celular quebrado está no fundo da sala (na parede central).")
        return "continuar"

    def _processar_god_mode_ataque(self, acao, jogo):
        """Trata o ataque cheat em god mode. Retorna string de resultado ou None se não aplicável."""
        if acao in ["atacar", "bater", "chutar", "lutar"] and getattr(jogo, 'god_mode', False):
            self.ui.exibir(f"{DOS_AMARELO}[GOD MODE] Você corre na direção do Minotauro e dá uma voadora com os dois pés no peito dele!{RESET}")
            self.ui.exibir(f"{DOS_AMARELO}A fera despenca para trás, choraminga em som de estática e foge rompendo as paredes.{RESET}")
            self.ui.pausar(1.5)
            return "vitoria_minotauro"
        return None

    def _processar_movimento_e_acoes(self, acao, jogo):
        """Aplica a ação escolhida (movimento, esperar, itens, cortar fios, sair).
        Retorna True se o turno foi consumido (o minotauro deve se mover em seguida)."""
        ui = self.ui
        turno_gasto = False

        if acao == "ir esquerda":
            if self.px > GRID_MIN_X: self.px -= 1
            else: ui.exibir("Você bate a cara na parede...")
            turno_gasto = True 
        elif acao == "ir direita":
            if self.px < GRID_MAX_X: self.px += 1
            else: ui.exibir("Você bate a cara na parede...")
            turno_gasto = True
        elif acao == "ir frente":
            if self.py < GRID_MAX_Y: self.py += 1
            else: ui.exibir("Você bateu na parede do fundo...")
            turno_gasto = True
        elif acao in ["ir trás", "ir tras", "ir atrás", "ir atras"]:
            if self.py > GRID_MIN_Y: self.py -= 1
            else: ui.exibir("Você bate as costas na porta de metal. Ela não abre...")
            turno_gasto = True
        elif acao == "esperar": 
            ui.exibir("Você fica imóvel aguardando...")
            turno_gasto = True
        elif acao == "pegar tesoura":
            if self.px == POS_FUSIVEL_X and self.py == POS_FUSIVEL_Y and self.tesoura_chao:
                jogo.inventario.append("tesoura")
                self.tesoura_chao = False
                ui.exibir(" ✂ Você derruba a tesoura sem querer, fazendo um barulho, mas guarda na sua bolsa")
                
                turno_gasto = True
            else: 
                ui.exibir("Não tem tesoura aqui.")
        elif acao == "cortar fios":
            if self.px == POS_FUSIVEL_X and self.py == POS_FUSIVEL_Y and not self.fios_cortados:
                if "tesoura" in jogo.inventario:
                    ui.exibir(f"\n{DOS_VERMELHO}Você corta os fios principais. Faíscas voam no seu rosto, mas não causam queimaduras.{RESET}")
                    ui.exibir(f"{DOS_VERMELHO} ✂ Sua tesoura quebra com a força do choque elétrico{RESET}")
                    ui.exibir(f"{DOS_VERMELHO}Ele sabe onde você está.{RESET}")
                    ui.exibir(f"{DOS_VERMELHO}CORRA DE VOLTA PARA A PORTA{RESET}")
                    
                    jogo.inventario.remove("tesoura")
                    jogo.inventario.append("tesoura quebrada")
                    jogo.inventario.append("fios cortados")
                    ui.exibir(f"{DOS_AMARELO}Você guarda os 'fios cortados' na mochila.{RESET}")
                    
                    self.fios_cortados = True
                    jogo.fios_cortados_inventario = True
                    turno_gasto = True
                else: 
                    ui.exibir("Você precisa de uma tesoura inteira para cortar os fios.")
                    turno_gasto = True
            else: 
                ui.exibir("Não há mais fios aqui.")
                turno_gasto = True
        elif acao == "sair":
            if self.px == POS_SAIDA_X and self.py == POS_SAIDA_Y:
                if self.fios_cortados and "fios cortados" in jogo.inventario:
                    ui.exibir(f"\n{DOS_VERDE}Você se joga contra a maçaneta, abre a porta e a tranca com toda a força! Você sobreviveu!{RESET}")
                    ui.pausar(1.5)
                    return "vitoria_minotauro"
                else:
                    ui.exibir("Você está na porta de saída. Você precisa cortar e pegar os fios elétricos.")
                    turno_gasto = True
            else:
                ui.exibir("A porta de saída não fica aqui. Tente voltar para trás.")
        else: 
            ui.exibir(f"{DOS_AMARELO}Comando não reconhecido no escuro. Você gasta segundos tropeçando...{RESET}")
            turno_gasto = True 

        return turno_gasto

    def processar_turno(self, acao, jogo):
        ui = self.ui

        resultado = self._processar_investigacao(acao, jogo)
        if resultado:
            return resultado

        resultado = self._processar_god_mode_ataque(acao, jogo)
        if resultado:
            return resultado

        dist_ui = abs(self.px - self.mx) + abs(self.py - self.my)
        px_old, py_old = self.px, self.py
        mx_old, my_old = self.mx, self.my

        resultado_acao = self._processar_movimento_e_acoes(acao, jogo)
        
        if resultado_acao in ("vitoria_minotauro",):
            return resultado_acao

        turno_gasto = bool(resultado_acao)

        
        resultado_colisao = self._resolver_colisao(jogo, dist_ui, px_old, py_old, mx_old, my_old, quem_moveu="jogador")
        if resultado_colisao: return resultado_colisao

        if turno_gasto:
            if not getattr(jogo, 'god_mode', False):
                self.bateria -= 1
                if self.bateria <= 0:
                    ui.exibir("\n A sua lanterna apaga, você entra em desespero e bate na bateria fazendo barulho.")
                    ui.pausar(2)
                    ui.exibir("\n Você sente uma mão atravessando seu estômago por trás, não há nada a se fazer.")
                    return "morte"
                
            mx_old, my_old = self.mx, self.my
            self.mover_minotauro()
                
            
            resultado_colisao = self._resolver_colisao(jogo, dist_ui, px_old, py_old, mx_old, my_old, quem_moveu="monstro")
            if resultado_colisao: return resultado_colisao
                
        return "continuar"