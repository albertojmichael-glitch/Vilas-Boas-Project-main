use crate::audio::AudioManager;
use crate::data::{salas_terminais, CHANCE_EVENTO_ALEATORIO, CHANCE_TROPECO_ESCURO_PCT};
use crate::media::MediaManager;
use crate::parser::normalizar;
use crate::state::GameState;
use crate::ui::{DOS_AMARELO, DOS_BRANCO, DOS_VERDE, DOS_VERMELHO, RESET};
use crate::utils::{desbloquear_conquista, encontrar_melhor_match};
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
        if jogo.god_mode && ["atacar", "bater", "chutar", "lutar"].contains(&comando.as_str()) {
            ui.animar(
                "Você dá um soco de direita com força sobre-humana na mandíbula de metal!",
                DOS_AMARELO,
            );
            ui.animar(
                "O animatrônico solta faíscas e recua atordoado para a escuridão!",
                DOS_AMARELO,
            );
            ui.pausa(2.0);
            jogo.estado_atual = "JOGO".to_string();
            jogo.posicao_perseguidor = "longe".to_string();
            return Some("combate".to_string());
        } else {
            ui.animar(
                "Você tenta reagir na escuridão, mas garras de ferro perfuram seu peito...",
                DOS_VERMELHO,
            );
            jogo.sala_atual = "morte".to_string();
            ui.pausa(2.0);
            return Some("morte".to_string());
        }
    }

    if let Some(sala) = jogo.mapa.get(&jogo.sala_atual) {
        if sala
            .conexoes
            .keys()
            .any(|chave| normalizar(chave) == comando)
        {
            comando = format!("ir {}", comando);
        } else if sala
            .inspecionaveis
            .keys()
            .any(|chave| normalizar(chave) == comando)
        {
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
        "c" | "juntar" => "combinar",
        "j" => "jogar",
        "x" | "ex" | "o" | "olhar" | "ver" | "investigar" => "examinar",
        "i" | "inv" => "inventario",
        v => v,
    };

    if argumento.is_empty() && ["ir", "pegar", "largar", "usar", "combinar"].contains(&verbo) {
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
            ui.exibir("Comando não reconhecido.");
            if jogo.erros_consecutivos + 1 >= crate::data::LIMITE_ERROS_DICA {
                ui.pausa(0.5);
                ui.exibir(&format!(
                    "\n{DOS_AMARELO}[DICA DO SISTEMA] Parece que você está com dificuldades. Tente verbos simples:{RESET}\n\
                     {DOS_BRANCO} ☞ 'olhar' {DOS_AMARELO}(para examinar a sala){RESET}\n\
                     {DOS_BRANCO} ☞ 'ir frente / trás / esquerda / direita'{RESET}\n\
                     {DOS_BRANCO} ☞ 'pegar [item]' ou 'usar [item]'{RESET}\n\
                     {DOS_BRANCO} ☞ 'inventario' {DOS_AMARELO}(para ver sua mochila){RESET}"
                ));
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
    let palavras_ignoradas = ["para", "pro", "pra", "em", "a", "o", "as", "os", "na", "no"];
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
        ui.exibir(&format!(
            "{DOS_AMARELO}Você toma distância e dá um chute violento na porta emperrada!{RESET}"
        ));
        ui.exibir(&format!(
            "{DOS_VERDE}A madeira velha cede e a porta escancara.{RESET}"
        ));
        abrir_sala_gerador(jogo);
        ui.pausa(2.0);
        return true;
    }

    let Some(sala) = jogo.mapa.get(&jogo.sala_atual) else {
        return false;
    };
    let saidas = sala.saidas_listadas();
    let opcoes: Vec<_> = saidas.iter().map(|(chave, _)| chave.clone()).collect();
    let destino_exato = saidas
        .iter()
        .find_map(|(_, destino)| (normalizar(destino) == direcao).then(|| destino.clone()));
    let destino = destino_exato.or_else(|| {
        encontrar_melhor_match(&direcao, &opcoes, 0.70).and_then(|chave| sala.obter_saida(&chave))
    });
    let Some(destino) = destino else {
        ui.exibir(&format!("Você não pode ir para '{direcao_in}'."));
        if !opcoes.is_empty() {
            ui.exibir(&format!(
                "{DOS_BRANCO}Saídas disponíveis: {}{RESET}",
                opcoes.join(", ")
            ));
        }
        ui.pausa(1.5);
        return false;
    };
    if !jogo.mapa.contains_key(&destino) && !salas_terminais().contains(&destino.as_str()) {
        ui.exibir("O caminho está bloqueado ou não há saída para essa direção.");
        return false;
    }

    ui.limpar();
    jogo.turnos_mesma_sala = 0;
    let mut rng = rand::thread_rng();
    if jogo.turnos_luz <= 0
        && !jogo.god_mode
        && jogo.dificuldade_escolhida != "DESAFIO_BREU"
        && rng.gen_range(1..=100) <= CHANCE_TROPECO_ESCURO_PCT
    {
        ui.exibir("\n No escuro, você perde a noção da direção, e acaba tropeçando no proprio pé, e cai no chão");
        let morreu = jogo.aplicar_dano(1);
        ui.exibir(&format!(" Você se machucou na queda. (HP: {})", jogo.hp));
        ui.pausa(2.0);
        if morreu {
            ui.exibir("\n Você cai no chão e quebra sua perna, você não consegue mais andar, e escuta barulhos vindo na sua direção");
            ui.pausa(2.0);
        }
        return true;
    }

    jogo.sala_atual = destino;
    if jogo.mapa.contains_key(&jogo.sala_atual) && rng.gen::<f64>() < CHANCE_EVENTO_ALEATORIO {
        if let Some(evento) = jogo.eventos_aleatorios.choose(&mut rng) {
            let drop = evento.item_recompensa.as_deref();
            if drop != Some("bateria nova") || !jogo.bateria_ajuda_gerada {
                ui.pausa(0.5);
                ui.exibir(&format!("\n{DOS_AMARELO}⚠ {}{RESET}", evento.texto));
                if let Some(item) = drop {
                    let sala = jogo.mapa.get_mut(&jogo.sala_atual).expect("sala válida");
                    if !jogo.inventario.iter().chain(&sala.itens).any(|i| i == item) {
                        sala.itens.push(item.to_string());
                        if item == "bateria nova" {
                            jogo.bateria_ajuda_gerada = true;
                        }
                    }
                }
            }
        }
    }
    ui.pausa(0.5);
    true
}

fn abrir_sala_gerador(jogo: &mut GameState) {
    if let Some(corredor) = jogo.mapa.get_mut("corredor") {
        corredor
            .conexoes
            .insert("03".to_string(), Some("sala do gerador".to_string()));
    }
    jogo.sala_atual = "sala do gerador".to_string();
}

/// Port 1:1 de `cmd_pegar` do `server/villas_boas/actions/inventory.py`
pub fn cmd_pegar(
    item: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    _audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    let Some(item_real) = jogo
        .mapa
        .get(&jogo.sala_atual)
        .and_then(|sala| encontrar_melhor_match(item, &sala.itens, 0.70))
    else {
        ui.exibir(&format!("Não há nenhum '{item}' aqui para pegar."));
        return false;
    };

    if item_real == "bolsa" {
        let limite = jogo.equipar_bolsa();
        ui.exibir(&format!(
            "{DOS_VERDE}Você equipou a bolsa! Seu limite de inventário aumentou permanentemente (Total: {limite} slots).{RESET}"
        ));
    } else if jogo.adicionar_ao_inventario(&item_real) {
        ui.exibir(&format!("{DOS_VERDE}Você pegou: {item_real}{RESET}"));
        let _ = media.acionar_imagem(&item_real, false, false);
    } else {
        ui.exibir(&format!(
            "{DOS_VERMELHO}Sua mochila está cheia! Você precisa largar algo antes.{RESET}"
        ));
        return false;
    }

    if let Some(sala) = jogo.mapa.get_mut(&jogo.sala_atual) {
        remover_um(&mut sala.itens, &item_real);
    }
    true
}

fn remover_um(itens: &mut Vec<String>, item: &str) {
    if let Some(indice) = itens.iter().position(|existente| existente == item) {
        itens.remove(indice);
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

    match item_real.as_str() {
        "lanterna" => {
            ui.exibir("Você já está usando a lanterna automaticamente (quando tem bateria).");
        }
        "isqueiro" => {
            if let Some(arte) = jogo.ascii_arts.get("ARTE_ISQUEIRO") {
                ui.animar_com_intervalo(&format!("{DOS_AMARELO}{arte}{RESET}"), DOS_AMARELO, 0.015);
            }
            if !jogo.acender_isqueiro() {
                ui.exibir(&format!("{DOS_AMARELO}Você acende o isqueiro, mas a sua lanterna já ilumina o local muito bem.{RESET}"));
                return false;
            }
            ui.exibir(&format!("{DOS_AMARELO}Você risca a pedra do isqueiro. Uma chama tremeluzente ilumina as sombras.{RESET}"));
            ui.exibir(&format!("{DOS_VERDE}A luz fraca permite enxergar o chão, mas o gás parece que vai durar pouco.{RESET}"));
        }
        "chave dos fundos" => {
            if jogo.sala_atual == "porta dos fundos" {
                ui.exibir(&format!("{DOS_VERDE}Você insere a chave suja de graxa na fechadura e força. Ela gira com um estalo alto.{RESET}"));
                ui.exibir(&format!("{DOS_AMARELO}A pesada porta de metal se escancara, revelando um corredor denso e escuro.{RESET}"));
                if let Some(sala) = jogo.mapa.get_mut("porta dos fundos") {
                    sala.descricao =
                        "A pesada porta de metal está aberta, levando para a área de serviço."
                            .to_string();
                    sala.conexoes
                        .insert("frente".to_string(), Some("sala dos fundos".to_string()));
                }
                remover_um(&mut jogo.inventario, &item_real);
            } else {
                ui.exibir("Não há nenhuma fechadura por aqui que se encaixe nessa chave.");
            }
        }
        "bateria nova" => {
            if jogo.recarregar_lanterna() == "quebrada_breu" {
                ui.exibir("\nVocê tenta colocar a bateria nova, mas a sua lanterna está permanentemente quebrada.");
            } else {
                ui.exibir(&format!("{DOS_VERDE}Você abre a parte inferior da lanterna e insere a bateria nova.{RESET}"));
                ui.exibir(&format!(
                    "{DOS_AMARELO}A luz da lanterna fica forte e ofuscante!{RESET}"
                ));
            }
        }
        "disquete1" | "disquete2" | "disquete3" => usar_disquete(&item_real, jogo, ui),
        "tabua pequena de madeira" | "tábua pequena de madeira" => {
            if jogo.sala_atual == "03" {
                ui.exibir(&format!("{DOS_AMARELO}Você usa a tábua como alavanca e força a porta emperrada...{RESET}"));
                ui.exibir(&format!(
                    "{DOS_VERDE}CRASH! A porta cede e abre. A tábua quebra no processo.{RESET}"
                ));
                remover_um(&mut jogo.inventario, &item_real);
                abrir_sala_gerador(jogo);
                ui.pausa(2.0);
            } else {
                ui.exibir("Não há onde usar a tábua aqui.");
            }
        }
        "remedio" | "doce" => {
            if !jogo.curar(&item_real, 1) {
                ui.exibir("Você já está se sentindo bem. Melhor guardar isso para depois.");
                return false;
            }
            if item_real == "remedio" {
                ui.exibir(&format!("{DOS_VERDE}Você engole o relaxante muscular vencido. Seus músculos perdem o estresse e a dor diminui (+1 HP).{RESET}"));
            } else {
                ui.exibir(&format!("{DOS_VERDE}Você mastiga o doce velho. O pico de açúcar te dá um pouco mais de resistência (+1 HP).{RESET}"));
                desbloquear_conquista("glicose", jogo, ui, audio);
            }
        }
        _ => ui.exibir(&format!("Você não sabe como usar '{item_real}' aqui.")),
    }
    true
}

fn usar_disquete(item: &str, jogo: &mut GameState, ui: &mut TerminalBuffer) {
    if jogo.sala_atual != "01" {
        ui.exibir(&format!("{DOS_BRANCO}Você segura o velho {item}, mas não há nenhum computador neste cômodo para lê-lo. Talvez na sala de segurança?{RESET}"));
        return;
    }
    ui.exibir(&format!(
        "{DOS_VERDE}Você insere o {item} sujo no drive do terminal de segurança...{RESET}"
    ));
    ui.pausa(1.5);
    ui.exibir(&format!("{DOS_BRANCO}LENDO A:\\ ...{RESET}"));
    if let Some(arte) = jogo.ascii_arts.get("ARTE_DISQUETE") {
        ui.animar_com_intervalo(&format!("{DOS_BRANCO}{arte}{RESET}"), DOS_BRANCO, 0.015);
        ui.pausa(1.0);
    }
    match item {
        "disquete1" => {
            ui.animar_com_intervalo(
                &format!("{DOS_AMARELO}ARQUIVO RECUPERADO: ANGELA.TXT{RESET}"),
                DOS_AMARELO,
                0.05,
            );
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Hoje vim mostrar para meu esposo João, meu local de trabalho, o Vilas Boas. Talvez não tenha sido uma boa ideia.'{RESET}"), DOS_BRANCO, 0.06);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'A gente brigou feio no meio do salão, pois aparentemente ele achava que tinha alguém me observando atrás das cortinas, sendo que não... Não tinha nada lá além de poeira e peças enferrujadas. Ele está perdendo a cabeça.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Ele foi falar com meu chefe, o Sr. Renato, lá na salas dos fundos, enquanto eu escrevo isso.'{RESET}"), DOS_BRANCO, 0.08);
            ui.animar_com_intervalo(&format!("{DOS_VERMELHO}'Talvez... Seja loucura minha, mas eu vi alguem me chamando para a cozinha privada pela janela do escritório, vou ir lá ver.'{RESET}"), DOS_VERMELHO, 0.05);
            ui.animar_com_intervalo(
                &format!("{DOS_VERMELHO}'Ela foi libertada.'{RESET}"),
                DOS_VERMELHO,
                0.1,
            );
        }
        "disquete2" => {
            ui.animar_com_intervalo(
                &format!("{DOS_AMARELO}ARQUIVO RECUPERADO: MICHEL.TXT{RESET}"),
                DOS_AMARELO,
                0.05,
            );
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Depois que a Caroline partiu, não encontramos ninguém para alavancar esse projeto.'{RESET}"), DOS_BRANCO, 0.06);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Acreditamos que não há salvação mais, vamos abandonar tudo e deixar as traças. Tudo o que a gente já devia ter feito.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Perdemos tudo, e agora tem coisas no restaurante, que mesmo a gente causando isso... Nós nos arrependemos, e vimos que tudo isso não passa de uma vergonha.'{RESET}"), DOS_BRANCO, 0.06);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Sinto culpa por tudo que fiz. Não sei como me defender, sei que sou culpado, e se isso tudo vier à tona, irei pegar prisão perpétua sem dúvidas.'{RESET}"), DOS_BRANCO, 0.06);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Sou um covarde que levou todas essas pessoas à morte. Não vou mais deixar que isso me consuma, vou ir embora enquanto há tempo.'{RESET}"), DOS_BRANCO, 0.06);
            ui.animar_com_intervalo(
                &format!("{DOS_VERMELHO}Ele fugiu.{RESET}"),
                DOS_VERMELHO,
                0.1,
            );
        }
        "disquete3" => {
            ui.buffer_efeito("@@START_FLASH_SECRETS@@");
            ui.animar_com_intervalo(
                &format!("{DOS_AMARELO}ARQUIVO RECUPERADO: RENATO.TXT{RESET}"),
                DOS_AMARELO,
                0.05,
            );
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'1994, 3 de setembro. O restaurante está indo bem, mantendo uma clientela fiel. Escrevo isso como relatório.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Sou Renato Fidelis Gomes, fundador deste local. Eu mesmo fiz esses animatrônicos, juntei placas e peças. Minhas obras-primas.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Derramei muito sangue e suor. Tenho fantasias antigas no estoque: uma de lebre rosa sem nome, a do Senhor Raposa, e a de jacaré, que guardo bem.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Sou um inventor fadado a cuidar da administração sozinho. Tenho orgulho da minha primeira criação, um animatrônico de touro com 3 rostos.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Deixo ele na sala de energia. Fiz um mini programa de sonar que lança ondas nas paredes. Se algo estiver no radar, ele vai atrás.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'O porco Jon mapeia o local inteiro e procura as entradas. Se o caminho mais curto for pela tubulação, ele vai pela tubulação.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'O mosqueteiro Rick é mais direto, mas precisa de manutenção. O índio Jones segue vozes e induz pensamentos nos clientes. Não sei qual o limite.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_VERMELHO}'E o cozinheiro Alberto tem acesso ao sistema inteiro do restaurante, acho isso preocupante.'{RESET}"), DOS_VERMELHO, 0.06);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'A ideia surgiu quando visitei Hurricane, em Utah. Descobri a Chuck E. Cheese e a Fazbear Pizza. Fiquei encantado com as atrações.'{RESET}"), DOS_BRANCO, 0.05);
            ui.animar_com_intervalo(&format!("{DOS_BRANCO}'Penso que se esse lugar crescer, farei parcerias com eles para trazer ao Brasil. Escrevi muito, hora de dar Adeus. (FIM)'{RESET}"), DOS_BRANCO, 0.06);
            ui.buffer_efeito("@@STOP_FLASH_SECRETS@@");
        }
        _ => {}
    }
    ui.pausa(2.0);
    ui.exibir(&format!("{DOS_VERMELHO}O drive faz um ruído horrível e ejeta o {item} arranhado. Ele está arruinado.{RESET}"));
    remover_um(&mut jogo.inventario, item);
    ui.pausa(2.0);
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
        remover_um(&mut jogo.inventario, &i1);
        remover_um(&mut jogo.inventario, &i2);
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
    ui.exibir(&format!(
        "{}==============================={}",
        DOS_AMARELO, RESET
    ));
}

