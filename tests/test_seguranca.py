import uuid

import pytest
from extensions import MEMORIA_SESSOES, RedisSessionStore, db, oauth
from models import Compartilhamento, Jogador, SaveJogo, Telemetria
from pydantic import ValidationError
from routes.helpers import ComandoRequest
from services.save_service import salvar_save_web
from state import GameState


def test_bloqueio_injecao_sql():
    with pytest.raises(ValidationError):
        ComandoRequest(comando="ir frente; DROP TABLE saves;")


def test_bloqueio_xss_cross_site_scripting():
    with pytest.raises(ValidationError):
        ComandoRequest(comando="<script>alert('hack')</script>")


def test_comando_extremamente_longo():
    with pytest.raises(ValidationError):
        ComandoRequest(comando="frente " * 100)


def test_comando_valido_passa_normalmente():
    req = ComandoRequest(comando="abrir cofre")
    assert req.comando == "abrir cofre"


@pytest.mark.parametrize(
    "caminho_sensivel",
    [
        "/config.py",
        "/models.py",
        "/data/game_data.json",
        "/instance/local_testes.db",
        "/villas_boas.log",
        "/migrations/alembic.ini",
    ],
)
def test_arquivos_internos_do_servidor_nao_sao_expostos(cliente, caminho_sensivel):
    resposta = cliente.get(caminho_sensivel)
    assert resposta.status_code == 404


def test_validacao_origin_bloqueia_origens_externas(cliente):
    headers_maliciosos = {"Origin": "https://site-atacante.evil"}
    assert cliente.post("/iniciar", headers=headers_maliciosos).status_code == 403
    assert (
        cliente.post(
            "/comando", json={"comando": "dir"}, headers=headers_maliciosos
        ).status_code
        == 403
    )
    assert (
        cliente.post("/share/generate", headers=headers_maliciosos).status_code == 403
    )
    assert (
        cliente.post(
            "/save/import",
            json={"dados_seguros": "abc"},
            headers=headers_maliciosos,
        ).status_code
        == 403
    )


def test_validacao_origin_aceita_origem_permitida(cliente):
    headers_validos = {"Origin": "http://localhost:5000"}
    assert cliente.post("/iniciar", headers=headers_validos).status_code == 200


def test_admin_rejeita_token_na_query_string_e_aceita_header(app_ctx, cliente):
    from flask import current_app

    token = current_app.config["ADMIN_TOKEN"]

    # Token na query string deve ser negado
    resp_query = cliente.get(f"/admin/analytics?token={token}")
    assert resp_query.status_code == 403

    # Sem token deve ser negado
    assert cliente.get("/admin/analytics").status_code == 403

    # Com header X-Admin-Token deve ser autorizado
    headers = {"X-Admin-Token": token}
    resp_header = cliente.get("/admin/analytics", headers=headers)
    assert resp_header.status_code == 200
    dados = resp_header.get_json()
    assert "geral" in dados and "autopsia_da_morte" in dados


def test_rotas_admin_auditoria_saves_e_replay(app_ctx, cliente):
    from flask import current_app

    headers = {"X-Admin-Token": current_app.config["ADMIN_TOKEN"]}

    jogador = Jogador(
        iniciais="AAA",
        email="teste@vilasboas.local",
        tempo_segundos=120.5,
        final_alcancado="saida",
        dificuldade="NORMAL",
    )
    db.session.add(jogador)
    db.session.flush()

    tele = Telemetria(
        sid_sessao="sessao-1",
        jogador_id=jogador.id,
        evento="MORTE",
        sala="cozinha",
        dificuldade="NORMAL",
        detalhes="Perseguidor",
        luz_restante=5,
        nivel_barulho=80,
        inventario_qtd=2,
        log_comandos=["ir frente", "olhar"],
    )
    db.session.add(tele)
    salvar_save_web(GameState(), str(uuid.uuid4()))
    db.session.commit()

    r_auditoria = cliente.get("/admin/auditoria-mortes", headers=headers)
    assert r_auditoria.status_code == 200
    assert len(r_auditoria.get_json()["mortes_identificadas"]) == 1

    r_saves = cliente.get("/admin/saves?page=1", headers=headers)
    assert r_saves.status_code == 200
    assert r_saves.get_json()["total_saves"] >= 1

    r_replay = cliente.get(f"/admin/replay/{tele.id}", headers=headers)
    assert r_replay.status_code == 200
    assert r_replay.get_json()["sala_final"] == "cozinha"

    r_replay_404 = cliente.get("/admin/replay/999999", headers=headers)
    assert r_replay_404.status_code == 404


def test_fluxo_share_generate_e_expiracao(app_ctx, cliente):
    # GET em /share/generate deve retornar 405
    assert cliente.get("/share/generate").status_code == 405

    # Sem sessão ativa retorna 401
    assert cliente.post("/share/generate").status_code == 401

    # Com sessão e save ativo gera link
    cliente.post("/iniciar")
    with cliente.session_transaction() as sess:
        sid = sess["sid"]
    salvar_save_web(GameState(), sid)

    r_gen = cliente.post("/share/generate")
    assert r_gen.status_code == 200
    link = r_gen.get_json()["link"]
    token = link.rstrip("/").split("/")[-1]

    # Link expirado retorna 410
    share = Compartilhamento.query.filter_by(share_token=token).first()
    share.expires_at = 1.0
    db.session.commit()
    assert cliente.get(f"/share/{token}").status_code == 410
    assert cliente.post(f"/share/{token}").status_code == 410


