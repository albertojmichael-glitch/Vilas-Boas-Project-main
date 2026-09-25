import logging
from flask import Blueprint, jsonify, request
from sqlalchemy import func
from sqlalchemy.exc import SQLAlchemyError
from extensions import db, limiter
from models import Telemetria, Jogador, SaveJogo
from security import requer_admin

logger = logging.getLogger(__name__)
admin_bp = Blueprint('admin', __name__, url_prefix='/admin')

def _ranking(coluna, evento, limite=None):
    q = db.session.query(coluna, func.count().label("total")).filter(Telemetria.evento == evento).group_by(coluna).order_by(func.count().desc())
    return [{"_id": chave, "total": total} for chave, total in (q.limit(limite).all() if limite else q.all())]

@admin_bp.route("/analytics", methods=["GET"])
@requer_admin
def ver_telemetria():
    mortes = Telemetria.query.filter_by(evento="MORTE").count()
    vitorias = Telemetria.query.filter_by(evento="VITORIA").count()
    avg_luz, avg_barulho, avg_inv = db.session.query(
        func.avg(Telemetria.luz_restante), func.avg(Telemetria.nivel_barulho), func.avg(Telemetria.inventario_qtd)
    ).filter(Telemetria.evento == "MORTE").first() or (0, 0, 0)

    return jsonify({
        "geral": {"mortes_totais": mortes, "vitorias_totais": vitorias},
        "top_salas_mortais": _ranking(Telemetria.sala, "MORTE", 5),
        "causas_de_morte": _ranking(Telemetria.detalhes, "MORTE", 5),
        "finais_alcancados": _ranking(Telemetria.detalhes, "VITORIA"),
        "autopsia_da_morte": {
            "luz_media_restante": round(float(avg_luz or 0), 2),
            "barulho_medio": round(float(avg_barulho or 0), 2),
            "itens_na_mochila": round(float(avg_inv or 0), 2),
        },
    })

@admin_bp.route("/auditoria-mortes", methods=["GET"])
@requer_admin
def auditoria_mortes():
    resultados = db.session.query(Jogador.email, Telemetria.sala, Telemetria.log_comandos).join(Telemetria, Jogador.id == Telemetria.jogador_id).filter(Telemetria.evento == "MORTE").limit(50).all()
    return jsonify({"mortes_identificadas": [{"vitima": e, "morreu_na_sala": s, "ultimas_palavras": c[-3:] if c else []} for e, s, c in resultados]})

@admin_bp.route("/saves", methods=["GET"])
@limiter.limit("10 per minute")
@requer_admin
def listar_saves_paginados():
    page = max(request.args.get("page", 1, type=int), 1)
    paginacao = SaveJogo.query.order_by(SaveJogo.atualizado_em.desc()).paginate(page=page, per_page=10, error_out=False)
    return jsonify({
        "page": page, "limit": 10, "total_saves": paginacao.total, "total_pages": paginacao.pages,
        "saves": [{"sid": s.sid, "atualizado_em": s.atualizado_em} for s in paginacao.items]
    })

@admin_bp.route("/replay/<int:id_replay>", methods=["GET"])
@requer_admin
def obter_replay(id_replay):
    try:
        doc = db.session.get(Telemetria, id_replay)
        if not doc:
            return jsonify({"erro": "Replay não encontrado ou expirado."}), 404

        return jsonify({
            "jogador": doc.evento,
            "sala_final": doc.sala,
            "log_comandos": doc.log_comandos or [],
        })
    except SQLAlchemyError:
        db.session.rollback()
        logger.exception("Erro ao buscar replay")
        return jsonify({"erro": "Erro interno do servidor"}), 500