"""Corrupção de dados: nenhum arquivo, payload ou registro quebrado pode gerar
erro 500, derrubar a sessão ou apagar progresso."""

import json
import random
import uuid

import pytest
from extensions import db
from models import SaveJogo
from services.save_service import obter_ou_recuperar_jogo, salvar_save_web
from state import GameState

ORIGEM = {"Origin": "http://localhost:5000"}


@pytest.fixture(autouse=True)
def sem_rate_limit(monkeypatch):
    from extensions import limiter

    monkeypatch.setattr(limiter, "enabled", False)


@pytest.fixture
def save_exportado(cliente):
    cliente.post("/iniciar", headers=ORIGEM)
    for cmd in ("dir", "1", "ir direita"):
        cliente.post("/comando", json={"comando": cmd}, headers=ORIGEM)
    return cliente.get("/save/export").get_json()


def _importar_bruto(cliente, corpo: bytes):
    return cliente.post(
        "/save/import",
        data=corpo,
        headers=ORIGEM,
        content_type="application/json",
    )


def test_bytes_corrompidos_nunca_geram_500(cliente, save_exportado):
    original = json.dumps(save_exportado, ensure_ascii=False).encode("utf-8")
    rng = random.Random(1982)
    for _ in range(150):
        corrompido = bytearray(original)
        for _ in range(rng.randint(1, 8)):
            corrompido[rng.randrange(len(corrompido))] = rng.randrange(256)
        resp = _importar_bruto(cliente, bytes(corrompido))
        assert resp.status_code in (200, 400, 403), resp.status_code


@pytest.mark.parametrize("corte", [1, 10, 100, 1000])
def test_arquivo_truncado_e_recusado(cliente, save_exportado, corte):
    texto = json.dumps(save_exportado).encode()
    resp = _importar_bruto(cliente, texto[: len(texto) - corte])
    assert resp.status_code == 400


@pytest.mark.parametrize(
    "payload",
    [
        None,
        [],
        "texto",
        123,
        {"formato": "vilas-boas-save", "versao": 2},
        {"formato": "vilas-boas-save", "versao": 2, "estado": "x", "assinatura": 1},
        {"formato": "vilas-boas-save", "versao": 99, "estado": {}, "assinatura": ""},
        {"dados_seguros": 42},
    ],
)
def test_estruturas_estranhas_sao_recusadas(cliente, payload):
    resp = cliente.post("/save/import", json=payload, headers=ORIGEM)
    assert resp.status_code in (400, 403)
    assert "erro" in resp.get_json()


def test_arquivo_gigante_e_recusado(cliente):
    resp = _importar_bruto(cliente, b'{"x":"' + b"a" * 300_000 + b'"}')
    assert resp.status_code == 413


def test_import_recusado_nao_apaga_o_progresso(cliente, save_exportado):
    adulterado = json.loads(json.dumps(save_exportado))
    adulterado["estado"]["hp"] = 99
    assert cliente.post("/save/import", json=adulterado, headers=ORIGEM).status_code
    depois = cliente.get("/save/export").get_json()
    assert depois["estado"]["sala_atual"] == save_exportado["estado"]["sala_atual"]


@pytest.mark.parametrize(
    "corpo",
    [b"", b"nao e json", b'{"comando": 123}', b'{"comando": ["a"]}', b"[1,2]"],
)
def test_comando_com_payload_quebrado_nao_derruba(cliente, corpo):
    cliente.post("/iniciar", headers=ORIGEM)
    resp = cliente.post(
        "/comando", data=corpo, headers=ORIGEM, content_type="application/json"
    )
    assert resp.status_code in (200, 400)


@pytest.mark.parametrize(
    "dados",
    [
        "",
        "   ",
        "{",
        "null",
        "[]",
        '{"hp": "muito"}',
        "gAAAAABinvalido",
        json.dumps({"estado_atual": "JOGO", "sala_atual": "lugar nenhum"}),
    ],
)
def test_registro_quebrado_no_banco_vira_sem_save(app_ctx, dados):
    sid = uuid.uuid4()
    db.session.add(SaveJogo(sid=sid, dados=dados))
    db.session.commit()
    assert obter_ou_recuperar_jogo(str(sid), {}) is None
    assert db.session.get(SaveJogo, sid) is not None  # nunca apaga


def test_estado_em_memoria_corrompido_nao_sobrescreve_o_banco(app_ctx):
    sid = str(uuid.uuid4())
    bom = GameState(estado_atual="JOGO", sala_atual="balcão")
    assert salvar_save_web(bom, sid)
    corrompido = GameState(estado_atual="JOGO")
    corrompido.sala_atual = "sala que nao existe"
    corrompido.hp = -3
    assert salvar_save_web(corrompido, sid) is False
    assert obter_ou_recuperar_jogo(sid, {}).sala_atual == "balcão"
