import uuid, json, time, logging
from flask import Blueprint, request, jsonify, session, redirect, current_app, send_from_directory
from pydantic import ValidationError
from cryptography.fernet import InvalidToken

from extensions import db, limiter, MEMORIA_SESSOES
from state import GameState
from models import Compartilhamento, SaveJogo
from views import imprimir_tela_boot
from villas_boas.engine.core import processar_fluxo_jogo

from routes.helpers import ComandoRequest, WebUIHandler, obter_sid_seguro, gerar_resposta_json
from services.save_service import CIPHER_SUITE, save_existe, carregar_save_web, salvar_save_web, obter_ou_recuperar_jogo

logger = logging.getLogger(__name__)
game_bp = Blueprint('game', __name__)

@game_bp.route("/iniciar", methods=["GET"])
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

@game_bp.route("/comando", methods=["GET", "POST"])
@limiter.limit("60 per minute")
@limiter.limit("500 per hour")
def receber_comando():
    if request.method == "GET":
        return send_from_directory(current_app.config.get("BASE_DIR"), "index.html")

    sid = obter_sid_seguro()
    if not sid or sid not in MEMORIA_SESSOES:
        sid = str(uuid.uuid4())
        session["sid"] = sid
        session.permanent = True
        MEMORIA_SESSOES[sid] = GameState()
        MEMORIA_SESSOES[sid].estado_atual = "AGUARDANDO_DIR"

    jogo = obter_ou_recuperar_jogo(sid, MEMORIA_SESSOES)
    if not jogo:
        return jsonify({"erro": "Sessão expirada. Digite 'iniciar' para recomeçar."}), 404

    jogo.ui_handler = WebUIHandler()
    try:
        requisicao = ComandoRequest(**(request.json or {}))
        comando = requisicao.comando
        session["permite_telemetria"] = requisicao.telemetria
    except ValidationError:
        jogo.ui_handler.buffer.append("@@TYPE@@vermelho@@0@@[ ERRO DE SEGURANÇA ]: Payload inválido.")
        return gerar_resposta_json(jogo), 400

    if comando: jogo.log_comandos.append(comando)

    try:
        processar_fluxo_jogo(comando, jogo, tem_save=save_existe(sid), callback_load_save=lambda j: carregar_save_web(j, sid))
        if getattr(jogo, "estado_atual", "") in ["JOGO", "COMBATE_ANIMATRONICO"]:
            salvar_save_web(jogo, sid)
        MEMORIA_SESSOES[sid] = jogo
    except Exception as e:
        logger.exception("Erro crítico na Engine")
        jogo.ui_handler.buffer.append("@@TYPE@@vermelho@@0@@[ERRO INTERNO]: Falha ao processar a ação.")

    return gerar_resposta_json(jogo)

@game_bp.route("/save/export", methods=["GET"])
@limiter.limit("5 per minute")
def exportar_save():
    sid = obter_sid_seguro()
    jogo = obter_ou_recuperar_jogo(sid, MEMORIA_SESSOES) if sid else None
    if not jogo: return jsonify({"erro": "Nenhum jogo ativo para processar o save."}), 404

    dados_criptografados = CIPHER_SUITE.encrypt(json.dumps(jogo.to_dict(), ensure_ascii=False).encode("utf-8")).decode("utf-8")
    resumo_publico = {"sala": jogo.sala_atual, "hp": jogo.hp, "inventario_tamanho": len(jogo.inventario)}
    return jsonify({"resumo": resumo_publico, "dados_seguros": dados_criptografados})

@game_bp.route("/save/import", methods=["POST"])
@limiter.limit("10 per minute")
def importar_save():
    sid = obter_sid_seguro() or str(uuid.uuid4())
    session["sid"], session.permanent = sid, True

    dados_seguros = (request.json or {}).get("dados_seguros")
    if not dados_seguros or not isinstance(dados_seguros, str): return jsonify({"erro": "Formato não suportado."}), 400
    if len(dados_seguros) > 50000: return jsonify({"erro": "Tamanho excede o permitido."}), 413

    try:
        dados_save = json.loads(CIPHER_SUITE.decrypt(dados_seguros.encode("utf-8")).decode("utf-8"))
        novo_jogo = GameState.from_dict(dados_save)
        novo_jogo.ui_handler = WebUIHandler()
        MEMORIA_SESSOES[sid] = novo_jogo
        salvar_save_web(novo_jogo, sid)
        return jsonify({"sucesso": True, "mensagem": "Save importado."})
    except InvalidToken:
        return jsonify({"erro": "[ ERRO CRÍTICO ] ASSINATURA INVÁLIDA."}), 403
    except (ValueError, TypeError, KeyError):
        return jsonify({"erro": "Arquivo de save inválido."}), 400

@game_bp.route("/share/generate", methods=["GET"])
@limiter.limit("5 per minute")
def gerar_link():
    sid = obter_sid_seguro()
    if not sid: return jsonify({"erro": "Sem save ativo"}), 401
    share = Compartilhamento(original_sid=sid, expires_at=time.time() + 3600)
    db.session.add(share)
    db.session.commit()
    return jsonify({"link": f"{request.host_url}share/{share.share_token}", "mensagem": "Link válido por 1 hora."})

@game_bp.route("/share/<share_token>", methods=["GET"])
def carregar_compartilhado(share_token):
    share = db.session.get(Compartilhamento, share_token)
    if not share: return "Link inválido.", 404
    if time.time() > share.expires_at:
        db.session.delete(share); db.session.commit(); return "Expirado.", 410

    original = db.session.get(SaveJogo, share.original_sid)
    
    try:
        dados = json.loads(CIPHER_SUITE.decrypt(original.dados.encode("utf-8")).decode("utf-8"))
        novo_sid = str(uuid.uuid4())
        db.session.add(SaveJogo(sid=novo_sid, dados=original.dados))
        db.session.commit()
        session["sid"], session.permanent = novo_sid, True
        MEMORIA_SESSOES[novo_sid] = GameState.from_dict(dados)
        return redirect("/")
    except (ValueError, TypeError, KeyError):
        return "Save corrompido.", 500

@game_bp.route("/achievements", methods=["GET"])
def listar_conquistas():
    jogo = obter_ou_recuperar_jogo(obter_sid_seguro(), MEMORIA_SESSOES)
    conquistas = getattr(jogo, "conquistas", []) if jogo else []
    return jsonify({"conquistas": conquistas, "total": len(conquistas)})