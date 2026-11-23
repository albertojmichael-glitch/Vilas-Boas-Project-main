import base64
import hashlib
import json
import logging
import uuid

from config import Config
from cryptography.fernet import Fernet, InvalidToken
from extensions import db
from models import SaveJogo
from sqlalchemy.exc import SQLAlchemyError

from services.save_validacao import SaveInvalido, validar_estado_save

logger = logging.getLogger(__name__)

# Cifra do save *em repouso* no servidor. A chave nunca sai do servidor.
_key_hash = hashlib.sha256(str(Config.SAVE_ENCRYPTION_KEY).encode("utf-8")).digest()
CIPHER_SUITE = Fernet(base64.urlsafe_b64encode(_key_hash))


def to_uuid(sid):
    if isinstance(sid, uuid.UUID):
        return sid
    try:
        return uuid.UUID(str(sid))
    except (ValueError, TypeError, AttributeError):
        return None


def _processar_dados_save(dados_brutos):
    """Converte o registro do banco em dict, sem nunca levantar exceção.

    Aceita o formato atual (Fernet) e saves legados gravados em JSON puro."""
    if isinstance(dados_brutos, dict):
        return dados_brutos
    if not isinstance(dados_brutos, str) or not dados_brutos.strip():
        return None
    texto = dados_brutos.strip()
    try:
        texto = CIPHER_SUITE.decrypt(texto.encode("utf-8")).decode("utf-8")
    except InvalidToken:
        if not texto.startswith("{"):
            logger.error("Save com cifra inválida (chave trocada ou dado corrompido).")
            return None
        logger.info("Save legado em JSON puro encontrado; será migrado para cifrado.")
    try:
        dados = json.loads(texto)
    except (json.JSONDecodeError, UnicodeDecodeError):
        logger.error("Save com JSON corrompido no banco.")
        return None
    return dados if isinstance(dados, dict) else None


def carregar_estado_validado(dados_brutos, permitir_dev=False):
    """Registro do banco -> GameState validado, ou None (nunca levanta exceção)."""
    dados = _processar_dados_save(dados_brutos)
    if dados is None:
        return None
    try:
        return validar_estado_save(dados, permitir_dev=permitir_dev)
    except SaveInvalido as exc:
        logger.warning(f"Save rejeitado na validação: {exc}")
        return None


def serializar_estado(jogo, permitir_dev=True):
    """Valida e serializa o estado; levanta SaveInvalido se não for salvável.

    O JSON gerado é relido antes de seguir: nada corrompido chega ao banco."""
    dados = json.loads(jogo.model_dump_json())
    validado = validar_estado_save(dados, permitir_dev=permitir_dev)
    texto = validado.model_dump_json()
    json.loads(texto)  # round-trip: garante JSON íntegro antes de persistir
    return texto


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
        if not registro:
            return False
        validado = carregar_estado_validado(registro.dados, permitir_dev=True)
        if validado is None:
            return False
        jogo.carregar_estado(validado.to_dict())
        _migrar_se_legado(registro, validado)
        return True
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Falha na leitura do banco de dados ao carregar progresso.")
    return False


def _migrar_se_legado(registro, jogo):
    """Regrava cifrado um save legado que estava em JSON puro."""
    if isinstance(registro.dados, str) and registro.dados.lstrip().startswith("{"):
        try:
            registro.dados = CIPHER_SUITE.encrypt(
                serializar_estado(jogo).encode("utf-8")
            ).decode("utf-8")
            db.session.commit()
        except (SaveInvalido, SQLAlchemyError):
            db.session.rollback()
            logger.exception("Falha ao migrar save legado; o original foi mantido.")


CAPITULOS = {
    "entrada": "Entrada",
    "sala de fliperamas": "Fliperamas",
    "01": "Sala 01",
    "amanhecer": "Amanhecer",
}


def progresso_web(sid):
    try:
        chave = to_uuid(sid)
        registro = db.session.get(SaveJogo, chave) if chave else None
        meta = (registro.metadados or {}) if registro else {}
        return {
            "tem_save": registro is not None,
            "pode_voltar": bool(meta.get("anterior")),
            "capitulos": [
                {"id": k, "nome": v}
                for k, v in CAPITULOS.items()
                if k in meta.get("capitulos", {})
            ],
        }
    except SQLAlchemyError:
        db.session.rollback()
        return {"tem_save": False, "pode_voltar": False, "capitulos": []}


def restaurar_marco(sid, capitulo=None):
    chave = to_uuid(sid)
    if not chave:
        return None
    registro = db.session.get(SaveJogo, chave, with_for_update=True)
    if not registro:
        return None
    meta = dict(registro.metadados or {})
    dados = (
        meta.get("capitulos", {}).get(capitulo) if capitulo else meta.get("anterior")
    )
    jogo = carregar_estado_validado(dados, permitir_dev=True)
    if jogo:
        meta.pop("anterior", None)
        registro.dados = dados
        registro.metadados = meta
        db.session.commit()
    return jogo


def salvar_save_web(jogo, sid, nova_partida=False):
    """Persiste o progresso. Se o estado não for válido, mantém o último save bom."""
    chave = to_uuid(sid)
    if not chave:
        return False

    try:
        texto = serializar_estado(jogo)
    except (SaveInvalido, ValueError, TypeError) as exc:
        logger.error(f"Estado não persistido (último save bom mantido): {exc}")
        return False

    try:
        dados_encriptados = CIPHER_SUITE.encrypt(texto.encode("utf-8")).decode("utf-8")
        registro = db.session.get(SaveJogo, chave, with_for_update=True)
        if registro is None:
            registro = SaveJogo(sid=chave)
            db.session.add(registro)
        meta = dict(registro.metadados or {})
        capitulos = {} if nova_partida else dict(meta.get("capitulos", {}))
        if nova_partida:
            meta.pop("anterior", None)
        elif registro.dados:
            meta["anterior"] = registro.dados
        for marco in (jogo.sala_atual, "amanhecer" if jogo.noite_vencida else None):
            if marco in CAPITULOS:
                capitulos.setdefault(marco, dados_encriptados)
        meta["capitulos"] = capitulos
        registro.dados = dados_encriptados
        registro.metadados = meta

        db.session.commit()
        return True
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro crítico ao gravar progresso na infraestrutura de dados.")
        return False


def obter_ou_recuperar_jogo(sid, memoria_sessoes):
    if not sid:
        return None

    jogo = memoria_sessoes.get(str(sid))
    if jogo:
        return jogo

    chave = to_uuid(sid)
    if not chave:
        return None

    try:
        registro = db.session.get(SaveJogo, chave)
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Falha ao consultar o save no banco.")
        return None
    if not registro:
        return None

    estado_jogo = carregar_estado_validado(registro.dados, permitir_dev=True)
    if estado_jogo is None:
        return None
    _migrar_se_legado(registro, estado_jogo)
    memoria_sessoes[str(sid)] = estado_jogo
    return estado_jogo