def test_auth_login_e_callback_sem_vazar_traceback(app_ctx, cliente, monkeypatch):
    # Login redireciona para provedor OAuth e grava iniciais
    monkeypatch.setattr(
        oauth.google,
        "authorize_redirect",
        lambda redirect_uri: ("redirecionado", 302),
    )
    r_login = cliente.get("/login/google?iniciais=abcde")
    assert r_login.status_code == 302
    with cliente.session_transaction() as sess:
        assert sess["arcade_initials"] == "ABC"

    # Callback sem sessão retorna 400
    assert cliente.get("/auth/google/callback").status_code == 400

    # Callback com erro não vaza traceback nem tags <pre>
    sid = str(uuid.uuid4())
    jogo = GameState()
    jogo.tempo_total_segundos = 95.0
    jogo.sala_atual = "saida"
    MEMORIA_SESSOES[sid] = jogo
    with cliente.session_transaction() as sess:
        sess["sid"] = sid
        sess["arcade_initials"] = "VBS"

    def _falhar_token():
        raise ValueError("Segredo interno SQL / caminho /server/routes/auth.py")

    monkeypatch.setattr(oauth.google, "authorize_access_token", _falhar_token)
    r_erro = cliente.get("/auth/google/callback")
    assert r_erro.status_code == 500
    texto_erro = r_erro.get_data(as_text=True)
    assert "Traceback" not in texto_erro
    assert "<pre" not in texto_erro
    assert "Segredo interno" not in texto_erro

    # Callback com sucesso grava jogador com hash de e-mail (LGPD) e vincula save
    from security import hash_identificador_email

    salvar_save_web(jogo, sid)
    monkeypatch.setattr(
        oauth.google,
        "authorize_access_token",
        lambda: {"userinfo": {"email": "campeao@vilasboas.local"}},
    )
    r_ok = cliente.get("/auth/google/callback")
    assert r_ok.status_code == 302
    assert "leaderboard=sucesso" in r_ok.headers["Location"]
    email_hash = hash_identificador_email("campeao@vilasboas.local")
    jog = Jogador.query.filter_by(email=email_hash).first()
    assert jog is not None and jog.iniciais == "VBS"
    assert Jogador.query.filter_by(email="campeao@vilasboas.local").first() is None


def test_comando_aceita_pontuacao_natural_portugues():
    req = ComandoRequest(comando="olhar, abrir cofre? socorro!")
    assert req.comando == "olhar, abrir cofre? socorro!"


def test_sqlalchemy_engine_options_preserva_variaveis_pool(monkeypatch):
    from config import Config

    monkeypatch.setenv("DB_POOL_SIZE", "12")
    monkeypatch.setenv("DB_MAX_OVERFLOW", "24")
    monkeypatch.setenv("DB_POOL_RECYCLE", "600")
    monkeypatch.setenv("DB_POOL_TIMEOUT", "45")
    Config.validar_e_configurar()

    opts = Config.SQLALCHEMY_ENGINE_OPTIONS
    assert opts["pool_size"] == 12
    assert opts["max_overflow"] == 24
    assert opts["pool_recycle"] == 600
    assert opts["pool_timeout"] == 45


def test_log_comandos_limitado_aos_ultimos_n():
    from state import MAX_LOG_COMANDOS

    jogo = GameState()
    for i in range(MAX_LOG_COMANDOS + 50):
        jogo.registrar_comando(f"cmd {i}")

    assert len(jogo.log_comandos) == MAX_LOG_COMANDOS
    assert jogo.log_comandos[0] == "cmd 50"


def test_redis_session_store_operacoes():
    try:
        import fakeredis

        cliente_redis = fakeredis.FakeStrictRedis(decode_responses=True)
    except ImportError:

        class _FallbackFakeRedis:
            def __init__(self):
                self.store = {}

            def exists(self, key):
                return 1 if key in self.store else 0

            def get(self, key):
                return self.store.get(key)

            def setex(self, key, ttl, value):
                self.store[key] = value

            def delete(self, key):
                self.store.pop(key, None)

            def flushdb(self):
                self.store.clear()

        cliente_redis = _FallbackFakeRedis()

    store = RedisSessionStore(cliente_redis)
    estado = GameState()
    estado.sala_atual = "cozinha"

    store["sid-1"] = estado
    assert "sid-1" in store
    recuperado = store.get("sid-1")
    assert recuperado is not None and recuperado.sala_atual == "cozinha"
    assert store.get("inexistente") is None

    removido = store.pop("sid-1", None)
    assert removido is not None and "sid-1" not in store

    store["sid-2"] = estado
    store.clear()
    assert "sid-2" not in store
