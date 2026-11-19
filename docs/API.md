# API do Vilas Boas OS

Endpoints do servidor Flask (`server/`). Em produção o front chama a API em `/api/...`; a Vercel reescreve `/api` para o projeto da API (veja `vercel.json`).

## Regras gerais

- **Sessão:** cookie de sessão HTTP. O `sid` identifica o estado do jogo no servidor; o navegador nunca guarda o estado do jogo.
- **Origem:** rotas que alteram estado exigem um header `Origin` permitido (`FRONTEND_URL` / `ALLOWED_ORIGINS`). Caso contrário respondem `403`.
- **Limites:** `Flask-Limiter` por IP. Ao estourar o limite a resposta é `429`.
- **Erros:** respostas de erro trazem `{"erro": "mensagem"}`.

## Jogo

### `POST /iniciar` — 10/min
Cria uma sessão nova e devolve a tela de boot. Não tem corpo.

### `POST /comando` — 120/min e 500/h
Corpo:
```json
{"comando": "ir frente", "telemetria": true}
```
`comando` tem até 256 caracteres, e os caracteres `< > ; ``` ` são recusados. Corpo que não é um objeto JSON → `400`.

Resposta `200`:
```json
{
  "linhas": ["@@TYPE@@verde@@15@@<span class=\"verde\">...</span>"],
  "estado": {
    "hp": 3, "inventario": ["lanterna"], "luz_restante": 3,
    "bolsas_coletadas": 0, "estado_jogo": "JOGO", "tempo_final": 0.0,
    "som": 15, "sala": "HALL DE ENTRADA", "final_alcancado": "hall de entrada"
  },
  "novas_conquistas": []
}
```
As `linhas` são instruções para o terminal: `@@TYPE@@cor@@ms@@texto` (texto já escapado para HTML), `@@CLEAR@@`, `@@PAUSE@@ms`, `@@JUMPSCARE@@` etc.

### `GET /achievements`
Conquistas da sessão atual: `{"conquistas": [...], "total": n}`.

## Saves

O progresso fica **cifrado no banco** (Fernet). O arquivo exportado é **legível e assinado** (HMAC-SHA256): o jogador pode ler, mas qualquer alteração invalida a assinatura. Todo estado recebido passa por `server/services/save_validacao.py`.

### `GET /save/export` — 5/min
```json
{
  "formato": "vilas-boas-save", "versao": 2, "criado_em": "2026-11-19T15:00:00+00:00",
  "resumo": {"sala": "balcão", "hp": 2, "dificuldade": "PESADELO", "inventario_tamanho": 3},
  "estado": { "...estado completo do jogo...": "..." },
  "assinatura": "<hex de 64 caracteres>"
}
```
Respostas: `404` sem jogo ativo; `409` se o jogo atual não puder ser exportado (por exemplo, no meio de um minigame).

### `POST /save/import` — 10/min
Corpo: o arquivo exportado (v2) ou o formato antigo `{"dados_seguros": "<token Fernet>"}`.

| Código | Quando |
| :---: | :--- |
| `200` | Save importado. |
| `400` | JSON malformado, formato desconhecido ou estado impossível; a mensagem diz a regra violada. |
| `403` | Assinatura inválida (arquivo alterado) ou origem não permitida. |
| `413` | Arquivo com mais de 200 KB. |

Um import recusado nunca altera o save atual.

### `POST /share/generate` — 5/h
Gera um link temporário (1 hora) para clonar o save atual: `{"link": ".../share/<token>", "mensagem": "..."}`.

### `GET|POST /share/<token>`
`GET` mostra uma página de confirmação; ela existe para que bots de pré-visualização não consumam o link. `POST` valida o save original, clona-o para uma sessão nova e redireciona para `/`.

## Conta e ranking

- `GET /login/google` (5/min) e `GET /auth/google/callback`: login Google OAuth para o ranking. O e-mail só é guardado como hash SHA-256 (LGPD).

## Administração

Todas exigem o header `X-Admin-Token` com o valor de `ADMIN_TOKEN`; um token na query string é recusado.

- `GET /admin/analytics`: métricas gerais e autópsia das mortes.
- `GET /admin/auditoria-mortes`: mortes recentes.
- `GET /admin/saves?page=n` (10/min): saves paginados.
- `GET /admin/replay/<id>`: replay de uma partida.

## Saúde

- `GET /`: `{"status": "online"}`.
- `GET /ping`: `Estou vivo!`.
