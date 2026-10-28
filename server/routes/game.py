import html
import json
import logging
import time
import uuid

from cryptography.fernet import InvalidToken
from extensions import MEMORIA_SESSOES, db, limiter
from flask import (
    Blueprint,
    jsonify,
    redirect,
    request,
    session,
)
from models import Compartilhamento, SaveJogo
from pydantic import ValidationError
from security import requer_origem_valida
from services.save_service import (
    CIPHER_SUITE,
    carregar_save_web,
    obter_ou_recuperar_jogo,
    salvar_save_web,
    save_existe,
    to_uuid,
)
from state import GameState
from views import imprimir_tela_boot
from villas_boas.engine.core import processar_fluxo_jogo

from routes.helpers import (
    ComandoRequest,
    WebUIHandler,
    gerar_resposta_json,
    obter_sid_seguro,
)

logger = logging.getLogger(__name__)
game_bp = Blueprint("game", __name__)


@game_bp.route("/iniciar", methods=["POST"])
@limiter.limit("10 per minute")
@requer_origem_valida
def iniciar_jogo():
    sid_antigo = session.get("sid")
    if sid_antigo and sid_antigo in MEMORIA_SESSOES:
        MEMORIA_SESSOES.pop(sid_antigo, None)

    session.clear()
    sid = str(uuid.uuid4())
    session["sid"] = sid
    session.permanent = True

    jogo = GameState()
    jogo.ui_handler = WebUIHandler()
    jogo.estado_atual = "AGUARDANDO_DIR"
    MEMORIA_SESSOES[sid] = jogo

    jogo.ui_handler.limpar()
    imprimir_tela_boot(jogo.ui_handler)

    resposta = gerar_resposta_json(jogo)
    resposta.headers["Cache-Control"] = "no-store"
    return resposta


@game_bp.route("/comando", methods=["POST"])
@limiter.limit("120 per minute")
@limiter.limit("500 per hour")
@requer_origem_valida
def receber_comando():
    sid = obter_sid_seguro()
    if not sid or sid not in MEMORIA_SESSOES:
        sid = str(uuid.uuid4())
        session["sid"] = sid
        session.permanent = True
        MEMORIA_SESSOES[sid] = GameState()
        MEMORIA_SESSOES[sid].estado_atual = "AGUARDANDO_DIR"

    jogo = obter_ou_recuperar_jogo(sid, MEMORIA_SESSOES)
    if not jogo:
        return jsonify(
            {"erro": "Sessão expirada. Digite 'iniciar' para recomeçar."}
        ), 404

    jogo.ui_handler = WebUIHandler()
    try:
        requisicao = ComandoRequest(**(request.json or {}))
        comando = requisicao.comando
        session["permite_telemetria"] = requisicao.telemetria
    except ValidationError:
        jogo.ui_handler.buffer.append(
            "@@TYPE@@vermelho@@0@@[ ERRO DE SEGURANÇA ]: Payload inválido."
        )
        return gerar_resposta_json(jogo), 400

    if comando:
        jogo.registrar_comando(comando)

    try:
        snapshot_antes = jogo.snapshot_persistencia()
        processar_fluxo_jogo(
            comando,
            jogo,
            tem_save=save_existe(sid),
            callback_load_save=lambda j: carregar_save_web(j, sid),
        )
        if getattr(jogo, "estado_atual", "") in ["JOGO", "COMBATE_ANIMATRONICO"]:
            if snapshot_antes != jogo.snapshot_persistencia():
                salvar_save_web(jogo, sid)
        MEMORIA_SESSOES[sid] = jogo
    except Exception:
        logger.exception("Erro crítico na Engine")
        jogo.ui_handler.buffer.append(
            "@@TYPE@@vermelho@@0@@[ERRO INTERNO]: Falha ao processar a ação."
        )

    return gerar_resposta_json(jogo)


@game_bp.route("/save/export", methods=["GET"])
@limiter.limit("5 per minute")
def exportar_save():
    sid = obter_sid_seguro()
    jogo = obter_ou_recuperar_jogo(sid, MEMORIA_SESSOES) if sid else None
    if not jogo:
        return jsonify({"erro": "Nenhum jogo ativo para processar o save."}), 404

    dados_criptografados = CIPHER_SUITE.encrypt(
        json.dumps(jogo.to_dict(), ensure_ascii=False).encode("utf-8")
    ).decode("utf-8")
    resumo_publico = {
        "sala": jogo.sala_atual,
        "hp": jogo.hp,
        "inventario_tamanho": len(jogo.inventario),
    }
    return jsonify({"resumo": resumo_publico, "dados_seguros": dados_criptografados})