/// Port 1:1 de `cmd_examinar` do `server/villas_boas/actions/inspection.py`
pub fn cmd_examinar(
    objeto: &str,
    jogo: &mut GameState,
    ui: &mut TerminalBuffer,
    _audio: &mut AudioManager,
    media: &mut MediaManager,
) -> bool {
    let Some(sala) = jogo.mapa.get(&jogo.sala_atual) else {
        return false;
    };
    let mut cenario: Vec<_> = sala.inspecionaveis.keys().cloned().collect();
    cenario.sort();
    if let Some(chave) = encontrar_melhor_match(objeto, &cenario, 0.70) {
        ui.exibir(&format!(
            "\n{DOS_VERDE}C:\\> ACESSANDO ARQUIVO DE DADOS...{RESET}"
        ));
        ui.pausa(1.0);
        if chave == "papeis" && jogo.sala_atual == "01" {
            if let Some(arte) = jogo.ascii_arts.get("ARTE_PASTA") {
                ui.animar_com_intervalo(arte, DOS_BRANCO, 0.015);
            }
        }
        ui.animar_com_intervalo(&sala.inspecionaveis[&chave], DOS_AMARELO, 0.03);
        ui.pausa(2.0);
    } else if let Some(item) = encontrar_melhor_match(objeto, &jogo.inventario, 0.70)
        .or_else(|| encontrar_melhor_match(objeto, &sala.itens, 0.70))
    {
        if item == "pelucias" {
            if let Some(arte) = jogo.ascii_arts.get("ARTE_PELUCIA") {
                ui.animar_com_intervalo(arte, DOS_AMARELO, 0.015);
            }
        }
        let descricao = jogo
            .descricoes_itens
            .get(&item)
            .map(String::as_str)
            .unwrap_or("Não há nada de especial nisso.");
        ui.exibir(&format!("\n{DOS_AMARELO} ☞ {descricao}{RESET}"));
    } else {
        ui.exibir(&format!(
            "Você não vê nenhum '{objeto}' aqui para examinar."
        ));
        return false;
    }
    let _ = media.acionar_imagem(objeto, false, false);
    true
}
