use crate::audio::AudioManager;
use crate::media::MediaManager;
use crate::state::GameState;
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::{desbloquear_conquista, encontrar_melhor_match};
use crate::views;
use crate::TerminalBuffer;
use rand::seq::SliceRandom;
use rand::Rng;

/// Port 1:1 de `processar_comando` do `server/villas_boas/actions/parser.py`
pub fn processar_comando(
    comando_in: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) -> Option<String> {
    let mut comando = comando_in.trim().to_string();

    if jogo.estado_atual == "COMBATE_ANIMATRONICO" {
        if jogo.god_mode
            && ["atacar", "bater", "chutar", "lutar"].contains(&comando.as_str())
        {
            ui.animar(
                "Você dá um soco de direita com força sobre-humana na mandíbula de metal!",
                DOS_AMARELO,
            );
            ui.animar(
                "O animatrônico solta faíscas e recua atordoado para a escuridão!",
                DOS_VERDE,
            );
            jogo.estado_atual = "JOGO".to_string();
            jogo.posicao_perseguidor = "longe".to_string();
            return Some("combate".to_string());
        } else {
            ui.animar(
                "Você tenta reagir na escuridão, mas garras de ferro perfuram seu peito...",
                DOS_VERMELHO,
            );
            jogo.sala_atual = "morte".to_string();
            views::dar_tela_de_morte(jogo, ui, audio, media);
            return Some("morte".to_string());
        }
    }

    if let Some(sala) = jogo.mapa.get(&jogo.sala_atual) {
        if sala.conexoes.contains_key(&comando) {
            comando = format!("ir {}", comando);
        } else if sala.inspecionaveis.contains_key(&comando) {
            comando = format!("examinar {}", comando);
        }
    }

    let partes: Vec<&str> = comando.split_whitespace().collect();
    if partes.is_empty() {
        return None;
    }

    let verbo_bruto = partes[0];
    let argumento = if partes.len() > 1 {
        partes[1..].join(" ")
    } else {
        String::new()
    };

    let verbo = match verbo_bruto {
        "p" => "pegar",
        "l" => "largar",
        "u" => "usar",
        "c" => "combinar",
        "j" => "jogar",
        "x" | "ex" | "o" | "olhar" | "ver" | "investigar" => "examinar",
        "i" | "inv" => "inventario",
        v => v,
    };

    if argumento.is_empty()
        && ["ir", "pegar", "largar", "usar", "combinar"].contains(&verbo)
    {
        let msg = match verbo {
            "ir" => "Ir para onde?",
            "pegar" => "Pegar o quê?",
            "largar" => "Largar o quê?",
            "usar" => "Usar o quê?",
            "combinar" => "Combinar o quê?",
            _ => "",
        };
        ui.exibir(&format!("{}{}{}", DOS_AMARELO, msg, RESET));
        return None;
    }

    if verbo == "examinar" && argumento.is_empty() {
        return Some("olhar".to_string());
    }

    match verbo {
        "ir" => {
            if cmd_ir(&argumento, jogo, ui, audio, media) {
                Some("ir".to_string())
            } else {
                None
            }
        }
        "pegar" => {
            if cmd_pegar(&argumento, jogo, ui, audio, media) {
                Some("pegar".to_string())
            } else {
                None
            }
        }
        "largar" => {
            if cmd_largar(&argumento, jogo, ui, audio, media) {
                Some("largar".to_string())
            } else {
                None
            }
        }
        "usar" => {
            if cmd_usar(&argumento, jogo, ui, audio, media) {
                Some("usar".to_string())
            } else {
                None
            }
        }
        "combinar" => {
            if cmd_combinar(&argumento, jogo, ui, audio, media) {
                Some("combinar".to_string())
            } else {
                None
            }
        }
        "examinar" => {
            if cmd_examinar(&argumento, jogo, ui, audio, media) {
                Some("examinar".to_string())
            } else {
                None
            }
        }
        "inventario" => {
            cmd_inventario(jogo, ui);
            Some("inventario".to_string())
        }
        _ => {
            ui.exibir(&format!(
                "{}Comando inválido. Digite 'ajuda' se precisar.{}",
                DOS_VERMELHO, RESET
            ));
            jogo.erros_consecutivos += 1;
            if jogo.erros_consecutivos >= 3 {
                ui.exibir(&format!(
                    "\n{}[DICA DO SISTEMA]: Tente usar verbos simples no início da frase (ex: 'ir frente', 'olhar mesa', 'pegar chave', 'usar item').{}",
                    DOS_AMARELO, RESET
                ));
                jogo.erros_consecutivos = 0;
            }
            None
        }
    }
}

