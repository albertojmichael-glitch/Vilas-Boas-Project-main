import json
import hashlib
import base64
import logging
import uuid
from cryptography.fernet import Fernet, InvalidToken
from sqlalchemy.exc import SQLAlchemyError

from config import Config
from extensions import db
from models import SaveJogo
from state import GameState

logger = logging.getLogger(__name__)

_key_hash = hashlib.sha256((Config.SECRET_KEY).encode()).digest()
CIPHER_SUITE = Fernet(base64.urlsafe_b64encode(_key_hash))

def to_uuid(sid):
    
    if isinstance(sid, uuid.UUID):
        return sid
    try:
        return uuid.UUID(str(sid))
    except (ValueError, TypeError, AttributeError):
        return None



def _processar_dados_save(dados_brutos):
    
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

def save_existe(sid):
    if not sid:
        return False
    try:
        chave = to_uuid(sid)
        return chave is not None and db.session.get(SaveJogo, chave) is not None
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao verificar existência de save")
        return False

def carregar_save_web(jogo, sid):
    chave = to_uuid(sid)
    if not chave:
        return False
    try:
        registro = db.session.get(SaveJogo, chave)
        if registro:
            dados = _processar_dados_save(registro.dados)
            if dados:
                jogo.carregar_estado(dados)
                return True
    except SQLAlchemyError:
        logger.exception("Falha na leitura do banco de dados ao carregar progresso.")
    return False

def salvar_save_web(jogo, sid):
    chave = to_uuid(sid)
    if not chave:
        return

    try:
        dados_encriptados = CIPHER_SUITE.encrypt(jogo.model_dump_json().encode('utf-8')).decode('utf-8')
        registro = db.session.get(SaveJogo, chave)
        
        if registro:
            registro.dados = dados_encriptados
        else:
            novo_registro = SaveJogo(sid=chave, dados=dados_encriptados)
            db.session.add(novo_registro)
            
        db.session.commit()
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro crítico ao gravar progresso na infraestrutura de dados.")

def obter_ou_recuperar_jogo(sid, memoria_sessoes):
    if not sid:
        return None
        
    jogo = memoria_sessoes.get(str(sid))
    if jogo:
        return jogo

    chave = to_uuid(sid)
    if not chave:
        return None
        
    registro = db.session.get(SaveJogo, chave)
    if not registro:
        return None 

    try:
        dados_recuperados = _processar_dados_save(registro.dados)
        if not dados_recuperados:
            return None
            
        estado_jogo = GameState.from_dict(dados_recuperados)
        memoria_sessoes[str(sid)] = estado_jogo
        return estado_jogo
        
    except Exception:
        logger.exception("Falha irrecuperável ao tentar reconstruir a sessão a partir do banco de dados.")
        return None