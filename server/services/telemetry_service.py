import queue
from sqlalchemy.exc import SQLAlchemyError
from extensions import db
from models import Telemetria


fila_telemetria = queue.Queue()

def worker_telemetria(app_instance):
    
    while True:
        lote = []
        try:
            evento = fila_telemetria.get(timeout=5.0)
            lote.append(evento)
            while len(lote) < 50:
                try:
                    lote.append(fila_telemetria.get_nowait())
                except queue.Empty:
                    break
        except queue.Empty:
            pass 

        if lote:
           
            with app_instance.app_context():
                try:
                    db.session.add_all(lote)
                    db.session.commit()
                except SQLAlchemyError as e:
                    db.session.rollback()
                    
                    db_orig = getattr(e, "orig", None)
                    pg_code = getattr(db_orig, "pgcode", "N/A") if db_orig else "N/A"
                    
                    app_instance.logger.error(
                        "Falha na transação do PostgreSQL",
                        extra={
                            "db_error_type": type(e).__name__,
                            "pg_code": pg_code,
                            "statement": getattr(e, "statement", "N/A"),
                            "detalhes": str(db_orig) if db_orig else str(e)
                        }
                    )

def registrar_telemetria(evento, sala, dificuldade, detalhes="", jogo=None, sid_sessao=None, permite_telemetria=True):
    
    if not permite_telemetria:
        return

    registro = Telemetria(
        evento=str(evento)[:50],
        sala=str(sala)[:50],
        dificuldade=str(dificuldade)[:50],
        detalhes=str(detalhes)[:256],
        sid_sessao=sid_sessao
    )

    if jogo:
        registro.hp_restante = getattr(jogo, "hp", 0)
        registro.luz_restante = getattr(jogo, "turnos_luz", 0)
        registro.nivel_barulho = getattr(jogo, "nivel_barulho", 0)
        registro.inventario_qtd = len(getattr(jogo, "inventario", []))
        registro.bolsas = getattr(jogo, "bolsas_coletadas", 0)
        registro.log_comandos = getattr(jogo, "log_comandos", [])

    fila_telemetria.put(registro)