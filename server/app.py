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
import hmac
import re
import html
from cryptography.fernet import Fernet, InvalidToken
from datetime import timedelta
from logging.handlers import RotatingFileHandler
from authlib.integrations.flask_client import OAuth
from werkzeug.middleware.proxy_fix import ProxyFix
from cachetools import TTLCache
from flask import (
    Flask,
    jsonify,
    redirect,
    request,
    send_from_directory,
    session,
    url_for,
)
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from pydantic import BaseModel, Field, field_validator, ValidationError
from sqlalchemy import func
from sqlalchemy.exc import SQLAlchemyError
from flask_migrate import Migrate

try:
    import redis
except ImportError:
    redis = None

from villas_boas.engine.core import processar_fluxo_jogo
from state import GameState
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET, UIHandler
from views import imprimir_tela_boot
from security import assinar_dados
from config import Config
from extensions import db, migrate, cors
from models import db, Jogador, SaveJogo, Telemetria, Compartilhamento

def create_app():
    """Application Factory: Monta o app sob demanda."""
    app_instance = Flask(__name__, static_folder=Config.BASE_DIR, static_url_path="/")
    
    # Injeta todas as variáveis do Config de uma vez só
    app_instance.config.from_object(Config)
    
    # Acopla as ferramentas ao app recém-criado
    db.init_app(app_instance)
    migrate.init_app(app_instance, db)
    cors.init_app(app_instance, supports_credentials=True, origins=Config.ALLOWED_ORIGINS)

    return app_instance

app = create_app()

# Configuração geral / segurança
app.secret_key = app.config.get("SECRET_KEY")
app.wsgi_app = ProxyFix(app.wsgi_app, x_proto=1, x_host=1)
app.config["PERMANENT_SESSION_LIFETIME"] = timedelta(days=30)
app.config["MAX_CONTENT_LENGTH"] = 1 * 1024 * 1024

_key_hash = hashlib.sha256((app.secret_key).encode()).digest()
CIPHER_SUITE = Fernet(base64.urlsafe_b64encode(_key_hash))

# Configuração geral / segurança

app.secret_key = SECRET_KEY or "DEV_SECRET_DO_NOT_USE_IN_PROD_1982"
app.wsgi_app = ProxyFix(app.wsgi_app, x_proto=1, x_host=1)
app.config["PERMANENT_SESSION_LIFETIME"] = timedelta(days=30)
app.config["MAX_CONTENT_LENGTH"] = 1 * 1024 * 1024

_key_hash = hashlib.sha256((app.secret_key).encode()).digest()
CIPHER_SUITE = Fernet(base64.urlsafe_b64encode(_key_hash))

oauth = OAuth(app)
google = oauth.register(
    name="google",
    client_id=os.environ.get("GOOGLE_CLIENT_ID"),
    client_secret=os.environ.get("GOOGLE_CLIENT_SECRET"),
    server_metadata_url="https://accounts.google.com/.well-known/openid-configuration",
    client_kwargs={"scope": "openid email profile"},
)

if IS_PRODUCTION:
    app.config.update(
        SESSION_COOKIE_SECURE=True,
        SESSION_COOKIE_HTTPONLY=True,
        SESSION_COOKIE_SAMESITE="None",
    )
    print(" Segurança de Cookies: Modo Produção ativado (Secure=True).")
else:
    app.config.update(
        SESSION_COOKIE_SECURE=False,
        SESSION_COOKIE_HTTPONLY=True,
        SESSION_COOKIE_SAMESITE="Lax",
    )
    print(" Segurança de Cookies: Modo Desenvolvimento (Secure=False).")


# Logging

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


limiter = Limiter(key_func=get_remote_address, app=app, storage_uri="memory://")


# sessões de jogo em memória (Redis ou TTLCache)

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

    def __delitem__(self, key):
        self.client.delete(key)


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



# validação de entrada

