import base64
import hashlib
import os
import secrets
from typing import ClassVar

BASE_DIR = os.path.abspath(os.path.dirname(__file__))


def is_production() -> bool:
    """Detecta de forma centralizada se a aplicação está em produção."""
    app_env = os.environ.get("APP_ENV", "").strip().lower()
    if app_env:
        return app_env == "production"
    return bool(
        os.environ.get("FLASK_ENV") == "production"
        or os.environ.get("PROD", "").strip().lower() in ("1", "true", "yes", "on")
    )


def _obter_database_url() -> str | None:
    for chave in ("DATABASE_URL", "POSTGRES_URL", "POSTGRES_PRISMA_URL"):
        val = os.environ.get(chave, "").strip()
        if val and not val.startswith("${{"):
            return val
    return None


def _garantir_fernet_key(raw_key: str) -> str:
    """Garante que a chave tenha 32 bytes em urlsafe base64 para o Fernet."""
    try:
        decodificada = base64.urlsafe_b64decode(raw_key.encode("utf-8"))
        if len(decodificada) == 32:
            return raw_key
    except Exception:
        pass
    digest = hashlib.sha256(raw_key.encode("utf-8")).digest()
    return base64.urlsafe_b64encode(digest).decode("utf-8")


class Config:
    BASE_DIR = BASE_DIR
    IS_PRODUCTION = is_production()
    SQLALCHEMY_ENGINE_OPTIONS: ClassVar[dict] = {
        "pool_pre_ping": True,
        "pool_recycle": int(os.environ.get("DB_POOL_RECYCLE", "300")),
        "pool_size": int(os.environ.get("DB_POOL_SIZE", "5")),
        "max_overflow": int(os.environ.get("DB_MAX_OVERFLOW", "10")),
        "pool_timeout": int(os.environ.get("DB_POOL_TIMEOUT", "30")),
    }
    SECRET_KEY = os.environ.get("SECRET_KEY") or os.environ.get("FLASK_SECRET_KEY")
    SAVE_ENCRYPTION_KEY = os.environ.get("SAVE_ENCRYPTION_KEY")
    ADMIN_TOKEN = os.environ.get("ADMIN_TOKEN")
    FRONTEND_URL = os.environ.get("FRONTEND_URL")
    ALLOWED_ORIGINS_ENV = os.environ.get("ALLOWED_ORIGINS", "")
    DATABASE_URL = _obter_database_url()
    REDIS_URL = os.environ.get("REDIS_URL") or os.environ.get("KV_URL")
    SESSION_COOKIE_DOMAIN = os.environ.get("SESSION_COOKIE_DOMAIN") or None
    SESSION_COOKIE_SAMESITE = os.environ.get("SESSION_COOKIE_SAMESITE", "Lax")

    @classmethod
    def validar_e_configurar(cls):
        cls.IS_PRODUCTION = is_production()
        cls.SECRET_KEY = (
            os.environ.get("SECRET_KEY")
            or os.environ.get("FLASK_SECRET_KEY")
            or "DEV_SECRET_DO_NOT_USE_IN_PROD_1982"
        )
        raw_save_key = os.environ.get("SAVE_ENCRYPTION_KEY") or cls.SECRET_KEY
        cls.SAVE_ENCRYPTION_KEY = _garantir_fernet_key(raw_save_key)

        cls.ADMIN_TOKEN = os.environ.get("ADMIN_TOKEN") or secrets.token_hex(16)
        cls.FRONTEND_URL = os.environ.get("FRONTEND_URL") or "http://localhost:5000"
        cls.ALLOWED_ORIGINS_ENV = os.environ.get("ALLOWED_ORIGINS", "")
        cls.DATABASE_URL = _obter_database_url()
        cls.REDIS_URL = os.environ.get("REDIS_URL") or os.environ.get("KV_URL")
        cls.SESSION_COOKIE_DOMAIN = os.environ.get("SESSION_COOKIE_DOMAIN") or None
        cls.SESSION_COOKIE_SAMESITE = os.environ.get("SESSION_COOKIE_SAMESITE", "Lax")

        if not cls.DATABASE_URL:
            cls.DATABASE_URL = "sqlite:////tmp/local_testes.db"
        elif cls.DATABASE_URL.startswith("postgres://"):
            cls.DATABASE_URL = cls.DATABASE_URL.replace(
                "postgres://", "postgresql://", 1
            )

        if cls.ALLOWED_ORIGINS_ENV:
            cls.ALLOWED_ORIGINS = [
                origem.strip()
                for origem in cls.ALLOWED_ORIGINS_ENV.split(",")
                if origem.strip()
            ]
        elif cls.IS_PRODUCTION:
            cls.ALLOWED_ORIGINS = [cls.FRONTEND_URL]
        else:
            cls.ALLOWED_ORIGINS = [
                "http://localhost:5000",
                "http://127.0.0.1:5000",
                cls.FRONTEND_URL,
            ]

        cls.SQLALCHEMY_DATABASE_URI = cls.DATABASE_URL
        cls.SQLALCHEMY_TRACK_MODIFICATIONS = False
        if cls.DATABASE_URL.startswith("sqlite"):
            cls.SQLALCHEMY_ENGINE_OPTIONS = {"pool_pre_ping": True}
        else:
            cls.SQLALCHEMY_ENGINE_OPTIONS = {
                "pool_pre_ping": True,
                "pool_recycle": int(os.environ.get("DB_POOL_RECYCLE", "300")),
                "pool_size": int(os.environ.get("DB_POOL_SIZE", "5")),
                "max_overflow": int(os.environ.get("DB_MAX_OVERFLOW", "10")),
                "pool_timeout": int(os.environ.get("DB_POOL_TIMEOUT", "30")),
            }


Config.validar_e_configurar()