"""Ambiente isolado para rodar milhares de partidas sem efeitos colaterais.

* Desliga a gravação de ``conquistas.json`` feita pelas telas de final.
* Desliga o ``time.sleep`` real do minigame do Minotauro.
* Salva e restaura o estado global do ``random`` (cada partida usa a própria semente).
"""

import random
from collections.abc import Iterator
from contextlib import contextmanager


@contextmanager
def ambiente_simulacao() -> Iterator[None]:
    import views

    from villas_boas.engine.minigames import minotauro

    estado_rng = random.getstate()
    registrar_final_original = views.registrar_final
    pausar_original = minotauro.pausar

    views.registrar_final = lambda _nome: False  # type: ignore[assignment]
    minotauro.pausar = lambda *_a, **_k: None
    try:
        yield
    finally:
        views.registrar_final = registrar_final_original
        minotauro.pausar = pausar_original
        random.setstate(estado_rng)
