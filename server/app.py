import os
import logging
from logging.handlers import RotatingFileHandler

from flask import send_from_directory

from config import Config
from app_factory import create_app

app = create_app()


# logging em arquivo

logging.basicConfig(
    level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s"
)
log_file = os.path.join(app.config.get("BASE_DIR"), "villas_boas.log")
file_handler = RotatingFileHandler(
    log_file, maxBytes=10_000_000, backupCount=5, encoding="utf-8"
)
file_handler.setFormatter(logging.Formatter("%(asctime)s [%(levelname)s] %(message)s"))
logging.getLogger().addHandler(file_handler)


# rotas estáticas

@app.route("/")
def raiz():
    return send_from_directory(app.config.get("BASE_DIR"), "index.html")


@app.route("/ping")
def ping():
    return "Estou vivo!", 200


@app.route("/style.css")
def serve_css():
    if os.path.exists(os.path.join(app.config.get("BASE_DIR"), "style.min.css")):
        return send_from_directory(app.config.get("BASE_DIR"), "style.min.css")
    return send_from_directory(app.config.get("BASE_DIR"), "style.css")


@app.route("/script.js")
def serve_js():
    if os.path.exists(os.path.join(app.config.get("BASE_DIR"), "script.min.js")):
        return send_from_directory(app.config.get("BASE_DIR"), "script.min.js")
    return send_from_directory(app.config.get("BASE_DIR"), "script.js")


@app.errorhandler(404)
@app.errorhandler(405)
def page_not_found(e):
    return send_from_directory(app.config.get("BASE_DIR"), "index.html")


if __name__ == "__main__":
    app.run(debug=True, port=5000)