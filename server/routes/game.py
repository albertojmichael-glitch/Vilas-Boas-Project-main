import html
import json
import logging
import time
import uuid
from datetime import UTC, datetime

from cryptography.fernet import InvalidToken
from extensions import MEMORIA_SESSOES, db, limiter
from flask import (
    Blueprint,
    Response,
    jsonify,
    redirect,
    request,
    session,
)
from models import Compartilhamento, SaveJogo
from pydantic import ValidationError
from security import (
    FORMATO_SAVE,
    VERSAO_SAVE,
    assinar_save,
    requer_origem_valida,
    verificar_assinatura_save,
)
from services.save_service import (
    CIPHER_SUITE,
    carregar_estado_validado,
    carregar_save_web,
    obter_ou_recuperar_jogo,
    salvar_save_web,
    save_existe,
    serializar_estado,
    to_uuid,
)
from services.save_validacao import SaveInvalido, validar_estado_save
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
TAMANHO_MAXIMO_SAVE = 200_000  # bytes; o save legível é maior que o token antigo
game_bp = Blueprint("game", __name__)


@game_bp.route("/iniciar", methods=["POST"])
@limiter.limit("10 per minute")
@requer_origem_valida
def iniciar_jogo():
    # O SID do cookie assinado identifica também o autosave, inclusive após importar.
    sid = obter_sid_seguro() or str(uuid.uuid4())
    session.clear()
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
    if not sid:
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
        if (
            getattr(jogo, "estado_atual", "") in ["JOGO", "COMBATE_ANIMATRONICO"]
            and snapshot_antes != jogo.snapshot_persistencia()
        ):
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
    """Exporta o save em JSON legível e assinado pelo servidor.

    Nada é escondido do jogador (ele controla o navegador de qualquer forma);
    a assinatura HMAC só garante que o arquivo não foi alterado."""
    sid = obter_sid_seguro()
    jogo = obter_ou_recuperar_jogo(sid, MEMORIA_SESSOES) if sid else None
    if not jogo:
        return jsonify({"erro": "Nenhum jogo ativo para processar o save."}), 404

    try:
        estado = json.loads(serializar_estado(jogo))
    except SaveInvalido as exc:
        return jsonify({"erro": f"O jogo atual não pode ser exportado: {exc}"}), 409

    arquivo = {
        "formato": FORMATO_SAVE,
        "versao": VERSAO_SAVE,
        "criado_em": datetime.now(UTC).isoformat(timespec="seconds"),
        "resumo": {
            "sala": jogo.sala_atual,
            "hp": jogo.hp,
            "dificuldade": jogo.dificuldade_escolhida,
            "inventario_tamanho": len(jogo.inventario),
        },
        "estado": estado,
        "assinatura": assinar_save(estado),
    }
    return Response(
        json.dumps(arquivo, ensure_ascii=False, indent=2), mimetype="application/json"
    )


def _estado_do_arquivo(payload):
    """Extrai o estado de um save importado (v2 assinado ou v1 legado).

    Devolve (estado, None) ou (None, (mensagem, status_http))."""
    if not isinstance(payload, dict):
        return None, ("Formato não suportado.", 400)

    if payload.get("formato") == FORMATO_SAVE:
        if payload.get("versao") != VERSAO_SAVE:
            return None, ("Versão de save não suportada.", 400)
        estado, assinatura = payload.get("estado"), payload.get("assinatura")
        if not verificar_assinatura_save(estado, assinatura):
            return None, (
                "[ ERRO CRÍTICO ] ASSINATURA INVÁLIDA: o arquivo foi alterado.",
                403,
            )
        return estado, None

    dados_seguros = payload.get("dados_seguros")  # formato v1 (Fernet), legado
    if isinstance(dados_seguros, str):
        try:
            texto = CIPHER_SUITE.decrypt(dados_seguros.encode("utf-8")).decode("utf-8")
            return json.loads(texto), None
        except InvalidToken:
            return None, ("[ ERRO CRÍTICO ] ASSINATURA INVÁLIDA.", 403)
        except (json.JSONDecodeError, UnicodeDecodeError):
            return None, ("Arquivo de save inválido.", 400)

    return None, ("Formato não suportado.", 400)


@game_bp.route("/save/import", methods=["POST"])
@limiter.limit("10 per minute")
@requer_origem_valida
def importar_save():
    if (request.content_length or 0) > TAMANHO_MAXIMO_SAVE:
        return jsonify({"erro": "Tamanho excede o permitido."}), 413
    payload = request.get_json(silent=True)

    estado, erro = _estado_do_arquivo(payload)
    if erro:
        return jsonify({"erro": erro[0]}), erro[1]

    try:
        novo_jogo = validar_estado_save(estado, permitir_dev=False)
    except SaveInvalido as exc:
        return jsonify({"erro": f"Save rejeitado: {exc}"}), 400

    sid = obter_sid_seguro() or str(uuid.uuid4())
    session["sid"], session.permanent = sid, True
    novo_jogo.ui_handler = WebUIHandler()
    if not salvar_save_web(novo_jogo, sid):
        return jsonify({"erro": "Falha ao gravar o save importado."}), 500
    MEMORIA_SESSOES[sid] = novo_jogo
    return jsonify({"sucesso": True, "mensagem": "Save importado."})


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
            (
                "<!DOCTYPE html><html lang='pt-BR'><head><meta charset='utf-8'>"
                "<title>Vilas Boas OS - Save Compartilhado</title></head>"
                "<body style='background:#000;color:#0f0;font-family:monospace;padding:2rem;'>"
                "<h1>[ VILAS BOAS OS - SAVE COMPARTILHADO ]</h1>"
                "<p>Confirme abaixo para carregar este save na sua sessão atual.</p>"
                f"<form method='POST' action='/share/{token_seguro}'>"
                "<button type='submit' style='background:#0f0;color:#000;border:0;padding:0.5rem 1rem;font-family:monospace;cursor:pointer;'>"
                "CARREGAR SAVE</button></form></body></html>"
            ),
            200,
        )

    from security import validar_origem_request

    if not validar_origem_request():
        return jsonify({"erro": "Origem não autorizada."}), 403

    try:
        novo_jogo = carregar_estado_validado(original.dados, permitir_dev=False)
        if novo_jogo is None:
            return "Os dados estruturais do save estão corrompidos e ilegíveis.", 500

        novo_sid = uuid.uuid4()
        if not salvar_save_web(novo_jogo, novo_sid):
            return "Não foi possível clonar o save compartilhado.", 500

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
