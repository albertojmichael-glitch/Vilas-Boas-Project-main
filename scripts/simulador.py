"""Atalho para o simulador de balanceamento (rodar da raiz do repositório).

    python scripts/simulador.py --partidas 5000 --dificuldade PESADELO NORMAL \
        --estrategia competente aleatoria

Equivale a ``PYTHONPATH=server python -m villas_boas.simulacao ...``.
"""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "server"))

from villas_boas.simulacao.__main__ import main

if __name__ == "__main__":
    raise SystemExit(main())
