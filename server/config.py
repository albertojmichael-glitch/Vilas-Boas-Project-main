import os
import sys
import secrets

BASE_DIR = os.path.abspath(os.path.dirname(__file__))

class Config:

    BASE_DIR = os.path.abspath(os.path.dirname(__file__))
    
    IS_PRODUCTION = bool(
        os.environ.get("FLASK_ENV") == "production"
        or os.environ.get("RENDER")
        or os.environ.get("RAILWAY_STATIC_URL")
        or os.environ.get("PROD")
    )

    
    SECRET_KEY = os.environ.get("SECRET_KEY") or os.environ.get("FLASK_SECRET_KEY")
    ADMIN_TOKEN = os.environ.get("ADMIN_TOKEN")
    FRONTEND_URL = os.environ.get("FRONTEND_URL")
    ALLOWED_ORIGINS_ENV = os.environ.get("ALLOWED_ORIGINS", "")
    DATABASE_URL = os.environ.get("DATABASE_URL")
    REDIS_URL = os.environ.get("REDIS_URL")

    @classmethod
    def validar_e_configurar(cls):
       
       
        if cls.IS_PRODUCTION:
            if not cls.SECRET_KEY or not cls.ADMIN_TOKEN:
                sys.exit("➣ ERRO FATAL: SECRET_KEY e/ou ADMIN_TOKEN ausentes na produção.")
            if not cls.FRONTEND_URL:
                sys.exit("➣ ERRO FATAL: FRONTEND_URL ausente na produção.")
            if not cls.DATABASE_URL:
                sys.exit("➣ ERRO FATAL: DATABASE_URL ausente na produção.")


        if not cls.SECRET_KEY:
            cls.SECRET_KEY = "DEV_SECRET_DO_NOT_USE_IN_PROD_1982"

      
        cls.FRONTEND_URL = cls.FRONTEND_URL or "http://localhost:5000"
        
        if not cls.DATABASE_URL:
            cls.DATABASE_URL = "sqlite:///local_testes.db"
            print(" Aviso: DATABASE_URL ausente. Usando SQLite local.")
        elif cls.DATABASE_URL.startswith("postgres://"):
            cls.DATABASE_URL = cls.DATABASE_URL.replace("postgres://", "postgresql://", 1)

    
        if not cls.IS_PRODUCTION and not cls.ADMIN_TOKEN:
            cls.ADMIN_TOKEN = secrets.token_hex(16)
            print(f"\n➣ AVISO DEV: ADMIN_TOKEN gerado localmente: {cls.ADMIN_TOKEN}\n")

       
        if cls.ALLOWED_ORIGINS_ENV:
            cls.ALLOWED_ORIGINS = [origem.strip() for origem in cls.ALLOWED_ORIGINS_ENV.split(",") if origem.strip()]
        elif cls.IS_PRODUCTION:
            cls.ALLOWED_ORIGINS = [cls.FRONTEND_URL]
        else:
            cls.ALLOWED_ORIGINS = ["http://localhost:5000", "http://127.0.0.1:5000", cls.FRONTEND_URL]

        
        cls.SQLALCHEMY_DATABASE_URI = cls.DATABASE_URL
        cls.SQLALCHEMY_TRACK_MODIFICATIONS = False
        cls.SQLALCHEMY_ENGINE_OPTIONS = {"pool_pre_ping": True, "pool_recycle": 300}


Config.validar_e_configurar()