#olamundo
import pytest
import os
import json
from unittest.mock import patch
from state import GameState
from villas_boas.actions import processar_comando
from villas_boas.utils import encontrar_melhor_match
from villas_boas.engine import MinigameMinotauro, MinigameSeguranca


class DummyUI:
    """Simula a interface do jogo para os testes não precisarem imprimir no console real"""
    def __init__(self):
        self.buffer = []
    def exibir(self, texto):
        self.buffer.append(texto)
    def limpar(self):
        pass
    def pausar(self, *args):
        pass
    def animar(self, texto, *args, **kwargs):
        self.buffer.append(texto)
@pytest.fixture
def jogo_base():
    """Retorna um GameState zerado e pronto para testes."""
    jogo = GameState()
    jogo.ui_handler = DummyUI()
    jogo.estado_atual = "JOGO"
    return jogo
def test_fuzzy_matching_itens():
    """Garante que o jogador pode errar algumas letras e o jogo ainda entende o item."""
    opcoes = ["lanterna", "bateria nova", "chave dos fundos", "fios cortados"]
    assert encontrar_melhor_match("lantrna", opcoes) == "lanterna"
    assert encontrar_melhor_match("chave fundos", opcoes) == "chave dos fundos"
    assert encontrar_melhor_match("bateria", opcoes) == "bateria nova"
    assert encontrar_melhor_match("pudim", opcoes) is None
def test_alias_movimentacao(jogo_base):
    """Garante que 'f' vira 'ir frente' e movimenta o jogador."""
    jogo_base.sala_atual = "entrada"
    processar_comando("f", jogo_base, jogo_base.mapa)
    assert jogo_base.sala_atual == "sala de jantar"
def test_inventario_cheio_sem_bolsa(jogo_base):
    """Testa se o jogo bloqueia pegar itens com limite de 3 (sem a bolsa)."""
    jogo_base.inventario = ["item1", "item2", "item3"]
    jogo_base.sala_atual = "entrada"
    jogo_base.mapa["entrada"]["itens"] = ["papel"]
    processar_comando("pegar papel", jogo_base, jogo_base.mapa)
    assert "papel" not in jogo_base.inventario
    assert len(jogo_base.inventario) == 3
    output = " ".join(jogo_base.ui_handler.buffer)
    assert "Sua mochila está cheia" in output
def test_inventario_aumenta_com_bolsa(jogo_base):
    """Garante que ter a bolsa aumenta o limite para 6 e permite pegar o papel."""
    jogo_base.inventario = ["item1", "item2", "bolsa"]
    jogo_base.sala_atual = "entrada"
    jogo_base.mapa["entrada"]["itens"] = ["papel"]
    processar_comando("pegar papel", jogo_base, jogo_base.mapa)
    assert "papel" in jogo_base.inventario
    assert len(jogo_base.inventario) == 4
def test_transicao_morte_animatronico(jogo_base):
    """Se tentar bater no animatronico SEM God Mode, tem que morrer instantaneamente."""
    jogo_base.estado_atual = "COMBATE_ANIMATRONICO"
    jogo_base.god_mode = False
    gastou_turno = processar_comando("bater", jogo_base, jogo_base.mapa)
    assert gastou_turno is True
    assert jogo_base.sala_atual == "morte"
    assert jogo_base.estado_atual == "FIM"
def test_vitoria_animatronico_god_mode(jogo_base):
    """Se tentar bater COM God Mode, ele espanta o monstro e volta pro jogo normal."""
    jogo_base.estado_atual = "COMBATE_ANIMATRONICO"
    jogo_base.god_mode = True
    processar_comando("bater", jogo_base, jogo_base.mapa)
    assert jogo_base.sala_atual != "morte"
    assert jogo_base.estado_atual == "JOGO"
    assert jogo_base.posicao_perseguidor == "longe"
def test_minotauro_derrota_escuro(jogo_base):
    """Testa se andar direto para o minotauro gera morte (Soft-lock preventivo)."""
    minigame = MinigameMinotauro(jogo_base)
    minigame.px, minigame.py = 0, 1
    minigame.mx, minigame.my = 0, 1
    resultado = minigame.processar_turno("esperar", jogo_base)
    assert resultado == "morte"
def test_seguranca_energia_esgotada(jogo_base):
    """Testa se a energia acaba ao ser muito gasta e aciona o apagão."""
    minigame = MinigameSeguranca(jogo_base)
    minigame.energia = 2
    minigame.porta_fechada = False
    minigame.processar_turno("fechar porta", jogo_base)
    assert minigame.porta_fechada is False
    output = " ".join(jogo_base.ui_handler.buffer)
    assert "Sem energia" in output
def test_salvar_e_carregar_autosave(jogo_base, tmp_path):
    """Cria um arquivo de save temporário, altera dados, salva e garante que carrega os dados exatos."""
    import state
    caminho_original = state.AUTOSAVE_FILE
    state.AUTOSAVE_FILE = tmp_path / "test_save.json"
    jogo_base.sala_atual = "01"
    jogo_base.hp = 1
    jogo_base.inventario = ["chave dos fundos"]
    sucesso_save = state.salvar_autosave(jogo_base)
    assert sucesso_save is True
    novo_jogo = GameState()
    sucesso_load = state.carregar_autosave(novo_jogo)
    assert sucesso_load is True
    assert novo_jogo.sala_atual == "01"
    assert novo_jogo.hp == 1
    assert "chave dos fundos" in novo_jogo.inventario
    state.AUTOSAVE_FILE = caminho_original