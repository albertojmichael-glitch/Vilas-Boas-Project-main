# Sistema de dificuldade

Todos os números ficam em [`server/data/balance_config.json`](../server/data/balance_config.json). A edição web lê esse arquivo em tempo de execução, e a offline (Rust) o embute na compilação (`offline/build.rs`). Mudou um número? As duas edições mudam juntas.

## Modos

| Modo | Vida | Noite de segurança | "Ela" | Observações |
| :--- | :---: | :--- | :--- | :--- |
| **NORMAL** (menu 1/2) | 3 | energia 100, fúria 1 | IA branda | Ficar parado 5 turnos chama a "Ela". |
| **PESADELO** (menu 3/4) | 2 | energia 70–76, fúria 2 | IA agressiva | Recursos escassos, inimigos rápidos. |
| **Speedrun** (menu 5 → 1) | 3 | como o NORMAL | — | Conquista ao vencer em menos de 5 min (neutro) ou 6 min (verdadeiro). |
| **Fantasma** (menu 5 → 2) | 3 | como o NORMAL | — | Barulho acima de 50% mata na hora. |
| **Breu total** (menu 5 → 3) | 3 | como o NORMAL | — | Lanterna quebrada; não há tropeço no escuro. |
| **Personalizado** (menu 6) | 1–10 | energia 100 | — | O jogador escolhe a bateria (1–99 turnos), os espaços extras na mochila (0–9) e a vida. |

As opções 2 e 4 são as mesmas partidas com o texto instantâneo.

## O que pesa em cada partida

**Luz**
- A lanterna começa com 3 turnos; cada bateria recarrega para 12. Salas iluminadas não gastam luz (fliperamas, hall de entrada, quarto de refrigeração, 01, balcão e mesas de jantar).
- No escuro: no 3º turno o pânico dispara o barulho para 100%. A cada 5 turnos no escuro perde-se 1 de vida.
- **Tropeço** (10% por movimento sem luz, fora de sala iluminada): tira 1 de vida, mas **nunca é o golpe final**. Com 1 de vida, o tropeço só faz barulho (`tropeco_pode_matar: false`).

**Barulho (visível no HUD como SOM)**
- Andar soma 15; ações comuns somam 5. `esperar`, `esconder`, `olhar` e `inventario` reduzem 10, e todo turno reduz mais 5.
- A partir de 50% o animatrônico entra em **alerta**; a partir de 80% ele **caça** o jogador pelo caminho mais curto.

**Perseguidores**
- **Animatrônico do barulho** (todos os modos): patrulha e caça conforme o barulho. Ao alcançar o jogador começa um combate; só `correr [direção]` salva.
- **"Ela"** (NORMAL e PESADELO): o perfil muda por dificuldade (tabela abaixo).
  - Dorme no início e depois patrulha ou rastreia o barulho.
  - Usa **dutos de ventilação exclusivos dela**: some por 1 turno e reaparece em outra sala com grade, a até 2 salas de distância. A grade range antes de ela sair.
  - Os passos são ouvidos com a direção de onde vêm.
  - O encontro vira combate: `correr` escapa com −1 de vida e a deixa atordoada.
- **Rotas de sobrevivência:** a "Ela" só chega a pé às salas sem grade (fliperamas, 01, 04, balcão e as salas pequenas dos fundos). Nenhum perseguidor entra na `01`, e lá dentro eles perdem o rastro.
- **Esconderijos:** balcão, mesas de jantar, sala de fliperamas, sala de pelúcias, cozinha principal e 04, por até 3 turnos seguidos. Escondido e em silêncio, ninguém te encontra.

| Perfil da "Ela" | NORMAL | PESADELO |
| :--- | :---: | :---: |
| Turnos dormindo no início | 10 | 3 |
| Chance de andar por turno | 30% | 65% |
| Chance de seguir o barulho (com barulho ≥ 50%) | 25% | 90% |
| Chance de usar o duto | 10% | 45% |
| Turnos atordoada após uma fuga | 5 | 2 |
| Distância em que os passos são ouvidos | 2 salas | 1 sala |

**Noite de segurança (sala 01)**
- São 24 turnos; o turno passa com `esperar`. A energia começa na faixa do modo, mais um bônus: +5 por turno de luz acima de 3 e +10 por bateria na mochila.
- Custos: a porta fechada consome 2 por turno. Câmeras, ouvir e ver os dutos custam a partir de 1; iluminar os dutos também.
- A fúria acelera o Rick; Jon e Caroline vêm pelos dutos.
- O `esperar` que chega às 06:00 já dá a vitória.

**Jon (fliperama)**
- Errar seguindo uma dica verdadeira tira 1 de vida.
- Errar porque a dica era falsa ("[SENSÓRIO CONFUSO]") custa luz e barulho, mas não vida.

## Metas e como medir

O simulador roda milhares de partidas determinísticas, com uma semente por partida, usando dois bots:

```bash
PYTHONPATH=server python -m villas_boas.simulacao --partidas 5000 \
    --dificuldade NORMAL PESADELO --estrategia competente aleatoria
```

| Cenário (bot `competente`) | Meta | Medido (5.000 partidas) |
| :--- | :---: | :---: |
| PESADELO | 15–30% de vitórias | ~27% |
| NORMAL | 55–80% de vitórias | ~72% |

`tests/test_balanceamento.py` trava essas faixas no CI. Ele também exige que não haja travamentos nem exceções e que o tropeço e o Jon nunca sejam causa de morte de um jogador atento. O lote pesado roda com `pytest -m slow`.

## Como ajustar com segurança

1. Mude só o `balance_config.json`.
2. Meça com o simulador (≥ 5.000 partidas por cenário) e confira o histograma de causas de morte.
3. Rode a suíte completa (`pytest`) e `cargo test` na pasta `offline/`.
4. Registre o porquê da mudança no `CHANGELOG.md`.
