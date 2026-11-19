"""Atalho legado para o simulador de balanceamento.

A antiga "rota dourada" com comandos fixos foi substituída pelo bot
``competente`` de ``villas_boas.simulacao``, que joga a partida inteira
(inclusive a noite de segurança) com a dificuldade configurada de verdade.

    PYTHONPATH=server python server/simulador_balanceamento.py --partidas 2000
"""

from villas_boas.simulacao.__main__ import main

if __name__ == "__main__":
    raise SystemExit(main())
