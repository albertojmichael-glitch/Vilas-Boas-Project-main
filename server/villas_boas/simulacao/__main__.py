"""CLI do simulador de balanceamento.

Exemplo:
    PYTHONPATH=server python -m villas_boas.simulacao --partidas 10000 \\
        --dificuldade PESADELO --estrategia competente --seed 1
"""

import argparse
import json
import sys
import time

from villas_boas.simulacao.estrategias import ESTRATEGIAS
from villas_boas.simulacao.simulador import formatar_relatorio, rodar_lote


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--partidas", type=int, default=2000)
    parser.add_argument(
        "--dificuldade",
        nargs="+",
        default=["PESADELO"],
        choices=["NORMAL", "PESADELO"],
    )
    parser.add_argument(
        "--estrategia",
        nargs="+",
        default=["competente"],
        choices=sorted(ESTRATEGIAS),
    )
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--max-comandos", type=int, default=None)
    parser.add_argument("--json", action="store_true", help="saída em JSON")
    args = parser.parse_args(argv)

    relatorios = []
    for dificuldade in args.dificuldade:
        for estrategia in args.estrategia:
            inicio = time.perf_counter()
            rel = rodar_lote(
                dificuldade, estrategia, args.partidas, args.seed, args.max_comandos
            )
            duracao = time.perf_counter() - inicio
            relatorios.append(rel)
            if not args.json:
                print(formatar_relatorio(rel))
                print(f"({duracao:.1f}s)\n")

    if args.json:
        json.dump([r.como_dict() for r in relatorios], sys.stdout, indent=2)
        print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