/// Port 1:1 de `cmd_ir` do `server/villas_boas/actions/movement.py`
pub fn cmd_ir(
    direcao_in: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    _audio: &mut AudioManager,
    _media: &mut MediaManager,
) -> bool {
    let palavras_ignoradas = [
        "para", "pro", "pra", "em", "a", "o", "as", "os", "na", "no",
    ];
    let palavras: Vec<&str> = direcao_in
        .split_whitespace()
        .filter(|p| !palavras_ignoradas.contains(p))
        .collect();
    let mut direcao = palavras.join(" ");

    if direcao.is_empty() {
        ui.exibir(&format!("{}Ir para onde?{}", DOS_AMARELO, RESET));
        return false;
    }

    if ["tras", "atras", "fundo"].contains(&direcao.as_str()) {
        direcao = "atrás".to_string();
    }

    if jogo.sala_atual == "03" && direcao == "frente" {
        ui.limpar();
        ui.animar(
            "Você chuta a parte de baixo da porta emperrada. Com um estalo alto da madeira podre, ela finalmente cede e se abre por completo!",
            DOS_VERDE,
        );
        if let Some(corredor) = jogo.mapa.get_mut("corredor") {
            corredor
                .conexoes
                .insert("03".to_string(), Some("sala do gerador".to_string()));
        }
        jogo.sala_atual = "sala do gerador".to_string();
        return true;
    }

    let opcoes_saidas: Vec<String> = jogo
        .mapa
        .get(&jogo.sala_atual)
        .map(|s| s.conexoes.keys().cloned().collect())
        .unwrap_or_default();

    let direcao_encontrada = encontrar_melhor_match(&direcao, &opcoes_saidas, 0.75);

    if let Some(dir_key) = direcao_encontrada {
        let destino_opt = jogo
            .mapa
            .get(&jogo.sala_atual)
            .and_then(|s| s.conexoes.get(&dir_key).cloned());

        match destino_opt {
            Some(None) => {
                ui.exibir(&format!(
                    "{}A passagem para '{}' está trancada!{}",
                    DOS_VERMELHO, dir_key, RESET
                ));
                false
            }
            Some(Some(destino)) => {
                ui.limpar();
                jogo.turnos_mesma_sala = 0;

                let mut rng = rand::thread_rng();
                if jogo.turnos_luz <= 0
                    && !jogo.god_mode
                    && jogo.dificuldade != "DESAFIO_BREU"
                    && rng.gen::<f64>() < 0.10
                {
                    ui.animar(
                        "TROPEÇO! Na escuridão total, você chuta uma cadeira caída e cai no chão!",
                        DOS_VERMELHO,
                    );
                    ui.animar(
                        "O barulho ecoa pelo restaurante inteiro... Ela ouviu isso.",
                        DOS_VERMELHO,
                    );
                    jogo.posicao_perseguidor = "perto".to_string();
                    return true;
                }

                jogo.sala_atual = destino;

                if rng.gen::<f64>() <= 0.25 && !jogo.eventos_aleatorios.is_empty() {
                    if let Some(evento) = jogo.eventos_aleatorios.choose(&mut rng).cloned() {
                        let is_bateria_repetida = evento.item_recompensa.as_deref()
                            == Some("bateria nova")
                            && jogo.bateria_ajuda_gerada;
                        if !is_bateria_repetida {
                            ui.animar(&format!("\n{}", evento.texto), DOS_AMARELO);
                            if let Some(item_rec) = evento.item_recompensa {
                                if let Some(sala_dest) = jogo.mapa.get_mut(&jogo.sala_atual) {
                                    sala_dest.itens.push(item_rec.clone());
                                }
                                ui.animar(
                                    &format!("Você nota algo novo no chão: {}.", item_rec),
                                    DOS_VERDE,
                                );
                                if item_rec == "bateria nova" {
                                    jogo.bateria_ajuda_gerada = true;
                                }
                            }
                        }
                    }
                }
                true
            }
            None => false,
        }
    } else {
        ui.exibir(&format!(
            "{}Não há caminho para '{}'.{}",
            DOS_VERMELHO, direcao, RESET
        ));
        false
    }
}

