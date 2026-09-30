# Projeto Vilas Boas (Sistema de Segurança Vilas Boas)

Um jogo de terror de sobrevivência em formato de ficção interativa (text-based adventure), construído sobre uma interface web que simula um terminal MS-DOS. O jogador deve usar comandos de texto natural para explorar um restaurante abandonado, resolver enigmas, gerenciar recursos e sobreviver.

* **Arquitetura e Infraestrutura*
Frontend: HTML, CSS e JavaScript puro (Vanilla). Hospedado na Vercel. Responsável pela renderização CRT, efeitos sonoros dinâmicos, controle de input e sistema de recuperação de saves locais.

Backend: Python com framework Flask. Hospedado no Railway. Processa a lógica de estado do jogo (Engine), parsing de comandos, IA do inimigo e banco de dados.

Banco de Dados: PostgreSQL via SQLAlchemy. Armazena telemetria, saves na nuvem, links de compartilhamento e o Quadro de Líderes (Leaderboard).

Autenticação: Google OAuth 2.0 integrado para registro seguro de jogadores no placar global.

* **Funcionalidades e Sistemas Principais*
Motor de Texto (Parser)
O jogo utiliza um sistema de processamento de linguagem natural básico que sanitiza o input do jogador, remove pontuações, ignora espaços extras e realiza aproximações (fuzzy matching). Suporta comandos completos (ex: "pegar a chave dos fundos") e atalhos rápidos (ex: "f" para "ir frente").

Inteligência Artificial Baseada em Som
O animatrônico inimigo não aparece de forma aleatória. Ele possui um estado interno de IA (Patrulha, Alerta, Caça) e navega pelo mapa usando o algoritmo de busca em largura (BFS).
Ações bruscas (como correr, permanecer na mesma sala por varios turnos e errar comandos sequencialmente) aumentam o "nível de barulho" do jogador, atraindo a criatura para a sala atual.

* **Gerenciamento de Sobrevivência*
Luz: O jogador inicia com bateria limitada na lanterna. Cada ação consome um turno. Permanecer no escuro absoluto por mais de 3 turnos, a chance de morte pelo homem das sombras resulta aumenta, 30% a cada turno, até o jogador morrer.

Inventário Restrito: O espaço inicial é limitado a 3 itens. O jogador deve encontrar mochilas no cenário para expandir a capacidade até o máximo de 9 espaços, exigindo decisões estratégicas sobre o que carregar.

Saúde (HP): O jogador possui 3 pontos de vida. Receber dano em minijogos ou confrontos reduz a vida. Chegar a 0 resulta em morte (Tela de Jumpscare).

* **Modos de Jogo*
Dificuldades Base: Normal e Pesadelo (inimigo mais rápido, menos bateria).

Desafios: Speedrun (limite de tempo real), Fantasma (morte instantânea ao fazer mais de 50% de barulho) e Breu Total (lanterna inicia descarregada).

Modo Personalizado: Permite alterar parâmetros vitais.

God Mode (Modo Deus): Sem mortes, sem sacrificios, explore o mapa sem medo, com lanterna, vida e espaços da bolsa infinitas, podendo gerar itens e se teletransportar (tp {sala}, gerar {item}).

* **Sistema de Saves (SafeStorage)*
Local: Sistema robusto com versionamento, shadow backups invisíveis e fallback automático para evitar perda de dados em caso de falha no navegador.

Nuvem e Exportação: Saves são exportados como arquivos JSON criptografados.

Compartilhamento: Geração de links únicos com validade de 1 hora para enviar o estado exato da partida para outro jogador.

* **Minijogos Integrados*

Além da exploração por texto, certas interações abrem instâncias de minijogos com regras próprias:

Cofre de Segurança: Um puzzle de dedução de 4 dígitos baseado em charadas textuais gravadas no cenário. Falhar sucessivamente bloqueia o cofre, entrega a chave da porta dos fundos.

A Fome de Jon: Navegação cega por dutos de ventilação guiada por dicas de áudio e texto. Errar o caminho resulta em dano elétrico, vence-lo te entrega a "moeda velha".

Consertos & Sorrisos: Usando a moeda velha, revela um quebra-cabeça de montagem de peças de um animatrônico, te entrega a partitura rasgada, bateria ou remedio, dependendo da montagem.

O Julgamento do Pianista: Usando a partitura rasgada, mostra um interrogatório fatal onde o jogador deve provar seu conhecimento sobre as tragédias do restaurante, entrega o cartão de segurança IV.

A Noite(Sala de Segurança): Com o cartão de segurança, o jogador pode iniciar o minigame da noite, onde você precisa sobreviver até as 6 da manhã, enfrentando todos os animatronicos com energia e recursos limitados. Vence-lo te leva ao final do jogo, onde o sol está quase nascendo, mas que você precisa terminar o que começou.

