import random
from state import GameState
from villas_boas.engine.core import processar_fluxo_jogo

class SilenciadorUI:
    """Uma interface falsa que não imprime nada na tela para o teste rodar rápido."""
    def exibir(self, texto): pass
    def animar(self, texto, tempo=0, cor="", jogo=None): pass
    def limpar(self): pass
    def pausar(self, segs): pass
    def obter_input(self, prompt): return ""

def rodar_simulacao(qtd_partidas=100, dificuldade="PESADELO"):
    mortes = 0
    vitorias = 0
    
    
    comandos_base = [
        "ir frente", "ir trás", "ir esquerda", "ir direita",
        "olhar", "pegar bateria nova", "usar bateria nova",
        "pegar chave", "usar isqueiro", "inventario"
    ]
    
    print(f"Iniciando {qtd_partidas} partidas no modo {dificuldade}...")
    
    for _ in range(qtd_partidas):
        
        jogo = GameState(ui_handler=SilenciadorUI())
        jogo.estado_atual = "JOGO"
        jogo.dificuldade_escolhida = dificuldade
        jogo.hp = 2 if dificuldade == "PESADELO" else 3
        jogo.sala_atual = "01"
        
        turnos_jogados = 0
        
        
        while jogo.estado_atual not in ["FIM", "MENU"] and turnos_jogados < 100:
            comando = random.choice(comandos_base)
            processar_fluxo_jogo(comando, jogo)
            turnos_jogados += 1
            
        if jogo.sala_atual == "morte":
            mortes += 1
        elif jogo.estado_atual == "FIM":
            vitorias += 1

    taxa_morte = (mortes / qtd_partidas) * 100
    
    print("-" * 30)
    print(f"Estatísticas de {qtd_partidas} jogos:")
    print(f"Taxa de Mortalidade: {taxa_morte:.1f}%")
    print(f"Vitórias Acidentais: {vitorias}")
    
    if taxa_morte > 95:
        print("⚠ ALERTA: O modo está punitivo demais (quase impossível).")
    elif taxa_morte < 40:
        print("⚠ ALERTA: O modo está muito fácil para um pesadelo.")
    else:
        print(" Balanceamento dentro do aceitável!")

if __name__ == "__main__":
    rodar_simulacao(500, "PESADELO")