# Vilas Boas OS (v1.982)

![Status](https://img.shields.io/badge/Status-Desenvolvimento-red?style=for-the-badge)
![Python](https://img.shields.io/badge/Python-3.11+-blue?style=for-the-badge&logo=python&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-3178C6?style=for-the-badge&logo=typescript&logoColor=white)
![Flask](https://img.shields.io/badge/Flask-3.x-black?style=for-the-badge&logo=flask&logoColor=white)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-4169E1?style=for-the-badge&logo=postgresql&logoColor=white)

> *"Rendenção dos meus erros. Perdão por não ter vindo antes."*

**Vilas Boas** é um jogo de terror psicológico, ficção interativa (*text-adventure*) e sobrevivência desenvolvido com estética retrô **MS-DOS / CRT**, arquitetura cliente-servidor desacoplada, inteligência artificial baseada em grafos (BFS) e persistência criptografada.

🔗 **[Jogue Agora no Navegador](https://vilas-boas-project.vercel.app/)**

---

## Sumário

1. [Arquitetura do Projeto](#arquitetura-do-projeto)
2. [Instalação e Execução Local](#instalação-e-execução-local)
3. [Variáveis de Ambiente](#variáveis-de-ambiente)
4. [Testes, Cobertura e Qualidade de Código](#testes-cobertura-e-qualidade-de-código)
5. [Mecânicas e Sistemas da Engine](#mecânicas-e-sistemas-da-engine)
6. [Arquivo Confidencial (Lore & Spoilers)](#arquivo-confidencial-lore--spoilers)

---

## Arquitetura do Projeto

O projeto é dividido entre o cliente web estático (hospedado na Vercel) e a API/Engine em Python (Flask):

- **`server/app.py`**: Ponto de entrada da API Web (Flask / Gunicorn).
- **`server/cli.py`**: Ponto de entrada da interface de linha de comando (CLI) para jogar diretamente no terminal.
- **`server/app_factory.py`**: Fábrica da aplicação Flask (CORS, Rate Limiting, OAuth, headers de segurança CSP/HSTS).
- **`server/villas_boas/`**: Núcleo da engine do jogo (parser de linguagem natural com distância de Levenshtein, IA do perseguidor por BFS, minigames e modo desenvolvedor).
- **`server/routes/`**: Rotas da API (`/iniciar`, `/comando`, `/save/*`, `/share/*`, `/auth/*`, `/admin/*`).
- **`server/services/`**: Persistência de saves criptografados com Fernet (AES-128-CBC + HMAC-SHA256) e worker assíncrono de telemetria.
- **`client/`**: Interface CRT em HTML5, CSS3 e TypeScript (`client/script.ts`) com sintetizador de áudio via Web Audio API.
- **`tests/`**: Suíte de testes automatizados (`pytest`) cobrindo engine, gameplay, rotas de API, autenticação, compartilhamento, painel admin, segurança e Redis.

---

## Instalação e Execução Local

### Pré-requisitos

- **Python 3.11+**
- **Node.js 20+** (opcional, apenas para recompilar o TypeScript do cliente)

### 1. Clonar e configurar o ambiente Python

```bash
git clone https://github.com/alberto/Vilas-Boas-Project.git
cd Vilas-Boas-Project

python -m venv venv
# Linux / macOS:
source venv/bin/activate
# Windows (PowerShell):
.\venv\Scripts\Activate.ps1

pip install -U pip
pip install -e ".[dev]"
```

> Também é possível usar `make setup` em ambientes Unix/WSL.

### 2. Executar a API Backend (Flask)

```bash
PYTHONPATH=server python server/app.py
```

A API responderá por padrão em `http://localhost:5000` (healthcheck em `GET /ping`).

### 3. Executar a versão de Terminal (CLI)

Para jogar diretamente no terminal sem navegador:

```bash
PYTHONPATH=server python server/cli.py
```

### 4. Executar o Frontend

Sirva a pasta `client/` com qualquer servidor HTTP estático (por exemplo, `python -m http.server 8080 --directory client`) ou compile alterações do TypeScript com:

```bash
tsc -p client/tsconfig.json
```

---

## Variáveis de Ambiente

Configure as variáveis abaixo em um arquivo `.env` (desenvolvimento) ou no painel do provedor de hospedagem (produção):

| Variável | Obrigatória em Produção | Descrição |
| :--- | :---: | :--- |
| `APP_ENV` | **Sim** | Define explicitamente o ambiente de execução (`production` ou `development`). |
| `SECRET_KEY` | **Sim** | Chave secreta usada exclusivamente para assinar cookies de sessão Flask e HMAC. |
| `SAVE_ENCRYPTION_KEY` | **Sim** | Chave secreta dedicada para derivar a chave Fernet de criptografia dos saves (desacoplada de `SECRET_KEY`). |
| `ADMIN_TOKEN` | **Sim** | Token secreto exigido no header `X-Admin-Token` para acessar as rotas `/admin/*`. |
| `FRONTEND_URL` | **Sim** | URL pública do frontend (ex.: `https://vilas-boas-project.vercel.app`) para CORS, validação de `Origin` e redirecionamento pós-OAuth. |
| `ALLOWED_ORIGINS` | Não | Lista de origens adicionais permitidas no CORS e na proteção CSRF/Origin, separadas por vírgula. |
| `SESSION_COOKIE_SAMESITE` | Não | Política `SameSite` do cookie de sessão (`Lax` por padrão; recomendado usar subdomínios próprios). |
| `SESSION_COOKIE_DOMAIN` | Não | Domínio compartilhado do cookie de sessão (ex.: `.seudominio.com` para `jogo.seudominio.com` e `api.seudominio.com`). |
| `DATABASE_URL` | **Sim** | URI de conexão com o banco de dados (`postgresql://...` em produção; usa SQLite local como fallback em desenvolvimento). |
| `DB_POOL_SIZE` / `DB_MAX_OVERFLOW` / `DB_POOL_RECYCLE` / `DB_POOL_TIMEOUT` | Não | Parâmetros de pool de conexões do SQLAlchemy para PostgreSQL. |
| `REDIS_URL` | Não | URI do Redis (`redis://...`) compartilhada entre os workers do Gunicorn para sessões (`RedisSessionStore`) e rate limiting (`Flask-Limiter`). |
| `GOOGLE_CLIENT_ID` | Não | Client ID do Google OAuth 2.0 para registro de recordes no Leaderboard. |
| `GOOGLE_CLIENT_SECRET` | Não | Client Secret do Google OAuth 2.0. |
| `DEV_MODE_ENABLED` | Não | Define se o modo desenvolvedor pode ser ativado (`1` / `true`). Ignorado e bloqueado automaticamente quando `IS_PRODUCTION` é verdadeiro. |
| `DEV_MODE_CODE` | Não | Código secreto personalizado necessário para ativar o modo desenvolvedor em ambiente local/teste. Não possui valor padrão. |

---

## Testes, Cobertura e Qualidade de Código

Para rodar a suíte completa de testes automatizados com relatório de cobertura:

```bash
PYTHONPATH=server pytest tests/ --cov=server --cov-report=term-missing -v
```

Para verificar formatação, lint (`ruff`) e checagem estática de tipos (`mypy`):

```bash
ruff check .
ruff format --check .
MYPYPATH=server mypy --explicit-package-bases server/
```

Ou utilizando o `Makefile`:

```bash
make test
make lint
make format
```

---

## Mecânicas e Sistemas da Engine

- **Parser Tolerante a Erros**: Normaliza acentuação, filtra *stopwords* e corrige erros de digitação usando distância de Levenshtein.
- **IA de Perseguição (BFS)**: O perseguidor navega pelo grafo do mapa usando Busca em Largura (*Breadth-First Search*), alternando entre patrulha e caça conforme o nível de ruído gerado pelo jogador.
- **Saves Criptografados e Compartilhamento Seguro**: Progresso exportado ou salvo no banco é criptografado com Fernet. Links de compartilhamento (`/share/<token>`) exigem confirmação via `POST` para evitar consumo acidental por bots de pré-visualização (WhatsApp, Discord).
- **Modo Desenvolvedor Isolado**: Disponível exclusivamente fora de produção mediante `DEV_MODE_ENABLED=1` e `DEV_MODE_CODE` definido no ambiente. Partidas com modo dev ativo são automaticamente excluídas do leaderboard e da telemetria.

---

## Arquivo Confidencial (Lore & Spoilers)

<details>
<summary><strong>⚠️ AVISO DE SPOILER — Clique para expandir os detalhes da história e enredo</strong></summary>

### A Tragédia do Restaurante Vilas Boas

Na década de 1982, **Renato Fidelis Gomes**, um engenheiro brilhante na área de robótica e sistemas embarcados, construiu animatrônicos avançados para o restaurante famoso em Curiiba. Porém, por conta de não haver ajudantes e trabalhadores na area de administração do local, Renato teve que abandonar a robotica e focar em papeis, dinheiro e tempo. O restaurante estava indo muito bem, famoso e tendo proposta de criar filiais, desabou em 1994, onde ocorreu o incidente mais famoso da cidade de Curtiba, o **IPD**, incidente das pessoas desaparecidas, onde 2 funcionarios, incluindo o gerente Renato Fidelis G. E uma cliente, desapareceram como pó ao fim do expediente. Em 1995, sem fluxo de clientes, e com diversas denuncias pela vigilancia sanitaria, o restaurante fechou definitivamente, com decisão de **Renato Fidelis Jr**, filho do dono, com previsão de abrir novamente nos anos 2000.

Em 2007, o restaurante está abandonado, mas com o *Renato* e *Michel* cuidando do restaurante, contratando serviços de limpeza e segurança. **Caroline A.**, uma estudante universitária, precisando de dinheiro rapido, consegue a vaga de segurança no restaurante, e logo começa a trabalhar. 
Caroline, após 4 dias sendo segurança, desaparece no restaurante, e não é mais achada. Nesses 4 dias, ela enviava diarimente emails para **Rogerio V.** Seu namorado que estava na cidade vizinha a procura de trabalho. Ao saber do desaparecimento, Rogerio não faz nada, se reclusa em casa, com medo e depressão profundo, covarde.

8 meses após Caroline desaparecer, Rogério não aguenta mais sentir culpa, afundando em suas lagrimas toda noite, e sonhos turbulentos, e vai a procura da sua redenção de seus erros e relapso, ele vai atrás de Caroline no restaurante.


### Os Finais Possíveis: O progresso da história depende de quão fundo o jogador decide ir na investigação:

- **Final Mediocre (`saida`)**: Rogerio, com medo e apavorado com o ambiente do restaurante, foge covardemente, e vai embora novamente para casa. 
- **Final Neutro (`final_neutro`)**: Rogério sobrevive até de manhã, vencendo a noite na sala de segurança, enfrentando todos os animatronicos, porém, sem pegar o item "fios cortados", ao chegar ao hall de entrada, o animatronico de caroline o cerca, tendo um dialogo profundo, reconhecendo o protagonista, porém, sua raiva sobresai, matando rogerio na sua frente, e o condenando a viver o terror em maquina. O ciclo de almas presas no maquinário permanece intacto.
- **Final Verdadeiro (`Libertação`)**: Rogerio consegue a chave dos fundos, vence o minigame do minotauro pegando os "fios cortados", vai na sala de fliperamas, faz cada um dos minigames, conseguindo passar na prova final, o julgamento do pianista, recebendo o cartão IV. Indo para sala de segurança, vencendo a noite, e indo ao hall de entrada, ateia fogo no local, e desliga o animatronico onde Caroline está, fazendo ela finalmente se recordar dele. Sem raiva e sem lamento, os dois se despedem, o animatronico pega fogo, e todos os espiritos, incluindo de caroline, descansam em paz.
- **Morte (`Assasinado`)**: Rogério é encontrado morto no restaurante as 8 da manhã do dia seguinte. Especialistas forenses relatam semelhanças com um ataque brutal feito por um predador, como um urso.
- **Final Bons Sonhos (`Durma...`)**: Rogério entra na sala de descanso, e decide dormir, preferindo entregar-se à insanidade dos pesadelos, ouvindo a porta da sala abrindo devargarzinho, sentindo braços etereos envolvendo seu pescoço.

</details>

---

## Licença

Distribuído sob a licença Apache 2.0. Veja o arquivo [LICENSE](LICENSE) para mais informações.


---

## Versão Desktop Offline (Rust — Windows `.exe` & Linux)

Além da versão web cliente-servidor (Python/Flask + TypeScript), o projeto conta com uma **reescrita nativa 1:1 em Rust** localizada na pasta `offline/`, compilada como um executável autônomo (*standalone*) que roda **100% sem necessidade de internet, navegador ou servidor Python**.

### Principais Funcionalidades da Edição Offline em Rust

- **Paridade 1:1 com a Engine Web**:
  - Todo o mapa, descrições, eventos aleatórios e artes ASCII de `server/data.py` são extraídos automaticamente em tempo de compilação pelo `offline/build.rs` e embutidos diretamente no binário.
  - Sistema completo de **IA de Perseguição por grafo (BFS)** com estados de Patrulha, Alerta e Caça, mecânica de barulho, tropeços no escuro e fuga (`correr <direção>`).
  - Parser de linguagem natural tolerante a erros de digitação (*fuzzy matching* por similaridade de subsequência), remoção de acentuação e *stopwords*, além de atalhos rápidos (`f`, `a`, `e`, `d`, `p`, `l`, `u`, `c`, `x`, `i`).
  - **Todos os 6 Minigames completos**:
    1. **Cofre Eletrônico (`MINIGAME_COFRE`)**: Pistas embaralhadas, senha de 4 dígitos e bloqueio com alarme após 3 erros.
    2. **A Fome de Jon (`MINIGAME_JON`)**: Fliperama de navegação sensorial nos dutos com pistas falsas (`[SENSÓRIO CONFUSO]`).
    3. **Consertos & Sorrisos (`MINIGAME_CONSERTOS`)**: Montagem de animatrônico em 3 etapas com recompensa bônus na combinação perfeita.
    4. **O Julgamento do Pianista (`MINIGAME_JULGAMENTO`)**: Interrogatório sobre a lore de 1982/1995 e as vítimas presas nas máquinas.
    5. **Labirinto de Alta Tensão do Minotauro (`MINIGAME_MINOTAURO`)**: Grid 5x5 com radar ASCII dinâmico, armadilhas elétricas, dicas direcionais por áudio/texto e corte dos cabos de alta tensão.
    6. **Turno da Noite na Sala de Segurança (`MINIGAME_SEGURANCA`)**: Sobrevivência das 12:00 AM às 06:00 AM (24 turnos) contra Rick, Jon, Caroline, Índio Jones e Alberto, incluindo câmeras, tubulação, gerador auxiliar, calor e reinicialização de sistemas via **CAPTCHAs no VILAS-BOAS OS (BSOD)**.
  - **Todos os 4 Finais + Easter Eggs**: Final Medíocre (`saida`), Final Bons Sonhos (`cama`), Final Neutro (`final_bom`) e Final Verdadeiro (`verdadeiro`), além do Kernel Panic (`del caroline.sys` / `format a:`), leitura dos 3 disquetes com flashes subliminares, código `2007` (Modo Deus) e código `1980` (Slideshow secreto).
- **Assets 100% Embutidos no Executável (`include_bytes!`)**:
  - Os 6 arquivos de áudio `.mp3` e todas as 33 imagens (incluindo a moldura da TV retrô, jumpscares, telas de pânico e as 21 imagens secretas) são compilados dentro do próprio binário, permitindo executar o `.exe` isoladamente em qualquer pasta.
- **Interface Gráfica Retrô (`eframe` / `egui`) e Sintetizador de Áudio (`rodio`)**:
  - **Intro da TV de Tubo**: Sequência interativa com botão Power, LED, chiado de estática analógica, barras de cor com tom de teste de 1000Hz e tela `VIDEO 2`.
  - **Efeito Typewriter & Bipes Mecânicos**: Impressão letra por letra com velocidade ajustável (`Instantâneo`, `0.5x`, `1x`, `2x`) e síntese de ondas sonoras em tempo real para teclas, alertas e erros.
  - **HUD `[ SISTEMA v20.08 ]`**: Indicadores de HP em blocos (`[███]`), LUZ, SOM, slots visuais de inventário (`[■][□]`), botões de uso rápido e D-Pad direcional clicável.
  - **Efeito CRT Scanlines, Toasts de Conquistas e Legendas de Acessibilidade**: Filtro visual de monitor antigo e legendas em tempo real (`[ Áudio: ... ]`) para todos os efeitos sonoros.
  - **Gerenciador de Saves e Quadro de Líderes Offline**: Autosave automático em `savegame.json`, importação/exportação de estado em JSON, ranking local persistente em `config.json` e geração de Card de Resumo copiável para a área de transferência.
  - **Modo Desenvolvedor Completo (`sudo admin 1982`)**: Comandos `tp`, `tp ia`, `gerar <item|tudo>`, `radar`, `dev radar on/off`, `dev ia on/off`, `dev mapa`, `dev status`, `dev dificuldade`, `dev ascii` e `dev imagem`.

### Como Compilar e Executar a Versão Offline

```bash
cd offline

# Executar diretamente em modo de desenvolvimento
cargo run --release

# Gerar o executável otimizado para o sistema atual
cargo build --release
```

- **Windows**: O executável independente será gerado em `offline/target/release/vilas_boas_offline.exe`.
- **Linux**: O binário nativo será gerado em `offline/target/release/vilas_boas_offline`.
