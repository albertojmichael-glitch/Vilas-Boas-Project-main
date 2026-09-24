import time
import uuid

from flask_sqlalchemy import SQLAlchemy

from sqlalchemy import JSON

db = SQLAlchemy()


class Jogador(db.Model):
    """Registro de jogadores e estatísticas para o leaderboard."""
    
    __tablename__ = "jogadores"

    id = db.Column(db.Integer, primary_key=True)
    iniciais = db.Column(db.String(3), nullable=False)
    email = db.Column(db.String(255), nullable=False)
    tempo_segundos = db.Column(db.Float, nullable=False)
    final_alcancado = db.Column(db.String(100))
    dificuldade = db.Column(db.String(50))
    timestamp = db.Column(db.Float, default=time.time, index=True)
    saves = db.relationship('SaveJogo', backref='jogador', lazy=True)
    historico_telemetria = db.relationship('Telemetria', backref='jogador', lazy=True)


class SaveJogo(db.Model):
    """Armazenamento de saves criptografados por sessão do jogador."""
    
    __tablename__ = "saves"

    sid = db.Column(db.String(36), primary_key=True)
    jogador_id = db.Column(db.Integer, db.ForeignKey('jogadores.id'), nullable=True)
    dados = db.Column(db.Text, nullable=False)  # texto Fernet
    atualizado_em = db.Column(db.Float, default=time.time, onupdate=time.time, index=True)


class Telemetria(db.Model):
    """Histórico de eventos e comandos de jogo para auditoria e analytics."""
    
    __tablename__ = "telemetria"

    id = db.Column(db.Integer, primary_key=True)
    jogador_id = db.Column(db.Integer, db.ForeignKey('jogadores.id'), nullable=True)
    sid_sessao = db.Column(db.String(36), index=True)
    evento = db.Column(db.String(50), index=True, nullable=False)
    sala = db.Column(db.String(50), index=True)
    dificuldade = db.Column(db.String(50))
    detalhes = db.Column(db.String(256))
    timestamp = db.Column(db.Float, default=time.time)
    hp_restante = db.Column(db.Integer)
    luz_restante = db.Column(db.Integer)
    nivel_barulho = db.Column(db.Integer)
    inventario_qtd = db.Column(db.Integer)
    bolsas = db.Column(db.Integer)
    log_comandos = db.Column(JSON, default=list)


class Compartilhamento(db.Model):
    """Gerenciamento de tokens de tempo limite para links de share."""
   
    __tablename__ = "compartilhamentos"

    share_token = db.Column(db.String(36), primary_key=True, default=lambda: str(uuid.uuid4()))
    original_sid = db.Column(db.String(36), nullable=False)
    expires_at = db.Column(db.Float, nullable=False)