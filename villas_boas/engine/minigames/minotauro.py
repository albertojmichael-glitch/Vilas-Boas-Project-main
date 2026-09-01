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
                    .-"    .$***$$$"""*e.
                 .-"    .e$"     "*$c  ^*b.
          .=*""""    .e$*"          "*bc  "*$e..
        .$"        .z*"               ^*$e.   "*****e.
        $$ee$c   .d"                     "*$.        3.
        ^*$E")$..$"                         * .ee==d%
           $.d$$$* * J$$$e*
            """""                             "$$$"    
'''



#minigame minotauro



class MinigameMinotauro(BaseMinigame):
    def __init__(self, jogo):
        self.px, self.py = 0, 0 
        self.mx, self.my = random.choice([-1, 0, 1]), random.choice([2, 3]) 
        self.tesoura_chao = True
        self.fios_cortados = False
        self.chance_sprint = getattr(jogo, 'chance_sprint_minotauro', 15)
        self.bateria = 9999 if getattr(jogo, 'god_mode', False) else 18

        self.ui = getattr(jogo, 'ui_handler', getattr(jogo, 'ui', None))
        
        
        if self.ui and hasattr(self.ui, 'exibir'):
            self.ui.exibir("\n" + "="*50)

        
            self.ui.exibir("Você entra na Sala de Energia... e a pesada porta de metal bate atrás de você.")
            pausar(2)
            self.ui.exibir("Você escuta uma respiração pesada.")
            self.ui.exibir("Ele está aqui.")
            jogo.ui_handler.pausar(2)

    def imprimir_status(self):
        if self.ui and hasattr(self.ui, 'exibir'):
            self.ui.exibir("\n" + "-"*30)
            texto_bat = "∞" if self.bateria > 100 else str(self.bateria)
            self.ui.exibir(f" Bateria da Lanterna: {texto_bat} turnos restantes")
            
            distancia = abs(self.px - self.mx) + abs(self.py - self.my)
            
            if distancia > 1: 
                self.ui.animar("[v] Você sente uma presença distante, talvez não haja perigo por enquanto.")
            elif distancia == 1:
                if random.random() < 0.2:
                    self.ui.exibir("[!] Os ecos, faiscas que caem do teto, e o barulho infernal de correntes eletricas te confundem... não dá pra saber de onde o som vem")
                else:
                    if self.mx < self.px: self.ui.animar("⚠ Você sente um ar pesado em sua esquerda.")
                    elif self.mx > self.px: self.ui.animar("⚠ Você enxerga um vulto a sua direita.")
                    elif self.my > self.py: self.ui.animar("⚠ Você não enxerga nada a sua frente, uma mancha negra cobre o fundo.")
                    elif self.my < self.py: self.ui.animar("⚠ Passos pesados são ouvidos atrás de você.")

            opcoes = "ir frente | ir trás | ir esquerda | ir direita | esperar"
            
            if self.px == 0 and self.py == 3 and not self.fios_cortados:
                self.ui.exibir(" ↯ Você encontrou a caixa de fusíveis na parede central!")
                if self.tesoura_chao:
                    self.ui.animar("✂ Há uma tesoura caída no chão.")
                    opcoes += " | pegar tesoura"
                opcoes += " | cortar fios"
                
            if self.fios_cortados:
                self.ui.exibir(f"{DOS_VERMELHO} ↯ OS FIOS ESTÃO CORTADOS! A SALA ESTÁ DESMORONANDO! FUJA PARA A SAÍDA!{RESET}")
                if self.px == 0 and self.py == 0:
                    self.ui.exibir(f"{DOS_VERDE}⍍ A porta de entrada está logo aqui! Você pode sair!{RESET}")
                    opcoes += " | sair"
                    
            self.ui.exibir(f"\n[{opcoes}]")

    def mover_minotauro(self):
        
        if random.random() < 0.60:
        
            if random.random() < 0.5:
                
                if self.px > self.mx: self.mx += 1
                elif self.px < self.mx: self.mx -= 1
            else:
                
                if self.py > self.my: self.my += 1
                elif self.py < self.my: self.my -= 1
        else:
            
            direcao = random.choice([(-1, 0), (1, 0), (0, -1), (0, 1)])
            self.mx += direcao[0]
            self.my += direcao[1]
            
        
        self.mx = max(-1, min(1, self.mx)) 
        self.my = max(0, min(3, self.my))

    def processar_turno(self, acao, jogo):
        ui = jogo.ui_handler

        if not ui:
            class DummyUI:
                def exibir(self, *args, **kwargs): pass
                def animar(self, *args, **kwargs): pass
                def pausar(self, *args, **kwargs): pass
            ui = DummyUI() 

        if acao in ["celular quebrado", "ver celular quebrado", "olhar celular quebrado", "examinar celular quebrado", "investigar celular quebrado", "celular"]:
            if self.px == 0 and self.py == 3: 
                desc = jogo.mapa.get("sala de energia", {}).get("inspecionaveis", {}).get("celular quebrado", "Parece ser dela...")
                ui.exibir(f"\n{DOS_AMARELO}🔎 {desc}{RESET}")
                return "continuar"
            else:
                ui.exibir("O celular quebrado está no fundo da sala (na parede central).")
                return "continuar"
        
        if acao in ["atacar", "bater", "chutar", "lutar"] and getattr(jogo, 'god_mode', False):
            ui.exibir(f"{DOS_AMARELO}[GOD MODE] Você corre na direção do Minotauro e dá uma voadora com os dois pés no peito dele!{RESET}")
            ui.exibir(f"{DOS_AMARELO}A fera despenca para trás, choraminga em som de estática e foge rompendo as paredes.{RESET}")
            ui.pausar(1.5)
            return "vitoria_minotauro"

        turno_gasto = False

        dist_ui = abs(self.px - self.mx) + abs(self.py - self.my)
        px_old, py_old = self.px, self.py
        
        if acao == "ir esquerda":
            if self.px > -1: self.px -= 1
            else: ui.exibir("Você bate a cara na parede...")
            turno_gasto = True 

        elif acao == "ir direita":
            if self.px < 1: self.px += 1
            else: ui.exibir("Você bate a cara na parede...")
            turno_gasto = True

        elif acao == "ir frente":
            if self.py < 3: self.py += 1
            else: ui.exibir("Você bateu na parede do fundo...")
            turno_gasto = True
            
        elif acao in ["ir trás", "ir tras", "ir atrás", "ir atras"]:
            if self.py > 0: self.py -= 1
            else: ui.exibir("Você bate as costas na porta de metal. Ela não abre...")
            turno_gasto = True

        elif acao == "esperar": 
            ui.exibir("Você fica imóvel aguardando...")
            turno_gasto = True

        elif acao == "pegar tesoura":
            if self.px == 0 and self.py == 3 and self.tesoura_chao:
                jogo.inventario.append("tesoura")
                self.tesoura_chao = False
                ui.exibir(" ✂ Você derruba a tesoura sem querer, fazendo um barulho, mas guarda na sua bolsa")
                if random.random() < 0.50:
                    self.mover_minotauro() 
                turno_gasto = True
            else: 
                ui.exibir("Não tem tesoura aqui.")

        elif acao == "cortar fios":
            if self.px == 0 and self.py == 3 and not self.fios_cortados:
                if "tesoura" in jogo.inventario:
                    ui.exibir(f"\\n{DOS_VERMELHO}Você corta os fios principais. Faíscas voam no seu rosto, mas não causam queimaduras.{RESET}")
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
            if self.px == 0 and self.py == 0:
                
                if self.fios_cortados and "fios cortados" in jogo.inventario:
                    ui.exibir(f"\\n{DOS_VERDE}Você se joga contra a maçaneta, abre a porta e a tranca com toda a força! Você sobreviveu!{RESET}")
                    ui.pausar(1.5)
                    return "vitoria_minotauro"
                else:
                    ui.exibir("Você está na porta de saída, Você precisa cortar e pegar os fios eletricos.")
                    turno_gasto = True
            else:
                ui.exibir("A porta de saída não fica aqui. Tente voltar para trás.")
        
        
        else: 
            ui.exibir(f"{DOS_AMARELO}Comando não reconhecido no escuro. Você gasta segundos tropeçando...{RESET}")
            turno_gasto = True 

        
        
        
        if self.px == self.mx and self.py == self.my:
            
            if dist_ui > 1:
                ui.exibir(f"\n{DOS_VERMELHO}⚠ Você dá um passo e bate o rosto em uma carcaça de metal fria! Você recua pelo susto.⚠{RESET}")
                self.px, self.py = px_old, py_old
            else:
                if getattr(jogo, 'god_mode', False):
                    ui.exibir(f"\n{DOS_AMARELO}[GOD MODE] Você destrói o animatrônico quando ele tenta te atacar, e sai da sala de energia!{RESET}")
                    ui.pausar(1.5)
                    return "vitoria_minotauro"
                else:
                    ui.exibir("\n Você andou direto para as mãos do monstro no escuro...")
                    ui.exibir("@@JUMPSCARE@@")
                    ui.pausar(1.5)
                    ui.exibir("\n No vazio, você morre sozinho, sem poder salvar ninguém. ")
                    return "morte"

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
                
            
            if self.px == self.mx and self.py == self.my:
                if dist_ui > 1: 
                    self.mx, self.my = mx_old, my_old
                    ui.exibir(f"\n{DOS_VERMELHO}⚠VOCÊ TROMBA COM ALGO GIGANTE E METÁLICO NO ESCURO! ELE ESTÁ BEM NA SUA FRENTE!⚠{RESET}")
                    ui.pausar(2)
                    return "continuar"
                else:
                    if getattr(jogo, 'god_mode', False):
                        ui.exibir(f"\n{DOS_AMARELO}[GOD MODE] O Minotauro pula em cima de você, mas é repelido por um escudo de energia! Ele desiste e foge.{RESET}")
                        ui.pausar(2)
                        return "vitoria_minotauro"
                    else:
                        ui.exibir(f"\n{DOS_VERMELHO} ☠ O Minotauro te encontrou no escuro. Mãos frias de metal te rasgam por inteiro ☠{RESET}")
                        ui.exibir("@@JUMPSCARE@@")
                        return "morte"
                
        return "continuar"