/// Port 1:1 de `cmd_pegar` do `server/villas_boas/actions/inventory.py`
pub fn cmd_pegar(
    item: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    let itens_na_sala: Vec<String> = jogo
        .mapa
        .get(&jogo.sala_atual)
        .map(|s| s.itens.clone())
        .unwrap_or_default();

    if let Some(item_real) = encontrar_melhor_match(item, &itens_na_sala, 0.70) {
        if item_real == "bolsa" {
            let novo_limite = jogo.equipar_bolsa();
            ui.animar(
                &format!(
                    "Você encontrou uma bolsa! Agora pode carregar até {} itens.",
                    novo_limite
                ),
                DOS_VERDE,
            );
            return true;
        }

        let limite = jogo.obter_limite_inventario();
        if jogo.inventario.len() >= limite {
            ui.exibir(&format!(
                "{}Mochila cheia ({}/{} itens)! Digite 'largar [item]' para abrir espaço.{}",
                DOS_VERMELHO,
                jogo.inventario.len(),
                limite,
                RESET
            ));
            return false;
        }

        jogo.adicionar_ao_inventario(&item_real);
        ui.exibir(&format!(
            "{}Pegou: {}. ({}/{}){}",
            DOS_VERDE,
            item_real,
            jogo.inventario.len(),
            limite,
            RESET
        ));
        let _ = media.acionar_imagem(&item_real, false, false);

        if jogo.atingiu_meta_acumulador() {
            desbloquear_conquista("acumulador", jogo, ui, audio);
        }
        true
    } else {
        ui.exibir(&format!(
            "{}Não há '{}' aqui.{}",
            DOS_VERMELHO, item, RESET
        ));
        false
    }
}

/// Port 1:1 de `cmd_largar` do `server/villas_boas/actions/inventory.py`
pub fn cmd_largar(
    item: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    _audio: &mut AudioManager,
    _media: &mut MediaManager,
) -> bool {
    let inv = jogo.inventario.clone();
    if let Some(item_real) = encontrar_melhor_match(item, &inv, 0.70) {
        jogo.largar_item(&item_real);
        ui.exibir(&format!(
            "{}Você largou '{}' no chão da sala. ({}/{}){}",
            DOS_AMARELO,
            item_real,
            jogo.inventario.len(),
            jogo.obter_limite_inventario(),
            RESET
        ));
        true
    } else {
        ui.exibir(&format!(
            "{}Você não tem '{}' na mochila.{}",
            DOS_VERMELHO, item, RESET
        ));
        false
    }
}

