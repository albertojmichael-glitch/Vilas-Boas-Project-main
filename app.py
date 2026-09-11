import functools
import json
import logging
import os
import sys  
import time
import secrets
import uuid
import base64
import hashlib
from cryptography.fernet import Fernet, InvalidToken
from datetime import timedelta
from logging.handlers import RotatingFileHandler
from pathlib import Path

import pymongo
from cachetools import TTLCache
from flask import Flask, jsonify, redirect, request, send_from_directory, session
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from pydantic import BaseModel, Field, ValidationError
from pymongo import MongoClient
from bson.objectid import ObjectId
from bson.errors import InvalidId

try:
    import redis
except ImportError:
    redis = None


from villas_boas.engine.core import processar_fluxo_jogo
from state import GameState
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET, UIHandler
from views import imprimir_tela_boot

BASE_DIR = os.path.abspath(os.path.dirname(__file__))


IS_PRODUCTION = bool(
    os.environ.get("FLASK_ENV") == "production"
    or os.environ.get("RENDER")
    or os.environ.get("RAILWAY_STATIC_URL")
    or os.environ.get("PROD")
)



SECRET_KEY = os.environ.get("SECRET_KEY") or os.environ.get("FLASK_SECRET_KEY")
ADMIN_TOKEN = os.environ.get("ADMIN_TOKEN")


if IS_PRODUCTION and not (SECRET_KEY and ADMIN_TOKEN):
    print("➣ ERRO FATAL: SECRET_KEY e/ou ADMIN_TOKEN não encontrados no ambiente de produção! ")
    sys.exit(1) 



app = Flask(__name__, static_folder=BASE_DIR, static_url_path="/")

app.secret_key = SECRET_KEY or "DEV_SECRET_DO_NOT_USE_IN_PROD_1982"
app.config["PERMANENT_SESSION_LIFETIME"] = timedelta(days=30)
app.config["MAX_CONTENT_LENGTH"] = 1 * 1024 * 1024 


_key_hash = hashlib.sha256((app.secret_key).encode()).digest()
CIPHER_SUITE = Fernet(base64.urlsafe_b64encode(_key_hash))


if IS_PRODUCTION:
    app.config.update(
        SESSION_COOKIE_SECURE=True,     
        SESSION_COOKIE_HTTPONLY=True,   
        SESSION_COOKIE_SAMESITE='Lax',  
    )
    print("🔒 Segurança de Cookies: Modo Produção ativado (Secure=True).")
else:
    app.config.update(
        SESSION_COOKIE_SECURE=False,
        SESSION_COOKIE_HTTPONLY=True,
        SESSION_COOKIE_SAMESITE='Lax',
    )
    print("🔓 Segurança de Cookies: Modo Desenvolvimento (Secure=False).")


logging.basicConfig(
    level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s"
)
log_file = os.path.join(BASE_DIR, "villas_boas.log")
file_handler = RotatingFileHandler(
    log_file, maxBytes=10_000_000, backupCount=5, encoding="utf-8"
)
file_handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)s] %(message)s"))
logging.getLogger().addHandler(file_handler)
logger = logging.getLogger(__name__)


SAVES_DIR_ENV = os.environ.get("SAVES_DIR", os.path.join(BASE_DIR, "saves"))
os.makedirs(SAVES_DIR_ENV, exist_ok=True)


if not ADMIN_TOKEN:
    ADMIN_TOKEN = secrets.token_urlsafe(32)
    logger.warning("⚠ ADMIN_TOKEN não definido no ambiente! Uma senha aleatória segura foi gerada para esta sessão.")


MONGO_URI = os.environ.get("MONGO_URI")
if MONGO_URI:
    mongo_client = MongoClient(MONGO_URI)
    db = mongo_client["villasboas_db"]
    saves_collection = db["saves"]
    telemetry_collection = db["telemetry"]
    shares_collection = db["shares"]
    logger.info("✅ Conectado ao MongoDB com sucesso...")
else:
    mongo_client = None
    logger.warning("⚠ Rodando sem Banco de Dados MongoDB. Usando arquivos locais.")

CORS(app, supports_credentials=True)
limiter = Limiter(key_func=get_remote_address, app=app, storage_uri="memory://")


REDIS_URL = os.environ.get("REDIS_URL")



