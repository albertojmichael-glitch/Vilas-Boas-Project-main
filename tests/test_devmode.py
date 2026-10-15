import pytest
from state import GameState
from villas_boas.engine import core
from villas_boas.engine.core import processar_fluxo_jogo

CODIGO = "sudo admin 1982"


class UIGravador:
    def __init__(self):
        self.buffer, self.saida = [], []

    def exibir(self, texto):
        self.saida.append(str(texto))

    def animar(self, texto, tempo=0, cor="", jogo=None):
        self.saida.append(str(texto))

    def limpar(self):
        pass

    def pausar(self, segs):
        pass

    def obter_input(self, prompt):
        return ""

    @property
    def texto(self):
        return "\n".join(self.saida)


@pytest.fixture
def jogo_menu():
    return GameState(ui_handler=UIGravador())  # estado inicial: MENU


@pytest.fixture
def jogo_dev(jogo_menu):
    processar_fluxo_jogo(CODIGO, jogo_menu)
    jogo_menu.ui_handler.saida.clear()
    return jogo_menu


def test_codigo_no_menu_inicia_partida_em_modo_dev(jogo_menu):
    processar_fluxo_jogo("SUDO  Admin 1982", jogo_menu)  # normalização: caixa e espaços
    assert jogo_menu.dev_mode and jogo_menu.dev_usado and jogo_menu.god_mode
    assert jogo_menu.estado_atual == "JOGO"
    assert jogo_menu.hp >= 9999 and jogo_menu.turnos_luz >= 9999


def test_codigo_no_meio_da_partida_mantem_a_sala():
    jogo = GameState(ui_handler=UIGravador())
    jogo.estado_atual, jogo.sala_atual = "JOGO", "hall de entrada"
    processar_fluxo_jogo(CODIGO, jogo)
    assert jogo.dev_mode and jogo.sala_atual == "hall de entrada"


def test_codigo_errado_nao_ativa(jogo_menu):
    processar_fluxo_jogo("sudo admin 1983", jogo_menu)
    assert not jogo_menu.dev_mode


def test_desabilitado_por_env(jogo_menu, monkeypatch):
    monkeypatch.setenv("DEV_MODE_ENABLED", "0")
    processar_fluxo_jogo(CODIGO, jogo_menu)
    assert not jogo_menu.dev_mode


def test_codigo_customizado_por_env(jogo_menu, monkeypatch):
    monkeypatch.setenv("DEV_MODE_CODE", "abra se same")
    processar_fluxo_jogo(CODIGO, jogo_menu)
    assert not jogo_menu.dev_mode
    processar_fluxo_jogo("abra se same", jogo_menu)
    assert jogo_menu.dev_mode


def test_teleporte_e_tp_ia(jogo_dev):
    processar_fluxo_jogo("tp cozinha principal", jogo_dev)
    assert jogo_dev.sala_atual == "cozinha principal"
    jogo_dev.ai_sala = "palco"
    processar_fluxo_jogo("tp ia", jogo_dev)
    assert jogo_dev.sala_atual == "palco"


def test_tp_inexistente_nao_move(jogo_dev):
    antes = jogo_dev.sala_atual
    processar_fluxo_jogo("tp xyzabc", jogo_dev)
    assert jogo_dev.sala_atual == antes
    assert "não existe" in jogo_dev.ui_handler.texto


def test_radar_mostra_sala_do_perseguidor(jogo_dev):
    jogo_dev.sala_atual, jogo_dev.ai_sala = "entrada", "sala de jantar"
    processar_fluxo_jogo("radar", jogo_dev)
    t = jogo_dev.ui_handler.texto
    assert (
        "sala de jantar" in t
        and "1 sala de distância" in t
        and "entrada > sala de jantar" in t
    )


def test_radar_automatico_apos_cada_turno_e_desligavel(jogo_dev):
    processar_fluxo_jogo("olhar", jogo_dev)
    assert "[RADAR]" in jogo_dev.ui_handler.texto
    processar_fluxo_jogo("dev radar off", jogo_dev)
    jogo_dev.ui_handler.saida.clear()
    processar_fluxo_jogo("olhar", jogo_dev)
    assert "[RADAR]" not in jogo_dev.ui_handler.texto