@game_bp.route("/save/import", methods=["POST"])
@limiter.limit("10 per minute")
@requer_origem_valida
def importar_save():
    sid = obter_sid_seguro() or str(uuid.uuid4())
    session["sid"], session.permanent = sid, True

    dados_seguros = (request.json or {}).get("dados_seguros")
    if not dados_seguros or not isinstance(dados_seguros, str):
        return jsonify({"erro": "Formato não suportado."}), 400
    if len(dados_seguros) > 50000:
        return jsonify({"erro": "Tamanho excede o permitido."}), 413

    try:
        dados_save = json.loads(
            CIPHER_SUITE.decrypt(dados_seguros.encode("utf-8")).decode("utf-8")
        )
        novo_jogo = GameState.from_dict(dados_save)
        novo_jogo.ui_handler = WebUIHandler()
        MEMORIA_SESSOES[sid] = novo_jogo
        salvar_save_web(novo_jogo, sid)
        return jsonify({"sucesso": True, "mensagem": "Save importado."})
    except InvalidToken:
        return jsonify({"erro": "[ ERRO CRÍTICO ] ASSINATURA INVÁLIDA."}), 403
    except (ValueError, TypeError, KeyError):
        return jsonify({"erro": "Arquivo de save inválido."}), 400


@game_bp.route("/share/generate", methods=["POST"])
@limiter.limit("5 per hour")
@requer_origem_valida
def gerar_link():
    sid = obter_sid_seguro()
    if not sid:
        return jsonify({"erro": "Sem save ativo"}), 401
    share = Compartilhamento(original_sid=sid, expires_at=time.time() + 3600)
    db.session.add(share)
    db.session.commit()
    return jsonify(
        {
            "link": f"{request.host_url}share/{share.share_token}",
            "mensagem": "Link válido por 1 hora.",
        }
    )


@game_bp.route("/share/<share_token>", methods=["GET", "POST"])
def carregar_compartilhado(share_token):
    share = db.session.execute(
        db.select(Compartilhamento).filter_by(share_token=share_token)
    ).scalar_one_or_none()

    if not share:
        return "Link inválido ou inexistente.", 404

    if time.time() > share.expires_at:
        if request.method == "POST":
            db.session.delete(share)
            db.session.commit()
        return "Este link temporário expirou e foi removido do sistema.", 410

    chave_original = to_uuid(share.original_sid)
    original = db.session.get(SaveJogo, chave_original) if chave_original else None

    if not original:
        return (
            "O save original associado a este link foi destruído ou deixou de existir.",
            404,
        )

    if request.method == "GET":
        token_seguro = html.escape(share_token)
        return (
            "<!DOCTYPE html><html lang='pt-BR'><head><meta charset='utf-8'>"
            "<title>Vilas Boas OS - Save Compartilhado</title></head>"
            "<body style='background:#000;color:#0f0;font-family:monospace;padding:2rem;'>"
            "<h1>[ VILAS BOAS OS - SAVE COMPARTILHADO ]</h1>"
            "<p>Confirme abaixo para carregar este save na sua sessão atual.</p>"
            f"<form method='POST' action='/share/{token_seguro}'>"
            "<button type='submit' style='background:#0f0;color:#000;border:0;padding:0.5rem 1rem;font-family:monospace;cursor:pointer;'>"
            "CARREGAR SAVE</button></form></body></html>",
            200,
        )

    from security import validar_origem_request

    if not validar_origem_request():
        return jsonify({"erro": "Origem não autorizada."}), 403

    try:
        dados_descriptografados = CIPHER_SUITE.decrypt(
            original.dados.encode("utf-8")
        ).decode("utf-8")
        dados = json.loads(dados_descriptografados)
        novo_jogo = GameState.from_dict(dados)

        novo_sid = uuid.uuid4()
        db.session.add(SaveJogo(sid=novo_sid, dados=original.dados))
        db.session.commit()

        session["sid"] = str(novo_sid)
        session.permanent = True
        MEMORIA_SESSOES[str(novo_sid)] = novo_jogo

        return redirect("/")
    except Exception:
        db.session.rollback()
        logger.exception(
            "Falha severa na decodificação e clonagem de save compartilhado."
        )
        return "Os dados estruturais do save estão corrompidos e ilegíveis.", 500


@game_bp.route("/achievements", methods=["GET"])
def listar_conquistas():
    jogo = obter_ou_recuperar_jogo(obter_sid_seguro(), MEMORIA_SESSOES)
    conquistas = getattr(jogo, "conquistas", []) if jogo else []
    return jsonify({"conquistas": conquistas, "total": len(conquistas)})
