# instruções de escrita

este arquivo registra o jeito de escrever do mantenedor e deve orientar os textos produzidos por pessoas e agentes neste projeto.

## regra obrigatória: começar com letra minúscula

sempre comece com a primeira letra minúscula, em qualquer texto: títulos, README, documentação, títulos e descrições de PR, comentários de PR, comentários no código e mensagens relacionadas ao projeto.

a regra vale para os estilos formal e informal. depois de marcadores como `#`, `-`, `//` ou `/*`, a primeira letra do texto também deve ser minúscula.

essa preferência é sobre a escrita: preserve a sintaxe de código, identificadores, caminhos, comandos e nomes de arquivos quando a capitalização for necessária para funcionar.

## escrita formal

use um português simples e relaxado, mantendo o respeito e a decência formal. evite palavras cultas, frases rebuscadas, burocracia e explicações enroladas. escreva de um jeito natural e fácil de entender.

exemplos do mantenedor, com o início em minúscula:

> e, confesso, não achava difícil. até gostava.

> nada, eu amei

> estava vendo o site para o estudo de linguagem de programação, muito bom. talvez assim que aprenda um pouco.

use esse estilo em README, documentação, descrições de PR e explicações que precisem de mais cuidado. mantenha as informações técnicas precisas, com frases diretas.

## escrita informal

esse é o jeito comum do mantenedor: relaxado, simples, fácil de entender e com abreviações naturais, como `dps`, `aq`, `pq` e `to`.

exemplos do mantenedor:

> vou chegar em casa dps das 20h

> deixe o po de fluorito no seu bau

> to escrevendo uma cosia aq pra minha namorada pq amanha é 11 meses de namoro

use esse modelo para comentários de PR e conversas rápidas do projeto. as abreviações podem aparecer quando forem naturais e compreensíveis; não precisa colocar abreviação em toda frase nem criar erros de digitação para imitar o estilo.

## títulos e descrições de PR

comece sempre com letra minúscula. faça títulos curtos e claros, dizendo o que mudou. nas descrições, use o formal simples e informe o motivo da mudança e como ela foi conferida.

exemplos de títulos:

- melhora a navegação do studio 3d
- corrige o progresso salvo das trilhas
- ajusta o contraste no modo escuro

exemplo de descrição:

> melhora a leitura dos botões e organiza as opções do painel. conferi o fluxo no app e rodei os testes relacionados.

## comentários de PR

use o estilo informal, começando com letra minúscula. seja breve e direto, com contexto suficiente para a outra pessoa entender o ponto.

exemplos:

- ajustei isso aq e conferi no app
- acho que dá pra simplificar essa parte
- esse valor pode vir vazio, vale conferir antes de usar

## comentários no código

comece sempre com letra minúscula. escreva de forma simples, rápida e bem relaxada, até com um jeito meio preguiçoso, sem deixar o comentário confuso.

exemplos do estilo pedido:

```cpp
// tal coisa aq
/* isso é tal coisa */
```

exemplos aplicados ao projeto:

```cpp
// guarda a seleção pra usar dps
// evita dividir por zero aq
/* atualiza a textura depois da pintura */
```

comente quando isso ajudar a entender uma decisão, uma limitação ou um comportamento pouco óbvio. mantenha o comentário curto e correto; quando o assunto precisar de detalhe, explique o necessário com palavras simples.

## aplicação no projeto

siga estas instruções ao criar ou editar textos. ao tocar em um trecho, adapte a escrita desse trecho ao estilo pedido. mudanças grandes em textos antigos devem fazer parte do escopo da tarefa.

em caso de dúvida sobre o tom, use formal simples na documentação e nas descrições de PR, e informal nos comentários de PR e de código. o início em minúscula é obrigatório em todos eles.
