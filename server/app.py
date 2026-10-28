import logging
import os
from logging.handlers import RotatingFileHandler

from app_factory import create_app
from flask import jsonify

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


@app.route("/")
def raiz():
    return jsonify({"status": "online", "servico": "Vilas Boas API"}), 200


@app.route("/ping")
def ping():
    return "Estou vivo!", 200


@app.errorhandler(404)
def page_not_found(e):
    return jsonify({"erro": "Recurso não encontrado."}), 404


@app.errorhandler(405)
def method_not_allowed(e):
    return jsonify({"erro": "Método não permitido."}), 405


if __name__ == "__main__":
    port = int(os.environ.get("PORT", "5000"))

    app.run(host="0.0.0.0", port=port)
