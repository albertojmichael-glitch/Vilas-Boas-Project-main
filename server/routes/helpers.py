import uuid
import re
import html
import logging
from flask import session, jsonify
from pydantic import BaseModel, Field, field_validator
from ui import DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET, UIHandler

logger = logging.getLogger(__name__)

class ComandoRequest(BaseModel):
    comando: str = Field(default="", max_length=256, pattern=r"^[a-zA-Z0-9\s\"\'\-\_áéíóúâêôãõçÁÉÍÓÚÂÊÔÃÕÇ\.\:]+$")
    telemetria: bool = Field(default=True)

    @field_validator("comando", mode="before")
    def limpar_comando(cls, v):
        return v.replace("\x00", "").replace("\0", "") if isinstance(v, str) else ""

def obter_sid_seguro():
    sid_bruto = session.get("sid")
    if not sid_bruto: return None
    try:
        return str(uuid.UUID(str(sid_bruto)))
    except (ValueError, TypeError, AttributeError):
        logger.warning(f"Alerta de Segurança: SID inválido/adulterado: {sid_bruto}")
        return None

class WebUIHandler(UIHandler):
    def __init__(self):
        self.buffer = []
    def limpar(self):
        self.buffer.append("@@CLEAR@@")
    def pausar(self, segs):
        self.buffer.append(f"@@PAUSE@@{int(segs * 1000)}")
    def exibir(self, texto):
        self.animar(texto, 0.015)
    def animar(self, texto, tempo=0.03, cor="", jogo=None):
        cor_nome = "verde"
        if cor == DOS_BRANCO: cor_nome = "branco"
        elif cor == DOS_AMARELO: cor_nome = "amarelo"
        elif cor == DOS_VERMELHO: cor_nome = "vermelho"
        if jogo and getattr(jogo, "fast_mode", False): tempo = 0
        self.buffer.append(f"@@TYPE@@{cor_nome}@@{int(tempo * 1000)}@@{texto}")
    def obter_input(self, prompt_text):
        return ""

def ansi_para_html(texto_ansi):
    mapa_cores = {DOS_VERDE: "verde", DOS_BRANCO: "branco", DOS_AMARELO: "amarelo", DOS_VERMELHO: "vermelho"}
    padrao = re.compile("(" + "|".join(re.escape(c) for c in list(mapa_cores.keys()) + [RESET]) + ")")
    partes = padrao.split(texto_ansi)
    html_out, aberto = [], False
    for parte in partes:
        if parte in mapa_cores:
            if aberto: html_out.append("</span>")
            html_out.append(f'<span class="{mapa_cores[parte]}">')
            aberto = True
        elif parte == RESET:
            if aberto: html_out.append("</span>"); aberto = False
        else:
            html_out.append(html.escape(parte))
    if aberto: html_out.append("</span>")
    return "".join(html_out)

def gerar_resposta_json(jogo):
    linhas = [ansi_para_html(linha) for linha in jogo.ui_handler.buffer if linha.strip() != ""] if jogo and hasattr(jogo, "ui_handler") else []
    if jogo and hasattr(jogo, "ui_handler"): jogo.ui_handler.buffer.clear()
    
    conquistas_enviadas = getattr(jogo, "novas_conquistas_turno", []).copy()
    if hasattr(jogo, "novas_conquistas_turno"): jogo.novas_conquistas_turno.clear()

    estado_jogo = getattr(jogo, "estado_atual", "")
    sala_exibicao = "SISTEMA" if estado_jogo in ["MENU", "AGUARDANDO_DIR"] else getattr(jogo, "sala_atual", "SISTEMA").upper()

    return jsonify({
        "linhas": linhas,
        "estado": {
            "hp": getattr(jogo, "hp", 0),
            "inventario": getattr(jogo, "inventario", []),
            "luz_restante": getattr(jogo, "turnos_luz", 0),
            "bolsas_coletadas": getattr(jogo, "bolsas_coletadas", 0),
            "estado_jogo": estado_jogo,
            "tempo_final": getattr(jogo, "tempo_total_segundos", 0.0),
            "som": getattr(jogo, "nivel_barulho", 0),
            "sala": sala_exibicao,
            "final_alcancado": getattr(jogo, "sala_atual", "Desconhecido"),
        },
        "novas_conquistas": conquistas_enviadas,
    })