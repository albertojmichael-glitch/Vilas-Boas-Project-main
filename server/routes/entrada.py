"""Contrato HTTP comum da API. Rotas sem parâmetros recusam dados extras."""

from flask import jsonify, request

CONSULTAS = {
    "admin.listar_saves_paginados": {"page"},
    "auth.login_google": {"iniciais"},
    "auth.auth_google_callback": {
        "code",
        "state",
        "scope",
        "authuser",
        "prompt",
        "hd",
        "iss",
        "error",
        "error_description",
        "error_uri",
        "session_state",
    },
}
CORPOS = {
    "game.receber_comando": 4096,
    "game.navegar": 1024,
    "game.importar_save": 200_000,
}


def validar_requisicao():
    if not request.endpoint or request.method == "OPTIONS":
        return None
    if len(request.query_string) > 8192:
        return jsonify(erro="Consulta muito longa."), 414
    permitidos = CONSULTAS.get(request.endpoint, set())
    for chave, valores in request.args.lists():
        if chave not in permitidos or len(valores) != 1 or len(valores[0]) > 4096:
            return jsonify(erro="Parâmetros de consulta inválidos."), 400
    limite = CORPOS.get(request.endpoint)
    if limite is None:
        if request.get_data():
            return jsonify(erro="Esta operação não aceita corpo."), 400
        return None
    if request.content_length is not None and request.content_length > limite:
        return jsonify(erro="Pedido muito grande."), 413
    if not request.is_json:
        return jsonify(erro="Use o formato application/json."), 415
    if len(request.get_data()) > limite:
        return jsonify(erro="Pedido muito grande."), 413
    try:
        corpo = request.get_json(silent=True)
    except RecursionError:
        corpo = None
    if not isinstance(corpo, dict):
        return jsonify(erro="Envie um objeto JSON válido."), 400
    return None