class RedisSessionStore:
    def __init__(self, client):
        self.client = client

    def __contains__(self, key):
        return self.client.exists(key) > 0

    def __getitem__(self, key):
        data = self.client.get(key)
        if data:
            return GameState.from_dict(json.loads(data))
        raise KeyError(key)

    def __setitem__(self, key, value):
        
        self.client.setex(key, 3600, json.dumps(value.to_dict()))


if REDIS_URL and redis is not None:
    try:
        redis_client = redis.Redis.from_url(REDIS_URL, decode_responses=True)
        redis_client.ping()

        
        MEMORIA_SESSOES = RedisSessionStore(redis_client)
        logger.info("Conectado ao Redis com sucesso.")

    except Exception:
        logger.exception(
            "Falha inesperada ao conectar no Redis — caindo para TTLCache."
        )
        MEMORIA_SESSOES = TTLCache(maxsize=1000, ttl=3600)
else:
    logger.info("Usando TTLCache na memória RAM local.")
    MEMORIA_SESSOES = TTLCache(maxsize=1000, ttl=3600)


class ComandoRequest(BaseModel):
    comando: str = Field(
        default="", 
        max_length=256, 
        
        pattern=r"^[a-zA-Z0-9\s\"\'\-\_áéíóúâêôãõçÁÉÍÓÚÂÊÔÃÕÇ\.\:]+$"
    )
    telemetria: bool = Field(default=True)


def obter_sid_seguro():
    """Garante que o SID lido do cookie é um UUID válido e não um script de injeção"""
    sid_bruto = session.get("sid")
    if not sid_bruto:
        return None
    try:
        return str(uuid.UUID(str(sid_bruto)))
    except (ValueError, TypeError, AttributeError):
        logger.warning(f"Alerta de Segurança: SID inválido/adulterado: {sid_bruto}")
        return None

def requer_admin(f):
    @functools.wraps(f)
    def decorated(*args, **kwargs):
        token = request.headers.get("X-Admin-Token") or request.args.get("token")
        ip_origem = get_remote_address()
        
        if not token or token != ADMIN_TOKEN:
            
            logger.warning(f"TENTATIVA INVASÃO ADMIN: IP {ip_origem} tentou acessar {request.path}")
            return jsonify({"erro": "Acesso negado. Credenciais inválidas."}), 403
            
        
        logger.info(f" ACESSO ADMIN: IP {ip_origem} visualizando {request.path}")
        return f(*args, **kwargs)
    return decorated


class WebUIHandler(UIHandler):
    def __init__(self):
        self.buffer = []

    def limpar(self):
        self.buffer.append("@@CLEAR@@")

    def pausar(self, segs):
        ms = int(segs * 1000)
        self.buffer.append(f"@@PAUSE@@{ms}")

    def exibir(self, texto):
        self.animar(texto, 0.015)

    def animar(self, texto, tempo=0.03, cor="", jogo=None):
        cor_nome = "verde"
        if cor == DOS_BRANCO:
            cor_nome = "branco"
        elif cor == DOS_AMARELO:
            cor_nome = "amarelo"
        elif cor == DOS_VERMELHO:
            cor_nome = "vermelho"

        if jogo and getattr(jogo, "fast_mode", False):
            tempo = 0
        ms = int(tempo * 1000)
        self.buffer.append(f"@@TYPE@@{cor_nome}@@{ms}@@{texto}")

    def obter_input(self, prompt_text):
        return ""


def ansi_para_html(texto_ansi):
    import re
    import html  
    
    mapa_cores = {
        DOS_VERDE: "verde", DOS_BRANCO: "branco",
        DOS_AMARELO: "amarelo", DOS_VERMELHO: "vermelho",
    }
    padrao = re.compile("(" + "|".join(re.escape(c) for c in list(mapa_cores.keys()) + [RESET]) + ")")
    partes = padrao.split(texto_ansi)
    html_out, aberto = [], False
    
    for parte in partes:
        if parte in mapa_cores:
            if aberto: html_out.append("</span>")
            html_out.append(f'<span class="{mapa_cores[parte]}">')
            aberto = True
        elif parte == RESET:
            if aberto:
                html_out.append("</span>")
                aberto = False
        else:
            
            html_out.append(html.escape(parte))
            
    if aberto: html_out.append("</span>")
    return "".join(html_out)


def obter_caminho_autosave(sid):
    return Path(SAVES_DIR_ENV) / f"autosave_{sid}.json"


