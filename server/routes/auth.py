import logging
from flask import Blueprint, redirect, url_for, session, request, current_app
from extensions import db, oauth, MEMORIA_SESSOES
from models import Jogador, SaveJogo, Telemetria
from services.save_service import obter_ou_recuperar_jogo

logger = logging.getLogger(__name__)
auth_bp = Blueprint('auth', __name__)

@auth_bp.route("/login/google")
def login_google():
    session["arcade_initials"] = request.args.get("iniciais", "???").upper()[:3]
    return oauth.google.authorize_redirect(url_for("auth.auth_google_callback", _external=True))

@auth_bp.route("/auth/google/callback")
def auth_google_callback():
    sid = session.get("sid")
    if not sid or sid not in MEMORIA_SESSOES: return "Sessão de jogo não encontrada.", 400

    try:
        user_info = oauth.google.authorize_access_token().get("userinfo")
        jogo = obter_ou_recuperar_jogo(sid, MEMORIA_SESSOES)
        
        if not jogo:
            logger.warning("OAuth concluído, mas o jogo sumiu.")
            return redirect(f"{current_app.config.get('FRONTEND_URL')}/?leaderboard=falha_sessao")
            
        if getattr(jogo, "tempo_total_segundos", 0) > 0:
            novo_jogador = Jogador(
                iniciais=session.get("arcade_initials", "UNK"),
                email=user_info.get("email"),
                tempo_segundos=jogo.tempo_total_segundos,
                final_alcancado=jogo.sala_atual,
                dificuldade=jogo.dificuldade_escolhida,
            )
            db.session.add(novo_jogador); db.session.flush() 
            
            save_atual = db.session.get(SaveJogo, sid)
            if save_atual: save_atual.jogador_id = novo_jogador.id
            Telemetria.query.filter_by(sid_sessao=sid).update({"jogador_id": novo_jogador.id})
            db.session.commit()

        return redirect(f"{current_app.config.get('FRONTEND_URL')}/?leaderboard=sucesso")
    except Exception as e:
        db.session.rollback()
        logger.error(f"Erro no OAuth do Google: {e}")
        return "Falha na autenticação.", 500