class ComandoRequest(BaseModel):
    comando: str = Field(
        default="",
        max_length=256,
        pattern=r"^[a-zA-Z0-9\s\"\'\-\_áéíóúâêôãõçÁÉÍÓÚÂÊÔÃÕÇ\.\:]+$",
    )
    telemetria: bool = Field(default=True)

    @field_validator("comando", mode="before")
    def limpar_comando(cls, v):
        if not isinstance(v, str):
            return ""
        return v.replace("\x00", "").replace("\0", "")


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

        if not token or not hmac.compare_digest(str(token), str(ADMIN_TOKEN)):
            logger.warning(
                f"TENTATIVA INVASÃO ADMIN: IP {ip_origem} tentou acessar {request.path}"
            )
            return jsonify({"erro": "Acesso negado. Credenciais inválidas."}), 403

        logger.info(f" ACESSO ADMIN: IP {ip_origem} visualizando {request.path}")
        return f(*args, **kwargs)

    return decorated



# UI Web

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
    mapa_cores = {
        DOS_VERDE: "verde",
        DOS_BRANCO: "branco",
        DOS_AMARELO: "amarelo",
        DOS_VERMELHO: "vermelho",
    }
    padrao = re.compile(
        "(" + "|".join(re.escape(c) for c in list(mapa_cores.keys()) + [RESET]) + ")"
    )
    partes = padrao.split(texto_ansi)
    html_out, aberto = [], False

    for parte in partes:
        if parte in mapa_cores:
            if aberto:
                html_out.append("</span>")
            html_out.append(f'<span class="{mapa_cores[parte]}">')
            aberto = True
        elif parte == RESET:
            if aberto:
                html_out.append("</span>")
                aberto = False
        else:
            html_out.append(html.escape(parte))

    if aberto:
        html_out.append("</span>")
    return "".join(html_out)



# telemetria (PostgreSQL)

def registrar_telemetria(evento, sala, dificuldade, detalhes="", jogo=None):
    if not session.get("permite_telemetria", True):
        return

    try:
        sid_atual = session.get("sid")
        registro = Telemetria(

            evento=str(evento)[:50],
            sala=str(sala)[:50],
            dificuldade=str(dificuldade)[:50],
            detalhes=str(detalhes)[:256],
            sid_sessao=sid_atual
        )

        if jogo:
            registro.hp_restante = getattr(jogo, "hp", 0)
            registro.luz_restante = getattr(jogo, "turnos_luz", 0)
            registro.nivel_barulho = getattr(jogo, "nivel_barulho", 0)
            registro.inventario_qtd = len(getattr(jogo, "inventario", []))
            registro.bolsas = getattr(jogo, "bolsas_coletadas", 0)
            registro.log_comandos = getattr(jogo, "log_comandos", [])

        db.session.add(registro)
        db.session.commit()

    except SQLAlchemyError as e:
        db.session.rollback()
        logger.error(f"Erro na telemetria: {e}")



# saves (PostgreSQL, criptografados com Fernet)

def _processar_dados_save(dados_brutos):
    """Descriptografa se for texto seguro, ou lê normalmente se for um save antigo."""
    if isinstance(dados_brutos, str):
        try:
            dados_json = CIPHER_SUITE.decrypt(dados_brutos.encode("utf-8")).decode(
                "utf-8"
            )
            return json.loads(dados_json)
        except InvalidToken:
            logger.error(
                "Tentativa de carregar save com chave de criptografia inválida!"
            )
            return None
        except json.JSONDecodeError:
            pass

    return dados_brutos if isinstance(dados_brutos, dict) else json.loads(dados_brutos)


def save_existe(sid):
    try:
        return db.session.get(SaveJogo, sid) is not None
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao verificar existência de save")
        return False


def carregar_save_web(jogo):
    sid = obter_sid_seguro()
    if not sid:
        return False

    try:
        registro = db.session.get(SaveJogo, sid)
        if registro:
            dados = _processar_dados_save(registro.dados)
            if dados:
                novo_jogo = GameState.from_dict(dados)
                for k, v in novo_jogo.__dict__.items():
                    if k != "ui_handler":
                        setattr(jogo, k, v)
                return True
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao buscar save criptografado no PostgreSQL")
    except Exception:
        logger.exception("Erro ao processar save carregado")

    return False