def registrar_telemetria(evento, sala, dificuldade, detalhes="", jogo=None):
    if not mongo_client or not session.get("permite_telemetria", True):
        return

    try:
        evento_seguro = str(evento)[:50]
        sala_segura = str(sala)[:50]
        dif_segura = str(dificuldade)[:50]
        det_seguros = str(detalhes)[:256]
        
        if any(v.startswith('$') for v in [evento_seguro, sala_segura, dif_segura]):
            return

        
        doc = {
            "evento": evento_seguro,
            "sala": sala_segura,
            "dificuldade": dif_segura,
            "detalhes": det_seguros,
            "timestamp": time.time(),
        }
        
        
        if jogo:
            doc.update({
                "hp_restante": getattr(jogo, 'hp', 0),
                "luz_restante": getattr(jogo, 'turnos_luz', 0),
                "nivel_barulho": getattr(jogo, 'nivel_barulho', 0),
                "inventario_qtd": len(getattr(jogo, 'inventario', [])),
                "bolsas": getattr(jogo, 'bolsas_coletadas', 0),
                "log_comandos": getattr(jogo, 'log_comandos', [])
            })

        telemetry_collection.insert_one(doc)
        
    
    except Exception as e:  # noqa: BLE001
        logger.error(f"Erro na telemetria: Falha de sanitização. Detalhes: {e}")



def carregar_save_web(jogo):
    sid = obter_sid_seguro()
    if not sid:
        return False

    def processar_dados_save(dados_brutos):
        """Descriptografa se for texto seguro, ou lê normalmente se for um save antigo."""
        if isinstance(dados_brutos, str):
            try:
                
                dados_json = CIPHER_SUITE.decrypt(dados_brutos.encode("utf-8")).decode("utf-8")
                return json.loads(dados_json)
            except InvalidToken:
                logger.error("Tentativa de carregar save com chave de criptografia inválida!")
                return None
            except json.JSONDecodeError:
                pass 
        
        return dados_brutos if isinstance(dados_brutos, dict) else json.loads(dados_brutos)

    if mongo_client:
        try:
            doc = saves_collection.find_one({"sid": sid})
            if doc and "dados" in doc:
                dados = processar_dados_save(doc["dados"])
                if dados:
                    novo_jogo = GameState.from_dict(dados)
                    for k, v in novo_jogo.__dict__.items():
                        if k != "ui_handler":
                            setattr(jogo, k, v)
                    return True
        except Exception:
            logger.exception("Erro ao buscar save criptografado no MongoDB")
    else:
        caminho = obter_caminho_autosave(sid)
        if caminho.exists():
            try:
                conteudo = caminho.read_text(encoding="utf-8")
                dados = processar_dados_save(conteudo)
                if dados:
                    novo_jogo = GameState.from_dict(dados)
                    for k, v in novo_jogo.__dict__.items():
                        if k != "ui_handler":
                            setattr(jogo, k, v)
                    return True
            except Exception:
                logger.exception("Erro ao carregar save local")

    return False

def salvar_save_web(jogo):
    sid = obter_sid_seguro()
    if not sid:
        return

    
    dados_json = json.dumps(jogo.to_dict(), ensure_ascii=False)
    dados_criptografados = CIPHER_SUITE.encrypt(dados_json.encode("utf-8")).decode("utf-8")

    if mongo_client:
        try:
            saves_collection.update_one(
                {"sid": sid},
                {"$set": {"sid": sid, "dados": dados_criptografados}},
                upsert=True,
            )
        except Exception:
            logger.exception("Erro ao salvar progresso blindado no MongoDB")
    else:
        try:
            caminho = obter_caminho_autosave(sid)
            caminho.write_text(dados_criptografados, encoding="utf-8")
        except Exception:
            logger.exception("Erro ao gerar autosave local blindado")


