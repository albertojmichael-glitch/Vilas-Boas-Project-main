import time
import uuid

from flask_sqlalchemy import SQLAlchemy

from sqlalchemy import JSON, func, Index

from extensions import db


class Jogador(db.Model):
    __tablename__ = "jogadores"

    id = db.Column(db.Integer, primary_key=True)
    iniciais = db.Column(db.String(3), nullable=False)
    
    email = db.Column(db.String(120), unique=True, index=True, nullable=True) 
    tempo_segundos = db.Column(db.Float, nullable=False)
    final_alcancado = db.Column(db.String(50))
    dificuldade = db.Column(db.String(20))
    timestamp = db.Column(db.Float, default=time.time, index=True)
    
    saves = db.relationship('SaveJogo', backref='jogador', lazy=True)
    historico_telemetria = db.relationship('Telemetria', backref='jogador', lazy=True)

class SaveJogo(db.Model):
    __tablename__ = "saves"
    
    
    __table_args__ = (
        Index('ix_save_jogador_atualizado', 'jogador_id', 'atualizado_em'),
    )

    
    sid = db.Column(db.Uuid(as_uuid=True), primary_key=True, default=uuid.uuid4)
    jogador_id = db.Column(db.Integer, db.ForeignKey('jogadores.id'), nullable=True)
    
    metadados = db.Column(JSON, default=dict)
    save_version = db.Column(db.Integer, default=1, nullable=False)
    checksum = db.Column(db.String(64), index=True)
    
   
    dados = db.Column(db.Text, nullable=False)
    
    criado_em = db.Column(db.DateTime(timezone=True), server_default=func.now())
    atualizado_em = db.Column(db.DateTime(timezone=True), server_default=func.now(), onupdate=func.now())

class Telemetria(db.Model):
    __tablename__ = "telemetria"
    
   
    __table_args__ = (
        Index('ix_telemetria_evento_sala', 'evento', 'sala'),
        Index('ix_telemetria_evento_timestamp', 'evento', 'timestamp'),
        Index('ix_telemetria_jogador_evento', 'jogador_id', 'evento'),
    )

    id = db.Column(db.Integer, primary_key=True)
    jogador_id = db.Column(db.Integer, db.ForeignKey('jogadores.id'), nullable=True)
    sid_sessao = db.Column(db.String(36), index=True)
    
    evento = db.Column(db.String(50), nullable=False)
    sala = db.Column(db.String(50))
    dificuldade = db.Column(db.String(20))
    detalhes = db.Column(db.String(256))
    
    hp_restante = db.Column(db.Integer)
    luz_restante = db.Column(db.Integer)
    nivel_barulho = db.Column(db.Integer)
    inventario_qtd = db.Column(db.Integer)
    bolsas = db.Column(db.Integer)
    log_comandos = db.Column(JSON)
    
    timestamp = db.Column(db.Float, default=time.time)


class Compartilhamento(db.Model):
   
   
    __tablename__ = "compartilhamentos"

    share_token = db.Column(db.String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    original_sid = db.Column(db.String(36), nullable=False)
    expires_at = db.Column(db.Float, nullable=False)