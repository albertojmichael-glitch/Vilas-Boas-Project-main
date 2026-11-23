"""Saves: legíveis no navegador, assinados e validados no servidor (issue #4)."""

import json
import uuid

import pytest
from config import Config
from extensions import db
from models import SaveJogo
from security import assinar_save
from services.save_service import (
    CIPHER_SUITE,
    obter_ou_recuperar_jogo,
    salvar_save_web,
)
from services.save_validacao import SaveInvalido, validar_estado_save
from state import GameState

ORIGEM = {"Origin": "http://localhost:5000"}


def _jogo_em_andamento(**campos):
    jogo = GameState(estado_atual="JOGO")
    jogo.configurar_dificuldade("PESADELO")
    for k, v in campos.items():
        setattr(jogo, k, v)
    return jogo


@pytest.fixture(autouse=True)
def sem_rate_limit(monkeypatch):
    # O limite por minuto das rotas acumularia entre os testes deste módulo.
    from extensions import limiter

    monkeypatch.setattr(limiter, "enabled", False)


@pytest.fixture
def sessao_em_jogo(cliente):
    cliente.post("/iniciar", headers=ORIGEM)
    for cmd in ("dir", "3", "ir direita"):
        cliente.post("/comando", json={"comando": cmd}, headers=ORIGEM)
    return cliente


def _exportar(cliente):
    resp = cliente.get("/save/export")
    assert resp.status_code == 200
    return resp.get_json()


def _importar(cliente, payload):
    return cliente.post("/save/import", json=payload, headers=ORIGEM)


def _assinado(estado):
    return {
        "formato": "vilas-boas-save",
        "versao": 2,
        "estado": estado,
        "assinatura": assinar_save(estado),
        "resumo": {"sala": estado.get("sala_atual")},
    }


# ---------------------------------------------------------------- export/import


def test_export_e_legivel_e_assinado(sessao_em_jogo):
    save = _exportar(sessao_em_jogo)
    assert save["formato"] == "vilas-boas-save" and save["versao"] == 2
    assert save["estado"]["sala_atual"] == "hall de entrada"
    assert save["estado"]["hp"] == 2
    assert len(save["assinatura"]) == 64
    assert "dados_seguros" not in save


def test_round_trip_export_import(sessao_em_jogo):
    save = _exportar(sessao_em_jogo)
    resp = _importar(sessao_em_jogo, save)
    assert resp.status_code == 200, resp.get_json()
    assert _exportar(sessao_em_jogo)["estado"]["sala_atual"] == "hall de entrada"


def test_save_adulterado_e_rejeitado_pela_assinatura(sessao_em_jogo):
    save = _exportar(sessao_em_jogo)
    save["estado"]["hp"] = 99
    resp = _importar(sessao_em_jogo, save)
    assert resp.status_code == 403
    assert "ASSINATURA" in resp.get_json()["erro"]


@pytest.mark.parametrize(
    ("campo", "valor", "motivo"),
    [
        ("hp", 99, "hp"),
        ("sala_atual", "sala secreta", "sala inexistente"),
        ("inventario", ["lanterna", "chave mestra"], "item desconhecido"),
        ("god_mode", True, "modo deus"),
        ("estado_atual", "MINIGAME_SEGURANCA", "não salvável"),
        ("turnos_luz", 500, "turnos_luz"),
        ("conquistas", ["inventada"], "conquista"),
    ],
)
def test_save_assinado_mas_impossivel_e_rejeitado(cliente, campo, valor, motivo):
    estado = json.loads(_jogo_em_andamento().model_dump_json())
    estado[campo] = valor
    resp = _importar(cliente, _assinado(estado))
    assert resp.status_code == 400
    assert motivo in resp.get_json()["erro"]


def test_saida_nova_no_mapa_e_ignorada():
    estado = json.loads(_jogo_em_andamento().model_dump_json())
    estado["mapa"]["entrada"]["esquerda"] = "final_bom"
    estado["mapa"]["sala inventada"] = {"descrição": "x"}
    jogo = validar_estado_save(estado)
    assert jogo.mapa["entrada"]["esquerda"] == "parede velha"
    assert "sala inventada" not in jogo.mapa


def test_mudancas_legitimas_do_mapa_sao_preservadas():
    jogo = _jogo_em_andamento()
    jogo.mapa["corredor"]["03"] = "sala do gerador"
    jogo.mapa["porta dos fundos"]["frente"] = "sala dos fundos"
    jogo.mapa["balcão"]["itens"] = ["pedra"]
    validado = validar_estado_save(json.loads(jogo.model_dump_json()))
    assert validado.mapa["corredor"]["03"] == "sala do gerador"
    assert validado.mapa["porta dos fundos"]["frente"] == "sala dos fundos"
    assert validado.mapa["balcão"]["itens"] == ["pedra"]