def gerar_resposta_json(jogo):
    linhas = []
    
    # 1. Captura e converte as linhas para HTML
    if jogo and hasattr(jogo, "ui_handler") and hasattr(jogo.ui_handler, "buffer"):
        linhas = [
            ansi_para_html(linha)
            for linha in jogo.ui_handler.buffer
            if linha.strip() != ""
        ]
        jogo.ui_handler.buffer.clear()

    # 2. Captura as conquistas do turno e limpa a mochila de envio
    conquistas_enviadas = getattr(jogo, 'novas_conquistas_turno', []).copy()
    if hasattr(jogo, 'novas_conquistas_turno'):
        jogo.novas_conquistas_turno.clear()

    # --- NOVO: Lógica restaurada para a "sala" no HUD ---
    estado_jogo = getattr(jogo, "estado_atual", "")
    if estado_jogo in ["MENU", "AGUARDANDO_DIR"]:
        sala_exibicao = "SISTEMA"
    else:
        sala_exibicao = getattr(jogo, "sala_atual", "SISTEMA").upper()

    # 3. Monta o pacote final
    resposta = {
        "linhas": linhas, 
        "estado": {
            "hp": getattr(jogo, "hp", 0),
            "inventario": getattr(jogo, "inventario", []),
            "luz_restante": getattr(jogo, "turnos_luz", 0),
            "bolsas_coletadas": getattr(jogo, "bolsas_coletadas", 0),
            "estado_jogo": estado_jogo,
            "tempo_final": getattr(jogo, "tempo_total_segundos", 0.0),
            "sala": sala_exibicao # <--- Resolvido!
        },
        "novas_conquistas": conquistas_enviadas
    }
    return jsonify(resposta)


@app.route("/")
def raiz():
    return send_from_directory(BASE_DIR, "index.html")


@app.route("/ping")
def ping():
    return "Estou vivo!", 200


@app.route("/style.css")
def serve_css():
    if os.path.exists(os.path.join(BASE_DIR, "style.min.css")):
        return send_from_directory(BASE_DIR, "style.min.css")
    return send_from_directory(BASE_DIR, "style.css")


@app.route("/script.js")
def serve_js():
    if os.path.exists(os.path.join(BASE_DIR, "script.min.js")):
        return send_from_directory(BASE_DIR, "script.min.js")
    return send_from_directory(BASE_DIR, "script.js")


@app.errorhandler(404)
@app.errorhandler(405)
def page_not_found(e):
    return send_from_directory(BASE_DIR, "index.html")


@app.route("/iniciar", methods=["GET"])
def iniciar_jogo():
    session.clear()
    sid = str(uuid.uuid4())
    session["sid"] = sid
    session.permanent = True
    session.modified = True

    jogo = GameState()
    jogo.ui_handler = WebUIHandler()
    jogo.estado_atual = "AGUARDANDO_DIR"

    MEMORIA_SESSOES[sid] = jogo

    imprimir_tela_boot(jogo.ui_handler)

    resposta = gerar_resposta_json(jogo)
    resposta.headers["Cache-Control"] = "no-store"
    return resposta


@app.route("/comando", methods=["GET", "POST"])
@limiter.limit("60 per minute")
@limiter.limit("500 per hour")
def receber_comando():
    if request.method == "GET":
        return send_from_directory(BASE_DIR, "index.html")

    sid = obter_sid_seguro()

    if not sid or sid not in MEMORIA_SESSOES:
        sid = str(uuid.uuid4())
        session["sid"] = sid
        session.permanent = True
        session.modified = True
        MEMORIA_SESSOES[sid] = GameState()
        MEMORIA_SESSOES[sid].estado_atual = "AGUARDANDO_DIR"

    jogo = MEMORIA_SESSOES[sid]
    jogo.ui_handler = WebUIHandler()

    dados = request.json or {}
    try:
        requisicao = ComandoRequest(**dados)
        comando = requisicao.comando
        session["permite_telemetria"] = requisicao.telemetria
    except ValidationError:
        # Se falhar na segurança, avisa a UI e aborta o turno
        jogo.ui_handler.buffer.append("@@TYPE@@vermelho@@0@@[ ERRO DE SEGURANÇA ]: Payload inválido.")
        return gerar_resposta_json(jogo), 400

    tem_save = obter_caminho_autosave(sid).exists()

    # Grava a fita de replay
    if comando:
        jogo.log_comandos.append(comando)

    try:
        # O processar_fluxo_jogo é o verdadeiro Maestro. Ele cuida de tudo!
        processar_fluxo_jogo(
            comando, jogo, tem_save=tem_save, callback_load_save=carregar_save_web
        )

        if getattr(jogo, "estado_atual", "") in ["JOGO", "COMBATE_ANIMATRONICO"]:
            salvar_save_web(jogo)

        MEMORIA_SESSOES[sid] = jogo

    except Exception as e:
        logger.exception("Erro critico na Engine")
        jogo.ui_handler.buffer.append(
            "@@TYPE@@vermelho@@0@@[ERRO INTERNO]: O servidor falhou ao processar a ação."
        )
        if app.debug:
            jogo.ui_handler.buffer.append(
                f"@@TYPE@@amarelo@@0@@Detalhes (Apenas em Debug): {e!s}"
            )

    # Devolvemos a resposta finalizada para o frontend
    return gerar_resposta_json(jogo)



