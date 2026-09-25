import os
import threading
from datetime import timedelta
from flask import Flask
from werkzeug.middleware.proxy_fix import ProxyFix

from config import Config
from extensions import db, migrate, cors, limiter, oauth

def create_app():
    """Monta o aplicativo Flask e junta todas as peças."""
    app = Flask(__name__, static_folder=Config.BASE_DIR, static_url_path="/")
    
    app.config.from_object(Config)
    app.secret_key = app.config.get("SECRET_KEY")
    app.wsgi_app = ProxyFix(app.wsgi_app, x_proto=1, x_host=1)
    app.config["PERMANENT_SESSION_LIFETIME"] = timedelta(days=30)
    app.config["MAX_CONTENT_LENGTH"] = 1 * 1024 * 1024

    if app.config.get("IS_PRODUCTION"):
        app.config.update(SESSION_COOKIE_SECURE=True, SESSION_COOKIE_HTTPONLY=True, SESSION_COOKIE_SAMESITE="None")
    else:
        app.config.update(SESSION_COOKIE_SECURE=False, SESSION_COOKIE_HTTPONLY=True, SESSION_COOKIE_SAMESITE="Lax")

   
    db.init_app(app)
    migrate.init_app(app, db)

    
    raw_origins = str(Config.ALLOWED_ORIGINS)

    
    if "](" in raw_origins:
        raw_origins = raw_origins.split("](")[0].replace("[", "")

    
    origens_permitidas = [origem.strip() for origem in raw_origins.split(",") if origem.strip()]

    
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