"""Servidor dos testes E2E: front compilado (dist/) em '/' e a API Flask em '/api',
igual ao que a Vercel faz em produção (rewrite /api -> backend).

Uso: PYTHONPATH=server python tests/e2e/servidor_e2e.py [porta]
(rode ``npm run build`` antes para gerar ``dist/``).
"""

import mimetypes
import os
import sys
import tempfile
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]
DIST = RAIZ / "dist"

# Banco descartável e sem telemetria: os testes nunca tocam dados reais.
_tmp = tempfile.mkdtemp(prefix="vilas_e2e_")
os.environ.setdefault("DATABASE_URL", f"sqlite:///{_tmp}/e2e.db")
os.environ.setdefault("APP_ENV", "development")
os.environ.setdefault("ENABLE_TELEMETRY", "0")
os.environ.setdefault("FRONTEND_URL", "http://localhost:5173")
sys.path.insert(0, str(RAIZ / "server"))

from app import app as api
from werkzeug.middleware.dispatcher import DispatcherMiddleware
from werkzeug.serving import run_simple
from werkzeug.wrappers import Request, Response


@Request.application
def estaticos(request):
    caminho = request.path.lstrip("/") or "index.html"
    arquivo = (DIST / caminho).resolve()
    if DIST not in arquivo.parents and arquivo != DIST or not arquivo.is_file():
        return Response("não encontrado", status=404)
    tipo = mimetypes.guess_type(arquivo.name)[0] or "application/octet-stream"
    return Response(arquivo.read_bytes(), mimetype=tipo)


aplicacao = DispatcherMiddleware(estaticos, {"/api": api})

if __name__ == "__main__":
    porta = int(sys.argv[1]) if len(sys.argv) > 1 else 5173
    run_simple("localhost", porta, aplicacao, use_reloader=False, threaded=True)
