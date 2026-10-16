import logging

from authlib.integrations.base_client.errors import OAuthError
from extensions import MEMORIA_SESSOES, db, limiter, oauth
from flask import Blueprint, current_app, redirect, request, session, url_for
from models import Jogador, SaveJogo
from services.save_service import obter_ou_recuperar_jogo
from sqlalchemy.exc import SQLAlchemyError

logger = logging.getLogger(__name__)
auth_bp = Blueprint("auth", __name__)


@auth_bp.route("/login/google")
@limiter.limit("5 per minute")
def login_google():
    session["arcade_initials"] = request.args.get("iniciais", "???").upper()[:3]
    return oauth.google.authorize_redirect(
        url_for("auth.auth_google_callback", _external=True)
    )


@auth_bp.route("/auth/google/callback")
def auth_google_callback():
    sid = session.get("sid")
    if not sid or sid not in MEMORIA_SESSOES:
        return "Sessão de jogo não encontrada.", 400

    try:
        user_info = oauth.google.authorize_access_token().get("userinfo")
        jogo = obter_ou_recuperar_jogo(sid, MEMORIA_SESSOES)

        if not jogo:
            logger.warning("OAuth concluído, mas o jogo sumiu.")
            return redirect(
                f"{current_app.config.get('FRONTEND_URL')}/?leaderboard=falha_sessao"
            )

        if getattr(jogo, "tempo_total_segundos", 0) > 0 and not getattr(
            jogo, "dev_usado", False
        ):
            email_jogador = user_info.get("email")

            jogador = Jogador.query.filter_by(email=email_jogador).first()

            if jogador:
                if jogo.tempo_total_segundos < jogador.tempo_segundos:
                    jogador.tempo_segundos = jogo.tempo_total_segundos
                    jogador.final_alcancado = jogo.sala_atual
                    jogador.dificuldade = jogo.dificuldade_escolhida
                    jogador.iniciais = session.get("arcade_initials", "UNK")
            else:
                jogador = Jogador(
                    iniciais=session.get("arcade_initials", "UNK"),
                    email=email_jogador,
                    tempo_segundos=jogo.tempo_total_segundos,
                    final_alcancado=jogo.sala_atual,
                    dificuldade=jogo.dificuldade_escolhida,
                )
                db.session.add(jogador)

            db.session.flush()

            save_atual = db.session.get(SaveJogo, sid)
            if save_atual:
                save_atual.jogador_id = jogador.id

            from models import Telemetria

            Telemetria.query.filter_by(sid_sessao=sid).update(
                {"jogador_id": jogador.id}
            )

            db.session.commit()

        return redirect(
            f"{current_app.config.get('FRONTEND_URL')}/?leaderboard=sucesso"
        )

    except (SQLAlchemyError, OAuthError, ValueError):
        import traceback

        db.session.rollback()
        erro_detalhado = traceback.format_exc()
        logger.error(f"Erro no OAuth do Google:\n{erro_detalhado}")
        return (
            f"<h1>Erro 500 - Diagnóstico</h1><pre style='color: red; white-space: pre-wrap;'>{erro_detalhado}</pre>",
            500,
        )
