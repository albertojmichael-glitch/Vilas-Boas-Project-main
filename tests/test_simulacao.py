import random
from state import GameState
from villas_boas.engine.core import processar_fluxo_jogo

class SilenciadorUI:
    
    def __init__(self):
        self.buffer = []
    def exibir(self, texto): pass
    def animar(self, texto, tempo=0, cor="", jogo=None): pass
    def limpar(self): pass
    def pausar(self, segs): pass
    def obter_input(self, prompt): return ""

def rodar_bateria_testes(dificuldade, qtd_partidas=50):
    comandos_base = [
        "ir frente", "ir trás", "ir esquerda", "ir direita",
        "olhar", "pegar bateria nova", "usar bateria nova",
        "pegar chave", "usar isqueiro", "inventario",
        "correr direita", "esperar", "ajuda", "tp saida"
    ]
    
    partidas_concluidas = 0
    
    for _ in range(qtd_partidas):
        jogo = GameState(ui_handler=SilenciadorUI())
        jogo.estado_atual = "JOGO"
        jogo.dificuldade_escolhida = dificuldade
        jogo.hp = 2 if dificuldade == "PESADELO" else 3
        jogo.sala_atual = "entrada"
        
        turnos_jogados = 0
        
        
        while jogo.estado_atual not in ["FIM", "MENU"] and turnos_jogados < 100:
            comando = random.choice(comandos_base)
            processar_fluxo_jogo(comando, jogo)
            turnos_jogados += 1
            
        partidas_concluidas += 1
        
    return partidas_concluidas

def test_stress_dificuldade_normal():
    
    concluidas = rodar_bateria_testes("NORMAL", 50)
    assert concluidas == 50

def test_stress_dificuldade_pesadelo():
    
    concluidas = rodar_bateria_testes("PESADELO", 50)
    assert concluidas == 50