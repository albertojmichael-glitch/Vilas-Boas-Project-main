# 📖 Villas-Boas Project: API Reference

Esta documentação descreve os endpoints expostos pelo servidor Flask para comunicação com a interface Web MS-DOS.

## 🌐 Configurações Globais
* **Autenticação:** Baseada em Sessões HTTP (Cookies). O ID da sessão (`sid`) gerencia o estado da `GameState` no servidor.
* **Rate Limiting:** Implementado via `Flask-Limiter` baseado em IP.

---

## 1. `GET /iniciar`
Inicia uma nova sessão de jogo, destrói qualquer estado anterior atrelado ao usuário e retorna a tela de boot do sistema.

* **Rate Limit:** Sem limite estrito (herda fallback global).
* **Parâmetros:** Nenhum.

### Resposta de Sucesso (`200 OK`)
```json
{
  "estado": {
    "hp": "...",
    "inventario": [],
    "luz": "...",
    "saidas": [],
    "sala": "BOOT"
  },
  "linhas": [
    "@@TYPE@@branco@@15@@[ SISTEMA DE BOOT INICIADO ]",
    "@@TYPE@@verde@@15@@Aguardando comando 'dir'..."
  ]
}

2. POST /comando
Recebe a string de comando do jogador, processa na Engine e retorna o novo estado do jogo juntamente com as linhas narrativas renderizadas.

Rate Limit: 60 requisições por minuto (Proteção Anti-DoS).

Headers Necessários: Content-Type: application/json

Body (Request)

JSON
{
  "comando": "usar isqueiro no gerador"
}
Resposta de Sucesso (200 OK)
JSON
{
  "estado": {
    "hp": 3,
    "inventario": ["lanterna", "isqueiro"],
    "luz": 9,
    "saidas": ["Corredor", "Sala 1"],
    "sala": "SALA DE ENERGIA"
  },
  "linhas": [
    "<span class=\"vermelho\"> VOCÊ APLICA O FOGO NOS FIOS DE ÓLEO DO GERADOR </span>",
    "<span class=\"amarelo\">[ALARME]: VOCÊ TEM 7 TURNOS PARA CHEGAR À SAÍDA ANTES QUE TUDO DESABAPE!</span>"
  ]
}

🛑 Tratamento de Erros
A API utiliza códigos HTTP padrão para indicar falhas no processamento.

429 Too Many Requests
Disparado quando o jogador/bot excede 60 comandos em menos de 1 minuto.

Retorno Padrão HTML/JSON: Mensagem padrão do Flask-Limiter. A UI do jogo deve exibir "Conexão Instável" e pausar o input.

400 Bad Request (Buffer Overflow)
Disparado quando a string do campo comando excede 256 bytes. Tratado internamente para não quebrar a UI.

Código Retornado: 200 OK (Com payload de erro para desenhar no terminal do jogador).

JSON
{
  "estado": {},
  "linhas": [
    "@@TYPE@@vermelho@@15@@[ ERRO DE SISTEMA ] Buffer overflow detectado. Comando excede 256 bytes."
  ]
}
500 Internal Server Error
Disparado em caso de falha crítica na Engine Python (ex: Atributo não mapeado no Pydantic).

Código Retornado: 200 OK (Impede o jogo de travar silenciosamente).

JSON
{
  "estado": {},
  "linhas": [
    "@@TYPE@@vermelho@@0@@[ERRO INTERNO]: O servidor falhou ao processar a ação."
  ]
}

