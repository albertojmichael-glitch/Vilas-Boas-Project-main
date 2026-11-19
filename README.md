# Vilas Boas OS (v1.982)

![Status](https://img.shields.io/badge/Status-Desenvolvimento-red?style=for-the-badge)
![Python](https://img.shields.io/badge/Python-3.11+-blue?style=for-the-badge&logo=python&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5.x-3178C6?style=for-the-badge&logo=typescript&logoColor=white)
![Flask](https://img.shields.io/badge/Flask-3.x-black?style=for-the-badge&logo=flask&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-offline-000000?style=for-the-badge&logo=rust&logoColor=white)

> *"Redenção dos meus erros. Perdão por não ter vindo antes."*

**Vilas Boas** é um jogo de terror psicológico em texto (*text adventure*) e sobrevivência, com estética retrô **MS-DOS / CRT**. Ele roda no navegador (cliente TypeScript + API Python/Flask) e numa edição offline nativa em Rust.

🔗 **[Jogue agora no navegador](https://vilas-boas-project.vercel.app/)**

> ⚠️ **Aviso de conteúdo:** terror com morte violenta, desaparecimento, depressão e culpa, fogo, sustos súbitos com imagem e som alto e efeitos de tela piscando. O efeito CRT pode ser desligado nas configurações (⚙). O mesmo aviso aparece na tela de boot do jogo.

---

## Sumário

1. [Como jogar](#como-jogar)
2. [Arquitetura](#arquitetura)
3. [Rodando localmente](#rodando-localmente)
4. [Variáveis de ambiente](#variáveis-de-ambiente)
5. [Saves e segurança](#saves-e-segurança)
6. [Dificuldade e balanceamento](#dificuldade-e-balanceamento)
7. [Acessibilidade](#acessibilidade)
8. [Testes e qualidade](#testes-e-qualidade)
9. [Deploy](#deploy)
10. [Edição offline](#edição-offline)
11. [Arquivo confidencial (lore e spoilers)](#arquivo-confidencial-lore-e-spoilers)
12. [Licença](#licença)

---

## Como jogar

Ligue a TV, digite `dir` e escolha um modo no menu:

| Opção | Modo |
| :---: | :--- |
| 1 / 2 | NORMAL (2 = texto instantâneo) |
| 3 / 4 | PESADELO (4 = texto instantâneo) |
| 5 | Modos de desafio: Speedrun, Fantasma, Breu total |
| 6 | Personalizado: bateria, mochila e vida |

**Comandos principais**

| Comando | O que faz |
| :--- | :--- |
| `ir frente` / `f`, `t`, `e`, `d` | Anda (também aceita o nome da saída, ex.: `sala 1`, `01`) |
| `voltar` | Volta pela saída de trás |
| `correr [direção]` | Foge de um combate (custa barulho; fugir da "Ela" custa 1 de vida) |
| `olhar`, `examinar [objeto]` | Descreve a sala ou um objeto |
| `pegar`, `largar`, `usar [item]` | Itens (ex.: `usar bateria nova`) |
| `combinar [item] com [item]` | Combina itens |
| `inventario` / `i` | Mostra a mochila |
| `esconder` | Esconde-se no esconderijo da sala (até 3 turnos) |
| `esperar` | Passa o turno em silêncio e baixa o barulho |
| `sentar` | Assume a noite de segurança na sala 01 (exige o cartão) |
| `jogar jon` / `consertos` / `tocar piano` | Fliperamas da sala de fliperamas |
| `ajuda` | Abre o manual |

Comandos com erro de digitação recebem uma sugestão ("Você quis dizer 'pegar'?"). O progresso na web é salvo automaticamente a cada ação.

## Arquitetura

| Caminho | Conteúdo |
| :--- | :--- |
| `client/` | Interface CRT em HTML, CSS e TypeScript (`script.ts`), com áudio via Web Audio API. |
| `server/app.py`, `server/app_factory.py` | API Flask: CORS, rate limiting, OAuth, headers de segurança. |
| `server/cli.py` | O jogo direto no terminal, sem navegador. |
| `server/villas_boas/` | Motor do jogo: parser, ações, IA dos perseguidores (`ela.py`), minigames, modo dev e simulador (`simulacao/`). |
| `server/routes/`, `server/services/` | Rotas da API e serviços de save (cifra, assinatura, validação) e telemetria. |
| `server/data/` | Mapa, textos (`game_data.json`) e números de balanceamento (`balance_config.json`), compartilhados com a edição offline. |
| `offline/` | Edição nativa em Rust (egui), com as mesmas regras. |
| `tests/` | Testes `pytest` e E2E Playwright (`tests/e2e/`). |
| `docs/` | [API](docs/API.md) e [sistema de dificuldade](docs/DIFICULDADE.md). |

## Rodando localmente

**Pré-requisitos:** Python 3.11+ e Node.js 20+.

```bash
git clone https://github.com/albertojmichael-glitch/Vilas-Boas-Project-main.git
cd Vilas-Boas-Project-main
python -m venv venv
source venv/bin/activate          # Windows (PowerShell): .\venv\Scripts\Activate.ps1
pip install -e ".[dev]"
npm install
```

Também dá para usar `make setup` em Linux, macOS ou WSL.

**Jogo completo no navegador.** O front compilado e a API sobem juntos, como em produção:

```bash
npm run build
PYTHONPATH=server python tests/e2e/servidor_e2e.py
```

Abra `http://localhost:5173`.

**Só a API** (responde em `http://localhost:5000`, healthcheck em `GET /ping`):

```bash
PYTHONPATH=server python server/app.py
```

**Só o terminal (CLI):**

```bash
PYTHONPATH=server python server/cli.py
```

## Variáveis de ambiente

Em desenvolvimento, use um arquivo `.env`; em produção, o painel do provedor de hospedagem.

| Variável | Obrigatória em produção | Descrição |
| :--- | :---: | :--- |
| `APP_ENV` | **Sim** | Ambiente de execução (`production` ou `development`). |
| `SECRET_KEY` | **Sim** | Assina cookies de sessão e HMAC. **Sem ela o servidor não inicia em produção.** Fora de produção, usa uma chave de desenvolvimento e avisa no log. |
| `SAVE_ENCRYPTION_KEY` | **Sim** | Cifra (Fernet) dos saves guardados no banco. Trocá-la torna ilegíveis os saves já gravados. |
| `SAVE_SIGNING_KEY` | Não | Chave HMAC dos saves exportados. Se ausente, é derivada de `SECRET_KEY` com um rótulo próprio. |
| `ADMIN_TOKEN` | **Sim** | Token do header `X-Admin-Token` das rotas `/admin/*`. |
| `FRONTEND_URL` | **Sim** | URL pública do front (CORS, validação de `Origin`, retorno do OAuth). |
| `ALLOWED_ORIGINS` | Não | Origens extras permitidas, separadas por vírgula. |
| `SESSION_COOKIE_SAMESITE` / `SESSION_COOKIE_DOMAIN` | Não | Política e domínio do cookie de sessão. |
| `DATABASE_URL` | **Sim** | `postgresql://...` em produção; em desenvolvimento cai para SQLite local. |
| `DB_POOL_SIZE` / `DB_MAX_OVERFLOW` / `DB_POOL_RECYCLE` / `DB_POOL_TIMEOUT` | Não | Pool de conexões do SQLAlchemy. |
| `REDIS_URL` | Não | Redis compartilhado para sessões e rate limiting entre workers. |
| `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET` | Não | Login Google para o ranking. |
| `DEV_MODE_ENABLED` / `DEV_MODE_CODE` | Não | Libera o modo desenvolvedor fora de produção (sempre bloqueado em produção). |

## Saves e segurança

- **No servidor**, o progresso fica **cifrado** (Fernet) no banco. As chaves nunca saem do servidor.
- **No navegador**, nada é escondido. O save exportado (`formato: vilas-boas-save`, `versao: 2`) é um JSON **legível** com uma **assinatura HMAC** do servidor: dá para ler, mas qualquer alteração é recusada.
- **Todo estado recebido é validado** (`server/services/save_validacao.py`), seja na importação, no autosave ou num link compartilhado:
  - vida, luz, itens, salas e conquistas precisam ser possíveis;
  - o mapa é reconstruído a partir do original;
  - a dificuldade vem sempre do servidor.
- **Um estado inválido nunca sobrescreve o último save bom.** Saves antigos continuam carregando.
- **Links de compartilhamento** (`/share/<token>`) pedem confirmação via `POST`, para que bots de pré-visualização (WhatsApp, Discord) não consumam o link.
- **Modo dev** só fora de produção. Partidas com ele ficam fora do ranking e da telemetria.

Detalhes das rotas em [docs/API.md](docs/API.md).

## Dificuldade e balanceamento

Os números ficam em `server/data/balance_config.json`, e as duas edições usam o mesmo arquivo. O documento [docs/DIFICULDADE.md](docs/DIFICULDADE.md) explica cada modo, a IA da "Ela", luz, barulho, esconderijos, a noite de segurança e como ajustar com segurança.

O simulador mede o efeito de cada ajuste:

```bash
PYTHONPATH=server python -m villas_boas.simulacao --partidas 5000 \
    --dificuldade NORMAL PESADELO --estrategia competente aleatoria
```

A meta do PESADELO é **15–30%** de vitórias para um jogador competente (hoje ~27%); o NORMAL fica em ~72%.

## Acessibilidade

O cliente web é testado automaticamente contra a **WCAG 2.2 AA** (axe-core), no desktop e no celular:

- **Leitores de tela:** a saída do terminal é anunciada (`role="log"`, `aria-live`), e todos os botões e controles têm nome acessível.
- **Teclado:** dá para jogar só pelo teclado, e `Esc` fecha qualquer janela.
- **Visual:** contraste AA, zoom liberado no celular e o efeito CRT, a velocidade do texto e o volume dos sustos ajustáveis em ⚙.
- **Celular:** botões de toque com pelo menos 24 px e nenhuma rolagem horizontal.

## Testes e qualidade

```bash
PYTHONPATH=server pytest tests/                               # suíte padrão
PYTHONPATH=server pytest -m slow tests/test_balanceamento.py  # 40 mil partidas simuladas
npx playwright install chromium                               # só na primeira vez
npx playwright test                                           # acessibilidade e celular (E2E)
cd offline && cargo test                                      # edição offline
```

| Arquivo | Cobre |
| :--- | :--- |
| `test_comandos.py` | Parser, menu, sinônimos e cada saída, objeto e item de cada sala, mais entradas aleatórias que nunca podem derrubar o motor. |
| `test_saves.py`, `test_corrupcao.py` | Assinatura, validação, adulteração, arquivos truncados ou corrompidos, registros quebrados no banco. |
| `test_conteudo.py` | Termos ofensivos no texto do jogo (lista em `tests/dados/termos_ofensivos.txt`) e o aviso de conteúdo. |
| `test_balanceamento.py` | Taxas de vitória e regras de dificuldade. |
| `tests/e2e/*.spec.ts` | WCAG (axe), teclado, celular e as opções do menu no navegador. |

Lint, formatação e tipos:

```bash
ruff check . && ruff format --check .
MYPYPATH=server mypy --explicit-package-bases server/
tsc -p client/tsconfig.json --noEmit
```

## Deploy

O deploy de produção é feito pelo GitHub Actions (`.github/workflows/ci.yml`, job **deploy**):

- Roda **só em push na `main`** e **só depois** de passarem lint, testes, balanceamento, build do front e testes E2E.
- O deploy automático por git da Vercel está desligado (`"git": {"deploymentEnabled": false}` nos `vercel.json`), então nada vai ao ar sem os testes.

Secrets necessários no repositório (Settings → Secrets and variables → Actions):

| Secret | Uso |
| :--- | :--- |
| `VERCEL_TOKEN` | Token de acesso da Vercel. |
| `VERCEL_ORG_ID` | ID do time ou conta na Vercel. |
| `VERCEL_PROJECT_ID` | Projeto do **front** (raiz do repositório). |
| `VERCEL_PROJECT_ID_API` | Projeto da **API** (pasta `server/`). |
| `RAILWAY_TOKEN` + `RAILWAY_SERVICE` | Opcionais, se a API também roda no Railway. Nesse caso, desligue o deploy automático no painel do Railway. |

## Edição offline

Executável nativo em Rust para Windows e Linux, que salva o progresso localmente. Veja o [guia da edição offline](offline/README.md) para compilar ou baixar o artefato do GitHub Actions.

---

## Arquivo confidencial (lore e spoilers)

<details>
<summary><strong>⚠️ AVISO DE SPOILER — clique para expandir a história e os finais</strong></summary>

### A tragédia do Restaurante Vilas Boas

Em 1982, **Renato Fidelis Gomes**, um engenheiro brilhante na área de robótica e sistemas embarcados, construiu animatrônicos avançados para um restaurante famoso em Curitiba. Porém, por não haver ajudantes nem funcionários na administração do local, Renato teve que abandonar a robótica e focar em papéis, dinheiro e tempo. O restaurante ia muito bem, famoso e com propostas de abrir filiais, até desabar em 1994, quando ocorreu o incidente mais famoso de Curitiba: o **IPD**, incidente das pessoas desaparecidas, em que 2 funcionários, incluindo o gerente Renato Fidelis G., e uma cliente desapareceram como pó ao fim do expediente. Em 1995, sem fluxo de clientes e com diversas denúncias da vigilância sanitária, o restaurante fechou definitivamente, por decisão de **Renato Fidelis Jr.**, filho do dono, com previsão de reabrir nos anos 2000.

Em 2007, o restaurante está abandonado, mas *Renato* e *Michel* cuidam do lugar, contratando serviços de limpeza e segurança. **Caroline A.**, uma estudante universitária precisando de dinheiro rápido, consegue a vaga de segurança no restaurante e logo começa a trabalhar.
Após 4 dias como segurança, Caroline desaparece no restaurante e nunca mais é encontrada. Nesses 4 dias, ela enviava e-mails diariamente para **Rogério V.**, seu namorado, que estava na cidade vizinha à procura de trabalho. Ao saber do desaparecimento, Rogério não faz nada: se isola em casa, com medo e em depressão profunda, covarde.

8 meses após o desaparecimento de Caroline, Rogério não aguenta mais a culpa, afundando em lágrimas toda noite e em sonhos turbulentos, e vai em busca da redenção de seus erros: ele vai atrás de Caroline no restaurante.

### Os finais possíveis

O progresso da história depende de quão fundo o jogador decide ir na investigação:

- **Final Medíocre (`saida`)**: Rogério, com medo e apavorado com o ambiente do restaurante, foge covardemente e volta para casa.
- **Final Neutro (`final_neutro`)**: Rogério sobrevive até de manhã, vencendo a noite na sala de segurança e enfrentando todos os animatrônicos, mas sem pegar o item "fios cortados". Ao chegar ao hall de entrada, o animatrônico de Caroline o cerca. Há um diálogo profundo e ela reconhece o protagonista, porém sua raiva sobressai: ela mata Rogério e o condena a viver o terror dentro da máquina. O ciclo de almas presas no maquinário permanece intacto.
- **Final Verdadeiro (`Libertação`)**: Rogério consegue a chave dos fundos, vence o minigame do minotauro pegando os "fios cortados", vai à sala de fliperamas e faz cada um dos minigames, passando na prova final, o julgamento do pianista, e recebendo o cartão IV. Ele vai à sala de segurança, vence a noite e, no hall de entrada, ateia fogo no local e desliga o animatrônico onde Caroline está, fazendo-a finalmente se lembrar dele. Sem raiva e sem lamento, os dois se despedem, o animatrônico pega fogo e todos os espíritos, incluindo o de Caroline, descansam em paz.
- **Morte (`Assassinado`)**: Rogério é encontrado morto no restaurante às 8 da manhã do dia seguinte. Especialistas forenses relatam semelhanças com um ataque brutal de um predador, como um urso.
- **Final Bons Sonhos (`Durma...`)**: Rogério entra na sala de descanso e decide dormir, preferindo entregar-se à insanidade dos pesadelos, ouvindo a porta da sala se abrindo devagarzinho e sentindo braços etéreos envolvendo seu pescoço.

</details>

---

## Licença

Distribuído sob a licença Apache 2.0. Veja o arquivo [LICENSE](LICENSE).
