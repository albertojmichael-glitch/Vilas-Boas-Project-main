import hmac
import hashlib
import json
import os
import sys

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