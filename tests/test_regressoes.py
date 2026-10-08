import uuid

from extensions import db
from models import Compartilhamento, SaveJogo
from services.save_service import to_uuid, salvar_save_web, save_existe, CIPHER_SUITE
from state import GameState


def test_to_uuid_aceita_str_e_uuid_e_rejeita_lixo():
    u = uuid.uuid4()
    assert to_uuid(str(u)) == u
    assert to_uuid(u) == u
    assert to_uuid("nao-e-uuid") is None
    assert to_uuid(None) is None


def test_save_roundtrip_com_sid_str(app_ctx):
    sid = str(uuid.uuid4())
    salvar_save_web(GameState(), sid)
    assert save_existe(sid) is True
    assert db.session.get(SaveJogo, uuid.UUID(sid)) is not None


def test_share_com_save_original_inexistente_retorna_404(app_ctx, cliente):
    share = Compartilhamento(original_sid=str(uuid.uuid4()), expires_at=9e12)
    db.session.add(share)
    db.session.commit()
    assert cliente.get(f"/share/{share.share_token}").status_code == 404


def test_share_com_save_corrompido_nao_gera_500_nao_tratado(app_ctx, cliente):
    sid = uuid.uuid4()
    db.session.add(SaveJogo(sid=sid, dados="isto-nao-e-fernet"))
    share = Compartilhamento(original_sid=str(sid), expires_at=9e12)
    db.session.add(share)
    db.session.commit()
    assert cliente.get(f"/share/{share.share_token}").status_code == 500  # tratado, com mensagem


def test_share_feliz_cria_novo_save_e_redireciona(app_ctx, cliente):
    sid = uuid.uuid4()
    dados = CIPHER_SUITE.encrypt(GameState().model_dump_json().encode()).decode()
    db.session.add(SaveJogo(sid=sid, dados=dados))
    share = Compartilhamento(original_sid=str(sid), expires_at=9e12)
    db.session.add(share)
    db.session.commit()
    r = cliente.get(f"/share/{share.share_token}")
    assert r.status_code == 302


def test_telemetria_chega_na_fila_dentro_de_requisicao(app_ctx):
    from services import telemetry_service as ts
    from villas_boas.engine.core import registrar_telemetria_segura
    while not ts.fila_telemetria.empty():
        ts.fila_telemetria.get_nowait()
    sid = str(uuid.uuid4())
    from app import app
    with app.test_request_context():
        from flask import session
        session["sid"] = sid
        session["permite_telemetria"] = True
        registrar_telemetria_segura("MORTE", "cozinha", "NORMAL", "teste", GameState())
    reg = ts.fila_telemetria.get_nowait()
    assert reg.evento == "MORTE" and reg.sid_sessao == sid


def test_telemetria_respeita_opt_out(app_ctx):
    from services import telemetry_service as ts
    from villas_boas.engine.core import registrar_telemetria_segura
    while not ts.fila_telemetria.empty():
        ts.fila_telemetria.get_nowait()
    from app import app
    with app.test_request_context():
        from flask import session
        session["permite_telemetria"] = False
        registrar_telemetria_segura("MORTE", "cozinha", "NORMAL", "teste", GameState())
    assert ts.fila_telemetria.empty()
