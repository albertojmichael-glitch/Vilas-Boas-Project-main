"""Parser e comandos: tudo o que o jogo anuncia precisa funcionar.

Cobre o menu (inclusive as opções 5 e 6, que antes davam "OPÇÃO INVÁLIDA"),
sinônimos, ajuda, cada saída/objeto/item de cada sala, e entradas aleatórias
(fuzz) que nunca podem derrubar o motor.
"""

import random
import re

import pytest
from state import GameState
from villas_boas.actions.parser import ATALHOS_DIRECAO, normalizar, sugerir_comando
from villas_boas.engine.core import (
    SINONIMOS_POR_SALA,
    processar_fluxo_jogo,
    traduzir_sinonimo,
)
from villas_boas.simulacao.ambiente import ambiente_simulacao

ANSI = re.compile(r"\x1b\[[0-9;]*m")
ERROS = re.compile(
    r"Comando não reconhecido|Você não pode ir|OPÇÃO INVÁLIDA|Bad command|ERRO INTERNO",
    re.IGNORECASE,
)
SALAS_TERMINAIS = {"morte", "saida", "cama", "final_bom"}
INVENTARIO_COMPLETO = [
    "lanterna",
    "bateria nova",
    "isqueiro",
    "remedio",
    "chave dos fundos",
    "moeda velha",
    "partitura rasgada",
    "cartao de seguranca",
    "tabua pequena de madeira",
    "tesoura quebrada",
    "fita isolante",
]


@pytest.fixture(autouse=True)
def _ambiente():
    with ambiente_simulacao():
        yield


def _novo(estado="JOGO", sala="entrada", **campos):
    jogo = GameState()
    jogo.configurar_dificuldade("NORMAL")
    jogo.estado_atual = estado
    jogo.sala_atual = sala
    jogo.turnos_luz = 99
    jogo.ela_dormencia = 999
    for k, v in campos.items():
        setattr(jogo, k, v)
    return jogo


def _rodar(jogo, comando):
    inicio = len(jogo.ui_handler.buffer)
    processar_fluxo_jogo(comando, jogo)
    return " | ".join(
        ANSI.sub("", m).strip() for m in jogo.ui_handler.buffer[inicio:] if m.strip()
    )


# ------------------------------------------------------------------- parser


@pytest.mark.parametrize(
    ("bruto", "esperado"),
    [
        ("  IR   Frente!! ", "ir frente"),
        ("Pegar, 'bateria nova'.", "pegar bateria nova"),
        ("", ""),
        (None, ""),
        ("\tOLHAR\n", "olhar"),
    ],
)
def test_normalizar(bruto, esperado):
    assert normalizar(bruto) == esperado


def test_atalhos_de_direcao_cobrem_todas_as_direcoes():
    assert set(ATALHOS_DIRECAO.values()) == {
        "ir frente",
        "ir atrás",
        "ir esquerda",
        "ir direita",
    }


def test_sugestao_para_erro_de_digitacao():
    assert sugerir_comando("pgar") == "pegar"
    assert sugerir_comando("inventraio") == "inventario"
    assert sugerir_comando("xyzzy") is None


def test_comando_desconhecido_sugere_ajuda():
    texto = _rodar(_novo(), "dançar")
    assert "ajuda" in texto


# --------------------------------------------------------------------- menu


@pytest.mark.parametrize(
    ("opcao", "estado", "dificuldade"),
    [
        ("1", "JOGO", "NORMAL"),
        ("2", "JOGO", "NORMAL"),
        ("3", "JOGO", "PESADELO"),
        ("4", "JOGO", "PESADELO"),
        ("5", "MENU_DESAFIOS", None),
        ("6", "MENU_CUSTOM_BAT", None),
    ],
)
def test_todas_as_opcoes_do_menu_funcionam(opcao, estado, dificuldade):
    jogo = _novo(estado="MENU")
    texto = _rodar(jogo, opcao)
    assert "INVÁLIDA" not in texto.upper()
    assert jogo.estado_atual == estado
    if dificuldade:
        assert jogo.dificuldade_escolhida == dificuldade


@pytest.mark.parametrize(
    ("opcao", "dificuldade"),
    [("1", "DESAFIO_SPEEDRUN"), ("2", "DESAFIO_FANTASMA"), ("3", "DESAFIO_BREU")],
)
def test_modos_de_desafio(opcao, dificuldade):
    jogo = _novo(estado="MENU")
    texto = _rodar(jogo, "5")
    assert "MODOS DE DESAFIO" in texto
    _rodar(jogo, opcao)
    assert jogo.estado_atual == "JOGO"
    assert jogo.dificuldade_escolhida == dificuldade


def test_desafios_voltar_e_opcao_invalida():
    jogo = _novo(estado="MENU")
    _rodar(jogo, "5")
    assert "Opção inválida" in _rodar(jogo, "9")
    assert jogo.estado_atual == "MENU_DESAFIOS"
    _rodar(jogo, "4")
    assert jogo.estado_atual == "MENU"


def test_modo_personalizado_completo_e_com_limites():
    jogo = _novo(estado="MENU")
    _rodar(jogo, "6")
    assert "entre 1 e 99" in _rodar(jogo, "-5")
    assert "entre 1 e 99" in _rodar(jogo, "abc")
    assert jogo.estado_atual == "MENU_CUSTOM_BAT"
    _rodar(jogo, "20")
    _rodar(jogo, "6")
    assert "entre 1 e 10" in _rodar(jogo, "0")
    _rodar(jogo, "5")
    assert jogo.estado_atual == "JOGO"
    assert jogo.dificuldade_escolhida == "CUSTOM"
    assert (jogo.turnos_luz, jogo.bolsas_coletadas, jogo.hp) == (20, 2, 5)