def test_gerar_item_e_tudo(jogo_dev):
    processar_fluxo_jogo("gerar isqueiro", jogo_dev)
    assert "isqueiro" in jogo_dev.inventario
    processar_fluxo_jogo("gerar tudo", jogo_dev)
    assert len(jogo_dev.inventario) > 20
    assert "fios cortados" not in jogo_dev.inventario  # só explícito


def test_itens_usados_voltam_mas_largados_ficam(jogo_dev):
    jogo_dev.inventario = ["lanterna", "bateria nova"]
    jogo_dev.sala_atual = "entrada"
    processar_fluxo_jogo("usar bateria nova", jogo_dev)
    assert "bateria nova" in jogo_dev.inventario  # consumido -> restaurado
    processar_fluxo_jogo("largar bateria nova", jogo_dev)
    assert "bateria nova" not in jogo_dev.inventario  # largado de propósito


def test_vida_e_luz_infinitas(jogo_dev):
    jogo_dev.hp, jogo_dev.turnos_luz = 1, 0
    processar_fluxo_jogo("olhar", jogo_dev)
    assert jogo_dev.hp >= 9999 and jogo_dev.turnos_luz >= 9999


def test_perseguidor_solto_nao_mata(jogo_dev):
    processar_fluxo_jogo("dev ia on", jogo_dev)
    jogo_dev.sala_atual, jogo_dev.ai_sala, jogo_dev.nivel_barulho = (
        "entrada",
        "sala de jantar",
        100,
    )
    processar_fluxo_jogo(
        "ir frente", jogo_dev
    )  # anda até a sala do perseguidor: ele alcança no mesmo turno
    t = jogo_dev.ui_handler.texto
    assert "o modo dev te protegeu" in t
    assert jogo_dev.estado_atual == "JOGO" and jogo_dev.sala_atual != "morte"
    assert jogo_dev.ai_sala != jogo_dev.sala_atual


def test_ia_congelada_por_padrao(jogo_dev):
    jogo_dev.ai_sala, jogo_dev.nivel_barulho = "palco", 100
    for _ in range(5):
        processar_fluxo_jogo("olhar", jogo_dev)
    assert jogo_dev.ai_sala == "palco"


def test_dev_morrer_leva_ao_fim(jogo_dev):
    processar_fluxo_jogo("dev morrer", jogo_dev)
    assert jogo_dev.estado_atual == "FIM"


def test_dev_off_restaura_valores_normais(jogo_dev):
    processar_fluxo_jogo("dev off", jogo_dev)
    assert not jogo_dev.dev_mode and not jogo_dev.god_mode
    assert jogo_dev.hp < 100 and jogo_dev.dev_usado  # continua marcada como usada


def test_comandos_dev_nao_funcionam_sem_o_modo():
    jogo = GameState(ui_handler=UIGravador())
    jogo.estado_atual, jogo.sala_atual = "JOGO", "entrada"
    processar_fluxo_jogo("dev status", jogo)
    processar_fluxo_jogo("gerar tudo", jogo)
    assert len(jogo.inventario) <= 1 and not jogo.dev_mode


def test_reiniciar_depois_do_fim_limpa_o_modo(jogo_dev):
    processar_fluxo_jogo("dev morrer", jogo_dev)
    processar_fluxo_jogo("reiniciar", jogo_dev)
    assert not jogo_dev.dev_mode and not jogo_dev.dev_usado and not jogo_dev.god_mode


def test_dev_status_e_mapa_e_ajuda(jogo_dev):
    for cmd in ("dev", "dev status", "dev mapa"):
        processar_fluxo_jogo(cmd, jogo_dev)
    t = jogo_dev.ui_handler.texto
    assert "COMANDOS DO MODO DEV" in t and "STATUS DEV" in t and "MAPA" in t


def test_save_carrega_com_flags_dev(jogo_dev):
    novo = GameState.from_dict(jogo_dev.to_dict())
    assert novo.dev_mode and novo.dev_usado


def test_partida_dev_fica_fora_do_leaderboard_e_telemetria(jogo_dev, monkeypatch):
    chamadas = []
    import services.telemetry_service as ts

    monkeypatch.setattr(ts, "registrar_telemetria", lambda *a, **k: chamadas.append(1))
    from app import app

    with app.test_request_context():
        core.registrar_telemetria_segura("MORTE", "x", "NORMAL", "t", jogo_dev)
    assert chamadas == []