Labirinto do Minotauro: Ao entrar na sala dos fundos, e ir na slaa de energia, inicia o minigame do minotauro, onde você precisa atravessar uma sala com a entidade "minotauro" te perseguindo, com curtos circuitos em areas da sala, te obrigando a pensar, analisar, desviar e arriscar. Chegando na final, voc~^e precisa pegar o item "Fios Cortados", e seguir de volta para a saída, iniciando a fuga na sala de energia. Minigame obrigatório para pegar o item necessario para o final verdadeiro, "Fios Cortados". Exigindo movimentação precisa sob pressão.

##Comandos do Jogo

* **Movimentação:*

ir [frente/tras/esquerda/direita]

Atalhos: f, t, e, d, norte, sul, leste, oeste

correr [direção] (gasta mais energia e faz muito barulho, mas permite fuga)

Interação:

pegar [item]

largar [item]

usar [item]

combinar [item] com [item]

examinar ou olhar (para investigar o cenário atual)

examinar [objeto] (para inspecionar itens específicos)

* **Sistema e Acesso:*

inventario ou i

limpar ou cls (limpa o terminal)

ajuda

dir, mem, chkdsk (comandos do MS-DOS no menu principal)

###Lore e História (Spoilers)

O jogador assume o papel de Rogério, um homem atormentado por insônia, culpa e por memórias reprimidas depois que sua companheira, Caroline, vai trabalhar no restaurante, e desaparece misteriosamente.
8 meses depois do Desaparecimento, março de 2008, Rogério se encontra em reparar seus erros, e ir atrás da pessoa que ele mais ama.
***(JOGO BASEADO NOS CONTOS AUTORAIS -> "TERROR EM MAQUINA" E "DE VOLTA NO TERROR EM MAQUINA")***

O cenário é o restaurante familiar "Vilas Boas". No passado, eventos trágicos culminaram no desaparecimento e morte de três pessoas: Angela, João e Renato, no incidente conhecido como IPD, incidente das pessoas desaparecidas, ocorrido em 1994. A música e a alegria do local pararam definitivamente em 1995, fechado pela vigiancia sanitaria, pagamentos atrasados e debitos com empresas terceirizadas.

Rogério retorna ao restaurante à noite em uma tentativa de acabar com seus pesadelos e achar Caroline. No entanto, o sistema do edifício e as carcaças dos animatrônicos são acordadas pelo novo visitante, incluindo, o espirito de Caroline, preso na fantasia de um coelho rosa. que manifesta como uma entidade, que procura matar o jogador a qualquer custo, não reconhecendo a sua pessoa. Ela persegue tanto físicamente (perseguindo o jogador) quanto digitalmente (corrompendo a "BIOS" do jogo, exibindo a anomalia caroline.sys).

* **Os Finais Possíveis**
O progresso da história depende de quão fundo o jogador decide ir na investigação:

Morte: Rogério é encontrado morto no restaurante as 8 da manhã do dia seguinte. Especialistas forenses relatam semelhanças com um ataque brutal feito por um predador, como um urso.

Final Covarde: Rogério entra no restaurante, entra em pânico e usa a porta de saída imediatamente sem explorar. Ele sobrevive fisicamente, mas a assombração continua.

Final Bons Sonhos: Rogério entra na sala de descanso, e decide dormir, preferindo entregar-se à insanidade dos pesadelos, ouvindo a porta da sala abrindo devargarzinho, sentindo braços etereos envolvendo seu pescoço.

Final Neutro: Rogério sobrevive até de manhã, vencendo a noite na sala de segurança, enfrentando todos os animatronicos, porém, sem pegar o item "fios cortados", ao chegar ao hall de entrada, o animatronico de caroline o cerca, tendo um dialogo profundo, reconhecendo o protagonista, porém, sua raiva sobresai, matando rogerio na sua frente, e o condenando a viver o terror em maquina. O ciclo de almas presas no maquinário permanece intacto.

Final Verdadeiro: Rogerio consegue a chave dos fundos, vence o minigame do minotauro pegando os "fios cortados", vai na sala de fliperamas, faz cada um dos minigames, conseguindo passar na prova final, o julgamento do pianista, recebendo o cartão IV. Indo para sala de segurança, vencendo a noite, e indo ao hall de entrada, ateia fogo no local, e desliga o animatronico onde Caroline está, fazendo ela finalmente se recordar dele. Sem raiva e sem lamento, os dois se despedem, o animatronico pega fogo, e todos os espiritos, incluindo de caroline, descansam em paz.