/// Port 1:1 de `cmd_usar` do `server/villas_boas/actions/inventory.py`
pub fn cmd_usar(
    item: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    audio: &mut AudioManager,
    _media: &mut MediaManager,
) -> bool {
    let inv = jogo.inventario.clone();
    let Some(item_real) = encontrar_melhor_match(item, &inv, 0.70) else {
        ui.exibir(&format!(
            "{}Você não tem '{}' no inventário.{}",
            DOS_VERMELHO, item, RESET
        ));
        return false;
    };

    if item_real == "lanterna" {
        ui.exibir(&format!(
            "{}Você já está usando a lanterna automaticamente para iluminar o caminho.{}",
            DOS_AMARELO, RESET
        ));
        return false;
    } else if item_real == "isqueiro" {
        if let Some(arte) = jogo.ascii_arts.get("ARTE_ISQUEIRO") {
            ui.animar(&format!("{}{}{}", DOS_AMARELO, arte, RESET), DOS_AMARELO);
        }
        jogo.acender_isqueiro();
        ui.animar(
            "Você risca o isqueiro. A pequena chama dança na escuridão, iluminando um pouco ao seu redor, mas logo se apaga para não queimar seus dedos.",
            DOS_AMARELO,
        );
        return true;
    } else if item_real == "chave dos fundos" {
        if jogo.sala_atual == "porta dos fundos" {
            ui.animar(
                "A chave gira com um estalo pesado. A porta para a Sala dos Fundos se abre!",
                DOS_VERDE,
            );
            if let Some(sala) = jogo.mapa.get_mut("porta dos fundos") {
                sala.descricao =
                    "A pesada porta de ferro agora está destrancada, revelando a escuridão lá dentro."
                        .to_string();
                sala.conexoes
                    .insert("frente".to_string(), Some("sala dos fundos".to_string()));
            }
            jogo.inventario.retain(|i| i != "chave dos fundos");
            return true;
        } else {
            ui.exibir(&format!(
                "{}Não há onde usar esta chave aqui.{}",
                DOS_VERMELHO, RESET
            ));
            return false;
        }
    } else if item_real == "bateria nova" {
        let status = jogo.recarregar_lanterna();
        if status == "quebrada_breu" {
            ui.exibir(&format!(
                "{}O refletor da sua lanterna está quebrado (Modo Desafio). A bateria não serve de nada.{}",
                DOS_VERMELHO, RESET
            ));
            return false;
        } else if status == "recarregada" {
            ui.animar(
                "Você troca as pilhas da lanterna. A luz volta a brilhar forte!",
                DOS_VERDE,
            );
            return true;
        } else {
            ui.exibir(&format!(
                "{}Você precisa da lanterna na mochila para colocar a bateria!{}",
                DOS_VERMELHO, RESET
            ));
            return false;
        }
    } else if ["disquete1", "disquete2", "disquete3"].contains(&item_real.as_str()) {
        if jogo.sala_atual == "01" {
            if let Some(arte) = jogo.ascii_arts.get("ARTE_DISQUETE") {
                ui.animar(&format!("{}{}{}", DOS_VERDE, arte, RESET), DOS_VERDE);
            }
            ui.animar(
                &format!(
                    "Você insere o {} no leitor do computador. A tela pisca em verde...",
                    item_real.to_uppercase()
                ),
                DOS_VERDE,
            );

            if item_real == "disquete1" {
                ui.animar(
                    "\n[ARQUIVO DE TEXTO: ANGELA.TXT - 14 de Outubro de 1995]",
                    DOS_AMARELO,
                );
                ui.animar(
                    "\"Eu não aguento mais o turno da noite. Os animatrônicos na sala de peças...\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"Eles não desligam de verdade. Ontem ouvi a voz de uma mulher chorando dentro do traje rosa.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"Quando me aproximei da bancada, o braço metálico se mexeu sozinho e agarrou meu pulso.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"O Sr. Villas-Boas disse que é apenas defeito nos servos motores, mas eu sei o que eu vi.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"Se alguém ler isso, NÃO CONFIE NO SISTEMA DE SEGURANÇA. As portas falham quando Ela se aproxima.\"",
                    DOS_VERMELHO,
                );
            } else if item_real == "disquete2" {
                ui.animar(
                    "\n[ARQUIVO DE LOG: MICHEL.TXT - 02 de Novembro de 1995]",
                    DOS_AMARELO,
                );
                ui.animar(
                    "\"Eles trancaram a Sala dos Fundos. O cheiro que vem de lá de dentro é insuportável.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"Encontrei a chave reserva, mas o gerente mudou o código do cofre da minha sala.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"Aquele maldito piano no Salão de Jogos começou a tocar sozinho de madrugada de novo.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"As notas formam datas... nomes... Ele sabe o que fizemos em 1982. Ele lembra das vítimas.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"Preciso cortar os fios da unidade central na Sala de Energia antes que eles acordem de vez.\"",
                    DOS_VERMELHO,
                );
            } else if item_real == "disquete3" {
                ui.buffer_efeito("@@START_FLASH_SECRETS@@");
                ui.animar(
                    "\n[ARQUIVO CONFIDENCIAL: RENATO.TXT - DATA CORROMPIDA]",
                    DOS_VERMELHO,
                );
                ui.animar(
                    "\"O projeto de transferência neural falhou. A cobaia (Caroline) rejeitou a carcaça mecânica.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"A dor física do metal perfurando os nervos gerou um loop de agonia e fúria no sistema operacional.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"Ela infectou a rede principal (CAROLINE.SYS). Os outros bonecos agora obedecem à mente colmeia dela.\"",
                    DOS_BRANCO,
                );
                ui.animar(
                    "\"A única forma de libertar a consciência presa no metal é através da destruição térmica total.\"",
                    DOS_AMARELO,
                );
                ui.animar(
                    "\"Fogo. Queime os cabos. Queime o salão principal. Queime tudo até virar cinzas.\"",
                    DOS_VERMELHO,
                );
                ui.buffer_efeito("@@STOP_FLASH_SECRETS@@");
            }

            ui.animar(
                "\nO leitor faz um barulho estridente e engole o disquete, destruindo a fita magnética.",
                DOS_VERMELHO,
            );
            jogo.inventario.retain(|i| i != &item_real);
            return true;
        } else {
            ui.exibir(&format!(
                "{}Não há nenhum computador aqui para ler este disquete.{}",
                DOS_VERMELHO, RESET
            ));
            return false;
        }
    } else if item_real == "tabua pequena de madeira" || item_real == "tábua pequena de madeira"
    {
        if jogo.sala_atual == "03" {
            ui.animar(
                "Você usa a tábua de madeira como alavanca na fresta da porta emperrada...",
                DOS_AMARELO,
            );
            ui.animar(
                "CRACK! A madeira quebra ao meio, mas a tranca cede e a porta se abre para a Sala do Gerador!",
                DOS_VERDE,
            );
            if let Some(corredor) = jogo.mapa.get_mut("corredor") {
                corredor
                    .conexoes
                    .insert("03".to_string(), Some("sala do gerador".to_string()));
            }
            if let Some(sala03) = jogo.mapa.get_mut("03") {
                sala03
                    .conexoes
                    .insert("frente".to_string(), Some("sala do gerador".to_string()));
            }
            jogo.inventario.retain(|i| i != &item_real);
            return true;
        } else {
            ui.exibir(&format!(
                "{}Não há necessidade de usar isso aqui.{}",
                DOS_VERMELHO, RESET
            ));
            return false;
        }
    } else if item_real == "remedio" || item_real == "doce" {
        jogo.curar(&item_real, 1);
        if item_real == "remedio" {
            ui.animar(
                "Você engole os comprimidos a seco. Sua respiração desacelera e a dor diminui. (+1 HP)",
                DOS_VERDE,
            );
        } else {
            ui.animar(
                "Você come o doce velho. O açúcar dá um pico repentino de energia no seu sangue! (+1 HP)",
                DOS_VERDE,
            );
            desbloquear_conquista("glicose", jogo, ui, audio);
        }
        return true;
    }

    ui.exibir(&format!(
        "{}Você não vê utilidade para '{}' agora.{}",
        DOS_AMARELO, item_real, RESET
    ));
    false
}

