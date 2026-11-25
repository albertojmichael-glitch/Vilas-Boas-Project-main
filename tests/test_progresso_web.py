import pytest
from flask import current_app


def iniciar_partida(cliente):
    cliente.post("/iniciar")
    for comando in ["dir", "2"]:
        resposta = cliente.post("/comando", json={"comando": comando})
        assert resposta.status_code == 200
    return resposta.get_json()


def test_marcos_pertencem_a_partida(cliente):
    dados = iniciar_partida(cliente)
    assert dados["autosave"] == "salvo"
    assert dados["progresso"]["capitulos"] == [{"id": "entrada", "nome": "Entrada"}]

    def pedir(capitulo):
        return {"acao": "capitulo", "capitulo": capitulo}

    assert cliente.post("/navegar", json=pedir("01")).status_code == 409
    outro = current_app.test_client()
    assert outro.post("/navegar", json=pedir("entrada")).status_code == 401
    outro.post("/iniciar")
    assert outro.post("/navegar", json=pedir("entrada")).status_code == 409
    assert cliente.post("/navegar", json=pedir("entrada")).status_code == 200


def test_falha_de_gravacao_preserva_ultimo_save(cliente, monkeypatch):
    iniciar_partida(cliente)
    monkeypatch.setattr("routes.game.salvar_save_web", lambda *args, **kwargs: False)
    dados = cliente.post(
        "/comando", json={"comando": "pegar tabua pequena de madeira"}
    ).get_json()
    assert dados["autosave"] == "falhou"
    assert "tabua pequena de madeira" in dados["estado"]["inventario"]
    retomado = cliente.post("/navegar", json={"acao": "continuar"}).get_json()
    assert "tabua pequena de madeira" not in retomado["estado"]["inventario"]


@pytest.mark.parametrize(
    ("metodo", "rota", "opcoes", "status"),
    [
        ("get", "/ping?extra=1", {}, 400),
        ("get", "/achievements", {"data": "extra"}, 400),
        ("post", "/comando", {"json": {"comando": "dir", "telemetria": 1}}, 400),
        ("post", "/comando", {"json": {"comando": "a" * 4096}}, 413),
        ("post", "/navegar", {"json": {"acao": "reiniciar", "sid": "outra"}}, 400),
        ("post", "/navegar", {"json": {"acao": "capitulo", "capitulo": []}}, 400),
        ("get", "/admin/saves?page=1&page=2", {}, 400),
    ],
)
def test_contrato_http(cliente, metodo, rota, opcoes, status):
    assert getattr(cliente, metodo)(rota, **opcoes).status_code == status


def test_limite_do_boot(cliente):
    for _ in range(10):
        assert cliente.post("/iniciar").status_code == 200
    bloqueio = cliente.post("/iniciar")
    assert bloqueio.status_code == 429
    assert int(bloqueio.headers["Retry-After"]) >= 0
