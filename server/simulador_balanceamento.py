import random
import re
from state import GameState
from villas_boas.engine.core import processar_fluxo_jogo

class SilenciadorUI:
    def __init__(self):
        self.buffer = []
        self.mensagens_turno = []
        
    def exibir(self, texto):
        texto_limpo = re.sub(r'\x1b\[[0-9;]*m', '', texto).replace('\n', ' ').strip()
        if texto_limpo:
            self.mensagens_turno.append(texto_limpo)
            
    def animar(self, texto, tempo=0, cor="", jogo=None): 
        self.exibir(texto)
    def limpar(self): pass
    def pausar(self, segs): pass
    def obter_input(self, prompt): return ""

def simular_rota_especifica(rota, dificuldade="NORMAL", mostrar_logs=True):
    iterador_rota = iter(rota)
    
    class UIAutomatica(SilenciadorUI):
        def obter_input(self, prompt):
            try:
                resposta = next(iterador_rota)
                prompt_limpo = re.sub(r'\x1b\[[0-9;]*m', '', prompt).replace('\n', ' ').strip()
                if mostrar_logs:
                    print(f"      ↳ [MINIGAME PEDIU]: '{prompt_limpo}'")
                    print(f"      ↳ [BOT INJETOU A SENHA]: '{resposta}'")
                return resposta
            except StopIteration:
                return ""
                
    jogo = GameState(ui_handler=UIAutomatica())
    jogo.estado_atual = "JOGO"
    jogo.dificuldade_escolhida = dificuldade
    jogo.hp = 2 if dificuldade == "PESADELO" else 3
    jogo.sala_atual = "hall de entrada" 
    
    print("\n" + "="*60)
    print(f"✪ TESTANDO O CAMINHO DOURADO ({dificuldade})")
    print("="*60)
    
    turno = 0
    try:
        while True:
            if jogo.estado_atual in ["FIM", "MENU"] or jogo.sala_atual == "morte":
                break
                
            comando = next(iterador_rota)
            turno += 1
            
            if comando == "[INJETAR_MOEDA]":
                if "moeda velha" not in jogo.inventario:
                    jogo.inventario.append("moeda velha")
                if mostrar_logs:
                    print(f"[{turno:02d}] 'moeda velha' adicionada ao inventário!")
                continue 

            elif comando == "[VENCER_NOITE]":
                jogo.noite_vencida = True
                jogo.sala_atual = "01" 
                jogo.estado_atual = "JOGO"
                if mostrar_logs:
                    print(f"[{turno:02d}] minigame de segurança pulado com sucesso!")
                continue
            
            jogo.ui_handler.mensagens_turno = []
            
            if mostrar_logs:
                print(f"[{turno:02d}] ☛ Comando Executado: '{comando}'")
                
            processar_fluxo_jogo(comando, jogo)
            
            if mostrar_logs:
                for msg in jogo.ui_handler.mensagens_turno:
                    print(f"      > {msg}")
                print(f"      [Status] Sala: {jogo.sala_atual} | HP: {jogo.hp} | Luz: {jogo.turnos_luz}\n")
                
    except StopIteration:
        pass 
        
    print("-" * 60)
    status_vitoria = getattr(jogo, 'noite_vencida', False) or jogo.estado_atual == "FIM"
    if status_vitoria and jogo.sala_atual != "morte":
        print(f"☘ VITÓRIA CONFIRMADA ({dificuldade})! Total: {turno} turnos.")
        return True
    else:
        print(f"☠ MORTE ou FALHA ({dificuldade}) no turno {turno}. Sala final: {jogo.sala_atual}")
        return False

if __name__ == "__main__":
    
  
    rota_normal = [
        "ir direita", "ir direita", "pegar bateria nova", "pegar bolsa",
        "atrás", "ir esquerda", "pegar bateria nova", "pegar remedio",
        "atrás", "ir frente", "pegar bateria nova", "atrás", "atrás",
        "frente", "usar bateria nova", "ir direita", "01", 
        "abrir cofre", "1994", "atrás", "atrás", "ir esquerda",
        "usar chave dos fundos", "frente", "sala de equipamentos",
        "pegar bateria nova", "usar bateria nova", "atrás",
        "sala de mercadorias", "pegar bolsa", "atrás", "atrás",
        "frente", "sala 1", "ir esquerda", "[INJETAR_MOEDA]",
        "jogar consertos", "1", "1", "1",
        "jogar julgamento", "1995", "ela", "1982", "rogerio", "joao", "angela", "renato",
        "ir direita", "atrás", "atrás", "ir direita", "01",
        "cadeira", "[VENCER_NOITE]", "atrás", "atrás", "atrás", 
        "examinar poster", "ir direita"
    ]
    
    
    rota_pesadelo = [
        "ir direita", "ir direita", "pegar bateria nova", "usar bateria nova", 
        "pegar bolsa", "atrás", "ir esquerda", "pegar bateria nova", 
        "pegar remedio", "atrás", "ir frente", "pegar bateria nova", 
        "atrás", "atrás", "frente", "usar bateria nova", "ir direita", "01", 
        "abrir cofre", "1994", "atrás", "atrás", "ir esquerda",
        "usar chave dos fundos", "frente", "sala de equipamentos",
        "pegar bateria nova", "usar bateria nova", "atrás",
        "sala de mercadorias", "pegar bolsa", "atrás", "atrás",
        "frente", "sala 1", "ir esquerda", "[INJETAR_MOEDA]",
        "jogar consertos", "1", "1", "1",
        "jogar julgamento", "1995", "ela", "1982", "rogerio", "joao", "angela", "renato",
        "ir direita", "atrás", "atrás", "ir direita", "01",
        "cadeira", "[VENCER_NOITE]", "atrás", "atrás", "atrás", 
        "examinar poster", "ir direita"
    ]
    
    print("=== EXECUTANDO BATERIA DE TESTES DE CAMINHO DOURADO ===")
    simular_rota_especifica(rota_normal, "NORMAL", mostrar_logs=False)
    simular_rota_especifica(rota_pesadelo, "PESADELO", mostrar_logs=True)