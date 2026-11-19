import functools
import hashlib
import hmac
import json
import logging
import sys

from config import Config, is_production
from flask import current_app, jsonify, request
from flask_limiter.util import get_remote_address

logger = logging.getLogger(__name__)

IS_PRODUCTION = is_production()


def obter_secret_key() -> bytes:
    chave = Config.SECRET_KEY
    if is_production() and not chave:
        print(
            "➣ ERRO FATAL DE SEGURANÇA: SECRET_KEY ausente. Abortando para evitar fallback de desenvolvimento."
        )
        sys.exit(1)
    return str(chave).encode("utf-8")


def hash_identificador_email(email: str) -> str:
    """Gera hash SHA-256 irreversível do e-mail para conformidade com LGPD (minimização de PII)."""
    normalizado = (email or "").strip().lower()
    if not normalizado:
        raise ValueError("E-mail vazio não pode ser processado.")
    return hashlib.sha256(normalizado.encode("utf-8")).hexdigest()


def assinar_dados(dados_dict):
    dados_str = json.dumps(dados_dict, sort_keys=True)
    return hmac.new(obter_secret_key(), dados_str.encode(), hashlib.sha256).hexdigest()


# --- Saves exportados: legíveis pelo jogador, mas assinados pelo servidor ---
FORMATO_SAVE = "vilas-boas-save"
VERSAO_SAVE = 2


def json_canonico(dados) -> bytes:
    """Serialização estável (mesmos dados -> mesmos bytes) para assinar."""
    return json.dumps(
        dados, sort_keys=True, separators=(",", ":"), ensure_ascii=False
    ).encode("utf-8")


def _corpo_assinado(estado: dict) -> bytes:
    return json_canonico(
        {"formato": FORMATO_SAVE, "versao": VERSAO_SAVE, "estado": estado}
    )


def assinar_save(estado: dict) -> str:
    chave = str(Config.SAVE_SIGNING_KEY).encode("utf-8")
    return hmac.new(chave, _corpo_assinado(estado), hashlib.sha256).hexdigest()


def verificar_assinatura_save(estado: dict, assinatura: str) -> bool:
    if not isinstance(estado, dict) or not isinstance(assinatura, str):
        return False
    return hmac.compare_digest(assinar_save(estado), assinatura)


def requer_admin(f):
    @functools.wraps(f)
    def decorated(*args, **kwargs):
        token = request.headers.get("X-Admin-Token")
        ip_origem = get_remote_address()
        admin_token = current_app.config.get("ADMIN_TOKEN")

        if (
            not token
            or not admin_token
            or not hmac.compare_digest(str(token), str(admin_token))
        ):
            logger.warning(
                f"TENTATIVA INVASÃO ADMIN: IP {ip_origem} tentou acessar {request.path}"
            )
            return jsonify({"erro": "Acesso negado. Credenciais inválidas."}), 403

        logger.info(f" ACESSO ADMIN: IP {ip_origem} visualizando {request.path}")
        return f(*args, **kwargs)

    return decorated


def validar_origem_request():
    origin = request.headers.get("Origin")
    origens_config = current_app.config.get("ALLOWED_ORIGINS") or []
    if isinstance(origens_config, str):
        origens_config = [o.strip() for o in origens_config.split(",") if o.strip()]

    permitidas = {o.rstrip("/") for o in origens_config if o}
    if request.host_url:
        permitidas.add(request.host_url.rstrip("/"))
    frontend_url = current_app.config.get("FRONTEND_URL")
    if frontend_url:
        permitidas.add(str(frontend_url).rstrip("/"))

    if origin:
        return origin.rstrip("/") in permitidas

    return not current_app.config.get("IS_PRODUCTION", False)


def requer_origem_valida(f):
    @functools.wraps(f)
    def decorated(*args, **kwargs):
        if not validar_origem_request():
            logger.warning(
                f"CSRF/Origin bloqueado em {request.path}: Origin={request.headers.get('Origin')}"
            )
            return jsonify({"erro": "Origem não autorizada."}), 403
        return f(*args, **kwargs)

    return decorated
