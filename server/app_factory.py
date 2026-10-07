import os
import threading
from datetime import timedelta
from flask import Flask
from werkzeug.middleware.proxy_fix import ProxyFix
from sqlalchemy.exc import SQLAlchemyError

from config import Config
from extensions import db, migrate, cors, limiter, oauth

def create_app():
    
    app = Flask(__name__, static_folder=Config.BASE_DIR, static_url_path="/")


    app.config.from_object(Config)

    is_prod = app.config.get("IS_PRODUCTION", False)

    app.config.update(
        SESSION_COOKIE_SAMESITE="None" if is_prod else "Lax",
        SESSION_COOKIE_SECURE=is_prod,
        SESSION_COOKIE_HTTPONLY=True,
        PERMANENT_SESSION_LIFETIME=timedelta(days=30),
        MAX_CONTENT_LENGTH=1 * 1024 * 1024
    )

    app.secret_key = app.config.get("SECRET_KEY")
    app.wsgi_app = ProxyFix(app.wsgi_app, x_proto=1, x_host=1)

    db.init_app(app)
    migrate.init_app(app, db)

    raw_origins = Config.ALLOWED_ORIGINS

    if isinstance(raw_origins, str):
        
        raw_origins = raw_origins.replace("[", "").replace("]", "").replace("'", "").replace('"', "")
        origens_permitidas = [origem.strip() for origem in raw_origins.split(",") if origem.strip()]
    elif isinstance(raw_origins, list):
        origens_permitidas = raw_origins
    else:
        origens_permitidas = []

    
    cors.init_app(app, 
                  resources={r"/*": {"origins": origens_permitidas}},
                  supports_credentials=True,
                  allow_headers=["Content-Type", "Authorization", "X-Admin-Token"],
                  expose_headers=["Content-Type", "Authorization"])

    limiter.init_app(app)
    oauth.init_app(app)

    oauth.register(
        name="google",
        client_id=os.environ.get("GOOGLE_CLIENT_ID"),
        client_secret=os.environ.get("GOOGLE_CLIENT_SECRET"),
        server_metadata_url="https://accounts.google.com/.well-known/openid-configuration",
        client_kwargs={"scope": "openid email profile"},
    )

    
    with app.app_context():
        from routes.game import game_bp
        from routes.admin import admin_bp
        from routes.auth import auth_bp
        
        app.register_blueprint(game_bp)
        app.register_blueprint(admin_bp)
        app.register_blueprint(auth_bp)

        
        try:
            db.session.execute(db.text("ALTER TABLE saves ADD COLUMN metadados JSON DEFAULT '{}';"))
            db.session.commit()
            print("Coluna 'metadados' adicionada com sucesso.")
        except SQLAlchemyError:
            db.session.rollback()

        try:
            db.session.execute(db.text("ALTER TABLE saves ADD COLUMN save_version INTEGER DEFAULT 1 NOT NULL;"))
            db.session.commit()
            print("Coluna 'save_version' adicionada com sucesso.")
        except SQLAlchemyError:
            db.session.rollback()

        try:
            db.session.execute(db.text("ALTER TABLE saves ADD COLUMN checksum VARCHAR(64);"))
            db.session.commit()
            print("Coluna 'checksum' adicionada com sucesso.")
        except SQLAlchemyError:
            db.session.rollback()

        
        try:
            db.session.execute(db.text("ALTER TABLE saves ADD COLUMN criado_em TIMESTAMP DEFAULT CURRENT_TIMESTAMP"))
            db.session.commit()
            print("Coluna 'criado_em' adicionada com sucesso.")
        except SQLAlchemyError:
            db.session.rollback()

        
        try:
            db.session.execute(db.text("ALTER TABLE saves ADD COLUMN atualizado_em TIMESTAMP DEFAULT CURRENT_TIMESTAMP"))
            db.session.commit()
            print("Coluna 'atualizado_em' adicionada com sucesso.")
        except SQLAlchemyError:
            db.session.rollback()

    
    from services.telemetry_service import worker_telemetria
    thread = threading.Thread(target=worker_telemetria, args=(app,), daemon=True)
    thread.start()

   
    @app.after_request
    def aplicar_headers_de_seguranca(response):
        response.headers["Content-Security-Policy"] = (
            "default-src 'self'; "
            "script-src 'self'; "
            "style-src 'self' https://fonts.googleapis.com; "
            "font-src 'self' https://fonts.gstatic.com; "
            "object-src 'none'; "
            "frame-ancestors 'none';"
        )
        response.headers["X-Content-Type-Options"] = "nosniff"
        response.headers["X-Frame-Options"] = "DENY"
        if app.config.get("IS_PRODUCTION"):
            response.headers["Strict-Transport-Security"] = "max-age=31536000; includeSubDomains"
        return response

    return app