@app.route("/save/export", methods=["GET"])
@limiter.limit("5 per minute")
def exportar_save():
    sid = obter_sid_seguro()
    if not sid or sid not in MEMORIA_SESSOES:
        return jsonify({"erro": "Nenhuma sessão ativa."}), 404

    jogo = MEMORIA_SESSOES[sid]
    
    return jsonify(jogo.to_dict())


@app.route("/save/import", methods=["POST"])
@limiter.limit("10 per minute")
def importar_save():
    sid = obter_sid_seguro()
    if not sid:
        
        sid = str(uuid.uuid4())
        session["sid"] = sid
        session.permanent = True

    dados = request.json
    if not dados:
        return jsonify({"erro": "Nenhum dado recebido."}), 400

    try:
        
        novo_jogo = GameState.from_dict(dados)
        novo_jogo.ui_handler = WebUIHandler()  

        MEMORIA_SESSOES[sid] = novo_jogo
        salvar_save_web(novo_jogo)

        return jsonify({"sucesso": True, "mensagem": "Save importado com sucesso."})
    except (ValidationError, ValueError, TypeError, KeyError) as e:
        logger.error(f"Erro ao importar save via UI: {e}")
        return jsonify(
            {
                "erro": "Arquivo de save inválido, corrompido ou de uma versão incompatível."
            }
        ), 400



@app.route("/achievements", methods=["GET"])
def listar_conquistas():
    sid = obter_sid_seguro()
    if not sid or sid not in MEMORIA_SESSOES:
        return jsonify({"erro": "Sessão não encontrada", "conquistas": []})

    jogo = MEMORIA_SESSOES[sid]
    conquistas = getattr(jogo, "conquistas", [])
    return jsonify({"conquistas": conquistas, "total": len(conquistas)})


@app.route("/admin/analytics", methods=["GET"])
@requer_admin
def ver_telemetria():
    if not mongo_client:
        return jsonify({"erro": "Sem banco de dados conectado."}), 400
        
    mortes = telemetry_collection.count_documents({"evento": "MORTE"})
    vitorias = telemetry_collection.count_documents({"evento": "VITORIA"})
    
    
    pipeline_salas = [
        {"$match": {"evento": "MORTE"}},                 
        {"$group": {"_id": "$sala", "total": {"$sum": 1}}}, 
        {"$sort": {"total": -1}},                         
        {"$limit": 5}                                     
    ]
    ranking_mortes = list(telemetry_collection.aggregate(pipeline_salas))
    
    
    pipeline_motivos = [
        {"$match": {"evento": "MORTE"}},
        {"$group": {"_id": "$detalhes", "total": {"$sum": 1}}},
        {"$sort": {"total": -1}},
        {"$limit": 5}
    ]
    ranking_motivos = list(telemetry_collection.aggregate(pipeline_motivos))

    
    pipeline_medias = [
        {"$match": {"evento": "MORTE"}},
        {"$group": {
            "_id": None,
            "avg_luz": {"$avg": "$luz_restante"},
            "avg_barulho": {"$avg": "$nivel_barulho"},
            "avg_inv": {"$avg": "$inventario_qtd"}
        }}
    ]
    medias_raw = list(telemetry_collection.aggregate(pipeline_medias))
    medias = medias_raw[0] if medias_raw else {"avg_luz": 0, "avg_barulho": 0, "avg_inv": 0}
    if "_id" in medias:
        del medias["_id"]

    
    pipeline_finais = [
        {"$match": {"evento": "VITORIA"}},
        {"$group": {"_id": "$detalhes", "total": {"$sum": 1}}},
        {"$sort": {"total": -1}}
    ]
    ranking_finais = list(telemetry_collection.aggregate(pipeline_finais))
    
    return jsonify({
        "geral": {
            "mortes_totais": mortes,
            "vitorias_totais": vitorias,
        },
        "top_salas_mortais": ranking_mortes,
        "causas_de_morte": ranking_motivos,
        "finais_alcancados": ranking_finais,
        "autopsia_da_morte": {
            "luz_media_restante": round(medias.get("avg_luz") or 0, 2),
            "barulho_medio": round(medias.get("avg_barulho") or 0, 2),
            "itens_na_mochila": round(medias.get("avg_inv") or 0, 2)
        }
    })

