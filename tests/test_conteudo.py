"""Conteúdo exibido ao jogador: sem termos ofensivos e com aviso de conteúdo.

Varre todo texto que o jogo pode mostrar (dados do mapa, telas, mensagens do
motor web e da edição offline, páginas do cliente) contra a lista em
``tests/dados/termos_ofensivos.txt``. Para incluir um termo, basta acrescentar
uma linha nesse arquivo.
"""

import json
import re
import unicodedata
from pathlib import Path

import pytest
from state import HeadlessUIHandler
from views import AVISO_DE_CONTEUDO, imprimir_tela_boot

RAIZ = Path(__file__).resolve().parent.parent
LISTA = Path(__file__).parent / "dados" / "termos_ofensivos.txt"
TEMAS_DO_AVISO = ("morte", "desaparecimento", "depressão", "fogo", "sustos", "piscando")


def _normalizar(texto: str) -> str:
    sem_acento = unicodedata.normalize("NFD", texto.lower())
    return "".join(c for c in sem_acento if unicodedata.category(c) != "Mn")


def _termos() -> list[str]:
    linhas = LISTA.read_text(encoding="utf-8").splitlines()
    return [
        _normalizar(t.strip()) for t in linhas if t.strip() and not t.startswith("#")
    ]


def _textos_do_jogo() -> dict[str, str]:
    arquivos = [
        *RAIZ.joinpath("server").glob("*.py"),
        *RAIZ.joinpath("server", "villas_boas").rglob("*.py"),
        *RAIZ.joinpath("server", "data").glob("*.json"),
        *RAIZ.joinpath("offline", "src").glob("*.rs"),
        *RAIZ.joinpath("client").glob("*.html"),
        *RAIZ.joinpath("client").rglob("*.ts"),
    ]
    return {
        str(a.relative_to(RAIZ)): a.read_text(encoding="utf-8", errors="ignore")
        for a in arquivos
    }


def test_lista_de_termos_nao_esta_vazia():
    assert len(_termos()) >= 10


@pytest.mark.parametrize("arquivo", sorted(_textos_do_jogo()))
def test_nenhum_termo_ofensivo_no_texto_do_jogo(arquivo):
    texto = _normalizar(_textos_do_jogo()[arquivo])
    achados = [
        termo
        for termo in _termos()
        if re.search(r"\b" + re.escape(termo) + r"\b", texto)
    ]
    assert not achados, f"{arquivo}: {achados}"


def test_dados_do_mapa_sao_json_valido_e_sem_html():
    dados = json.loads(RAIZ.joinpath("server", "data", "game_data.json").read_text())

    def strings(no):
        if isinstance(no, str):
            yield no
        elif isinstance(no, dict):
            for v in no.values():
                yield from strings(v)
        elif isinstance(no, list):
            for v in no:
                yield from strings(v)

    for texto in strings(dados):
        assert "<script" not in texto.lower() and "javascript:" not in texto.lower()


def test_aviso_de_conteudo_aparece_no_boot():
    ui = HeadlessUIHandler()
    imprimir_tela_boot(ui)
    assert any(AVISO_DE_CONTEUDO in linha for linha in ui.buffer)
    for tema in TEMAS_DO_AVISO:
        assert tema in AVISO_DE_CONTEUDO


def test_aviso_de_conteudo_igual_nas_duas_edicoes():
    rust = RAIZ.joinpath("offline", "src", "views.rs").read_text(encoding="utf-8")
    assert f'"{AVISO_DE_CONTEUDO}"' in rust


def test_readme_documenta_o_aviso_de_conteudo():
    readme = RAIZ.joinpath("README.md").read_text(encoding="utf-8")
    assert "Aviso de conteúdo" in readme
