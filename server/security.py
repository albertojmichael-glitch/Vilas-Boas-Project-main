import hmac
import functools
import hashlib
import json
import os
import sys
import logging

from flask import request, jsonify, current_app
from flask_limiter.util import get_remote_address

logger = logging.getLogger(__name__)

IS_PRODUCTION = bool(
    os.environ.get("FLASK_ENV") == "production"
    or os.environ.get("RENDER")
    or os.environ.get("RAILWAY_STATIC_URL")
    or os.environ.get("PROD")
)

def obter_secret_key():
    chave = os.environ.get("SECRET_KEY") or os.environ.get("FLASK_SECRET_KEY")
    
    if IS_PRODUCTION and not chave:
        print("➣ ERRO FATAL DE SEGURANÇA: SECRET_KEY ausente. Abortando para evitar fallback de desenvolvimento.")
        sys.exit(1)
        
    return (chave or "DEV_SECRET_DO_NOT_USE_IN_PROD_1982").encode()

def assinar_dados(dados_dict):
    
    dados_str = json.dumps(dados_dict, sort_keys=True)
    return hmac.new(obter_secret_key(), dados_str.encode(), hashlib.sha256).hexdigest()


def requer_admin(f):
    @functools.wraps(f)
    def decorated(*args, **kwargs):
        token = request.headers.get("X-Admin-Token") or request.args.get("token")
        ip_origem = get_remote_address()
        admin_token = current_app.config.get("ADMIN_TOKEN")

        if not token or not admin_token or not hmac.compare_digest(str(token), str(admin_token)):
            logger.warning(f"TENTATIVA INVASÃO ADMIN: IP {ip_origem} tentou acessar {request.path}")
            return jsonify({"erro": "Acesso negado. Credenciais inválidas."}), 403

        logger.info(f" ACESSO ADMIN: IP {ip_origem} visualizando {request.path}")
        return f(*args, **kwargs)

    return decorated