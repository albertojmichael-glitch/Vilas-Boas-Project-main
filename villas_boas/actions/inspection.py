import logging
from villas_boas.utils import encontrar_melhor_match
from ui import DOS_VERDE, DOS_BRANCO, DOS_AMARELO, RESET
from data import descricoes_itens

logger = logging.getLogger(__name__)

def cmd_examinar(comando, jogo, mapa):
    ui = jogo.ui_handler
    item = comando.replace("examinar ", "").strip()
    
    sala = mapa.get(jogo.sala_atual, {})
    coisas_para_olhar = sala.get("inspecionaveis", {})
    
    match_cenario = encontrar_melhor_match(item, list(coisas_para_olhar.keys()))
    match_inv = encontrar_melhor_match(item, jogo.inventario)
    match_chao = encontrar_melhor_match(item, sala.get("itens", []))
    
    if match_cenario:
        ui.exibir(f"\n{DOS_VERDE}C:\\> ACESSANDO ARQUIVO DE DADOS...{RESET}")
        ui.pausar(1)
        
        if match_cenario == "papeis" and jogo.sala_atual == "01":
            try:
                from data import ARTE_PASTA
                ui.animar(f"{DOS_BRANCO}{ARTE_PASTA}{RESET}", 0.015, jogo=jogo)
            except ImportError as e:
                logger.debug(f"ARTE_PASTA indisponível no arquivo de dados: {e}")
            
        ui.animar(coisas_para_olhar[match_cenario], 0.03, DOS_AMARELO, jogo=jogo)
        ui.pausar(2)
        return True
        
    elif match_inv or match_chao:
        item_real = match_inv if match_inv else match_chao
        desc = descricoes_itens.get(item_real, "Não há nada de especial nisso.")
        ui.exibir(f"\n{DOS_AMARELO} ☞ {desc}{RESET}")
        return True
    else:
        ui.exibir(f"Você não vê nenhum '{item}' aqui para examinar.")
        return False