def salvar_save_web(jogo):
    sid = obter_sid_seguro()
    if not sid:
        return

    estado_dict = jogo.to_dict()
    
    
    metadados = {
        "hp": estado_dict.get("hp", 3),
        "sala": estado_dict.get("sala_atual", "desconhecida"),
        "inventario_tamanho": len(estado_dict.get("inventario", []))
    }

   
    json_str_ordenado = json.dumps(estado_dict, sort_keys=True, ensure_ascii=False)
    checksum_gerado = hashlib.sha256(json_str_ordenado.encode('utf-8')).hexdigest()

    try:
        registro = db.session.get(SaveJogo, sid)
        
        
        if registro and registro.checksum == checksum_gerado:
            return

      
        dados_json = json.dumps(estado_dict, ensure_ascii=False)
        dados_criptografados = CIPHER_SUITE.encrypt(dados_json.encode("utf-8")).decode("utf-8")

        if registro:
            registro.dados = dados_criptografados
            registro.metadados = metadados
            registro.checksum = checksum_gerado
            registro.save_version = 1
        else:
            db.session.add(SaveJogo(
                sid=sid, 
                dados=dados_criptografados,
                metadados=metadados,
                checksum=checksum_gerado,
                save_version=1
            ))
            
        db.session.commit()
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao salvar progresso no PostgreSQL")


def obter_ou_recuperar_jogo(sid):
    """
    Tenta obter o estado do jogo da memória. Se falhar (ex: restart do servidor),
    tenta recuperar o último estado salvo no PostgreSQL de forma invisível para o jogador.
    """
    
    jogo = MEMORIA_SESSOES.get(sid)
    if jogo:
        return jogo

    
    registro = db.session.get(SaveJogo, sid)
    if not registro:
        
        return None 

    try:
        
        dados_json = CIPHER_SUITE.decrypt(registro.dados.encode("utf-8")).decode("utf-8")
        estado_dict = json.loads(dados_json)
        
        
        jogo_recuperado = GameState()
        
        
        for chave, valor in estado_dict.items():
            if hasattr(jogo_recuperado, chave):
                setattr(jogo_recuperado, chave, valor)
            
        
        MEMORIA_SESSOES[sid] = jogo_recuperado
        
        return jogo_recuperado
        
    except Exception as e:
        logger.exception("Falha ao tentar recuperar a sessão corrompida do banco de dados.")
        return None
# resposta JSON

def gerar_resposta_json(jogo):
    linhas = []

    if jogo and hasattr(jogo, "ui_handler") and hasattr(jogo.ui_handler, "buffer"):
        linhas = [
            ansi_para_html(linha)
            for linha in jogo.ui_handler.buffer
            if linha.strip() != ""
        ]
        jogo.ui_handler.buffer.clear()

    conquistas_enviadas = getattr(jogo, "novas_conquistas_turno", []).copy()
    if hasattr(jogo, "novas_conquistas_turno"):
        jogo.novas_conquistas_turno.clear()

    estado_jogo = getattr(jogo, "estado_atual", "")
    if estado_jogo in ["MENU", "AGUARDANDO_DIR"]:
        sala_exibicao = "SISTEMA"
    else:
        sala_exibicao = getattr(jogo, "sala_atual", "SISTEMA").upper()

    resposta = {
        "linhas": linhas,
        "estado": {
            "hp": getattr(jogo, "hp", 0),
            "inventario": getattr(jogo, "inventario", []),
            "luz_restante": getattr(jogo, "turnos_luz", 0),
            "bolsas_coletadas": getattr(jogo, "bolsas_coletadas", 0),
            "estado_jogo": estado_jogo,
            "tempo_final": getattr(jogo, "tempo_total_segundos", 0.0),
            "som": getattr(jogo, "nivel_barulho", 0),
            "sala": sala_exibicao,
            "final_alcancado": getattr(jogo, "sala_atual", "Desconhecido"),
        },
        "novas_conquistas": conquistas_enviadas,
    }

    return jsonify(resposta)



# rotas estáticas

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


# rotas do jogo