/// Port 1:1 de `cmd_combinar` do `server/villas_boas/actions/inventory.py`
pub fn cmd_combinar(
    argumento: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    _audio: &mut AudioManager,
    _media: &mut MediaManager,
) -> bool {
    let partes: Vec<&str> = if argumento.contains(" com ") {
        argumento.splitn(2, " com ").collect()
    } else if argumento.contains(" e ") {
        argumento.splitn(2, " e ").collect()
    } else {
        ui.exibir(&format!(
            "{}Use o formato: combinar [item 1] com [item 2]{}",
            DOS_AMARELO, RESET
        ));
        return false;
    };

    let item1_str = partes[0].trim();
    let item2_str = partes[1].trim();
    let inv = jogo.inventario.clone();

    let item1 = encontrar_melhor_match(item1_str, &inv, 0.70);
    let item2 = encontrar_melhor_match(item2_str, &inv, 0.70);

    let (Some(i1), Some(i2)) = (item1, item2) else {
        ui.exibir(&format!(
            "{}Você precisa ter os dois itens na mochila para combiná-los.{}",
            DOS_VERMELHO, RESET
        ));
        return false;
    };

    if (i1 == "tesoura quebrada" && i2 == "fita isolante")
        || (i1 == "fita isolante" && i2 == "tesoura quebrada")
    {
        jogo.inventario.retain(|i| i != &i1 && i != &i2);
        jogo.inventario.push("tesoura".to_string());
        ui.animar(
            "Você enrola a fita isolante com força no cabo partido da tesoura. Agora ela está firme e pronta para cortar!",
            DOS_VERDE,
        );
        return true;
    }

    ui.exibir(&format!(
        "{}Combinar '{}' com '{}' não parece funcionar.{}",
        DOS_VERMELHO, i1, i2, RESET
    ));
    false
}

