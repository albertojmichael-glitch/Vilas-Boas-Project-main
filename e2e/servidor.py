"""Hospeda o build e a API real na mesma origem, com saves isolados para o navegador."""

import os
import subprocess
import sys
import time
from pathlib import Path
from tempfile import TemporaryDirectory
from threading import Thread
from uuid import uuid4

from flask import Flask, redirect, session
from werkzeug.middleware.dispatcher import DispatcherMiddleware
from werkzeug.serving import make_server

RAIZ = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(RAIZ / "server"))


def executar():
    with TemporaryDirectory() as pasta:
        os.environ.update(
            APP_ENV="test",
            SECRET_KEY=os.urandom(32).hex(),
            DATABASE_URL=f"sqlite:///{Path(pasta).as_posix()}/jogo.db",
            VERCEL="1",
        )
        import state
        from app_factory import create_app
        from extensions import db, limiter
        from services.save_service import salvar_save_web
        from state import GameState

        state.ARQUIVO_CONQUISTAS = Path(pasta) / "conquistas.json"
        api = create_app()
        limiter.enabled = False
        site = Flask(__name__, static_folder=str(RAIZ / "dist"), static_url_path="")
        site.secret_key = api.secret_key
        cenarios = {
            "combate": {
                "sala_atual": "entrada",
                "estado_atual": "COMBATE_ANIMATRONICO",
                "atacante_combate": "ela",
            },
            "seguranca": {"sala_atual": "01", "inventario": ["cartao de seguranca"]},
            "energia": {"sala_atual": "sala de energia"},
            "cofre": {"sala_atual": "01"},
            "arcade": {
                "sala_atual": "sala de fliperamas",
                "inventario": ["moeda velha"],
            },
            "cama": {"sala_atual": "04"},
            "neutro": {"sala_atual": "entrada", "noite_vencida": True},
            "verdadeiro": {
                "sala_atual": "entrada",
                "noite_vencida": True,
                "fios_cortados_inventario": True,
            },
        }

        @site.get("/")
        def index():
            return site.send_static_file("index.html")

        @site.get("/cenario/<nome>")
        def preparar_cenario(nome):
            if nome not in cenarios:
                return "Cenário desconhecido", 404
            parametros = {
                "estado_atual": "JOGO",
                "fast_mode": True,
                "tempo_inicio": time.time() - 300,
                **cenarios[nome],
            }
            jogo = GameState(**parametros)
            sid = str(uuid4())
            with api.app_context():
                assert salvar_save_web(jogo, sid)
            session["sid"] = sid
            return redirect("/")

        # O cookie da API também precisa acompanhar a navegação para /.
        api.config["SESSION_COOKIE_PATH"] = "/"
        app = DispatcherMiddleware(site, {"/api": api})
        with make_server("127.0.0.1", 0, app, threaded=True) as servidor:
            thread = Thread(target=servidor.serve_forever, daemon=True)
            thread.start()
            try:
                url = f"http://127.0.0.1:{servidor.server_port}"
                cli = RAIZ / "node_modules" / "@playwright" / "test" / "cli.js"
                resultado = subprocess.run(
                    ["node", str(cli), "test", *sys.argv[1:]],
                    cwd=RAIZ,
                    env={**os.environ, "E2E_BASE_URL": url},
                    check=False,
                )
                return resultado.returncode
            finally:
                servidor.shutdown()
                thread.join()
                with api.app_context():
                    db.engine.dispose()


if __name__ == "__main__":
    sys.exit(executar())
