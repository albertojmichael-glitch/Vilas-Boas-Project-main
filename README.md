# Vilas Boas OS (v1.982)

![Status](https://img.shields.io/badge/Status-Finalizado-brightgreen?style=for-the-badge)
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