/// Port 1:1 de `cmd_inventario` do `server/villas_boas/actions/inventory.py`
pub fn cmd_inventario(jogo: &GameState, ui: &mut TerminalBuffer) {
    let limite = jogo.obter_limite_inventario();
    ui.exibir(&format!(
        "\n{}=== MOCHILA ({}/{} espaços) ==={}",
        DOS_AMARELO,
        jogo.inventario.len(),
        limite,
        RESET
    ));
    if jogo.inventario.is_empty() {
        ui.exibir(&format!("{}Sua mochila está vazia.{}", DOS_BRANCO, RESET));
    } else {
        for item in &jogo.inventario {
            let desc = jogo
                .descricoes_itens
                .get(item)
                .map(|s| s.as_str())
                .unwrap_or("Sem descrição.");
            ui.exibir(&format!(
                "{}• {}:{} {}",
                DOS_VERDE,
                item.to_uppercase(),
                RESET,
                desc
            ));
        }
    }
    ui.exibir(&format!("{}==============================={}", DOS_AMARELO, RESET));
}

/// Port 1:1 de `cmd_examinar` do `server/villas_boas/actions/inspection.py`
pub fn cmd_examinar(
    objeto: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    _audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    let inspecionaveis_keys: Vec<String> = jogo
        .mapa
        .get(&jogo.sala_atual)
        .map(|s| s.inspecionaveis.keys().cloned().collect())
        .unwrap_or_default();

    if let Some(obj_real) = encontrar_melhor_match(objeto, &inspecionaveis_keys, 0.70) {
        let texto_inspecao = jogo
            .mapa
            .get(&jogo.sala_atual)
            .and_then(|s| s.inspecionaveis.get(&obj_real))
            .cloned()
            .unwrap_or_default();

        if obj_real == "papeis" && jogo.sala_atual == "01" {
            if let Some(arte) = jogo.ascii_arts.get("ARTE_PASTA") {
                ui.animar(&format!("{}{}{}", DOS_AMARELO, arte, RESET), DOS_AMARELO);
            }
        } else if obj_real == "pelucias" {
            if let Some(arte) = jogo.ascii_arts.get("ARTE_PELUCIA") {
                ui.animar(&format!("{}{}{}", DOS_AMARELO, arte, RESET), DOS_AMARELO);
            }
        }

        ui.animar(
            &format!(
                "{}[{}]:{} {}",
                DOS_AMARELO,
                obj_real.to_uppercase(),
                RESET,
                texto_inspecao
            ),
            DOS_BRANCO,
        );
        let _ = media.acionar_imagem(&obj_real, false, false);
        return true;
    }

    let inv = jogo.inventario.clone();
    if let Some(item_real) = encontrar_melhor_match(objeto, &inv, 0.70) {
        let desc = jogo
            .descricoes_itens
            .get(&item_real)
            .cloned()
            .unwrap_or_else(|| "Um item comum, não parece ter nada de especial.".to_string());
        ui.animar(
            &format!(
                "{}[{}]:{} {}",
                DOS_VERDE,
                item_real.to_uppercase(),
                RESET,
                desc
            ),
            DOS_BRANCO,
        );
        let _ = media.acionar_imagem(&item_real, false, false);
        return true;
    }

    let itens_chao: Vec<String> = jogo
        .mapa
        .get(&jogo.sala_atual)
        .map(|s| s.itens.clone())
        .unwrap_or_default();

    if let Some(item_real) = encontrar_melhor_match(objeto, &itens_chao, 0.70) {
        ui.exibir(&format!(
            "{}Está no chão. Digite 'pegar {}' para colocar na mochila.{}",
            DOS_AMARELO, item_real, RESET
        ));
        return false;
    }

    ui.exibir(&format!(
        "{}Não há '{}' aqui para examinar.{}",
        DOS_VERMELHO, objeto, RESET
    ));
    false
}
