import json
import hashlib
import base64
import logging
from cryptography.fernet import Fernet, InvalidToken
from sqlalchemy.exc import SQLAlchemyError

from config import Config
from extensions import db
from models import SaveJogo
from state import GameState

logger = logging.getLogger(__name__)

_key_hash = hashlib.sha256((Config.SECRET_KEY).encode()).digest()
CIPHER_SUITE = Fernet(base64.urlsafe_b64encode(_key_hash))

def _processar_dados_save(dados_brutos):
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

def save_existe(sid):
    if not sid:
        return False
    try:
        return db.session.get(SaveJogo, sid) is not None
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao verificar existência de save")
        return False

def carregar_save_web(jogo, sid):
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

def salvar_save_web(jogo, sid):
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

def obter_ou_recuperar_jogo(sid, memoria_sessoes):
    
    jogo = memoria_sessoes.get(sid)
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
            
        memoria_sessoes[sid] = jogo_recuperado
        return jogo_recuperado
        
    except Exception:
        logger.exception("Falha ao tentar recuperar a sessão corrompida do banco de dados.")
        return None