@app.route("/iniciar", methods=["GET"])
def iniciar_jogo():
    sid_antigo = session.get("sid")
    if sid_antigo and sid_antigo in MEMORIA_SESSOES:
        try:
            del MEMORIA_SESSOES[sid_antigo]
        except (KeyError, TypeError):
            pass

    session.clear()

    sid = str(uuid.uuid4())
    session["sid"] = sid
    session.permanent = True
    session.modified = True

    jogo = GameState()
    jogo.ui_handler = WebUIHandler()
    jogo.estado_atual = "AGUARDANDO_DIR"

    MEMORIA_SESSOES[sid] = jogo

    jogo.ui_handler.limpar()
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

    jogo = obter_ou_recuperar_jogo(sid)
    if not jogo:
        return jsonify({"erro": "Sessão expirada. Digite 'iniciar' para recomeçar."}), 404

    jogo.ui_handler = WebUIHandler()

    dados = request.json or {}
    try:
        requisicao = ComandoRequest(**dados)
        comando = requisicao.comando
        session["permite_telemetria"] = requisicao.telemetria
    except ValidationError:
        jogo.ui_handler.buffer.append(
            "@@TYPE@@vermelho@@0@@[ ERRO DE SEGURANÇA ]: Payload inválido."
        )
        return gerar_resposta_json(jogo), 400

    tem_save = save_existe(sid)

    if comando:
        jogo.log_comandos.append(comando)

    try:
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

    return gerar_resposta_json(jogo)


@app.route("/save/export", methods=["GET"])
@limiter.limit("5 per minute")
def exportar_save():
    sid = obter_sid_seguro()
    if not sid or sid not in MEMORIA_SESSOES:
        return jsonify({"erro": "Nenhum jogo ativo encontrado."}), 404

    jogo = obter_ou_recuperar_jogo(sid)
    if not jogo:
        return jsonify({"erro": "Nenhum jogo ativo para processar o save."}), 404

    dados_json = json.dumps(jogo.to_dict(), ensure_ascii=False)
    dados_criptografados = CIPHER_SUITE.encrypt(dados_json.encode("utf-8")).decode("utf-8")

    
    resumo_publico = {
        "sala": jogo.sala_atual,
        "hp": jogo.hp,
        "inventario_tamanho": len(jogo.inventario)
    }

    return jsonify({"resumo": resumo_publico, "dados_seguros": dados_criptografados})


@app.route("/save/import", methods=["POST"])
@limiter.limit("10 per minute")
def importar_save():
    sid = obter_sid_seguro()
    if not sid:
        sid = str(uuid.uuid4())
        session["sid"] = sid
        session.permanent = True

    payload = request.json
    dados_seguros = payload.get("dados_seguros") if payload else None

    if not dados_seguros:
        return jsonify({"erro": "Arquivo corrompido ou formato não suportado."}), 400

    try:
        dados_json = CIPHER_SUITE.decrypt(dados_seguros.encode("utf-8")).decode("utf-8")
        dados_save = json.loads(dados_json)
        
        novo_jogo = GameState.from_dict(dados_save)
        novo_jogo.ui_handler = WebUIHandler()

        MEMORIA_SESSOES[sid] = novo_jogo
        salvar_save_web(novo_jogo)

        return jsonify({"sucesso": True, "mensagem": "Save importado com sucesso."})
        
    except InvalidToken:
        logger.warning(f"Tentativa de adulteração de save barrada. SID: {sid}")
        return jsonify({"erro": "[ ERRO CRÍTICO ] ASSINATURA INVÁLIDA. TENTATIVA DE TRAPAÇA DETECTADA."}), 403
    except (ValidationError, ValueError, TypeError, KeyError) as e:
        logger.error(f"Erro ao importar save via UI: {e}")
        return jsonify({"erro": "Arquivo de save inválido."}), 400


@app.route("/achievements", methods=["GET"])
def listar_conquistas():
    sid = obter_sid_seguro()
    if not sid or sid not in MEMORIA_SESSOES:
        return jsonify({"erro": "Sessão não encontrada", "conquistas": []})

    jogo = obter_ou_recuperar_jogo(sid)
    if not jogo:
        # Retorna vazio silenciosamente para não quebrar o front-end
        return jsonify({"conquistas": [], "total": 0})

    conquistas = getattr(jogo, "conquistas", [])
    return jsonify({"conquistas": conquistas, "total": len(conquistas)})



# Admin / analytics (PostgreSQL)

