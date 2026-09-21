import hmac
import hashlib
import json
import os

def obter_secret_key():

    return os.environ.get("SECRET_KEY", "DEV_SECRET_DO_NOT_USE_IN_PROD_1982").encode()

def assinar_dados(dados_dict):
    
    dados_str = json.dumps(dados_dict, sort_keys=True)
    return hmac.new(obter_secret_key(), dados_str.encode(), hashlib.sha256).hexdigest()