def test_menu_opcao_invalida_continua_no_menu():
    jogo = _novo(estado="MENU")
    assert "INVÁLIDA" in _rodar(jogo, "99").upper()
    assert jogo.estado_atual == "MENU"


# ---------------------------------------------------------- comandos no jogo


@pytest.mark.parametrize("cmd", ["ajuda", "help", "manual", "comandos"])
def test_ajuda_mostra_o_guia_sem_gastar_turno(cmd):
    jogo = _novo()
    luz = jogo.turnos_luz
    texto = _rodar(jogo, cmd)
    assert "correr" in texto and "esconder" in texto
    assert jogo.turnos_luz == luz


def test_salvar_explica_o_salvamento_automatico():
    assert "automaticamente" in _rodar(_novo(), "salvar")


@pytest.mark.parametrize(
    ("sala", "sinonimo", "canonico"),
    [
        (sala, sin, canon)
        for sala, mapa in SINONIMOS_POR_SALA.items()
        for sin, canon in mapa.items()
    ],
)
def test_sinonimos_viram_o_comando_canonico(sala, sinonimo, canonico):
    assert traduzir_sinonimo(sinonimo, _novo(sala=sala)) == canonico


@pytest.mark.parametrize("cmd", ["tocar piano", "fome de jon", "usar moeda velha"])
def test_sinonimos_dos_fliperamas_iniciam_o_minigame(cmd):
    jogo = _novo(sala="sala de fliperamas", inventario=list(INVENTARIO_COMPLETO))
    texto = _rodar(jogo, cmd)
    assert not ERROS.search(texto)
    assert jogo.estado_atual.startswith("MINIGAME")


def test_voltar_anda_para_tras():
    jogo = _novo(sala="sala de jantar")
    _rodar(jogo, "voltar")
    assert jogo.sala_atual == "entrada"


def test_andar_ate_a_cadeira_nunca_quebra():
    # Regressão: com o evento aleatório de 25%, 'ir frente' na 01 dava KeyError.
    for semente in range(200):
        random.seed(semente)
        jogo = _novo(sala="01")
        texto = _rodar(jogo, "ir frente")
        assert "sentar" in texto
        assert jogo.sala_atual == "01"


def _comandos_da_sala(sala, dados):
    cmds = []
    for chave in dados:
        if chave in ("descrição", "itens", "inspecionaveis", "cofre_important"):
            continue
        cmds += [chave, f"ir {chave}"]
    for objeto in dados.get("inspecionaveis", {}):
        cmds += [f"examinar {objeto}", objeto, f"olhar {objeto}"]
    cmds += [f"pegar {item}" for item in dados.get("itens", [])]
    return cmds


def _casos_por_sala():
    for sala, dados in GameState().mapa.items():
        if sala in SALAS_TERMINAIS:
            continue
        for cmd in _comandos_da_sala(sala, dados):
            yield sala, cmd


@pytest.mark.parametrize(("sala", "cmd"), list(_casos_por_sala()))
def test_toda_saida_objeto_e_item_de_cada_sala_responde(sala, cmd):
    jogo = _novo(sala=sala, inventario=list(INVENTARIO_COMPLETO), bolsas_coletadas=5)
    texto = _rodar(jogo, cmd)
    assert not ERROS.search(texto), texto[:200]


@pytest.mark.parametrize(
    "cmd",
    [
        "olhar",
        "inventario",
        "i",
        "esperar",
        "usar lanterna",
        "usar bateria nova",
        "usar isqueiro",
        "usar remedio",
        "largar isqueiro",
        "combinar tesoura quebrada com fita isolante",
        "x",
    ],
)
def test_acoes_basicas_nunca_dao_erro(cmd):
    jogo = _novo(sala="sala de jantar", inventario=list(INVENTARIO_COMPLETO), hp=1)
    assert not ERROS.search(_rodar(jogo, cmd))


# --------------------------------------------------------------------- fuzz

ESTADOS_FUZZ = [
    "AGUARDANDO_DIR",
    "MENU",
    "MENU_DESAFIOS",
    "MENU_CUSTOM_BAT",
    "JOGO",
    "COMBATE_ANIMATRONICO",
    "MINIGAME_COFRE",
    "MINIGAME_JON",
    "MINIGAME_CONSERTOS_CABECA",
    "MINIGAME_JULGAMENTO_Q1",
    "FIM",
]
PEDACOS = [
    "ir",
    "pegar",
    "usar",
    "correr",
    "frente",
    "atrás",
    "01",
    "sala 1",
    "esconder",
    "jogar jon",
    "1",
    "6",
    "🙂",
    "ção",
    "%s",
    "{}",
    "\\",
    "..",
    "' OR 1=1 --",
    "a" * 300,
]


@pytest.mark.parametrize("estado", ESTADOS_FUZZ)
def test_entradas_aleatorias_nunca_derrubam_o_motor(estado):
    rng = random.Random(estado)
    for _ in range(300):
        jogo = _novo(estado=estado, sala=rng.choice(["entrada", "01", "sala 1"]))
        if estado == "MINIGAME_JON":
            jogo.jon_caminho_certo = ["f", "e", "d", "f"]
        if estado.startswith("MINIGAME_JULGAMENTO"):
            jogo.web_julgamento = {"pontos": 0, "vitimas": ["angela"]}
        comando = " ".join(rng.choice(PEDACOS) for _ in range(rng.randint(0, 4)))
        processar_fluxo_jogo(comando, jogo)  # não pode levantar exceção