@app.route("/api/replay/<id_replay>", methods=["GET"])
def obter_replay(id_replay):
    if not mongo_client:
        return jsonify({"erro": "Banco de dados inativo"}), 500
        
    try:
        doc = telemetry_collection.find_one({"_id": ObjectId(id_replay)})
        if not doc:
            return jsonify({"erro": "Replay não encontrado ou expirado."}), 404
            
        return jsonify({
            "jogador": doc.get("evento"),
            "sala_final": doc.get("sala"),
            "log_comandos": doc.get("log_comandos", [])
        })
    except InvalidId: 
        return jsonify({"erro": "ID de replay inválido."}), 400


@app.route("/share/generate", methods=["GET"])
@limiter.limit("5 per minute")
def gerar_link_compartilhamento():
    sid = obter_sid_seguro()
    if not sid:
        return jsonify({"erro": "Sem save ativo"}), 401
    if not mongo_client:
        return jsonify({"erro": "Banco de dados desativado"}), 400

    
    share_token = str(uuid.uuid4())
    expires_at = time.time() + 3600 

    
    shares_collection.insert_one(
        {"share_token": share_token, "original_sid": sid, "expires_at": expires_at}
    )

    url_share = f"{request.host_url}share/{share_token}"
    return jsonify({"link": url_share, "mensagem": "Link válido por 24 horas."})


@app.route("/share/<share_token>", methods=["GET"])
def carregar_save_compartilhado(share_token):
    if not mongo_client:
        return "Banco de dados desativado. Não é possível compartilhar.", 400

    share_doc = shares_collection.find_one({"share_token": share_token})
    if not share_doc:
        return "Link inválido ou não encontrado.", 404

    if time.time() > share_doc.get("expires_at", 0):
        shares_collection.delete_one({"share_token": share_token})
        return "Este link de compartilhamento expirou.", 410

    doc = saves_collection.find_one({"sid": share_doc["original_sid"]})
    if not doc:
        return "O save original foi deletado ou corrompido.", 404

    novo_sid = str(uuid.uuid4())
    session["sid"] = novo_sid
    session.permanent = True

    jogo_compartilhado = GameState.from_dict(doc["dados"])
    MEMORIA_SESSOES[novo_sid] = jogo_compartilhado

    saves_collection.insert_one(
        {"sid": novo_sid, "dados": jogo_compartilhado.to_dict()}
    )

    return redirect("/")


@app.route("/saves", methods=["GET"])
@limiter.limit("10 per minute")
@requer_admin
def listar_saves_paginados():
    if not mongo_client:
        return jsonify({"erro": "Banco de dados desativado."}), 400

    try:
        page = request.args.get("page", 1, type=int)
        limit = 10
        skip = (page - 1) * limit

        cursor = (
            saves_collection.find({}, {"_id": 0})
            .sort("dados.timestamp", -1)
            .skip(skip)
            .limit(limit)
        )
        saves = list(cursor)

        total_saves = saves_collection.count_documents({})
        total_pages = (total_saves + limit - 1) // limit

        return jsonify(
            {
                "page": page,
                "limit": limit,
                "total_saves": total_saves,
                "total_pages": total_pages,
                "saves": saves,
            }
        )

    except (pymongo.errors.PyMongoError, ValueError) as e:
        logger.error(f"Erro ao listar saves paginados: {e}")
        return jsonify({"erro": "Erro interno do servidor"}), 500


@app.after_request
def aplicar_headers_de_seguranca(response):
    
    response.headers['Content-Security-Policy'] = (
        "default-src 'self'; "
        "script-src 'self' 'unsafe-inline'; " 
        "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; "
        "font-src 'self' https://fonts.gstatic.com;"
    )
    
    
    response.headers['X-Content-Type-Options'] = 'nosniff'
    response.headers['X-Frame-Options'] = 'DENY' 
    
    
    if IS_PRODUCTION:
        response.headers['Strict-Transport-Security'] = 'max-age=31536000; includeSubDomains'
        
    return response


if __name__ == "__main__":
    app.run(debug=True, port=5000)
