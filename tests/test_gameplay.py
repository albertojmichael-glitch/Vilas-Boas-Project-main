import pytest
import time
from state import GameState
from villas_boas.engine.core import processar_fluxo_jogo, verificar_final_de_jogo
from villas_boas.actions.parser import normalizar


class MockUIHandler:
    
    def __init__(self):
        self.buffer = []
        
    def limpar(self): 
        self.buffer.clear()
        
    def exibir(self, txt): 
        self.buffer.append(txt)
        
    def animar(self, txt, vel, *args, **kwargs): 
        self.buffer.append(txt)
        
    def pausar(self, tempo): 
        pass

@pytest.fixture
def jogo_base():

    jogo = GameState()
    jogo.ui_handler = MockUIHandler()
    jogo.estado_atual = "JOGO"
    jogo.sala_atual = "01"
    jogo.hp = 3
    jogo.inventario = []
    return jogo


def test_parser_normaliza_comandos():
    assert normalizar("   PEgAR    CHaVe  ") == "pegar chave"
    assert normalizar("iR   FRENTE") == "ir frente"
    assert normalizar("    olhar   ") == "olhar"

def test_atalhos_de_movimento(jogo_base):
 
    jogo_base.sala_atual = "entrada"   
    processar_fluxo_jogo("correr frente", jogo_base)       
    assert jogo_base.nivel_barulho == 100

def test_movimento_sala_invalida(jogo_base):
    sala_original = jogo_base.sala_atual
  
    processar_fluxo_jogo("ir teto", jogo_base)
  
    assert jogo_base.sala_atual == sala_original



def test_limite_inventario_base(jogo_base):

    jogo_base.inventario = ["lanterna", "bateria", "faca"]
    
  
    jogo_base.mapa[jogo_base.sala_atual] = {"itens": ["chave de fenda"]}
    
    processar_fluxo_jogo("pegar chave de fenda", jogo_base)
    
  
    assert len(jogo_base.inventario) == 3
    assert "chave de fenda" not in jogo_base.inventario
    
    assert any("cheia" in msg.lower() for msg in jogo_base.ui_handler.buffer)

def test_expansao_inventario_com_bolsas(jogo_base):
    jogo_base.inventario = ["lanterna", "bateria", "faca"]
    jogo_base.bolsas_coletadas = 1 # +3 espaços (total 6)
    
    jogo_base.mapa[jogo_base.sala_atual] = {"itens": ["chave de fenda"]}
    processar_fluxo_jogo("pegar chave de fenda", jogo_base)
    
    assert len(jogo_base.inventario) == 4
    assert "chave de fenda" in jogo_base.inventario



def test_gatilho_final_verdadeiro(jogo_base):

    jogo_base.sala_atual = "hall de entrada"
    jogo_base.noite_vencida = True
    jogo_base.fios_cortados_inventario = True
    jogo_base.tempo_inicio = time.time() - 1000 
    

    fim_alcancado = verificar_final_de_jogo(jogo_base)
    
    assert fim_alcancado is True
    assert "final_verdadeiro" in jogo_base.conquistas
    assert jogo_base.tempo_total_segundos > 0

def test_morte_por_dano(jogo_base):
    jogo_base.hp = 1
  
    jogo_base.sala_atual = "morte"
    
    fim = verificar_final_de_jogo(jogo_base)
    
    assert fim is True
    assert "primeira_morte" in jogo_base.conquistas



def test_save_load_schema_integrity():
    jogo_original = GameState()
    jogo_original.hp = 1
    jogo_original.sala_atual = "sala de energia"
    jogo_original.inventario = ["cartao de seguranca", "bateria"]
    jogo_original.conquistas = ["primeira_morte"]
    
    
    payload_save = jogo_original.to_dict()
    

    jogo_restaurado = GameState.from_dict(payload_save)
    
   
    assert jogo_restaurado.hp == 1
    assert jogo_restaurado.sala_atual == "sala de energia"
    assert len(jogo_restaurado.inventario) == 2
    assert "cartao de seguranca" in jogo_restaurado.inventario
    assert "primeira_morte" in jogo_restaurado.conquistas