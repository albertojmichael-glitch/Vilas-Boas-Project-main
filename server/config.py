import logging
import os
import secrets
from typing import ClassVar

logger = logging.getLogger(__name__)
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
    SAVE_ENCRYPTION_KEY = (
        os.environ.get("SAVE_ENCRYPTION_KEY")
        or os.environ.get("SECRET_KEY")
        or os.environ.get("FLASK_SECRET_KEY")
    )
    ADMIN_TOKEN = os.environ.get("ADMIN_TOKEN")
    FRONTEND_URL = os.environ.get("FRONTEND_URL")
    ALLOWED_ORIGINS_ENV = os.environ.get("ALLOWED_ORIGINS", "")
    DATABASE_URL = os.environ.get("DATABASE_URL")
    REDIS_URL = os.environ.get("REDIS_URL")
    SESSION_COOKIE_DOMAIN = os.environ.get("SESSION_COOKIE_DOMAIN") or None
    SESSION_COOKIE_SAMESITE = os.environ.get("SESSION_COOKIE_SAMESITE", "Lax")

    @classmethod
    def validar_e_configurar(cls):
        cls.IS_PRODUCTION = is_production()
        cls.SECRET_KEY = os.environ.get("SECRET_KEY") or os.environ.get(
            "FLASK_SECRET_KEY"
        )
        cls.SAVE_ENCRYPTION_KEY = (
            os.environ.get("SAVE_ENCRYPTION_KEY") or cls.SECRET_KEY
        )
        cls.ADMIN_TOKEN = os.environ.get("ADMIN_TOKEN")
        cls.FRONTEND_URL = os.environ.get("FRONTEND_URL")
        cls.ALLOWED_ORIGINS_ENV = os.environ.get("ALLOWED_ORIGINS", "")
        cls.DATABASE_URL = os.environ.get("DATABASE_URL")
        cls.REDIS_URL = os.environ.get("REDIS_URL")
        cls.SESSION_COOKIE_DOMAIN = os.environ.get("SESSION_COOKIE_DOMAIN") or None
        cls.SESSION_COOKIE_SAMESITE = os.environ.get("SESSION_COOKIE_SAMESITE", "Lax")

        if not cls.SECRET_KEY:
            cls.SECRET_KEY = "DEV_SECRET_DO_NOT_USE_IN_PROD_1982"
        if not cls.SAVE_ENCRYPTION_KEY:
            cls.SAVE_ENCRYPTION_KEY = "DEV_SAVE_KEY_DO_NOT_USE_IN_PROD_1982"

        cls.FRONTEND_URL = cls.FRONTEND_URL or "http://localhost:5000"

        if not cls.DATABASE_URL:
            db_path = "/tmp/local_testes.db" if os.environ.get("VERCEL") else "local_testes.db"
            cls.DATABASE_URL = f"sqlite:///{db_path}"
            logger.warning("Aviso: DATABASE_URL ausente. Usando SQLite.")
        elif cls.DATABASE_URL.startswith("postgres://"):
            cls.DATABASE_URL = cls.DATABASE_URL.replace(
                "postgres://", "postgresql://", 1
            )

        if not cls.ADMIN_TOKEN:
            cls.ADMIN_TOKEN = secrets.token_hex(16)

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
        cls.SQLALCHEMY_ENGINE_OPTIONS = {
            "pool_pre_ping": True,
            "pool_recycle": int(os.environ.get("DB_POOL_RECYCLE", "300")),
            "pool_size": int(os.environ.get("DB_POOL_SIZE", "5")),
            "max_overflow": int(os.environ.get("DB_MAX_OVERFLOW", "10")),
            "pool_timeout": int(os.environ.get("DB_POOL_TIMEOUT", "30")),
        }


Config.validar_e_configurar()