def test_presets_vem_do_servidor_e_nao_do_save():
    estado = json.loads(_jogo_em_andamento().model_dump_json())
    estado["energia_min_noite"] = estado["energia_max_noite"] = 9999
    estado["perfil_ela"] = {"chance_mover": 0.0}
    jogo = validar_estado_save(estado)
    assert jogo.energia_max_noite < 9999
    assert jogo.perfil_ela["chance_mover"] > 0


def test_import_legado_v1_fernet_continua_funcionando(cliente):
    jogo = _jogo_em_andamento(sala_atual="balcão")
    token = CIPHER_SUITE.encrypt(jogo.model_dump_json().encode()).decode()
    resp = _importar(cliente, {"dados_seguros": token, "resumo": {"sala": "balcão"}})
    assert resp.status_code == 200, resp.get_json()


def test_import_rejeita_lixo(cliente):
    assert _importar(cliente, {"qualquer": 1}).status_code == 400
    assert _importar(cliente, {"dados_seguros": "nao-e-fernet"}).status_code == 403
    resp = cliente.post(
        "/save/import", data="isto nao e json", headers=ORIGEM, content_type="text"
    )
    assert resp.status_code == 415


# ------------------------------------------------------------ persistência


def test_estado_invalido_nao_sobrescreve_o_ultimo_save_bom(app_ctx):
    sid = str(uuid.uuid4())
    assert salvar_save_web(_jogo_em_andamento(sala_atual="balcão"), sid)
    ruim = _jogo_em_andamento()
    ruim.inventario.append("chave mestra")
    assert salvar_save_web(ruim, sid) is False
    recuperado = obter_ou_recuperar_jogo(sid, {})
    assert recuperado.sala_atual == "balcão"


def test_registro_corrompido_no_banco_nao_quebra_nem_e_apagado(app_ctx):
    sid = uuid.uuid4()
    db.session.add(SaveJogo(sid=sid, dados="{isto: nao e json"))
    db.session.commit()
    assert obter_ou_recuperar_jogo(str(sid), {}) is None
    assert db.session.get(SaveJogo, sid) is not None


def test_save_legado_em_json_puro_e_migrado_para_cifrado(app_ctx):
    sid = uuid.uuid4()
    texto = _jogo_em_andamento(sala_atual="balcão").model_dump_json()
    db.session.add(SaveJogo(sid=sid, dados=texto))
    db.session.commit()
    jogo = obter_ou_recuperar_jogo(str(sid), {})
    assert jogo is not None and jogo.sala_atual == "balcão"
    gravado = db.session.get(SaveJogo, sid).dados
    assert not gravado.startswith("{")
    assert json.loads(CIPHER_SUITE.decrypt(gravado.encode()))["sala_atual"] == "balcão"


def test_validador_nao_aceita_nao_objeto():
    with pytest.raises(SaveInvalido):
        validar_estado_save(["nao", "e", "objeto"])


# ---------------------------------------------------------------------- chaves


def test_chaves_de_cifra_e_assinatura_sao_diferentes():
    assert Config.SAVE_SIGNING_KEY
    assert Config.SAVE_SIGNING_KEY != Config.SAVE_ENCRYPTION_KEY
    assert Config.SAVE_SIGNING_KEY != Config.SECRET_KEY


def test_producao_sem_secret_key_nao_inicia(monkeypatch):
    original = dict(vars(Config))
    monkeypatch.setenv("APP_ENV", "production")
    monkeypatch.delenv("SECRET_KEY", raising=False)
    monkeypatch.delenv("FLASK_SECRET_KEY", raising=False)
    try:
        with pytest.raises(RuntimeError, match="SECRET_KEY"):
            Config.validar_e_configurar()
    finally:
        monkeypatch.undo()
        for chave, valor in original.items():
            if not chave.startswith("__"):
                setattr(Config, chave, valor)


def test_desenvolvimento_sem_secret_key_continua_subindo(monkeypatch):
    original = dict(vars(Config))
    monkeypatch.setenv("APP_ENV", "development")
    monkeypatch.delenv("SECRET_KEY", raising=False)
    monkeypatch.delenv("FLASK_SECRET_KEY", raising=False)
    try:
        Config.validar_e_configurar()
        assert Config.SECRET_KEY
    finally:
        monkeypatch.undo()
        for chave, valor in original.items():
            if not chave.startswith("__"):
                setattr(Config, chave, valor)
