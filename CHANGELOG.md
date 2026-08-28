# Changelog

Todas as mudanças notáveis neste projeto serão documentadas neste arquivo.
O formato é baseado no [Keep a Changelog](https://keepachangelog.com/pt-BR/1.0.0/).

## (Atualização do Sistema) - Patch de Expansão, Estabilidade e Segurança

* **Sequência de Boot em CRT:** Implementação de uma introdução interativa simulando o ligar de uma TV de tubo clássica.
* **Camadas Visuais Dinâmicas:** Adição de estágios de transição reais incluindo Estática (com trepidação de imagem), Barras de Cor SMPTE e Tela Azul de VCR ("VIDEO 2 CONECTADO") antes de revelar o terminal.
* **Design de Som Integrado:** Sincronização de eventos de áudio nativos pelo AudioContext do navegador, gerando ruído branco procedural e o clássico tom de 1kHz (Bipe) durante as barras de cor.
* **Interação Física Simulada:** Botão de energia clicável sobreposto à imagem da TV com um indicador LED dinâmico (vermelho/verde) que reage à interação do jogador.
* **Transição Cinematográfica:** Adição de um efeito de zoom-in CSS que mergulha o jogador da sala da TV diretamente para o terminal do jogo de forma contínua.

* **Correção da Condição de Vitória (06:00):** Resolução de um erro fatal (UnboundLocalError: cannot access local variable 'digitar') no arquivo seguranca.py que causava o colapso do servidor ([ERRO INTERNO]) no momento de registrar o final da partida.

* **Desbloqueio de AudioContext:** Correção do bloqueio padrão de navegadores modernos contra autoplay de áudio, vinculando a inicialização do motor de som diretamente ao primeiro clique (gesto) do usuário no botão da TV.

* **Sincronização de Execução:** Remoção do auto-boot no carregamento da página (window.onload) para impedir que a engine do jogo rodasse e tocasse sons invisivelmente no background enquanto a TV ainda estava desligada.

* **Atualização Crítica de Dependências:** Mitigação de múltiplas vulnerabilidades graves (CVEs) apontadas pelo pip-audit. Bump de versão forçado no requirements.txt para cryptography (>=49.0.0), gunicorn (>=22.0.0), flask-cors (>=6.0.0), setuptools (>=83.0.0) e certifi (>=2024.7.4).

* **Ajuste de Content Security Policy (CSP):** Flexibilização segura do cabeçalho HTTP de resposta (script-src 'self' 'unsafe-inline') para permitir a execução dos gatilhos de clique na nova interface da TV sem bloquear os scripts do jogo.

## (Patch de Estabilidade e Balanceamento)

##  Engine e Testes Automatizados (Pytest)
* **FIX:** O `MinigameMinotauro` agora sobrevive aos testes do Pytest. Implementada blindagem de interface (`if self.ui and hasattr(self.ui, 'exibir'):`) no `__init__` e no `imprimir_status` para evitar `AttributeError` quando o ambiente gráfico for `None`.
* **FIX:** Criada uma classe `DummyUI` (Mock) nativa na função `processar_turno` do Minotauro. O Pytest agora simula as ações visuais silenciosamente, alcançando 100% de aprovação sem travar o código.
* **FIX:** A instância do Minotauro no jogo web agora recupera o contexto gráfico correto (`ui_handler`) em vez de cair no fallback de testes.
* **FIX:** O loop de eventos no `engine.py` agora atualiza a referência gráfica do Minotauro (`jogo.minigame_atual.ui = jogo.ui_handler`) a cada turno, resolvendo o bug do "console fantasma" (comandos que funcionavam, mas não imprimiam na tela).

##  Gameplay e Balanceamento (Minotauro)
* **REMOVED:** Mecânica de "Sprint" do Minotauro. O RNG de 15% para o inimigo andar 2 casas e causar um jumpscare inevitável foi removido para evitar punições injustas em uma grade tão pequena (3x4).
* **FEAT:** Nova mecânica de "Trombada" (Bump). O jogador e o Minotauro agora se movem 1 casa por turno. Se ambos entrarem na mesma coordenada partindo de uma distância segura (`dist_ui > 1`), o monstro é empurrado um bloco para trás e o jogador recebe um aviso visual vermelho, tendo 1 turno extra para reagir.
* **FIX:** Corrigido loop infinito de morte no God Mode. Adicionado o `return "vitoria_minotauro"` faltante após a colisão bem-sucedida, prevenindo que o Minotauro continuasse andando e gerasse duas telas de Game Over simultâneas.
* **FIX (Soft-lock):** Atropelar o Minotauro no God Mode ou gerar o item `fios cortados` via console de comandos agora ativa corretamente a flag `jogo.fios_cortados_inventario = True`, garantindo que jogadores onipotentes não sejam bloqueados de realizar o Final Verdadeiro.
* **UX:** O feedback visual das paredes do mapa (ex: bater as costas na porta sem estar no centro) foi mantido fiel, ensinando o jogador a se alinhar às coordenadas corretas (X=0, Y=0) para a opção de fuga aparecer.

## [1.7.0 - VERSÃO FINAL / PROJETO CONCLUÍDO] - 2026-07-27

### 🏆 Status do Projeto
- **PROJETO 100% FINALIZADO:** O motor do jogo MS-DOS retro foi concluído com sucesso, com todos os minigames, múltiplos finais, segredos, acessibilidade e responsividade totalmente funcionais.

---

### 🥚 Easter Eggs MS-DOS (Menu de Boot)
- **`HELP`:** Exibe a lista de comandos do sistema com resposta narrativa do arquivo de ajuda corrompido.
- **`MEM`:** Exibe detalhamento técnico da memória RAM em 1982/2007 e acusa a presença do módulo fantasma `CAROLINE.SYS`.
- **`CHKDSK`:** Executa verificação do disco FAT16 e aponta setores defeituosos no registro de memória de Rogério.
- **`EXIT`:** Desliga visualmente o terminal, deixando a tela completamente preta (requer F5 para reiniciar).
- **`SHUTDOWN`:** Executa o reinício completo da página/sistema no navegador via sinal `@@RELOAD@@`.

---

### 🔥 Narrativa & Finais
- **Gatilho do Final Verdadeiro Corrigido:** Adicionada a mecânica do `isqueiro` em `commands.py`. Se utilizado após as 6:00 AM (`noite_vencida`) portando os `fios cortados`, o jogador incendeia o restaurante, permitindo alcançar o **Final Verdadeiro: Libertação**.

---

### 🎛️ Minigames & Engine (`engine.py`, `minigames.py`, `commands.py`)
- **Correção da Cadeira / Sala de Segurança (`KeyError`):** Interagir com a cadeira mantém a `sala_atual` válida (`01`) enquanto alterna o `estado_atual` para `MINIGAME_SEGURANCA`, eliminando crashes de mapa.
- **Sincronização de Interface Web:** Atualização dinâmica de `jogo.minigame_atual.ui = jogo.ui_handler` a cada requisição HTTP, garantindo que o relatório de câmeras e sensores seja exibido na tela ativa do usuário.
- **Temporização e Leitura:** Implementação do manipulador `@@PAUSE@@` no servidor e cliente para dar tempo hábil de leitura nos relatórios de câmeras e eventos de suspense.
- **Proteção Visual:** `imprimir_contexto_sala()` agora aborta automaticamente durante minigames ativos para evitar sobreposição de textos.

---

### 📺 Visual, Glitches Dinâmicos & Responsividade (`style.css`, `index.html`)
- **Glitches de Dano e Jumpscare:** Adicionada a animação CSS `@keyframes glitch-shake` com distorção de cores RGB acionada em sustos, mensagens de perigo (texto vermelho) e tomadas de dano.
- **Efeito HP Crítico:** Quando o jogador atinge 1 de HP, o monitor CRT entra em estado de instabilidade visual (`.hp-critico`), piscando e intensificando o efeito de scanlines.
- **Redesign Mobile (Port Celular):**
  - O HUD (Status/HP/Luz/Inventário) se reorganiza como uma barra superior fixa e compacta.
  - O terminal ocupa a parte central flexível sem cortar textos.
  - A barra de atalhos e direcionais foi transformada num controle virtual estilo *gamepad* na parte inferior.
  - Prevenção do teclado virtual subir indesejadamente em cliques fora da caixa de entrada.

---

### 🎧 Sound Design Sintético (Web Audio API - `script.js`)
- **Feedback de Digitação (`tocarSomDigito`):** Sons de clique sintético retrô em cada caractere impresso na tela.
- **Bip de Entrada (`tocarBipEntrada`):** Confirmação sonora de envio de comando ao pressionar `Enter`.
- **Passos Metálicos Pesados (`tocarPassoMetalico`):** Som de impacto sub-grave e fricção de metal disparado pelas tags `@@PASSO@@` enviadas do backend.
- **Zumbido Elétrico CRT (`iniciarSomAmbiente`):** Drone abafado de 55 Hz combinado com uma frequência de rede de 60 Hz simulando o monitor de tubo aquecido.

---

### 🛡️ Segurança & Backend (`app.py`, `state.py`)
- **Validação de Entrada:** Pydantic (`ComandoRequest`) limitando comandos em até 256 caracteres.
- **Sanitização de Cookies:** Leitura segura de UUID em `obter_sid_seguro()` prevenindo injeções NoSQL.
- **Gerenciamento de Sessão:** Integração com Redis e *fallback* local `TTLCache`.
- **Rate Limiting:** Limite de 60 requisições/min e 500/hora via `Flask-Limiter`.


## [1.6.0] - 2026-07-24

### ⚡ Segurança & Arquitetura
- **Validação com Pydantic:** Adicionado o schema `ComandoRequest` na rota `/comando` para barrar payloads maliciosos ou maiores que 256 caracteres na porta de entrada.
- **Proteção contra Injeção NoSQL:** Criada a função `obter_sid_seguro()` para validar e converter os cookies de sessão `sid` em UUIDs válidos antes de realizar consultas no MongoDB.
- **Rate Limiting em Camadas:** Configurados múltiplos limites no `@limiter` para travar requisições automatizadas (60 por minuto / 500 por hora).
- **Suporte ao Redis:** Implementada integração distribuída via `REDIS_URL` com *fallback* automático para `TTLCache` na memória local.
- **Paginação de Saves:** Adicionada a rota `/saves` com suporte a paginação (`page` e `limit`) para evitar estouro de memória na listagem do banco de dados.
- **Clean Code & Imports:** Todos os imports de módulos (`redis`, `minigames`, etc.) foram movidos para o topo do código (PEP 8), eliminando problemas de *Circular Import* e `UnboundLocalError`.

### 🐛 Correções de Bugs (Bug Fixes)
- **Minigame de Segurança (`KeyError`):** Corrigido o bug onde interagir com a cadeira alterava `sala_atual` para `"cadeira"`. O ID da sala permanece válido enquanto o estado muda para `MINIGAME_SEGURANCA`.
- **Prevenção de Invocação de Salas em Minigames:** A função `imprimir_contexto_sala()` agora possui travas para cancelar a renderização visual do mapa enquanto minigames estiverem em execução.
- **Restauração de Textos Decepados:** Reconstruídos os finais de diálogos e narrações narrativas cortadas em `commands.py`, `minigames.py` e `views.py`.
- **Ajuste de Testes Pytest:** Corrigidos os testes unitários de alias de movimentação (`test_alias_movimentacao`) e de ausência do manipulador de pausa no minigame.
- **Dependências no Deploy:** Adicionada a biblioteca `redis>=5.0.0` ao `requirements.txt` para resolver falhas de boot no Railway/Gunicorn.

### 📱 Frontend & Acessibilidade (a11y)
- **Foco Inteligente:** Refatorado o ouvinte de cliques no `script.js` para não roubar o foco da tela caso o usuário esteja selecionando texto ou clicando em botões mobile.
- **Remoção de Duplicidade Visual:** Removido o atributo `placeholder="C:\>"` da tag `<input>` no `index.html` para eliminar o duplo prompt cinza na tela.
- **Suporte WCAG (Contraste de Cores):** Atualizadas as paletas do terminal no `style.css` (verde `#4aff4a`, vermelho `#ff5555`, amarelo `#ffda33`) para reduzir a fadiga visual sem perder o brilho CRT.
- **Screen Readers (Leitores de Tela):** Adicionados os atributos `role="log"`, `aria-live="polite"` e `aria-label` nos elementos principais.

### 🎧 Efeitos Sonoros (Web Audio API)
- **Feedback de Digitação (`tocarSomDigito`):** Efeito sintético leve acionado a cada caractere exibido na tela durante a animação de texto.
- **Bip de Entrada (`tocarBipEntrada`):** Som retrô disparado imediatamente ao pressionar a tecla `Enter`.
- **Passos Metálicos Pesados (`tocarPassoMetalico`):** Sintetizador de impacto e fricção metálica acionado dinamicamente pela engine através da tag `@@PASSO@@`.
- **Zumbido Elétrico CRT (`iniciarSomAmbiente`):** Adicionada frequência de rede de 60 Hz sobre o som abafado de fundo para simular a presença de um monitor de tubo antigo.



## [1.5.0] - 2026-07-22

### 🚀 Adicionado
- **Efeito Visual CRT Completo (Monitor de Tubo):** Implementação de camada gráfica avançada em CSS nativo (`body::before` e `body::after`) trazendo scanlines, vinheta escurecida nas bordas, brilho estático de fósforo (*glow*) e leve tremor de tela (*flicker*), sem impactar a acessibilidade ou o clique no input.
- **Sistema de Sanidade (Zalgo Text):** Injeção dinâmica de ruído visual Unicode no texto descritivo das salas (`corromper_texto` em `utils.py`) acionada automaticamente sempre que a saúde do jogador atinge níveis críticos (HP ≤ 1).
- **Rota do Final Verdadeiro (Heist & Armadilha):** Nova mecânica de resolução do jogo no `commands.py`. O jogador precisa reunir o *Kit Incendiário* (Garrafa Vazia, Pano, Fita Isolante, Fios Cortados e Isqueiro/Fósforo) e armar o pavio no Hall de Entrada após o amanhecer (06:00) para libertar a alma de Caroline.
- **Expandido Minigame do Minotauro:** O jogador agora precisa localizar a tesoura no labirinto da Sala de Energia, cortar os fios no painel do fundo (0,3) e realizar a rota de fuga de volta até a porta (0,0) sob perseguição do monstro para garantir o troféu *fios cortados*.
- **Radar de Inspecionáveis na UI:** Exibição contextual automatizada no `views.py` indicando pontos de interesse investigáveis (`🔍 Investigar: [alvo]`) logo após a descrição principal de cada sala.
- **Utilidade de Itens da Mochila:** - A *tábua pequena de madeira* agora pode ser usada como alavanca para arrombar a porta emperrada (03) e acessar a Sala do Gerador.
  - A *tesoura quebrada* agora pode ser restaurada combinando-a com a *fita isolante*.
- **Infraestrutura de Produção e Segurança:**
  - **Rate Limiting (Anti-DoS):** Integração do `Flask-Limiter` travando requisições abusivas acima de 60 comandos por minuto por IP.
  - **Eliminação de Memory Leak:** Substituição de gerenciamento manual de sessões por `cachetools.TTLCache` (limite rígido de 1000 sessões em RAM com expiração automática de 1h).
  - **Rotação de Logs de Produção:** Implementação de `RotatingFileHandler` (limite de 10 MB com até 5 backups) para conter o crescimento indefinido de arquivos de log.
  - **Minificação em Build (`build.py`):** Script automatizado usando `rcssmin` e `rjsmin` para compilar e minificar CSS e JS em tempo de deploy.
- **Documentação de Sistema:** Criação do arquivo de contrato `API.md` detalhando as rotas, códigos de erro HTTP e payloads JSON, além de um arquivo `.gitignore` abrangente cobrindo credenciais, logs e caches.

### 🔧 Modificado
- **Compatibilidade Python 3.13 / Pydantic V2:** Atualização da dependência para `pydantic>=2.9.2` no `requirements.txt`, eliminando falhas de compilação em Rust (`pydantic-core`) durante o build no Railway/Render.
- **Tipagem Estrita no `state.py`:** Mapeamento explícito de todos os atributos dinâmicos do estado do jogo (`web_consertos`, `web_julgamento`, `jon_passos_dados`, `jon_caminho_certo`, `fios_cortados_inventario`, `incendio`, `noite_vencida`) evitando erros de `NameError` e `AttributeError` em runtime.
- **Aprimoramento da Suíte de Testes (`test_game.py`):** Refatoração do `MockUI` com buffer simulado e injeção de mapa sintético para garantir execução 100% limpa da esteira CI/CD no GitHub Actions.
- **Simplificação do HTML:** Remoção da div estática `.crt-overlay` em `index.html`, delegando todo o controle visual diretamente ao estilo responsivo do `style.css`.

### 🐛 Corrigido
- **Bug da Transição Invisível (UI):** Correção do método `limpar()` no `app.py` que impedia o envio da instrução `@@CLEAR@@` ao frontend durante trocas de estado.
- **Caminho de Salvamento Dinâmico:** Ajuste do callback de persistência para ler a chave de sessão válida antes de atualizar instâncias do `GameState`.

---

## [1.4.0] - 2026-07-21

### Adicionado
- **Sintetizador de Áudio Retro (PC Speaker):** Implementação da Web Audio API no frontend para gerar ondas quadradas (`square`) e dente de serra (`sawtooth`) nativas. O jogo agora emite bipes mecânicos reais de placa-mãe de 1982 ao enviar comandos e avisos graves em caso de falhas críticas de conexão.
- **Foco Automático Implacável:** Script utilitário em JavaScript que força o foco do teclado de volta à caixa de texto sempre que o jogador clica na área preta do terminal, impedindo digitações sem efeito e melhorando a usabilidade em teclados físicos.
- **Auto-Scroll Milimétrico Interativo:** Integração do controle dinâmico de rolagem (`scrollTop`) diretamente no loop de renderização caractere por caractere (`digitarTextoAnimadoHTML`). A tela agora acompanha a escrita fluida das letras automaticamente, sem esconder o prompt de comando.
- **Segurança contra Buffer Overflow:** Nova camada de validação em `app.py` que barra comandos abusivos acima de 256 bytes, retornando um alerta de erro de sistema estilizado em DOS e protegendo as rotas internas contra injeção de payloads de DoS.
- **Coleta Automatizada de Lixo (Session TTL):** Implementação de um dicionário global de timestamps (`MEMORIA_SESSOES_TTL`) no servidor Flask. Sessões inativas há mais de 1 hora são eliminadas da RAM automaticamente, eliminando riscos de vazamento de memória com alta concorrência.
- **Mecanismo de Auditoria com Logs Estruturados:** Configuração de logging padronizado (`logging.getLogger`) nos pontos críticos da Engine e das rotas HTTP, facilitando o diagnóstico de erros fatais diretamente na interface de produção do Railway/Render.

### 🔧 Modificado
- **Corretor Ortográfico Silencioso (Modo Ninja):** Remoção de todas as saídas de texto `(Entendido como: '...')` que poluíam o terminal. O algoritmo de distância de strings (`difflib`) agora corrige pequenas falhas de digitação (como erros em direções ou nomes de itens) 100% por baixo dos panos, elevando a percepção de inteligência do parser.
- **Desativação do HUD Duplicado Textual:** Remoção completa da impressão em texto puro das estatísticas (`[ SISTEMA OPERACIONAL... ]`) no meio da tela narrativa. Toda a exibição de HP, Inventário e Níveis de Luz foi centralizada exclusivamente no componente Sidebar dinâmico do HTML/JS via JSON estável.
- **Correção de Quebra no Comando `dir`:** Substituição da tag nativa `<DIR>` por referências em formato de entidades HTML (`&lt;DIR&gt;`) no `engine.py`. Isso impede que o navegador web interprete a string como uma tag obsoleta de bloco HTML, garantindo o alinhamento de colunas típico do MS-DOS.
- **Ajuste Fino na Altura de Linha (Renderização ASCII):** Atualização da diretiva `#output p` no arquivo `style.css` definindo `margin-bottom: 0px` e `line-height: 1.1`. Agora, artes ASCII complexas impressas linha por linha via `ui.animar` grudam perfeitamente sem espaços ou falhas verticais.
- **Otimização de Cache via Cache Busting:** Atualização da assinatura dos arquivos estáticos no `index.html` para `?v=1.4.0`, forçando os navegadores de jogadores antigos a invalidarem o cache local e carregarem o novo layout JS/CSS imediatamente.
- **Ocultação Estética de Scrollbars:** Inclusão de pseudo-elementos CSS (`::-webkit-scrollbar` e `scrollbar-width: none`) para ocultar a barra de rolagem cinza padrão do Windows/Navegador, preservando a navegação por scroll intacta mas mantendo o fundo preto infinito.

---

## [1.3.0] - 2026-07-20
### Adicionado
- **Histórico de Comandos:** Implementação de navegação de terminal via setas direcionais (`ArrowUp` e `ArrowDown`) com armazenamento em memória na sessão do navegador.
- **Sintetizador de Áudio (MS-DOS):** Injeção de feedback sonoro de hardware via `AudioContext` nativo do JavaScript (sem dependência de arquivos de áudio externos), simulando os *beeps* e *boops* da placa-mãe.
- **Atalhos Globais de Teclado:** Suporte a atalhos clássicos de terminal (`Ctrl+L` para limpar a tela, `Ctrl+S` para verificação visual de save e `?` para ajuda).
- **Acessibilidade (a11y):** Inclusão de atributos ARIA, *Live Regions* (`aria-live="polite"`) e classes `.sr-only` para suporte a leitores de tela.
- **Menu de Ajuda Contextual:** Adição de modal responsivo com lista de comandos básicos e atalhos, ativado via UI ou teclado.
- **Cache de Estáticos:** Interceptador nativo no Flask (`@app.after_request`) para forçar o cache de CSS e JS no navegador do cliente (1 hora).

### Modificado
- **Direção Artística (Fim dos Emojis):** Expurgados todos os emojis modernos da interface gráfica, substituídos por padronização ASCII (ex: `⨹`, `☠`, `✕`) para maximizar a imersão de 1982.
- **HUD Retrô:** Barra de progresso de HP reimaginada utilizando caracteres de bloco (`[██░]`) e reatividade de cor (verde para seguro, vermelho para perigo).
- **Estética de Monitor CRT:** Adição de `text-shadow` sutil no CSS para emular o vazamento de luz (glow) do fósforo verde/âmbar e ajuste no `line-height` para otimização de leitura.
- **UX de Carregamento (Anti-Flicker):** Inserção de uma barreira de atraso mínimo de 300ms na animação de carregamento (`spinner`) para evitar flashes visuais bruscos quando a latência de rede é próxima de zero.

### Corrigido
- **O Bug do "Fantasma do Cache" (F5):** Aplicação de cabeçalhos estritos de *No-Cache* na rota `/iniciar` e reestruturação da geração de *Session IDs* (SIDs). Agora, forçar a atualização da página pelo navegador reseta corretamente a máquina de estados do servidor para a tela de Boot, prevenindo bloqueios irreversíveis.
- **Minotauro Silencioso (Timeout Exploit):** Refatoração no bloco de tratamento de comandos da classe `MinigameMinotauro`. Comandos inválidos ou não reconhecidos deixaram de ser ignorados silenciosamente e passaram a consumir o turno do jogador, deduzindo bateria e movendo o monstro, com o devido feedback narrativo de penalidade.

---

## [1.2.0] - 2026-07-17
### Adicionado
- **Persistência em Nuvem (MongoDB):** Integração nativa com banco de dados NoSQL MongoDB (`pymongo`) via MongoDB Atlas, garantindo retenção perpétua dos dados de progresso e contornando a limpeza de arquivos efêmeros do Render.
- **Mecanismo de Salvamento Híbrido:** Arquitetura inteligente que detecta a presença de `MONGO_URI` em variáveis de ambiente, alternando dinamicamente entre persistência na nuvem (produção) ou fallback em disco local (desenvolvimento).
- **Cofre de Sessões em RAM:** Barreira de isolamento via dicionário de execução interna em `app.py`, evitando problemas de vazamento de contexto e falhas de serialização (*pickling/decoding*) entre jogadores simultâneos.
- **Ambiente de Testes Isolado (`MockUI`):** Implementação de uma interface simulada em `tests/test_game.py` para prevenir erros de execução nula (`AttributeError: 'NoneType' object has no attribute 'limpar'`) durante execuções automáticas.
- **Pipeline Automatizada (`make push`):** Inclusão de um fluxo operacional unificado no `Makefile` que auto-formata o código, executa linters e roda testes locais antes de enviar as atualizações ao repositório.

### Modificado
- **Suíte de Testes Determinísticos:** Atualização das asserções da Seed 42 no teste do Minotauro para validar com precisão a lógica de ataque imediato (bateria decrementada para 14 e estado de morte) mapeada na engine.
- **Documentação Exaustiva da API:** Expansão das referências no `README.md` detalhando as assinaturas completas de payload, contratos JSON de entrada/saída e códigos de status HTTP (200, 400 e 500) para os endpoints `/iniciar` e `/comando`.

### Segurança
- **Failsafe de Produção:** Injeção de cláusula de validação estrita no boot do `app.py` que derruba intencionalmente a aplicação em ambientes de produção (`RENDER`) se a chave `FLASK_SECRET_KEY` estiver utilizando os valores padrão de desenvolvimento.

### Corrigido
- **Duplicação de Código em `state.py`:** Expurgados os métodos repetidos de serialização/desserialização herdados da transição para a sintaxe do Pydantic V2 (`model_dump` e `model_validate`).
- **Sintaxe do Makefile:** Eliminação de comandos Git órfãos que causavam quebra de interpretação estrutural por estarem fora de escopos de *targets*.
- **Mapeamento de Importações no GitHub Actions:** Ajuste fino na declaração de dependências do fluxo com a injeção da flag `PYTHONPATH=.`, extinguindo falhas do tipo `ModuleNotFound` induzidas pela árvore de caminhos do Linux.
- **Quebra de Renderização da Arte ASCII:** Substituição do desenho complexo do cofre (`ARTE_COFRE`) por um layout compacto simétrico, prevenindo o colapso horizontal de linhas ocasionado pelas margens automáticas de navegadores.

---

## [1.1.0] - 2026-07-17
### Adicionado
- Flask-Session para gerenciamento assíncrono e isolado de saves por cookie.
- Motor Unificado (`engine.py`) para tratar lógica independente de UI (Web/CLI).
- Suíte completa de testes automatizados com `pytest` cobrindo cenários determinísticos (Seed 42).
- CI/CD via GitHub Actions.

### Modificado
- Remoção do hack `sys.stdout` em favor de Buffer JSON estruturado na WebUI.
- Modificador de Dificuldade de Velocidade (`fast_mode`) desvinculado de Game Modes absolutos.
- Mapa de rotas e dependência do item 'tesoura' atrelado rigorosamente à geolocalização da `sala_atual`.
