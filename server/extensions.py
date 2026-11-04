import json
import logging
import os
from typing import Any

from authlib.integrations.flask_client import OAuth
from cachetools import TTLCache
from flask_cors import CORS
from flask_limiter import Limiter
from flask_limiter.util import get_remote_address
from flask_migrate import Migrate
from flask_sqlalchemy import SQLAlchemy
from state import GameState

logger = logging.getLogger(__name__)

REDIS_URL = os.environ.get("REDIS_URL")

db = SQLAlchemy()
migrate = Migrate()
cors = CORS()
limiter = Limiter(
    key_func=get_remote_address,
    storage_uri=REDIS_URL or "memory://",
)
oauth = OAuth()

redis: Any = None
try:
    import redis as _redis

    redis = _redis
except ImportError:
    pass


class RedisSessionStore:
    def __init__(self, client: Any) -> None:
        self.client = client

    def __contains__(self, key: str) -> bool:
        return bool(self.client.exists(key))

    def __getitem__(self, key: str) -> GameState:
        data = self.client.get(key)
        if data:
            return GameState.from_dict(json.loads(data))
        raise KeyError(key)

    def __setitem__(self, key: str, value: GameState) -> None:
        self.client.setex(key, 3600, json.dumps(value.to_dict()))

    def __delitem__(self, key: str) -> None:
        self.client.delete(key)

    def get(self, key: str, default: Any = None) -> Any:
        try:
            return self[key]
        except KeyError:
            return default

    def pop(self, key: str, default: Any = None) -> Any:
        val = self.get(key, default)
        self.client.delete(key)
        return val

    def clear(self) -> None:
        if hasattr(self.client, "flushdb"):
            self.client.flushdb()


MEMORIA_SESSOES: Any
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