def _ranking(coluna, evento, limite=None):
    q = (
        db.session.query(coluna, func.count().label("total"))
        .filter(Telemetria.evento == evento)
        .group_by(coluna)
        .order_by(func.count().desc())
    )
    if limite:
        q = q.limit(limite)
    return [{"_id": chave, "total": total} for chave, total in q.all()]


@app.route("/admin/analytics", methods=["GET"])
@requer_admin
def ver_telemetria():
    try:
        mortes = Telemetria.query.filter_by(evento="MORTE").count()
        vitorias = Telemetria.query.filter_by(evento="VITORIA").count()

        ranking_mortes = _ranking(Telemetria.sala, "MORTE", 5)
        ranking_motivos = _ranking(Telemetria.detalhes, "MORTE", 5)
        ranking_finais = _ranking(Telemetria.detalhes, "VITORIA")

        avg_luz, avg_barulho, avg_inv = (
            db.session.query(
                func.avg(Telemetria.luz_restante),
                func.avg(Telemetria.nivel_barulho),
                func.avg(Telemetria.inventario_qtd),
            )
            .filter(Telemetria.evento == "MORTE")
            .one()
        )

        return jsonify(
            {
                "geral": {
                    "mortes_totais": mortes,
                    "vitorias_totais": vitorias,
                },
                "top_salas_mortais": ranking_mortes,
                "causas_de_morte": ranking_motivos,
                "finais_alcancados": ranking_finais,
                "autopsia_da_morte": {
                    "luz_media_restante": round(float(avg_luz or 0), 2),
                    "barulho_medio": round(float(avg_barulho or 0), 2),
                    "itens_na_mochila": round(float(avg_inv or 0), 2),
                },
            }
        )
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro no analytics")
        return jsonify({"erro": "Erro interno do servidor"}), 500

@app.route("/admin/auditoria-mortes", methods=["GET"])
@requer_admin
def auditoria_mortes():
    try:
        
        resultados = (
            db.session.query(Jogador.email, Telemetria.sala, Telemetria.log_comandos)
            .join(Telemetria, Jogador.id == Telemetria.jogador_id)
            .filter(Telemetria.evento == "MORTE")
            .limit(50)
            .all()
        )

        auditoria = []
        for email, sala, comandos in resultados:
            auditoria.append({
                "vitima": email,
                "morreu_na_sala": sala,
                "ultimas_palavras": comandos[-3:] if comandos else [] 
            })

        return jsonify({"mortes_identificadas": auditoria})
        
    except SQLAlchemyError as e:
        db.session.rollback()
        return jsonify({"erro": "Erro ao consultar a auditoria no banco de dados."}), 500


@app.route("/api/replay/<int:id_replay>", methods=["GET"])
def obter_replay(id_replay):
    try:
        doc = db.session.get(Telemetria, id_replay)
        if not doc:
            return jsonify({"erro": "Replay não encontrado ou expirado."}), 404

        return jsonify(
            {
                "jogador": doc.evento,
                "sala_final": doc.sala,
                "log_comandos": doc.log_comandos or [],
            }
        )
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao buscar replay")
        return jsonify({"erro": "Erro interno do servidor"}), 500



# compartilhamento de saves (PostgreSQL)

@app.route("/share/generate", methods=["GET"])
@limiter.limit("5 per minute")
def gerar_link_compartilhamento():
    sid = obter_sid_seguro()
    if not sid:
        return jsonify({"erro": "Sem save ativo"}), 401

    try:
        share = Compartilhamento(original_sid=sid, expires_at=time.time() + 3600)
        db.session.add(share)
        db.session.commit()
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao gerar link de compartilhamento")
        return jsonify({"erro": "Erro interno do servidor"}), 500

    url_share = f"{request.host_url}share/{share.share_token}"
    return jsonify({"link": url_share, "mensagem": "Link válido por 1 hora."})


