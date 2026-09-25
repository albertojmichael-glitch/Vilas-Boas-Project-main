import os
import json
import logging
from flask_sqlalchemy import SQLAlchemy
from flask_migrate import Migrate
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from authlib.integrations.flask_client import OAuth
from cachetools import TTLCache

logger = logging.getLogger(__name__)


db = SQLAlchemy()
migrate = Migrate()
cors = CORS()
limiter = Limiter(key_func=get_remote_address, storage_uri="memory://")
oauth = OAuth()


try:
    import redis
except ImportError:
    redis = None

from state import GameState

class RedisSessionStore:
    def __init__(self, client):
        self.client = client
    def __contains__(self, key):
        return self.client.exists(key) > 0
    def __getitem__(self, key):
        data = self.client.get(key)
        if data: return GameState.from_dict(json.loads(data))
        raise KeyError(key)
    def __setitem__(self, key, value):
        self.client.setex(key, 3600, json.dumps(value.to_dict()))
    def __delitem__(self, key):
        self.client.delete(key)

REDIS_URL = os.environ.get("REDIS_URL")

if REDIS_URL and redis is not None:
    try:
        redis_client = redis.Redis.from_url(REDIS_URL, decode_responses=True)
        redis_client.ping()
        MEMORIA_SESSOES = RedisSessionStore(redis_client)
        logger.info("Conectado ao Redis com sucesso.")
    except Exception:
        logger.exception("Falha ao conectar no Redis — caindo para TTLCache.")
        MEMORIA_SESSOES = TTLCache(maxsize=1000, ttl=3600)
else:
    logger.info("Usando TTLCache na memória RAM local.")
    MEMORIA_SESSOES = TTLCache(maxsize=1000, ttl=3600)