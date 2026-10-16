import pytest
from pydantic import ValidationError
from routes.helpers import ComandoRequest


def test_bloqueio_injecao_sql():
    with pytest.raises(ValidationError):
        ComandoRequest(comando="ir frente; DROP TABLE saves;")


def test_bloqueio_xss_cross_site_scripting():
    with pytest.raises(ValidationError):
        ComandoRequest(comando="<script>alert('hack')</script>")


def test_comando_extremamente_longo():
    with pytest.raises(ValidationError):
        ComandoRequest(comando="frente " * 100)


def test_comando_valido_passa_normalmente():
    req = ComandoRequest(comando="abrir cofre")
    assert req.comando == "abrir cofre"