@app.route("/share/<share_token>", methods=["GET"])
def carregar_save_compartilhado(share_token):
    try:
        share = db.session.get(Compartilhamento, share_token)
        if not share:
            return "Link inválido ou não encontrado.", 404

        if time.time() > share.expires_at:
            db.session.delete(share)
            db.session.commit()
            return "Este link de compartilhamento expirou.", 410

        original = db.session.get(SaveJogo, share.original_sid)
        if not original:
            return "O save original foi deletado ou corrompido.", 404

        dados = _processar_dados_save(original.dados)
        if not dados:
            return "O save original está corrompido.", 500

        novo_sid = str(uuid.uuid4())
        db.session.add(SaveJogo(sid=novo_sid, dados=original.dados))
        db.session.commit()

        session["sid"] = novo_sid
        session.permanent = True

        MEMORIA_SESSOES[novo_sid] = GameState.from_dict(dados)

        return redirect("/")
    except (SQLAlchemyError, InvalidToken, json.JSONDecodeError):
        db.session.rollback()
        logger.exception("Erro ao carregar save compartilhado")
        return "Erro ao carregar o save compartilhado.", 500


@app.route("/saves", methods=["GET"])
@limiter.limit("10 per minute")
@requer_admin
def listar_saves_paginados():
    try:
        page = max(request.args.get("page", 1, type=int), 1)
        limit = 10

        paginacao = SaveJogo.query.order_by(SaveJogo.atualizado_em.desc()).paginate(
            page=page, per_page=limit, error_out=False
        )

        return jsonify(
            {
                "page": page,
                "limit": limit,
                "total_saves": paginacao.total,
                "total_pages": paginacao.pages,
                "saves": [
                    {"sid": s.sid, "atualizado_em": s.atualizado_em}
                    for s in paginacao.items
                ],
            }
        )
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao listar saves paginados")
        return jsonify({"erro": "Erro interno do servidor"}), 500



# Login Google / Leaderboard (PostgreSQL)

@app.route("/login/google")
def login_google():
    iniciais = request.args.get("iniciais", "???").upper()[:3]
    session["arcade_initials"] = iniciais

    redirect_uri = url_for("auth_google_callback", _external=True)
    return google.authorize_redirect(redirect_uri)


@app.route("/auth/google/callback")
def auth_google_callback():
    sid = session.get("sid")
    if not sid or sid not in MEMORIA_SESSOES:
        return "Sessão de jogo não encontrada.", 400

    try:
        token = google.authorize_access_token()
        user_info = token.get("userinfo")
        email_jogador = user_info.get("email")
        
        
        jogo = obter_ou_recuperar_jogo(sid)
        if not jogo:
            logger.warning("OAuth concluído, mas o estado do jogo sumiu da memória e do banco.")
            return redirect(f"{FRONTEND_URL}/?leaderboard=falha_sessao")
            
        iniciais = session.get("arcade_initials", "UNK")
        if getattr(jogo, "tempo_total_segundos", 0) > 0:
            
            novo_jogador = Jogador(
                iniciais=iniciais,
                email=email_jogador,
                tempo_segundos=jogo.tempo_total_segundos,
                final_alcancado=jogo.sala_atual,
                dificuldade=jogo.dificuldade_escolhida,
            )
            db.session.add(novo_jogador)
            db.session.flush() 
            save_atual = db.session.get(SaveJogo, sid)
            if save_atual:
                save_atual.jogador_id = novo_jogador.id

            Telemetria.query.filter_by(sid_sessao=sid).update(
                {"jogador_id": novo_jogador.id}
            )
            db.session.commit()

        return redirect(f"{FRONTEND_URL}/?leaderboard=sucesso")

    except Exception as e:  # noqa: BLE001
        db.session.rollback()
        logger.error(f"Erro no OAuth do Google: {e}")
        return "Falha na autenticação com o Google.", 500



@app.after_request
def aplicar_headers_de_seguranca(response):
    response.headers["Content-Security-Policy"] = (
        "default-src 'self'; "
        "script-src 'self' 'unsafe-inline'; "
        "style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; "
        "font-src 'self' https://fonts.gstatic.com;"
    )

    response.headers["X-Content-Type-Options"] = "nosniff"
    response.headers["X-Frame-Options"] = "DENY"

    if IS_PRODUCTION:
        response.headers["Strict-Transport-Security"] = (
            "max-age=31536000; includeSubDomains"
        )

    return response


if __name__ == "__main__":
    app.run(debug=True, port=5000)