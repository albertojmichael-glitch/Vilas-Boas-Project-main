use crate::audio::Configuracoes;
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;

pub const DOS_VERDE: &str = "\x1b[92m";
pub const DOS_VERMELHO: &str = "\x1b[91m";
pub const DOS_AMARELO: &str = "\x1b[93m";
pub const DOS_BRANCO: &str = "\x1b[97m";
pub const DOS_CIANO: &str = "\x1b[96m";
pub const DOS_CINZA: &str = "\x1b[90m";
pub const RESET: &str = "\x1b[0m";

pub struct TerminalUI {
    pub animacao_ativa: bool,
}

impl Default for TerminalUI {
    fn default() -> Self {
        Self {
            animacao_ativa: true,
        }
    }
}

impl TerminalUI {
    pub fn limpar(&self) {
        let mut out = stdout();
        let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));
        let _ = out.flush();
    }

    pub fn exibir(&self, msg: &str) {
        println!("{}{}", msg, RESET);
    }

    pub fn animar(&self, msg: &str, delay_secs: f64, cor: &str) {
        if !self.animacao_ativa || delay_secs <= 0.0 {
            println!("{}{}{}", cor, msg, RESET);
            return;
        }
        let mut out = stdout();
        let _ = write!(out, "{}", cor);
        let delay = Duration::from_secs_f64(delay_secs);
        for ch in msg.chars() {
            let _ = write!(out, "{}", ch);
            let _ = out.flush();
            thread::sleep(delay);
        }
        println!("{}", RESET);
    }

    pub fn efeito_jumpscare(&self, ativo: bool) {
        if !ativo {
            return;
        }
        let mut out = stdout();
        for _ in 0..3 {
            let _ = write!(out, "\x1b[41m\x1b[97m");
            let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));
            let _ = out.flush();
            thread::sleep(Duration::from_millis(60));
            let _ = write!(out, "{}", RESET);
            let _ = execute!(out, Clear(ClearType::All), MoveTo(0, 0));
            let _ = out.flush();
            thread::sleep(Duration::from_millis(60));
        }
    }
}

pub fn imprimir_tela_boot(ui: &TerminalUI) {
    ui.limpar();
    ui.exibir(&format!(
        "{}VILAS BOAS OS [Versão 1.982 - MODO OFFLINE]{}",
        DOS_VERDE, RESET
    ));
    ui.exibir(&format!(
        "{}(C) Copyright 1982 Vilas Boas Corp. Todos os direitos reservados.{}\n",
        DOS_CINZA, RESET
    ));
    ui.exibir(&format!(
        "{}Digite {}dir{} para acessar os diretórios:{}",
        DOS_VERDE, DOS_BRANCO, DOS_VERDE, RESET
    ));
}

pub fn imprimir_menu_principal(ui: &TerminalUI, tem_save: bool) {
    ui.exibir(&format!(
        "\n{}================[ DIRETÓRIO C:\\VILAS_BOAS ]================{}",
        DOS_VERDE, RESET
    ));
    ui.exibir(&format!("{}  [1] INICIAR NOVA PARTIDA{}", DOS_BRANCO, RESET));
    if tem_save {
        ui.exibir(&format!(
            "{}  [2] CONTINUAR PARTIDA SALVA{}",
            DOS_AMARELO, RESET
        ));
    } else {
        ui.exibir(&format!(
            "{}  [2] CONTINUAR (Nenhum save encontrado){}",
            DOS_CINZA, RESET
        ));
    }
    ui.exibir(&format!("{}  [3] DESAFIOS E CONQUISTAS{}", DOS_BRANCO, RESET));
    ui.exibir(&format!(
        "{}  [4] GALERIA E ARQUIVOS SECRETOS{}",
        DOS_BRANCO, RESET
    ));
    ui.exibir(&format!(
        "{}  [5] CONFIGURAÇÕES (ÁUDIO, JUMPSCARE E TELA){}",
        DOS_CIANO, RESET
    ));
    ui.exibir(&format!("{}  [6] SAIR PARA O SISTEMA{}", DOS_BRANCO, RESET));
    ui.exibir(&format!(
        "{}==========================================================={}",
        DOS_VERDE, RESET
    ));
    ui.exibir(&format!("{}Escolha uma opção:{}", DOS_VERDE, RESET));
}

pub fn imprimir_menu_configuracoes(ui: &TerminalUI, cfg: &Configuracoes) {
    ui.exibir(&format!(
        "\n{}==============[ CONFIGURAÇÕES DO SISTEMA ]=============={}",
        DOS_CIANO, RESET
    ));
    ui.exibir(&format!(
        "  [1] Volume Geral      : {:.0}%  (ex: digite 'geral 80')",
        cfg.volume_geral * 100.0
    ));
    ui.exibir(&format!(
        "  [2] Volume Jumpscare  : {:.0}%  (ex: digite 'jumpscare 30')",
        cfg.volume_jumpscare * 100.0
    ));
    ui.exibir(&format!(
        "  [3] Volume Ambiente   : {:.0}%  (ex: digite 'ambiente 50')",
        cfg.volume_ambiente * 100.0
    ));
    ui.exibir(&format!(
        "  [4] Volume Efeitos    : {:.0}%  (ex: digite 'efeitos 70')",
        cfg.volume_efeitos * 100.0
    ));
    ui.exibir(&format!(
        "  [5] Áudio Mudo        : {}   (digite 'mudo' para alternar)",
        if cfg.mudo { "SIM" } else { "NÃO" }
    ));
    ui.exibir(&format!(
        "  [6] Flash Jumpscare   : {}   (digite 'flash' para alternar)",
        if cfg.jumpscare_visual { "ON" } else { "OFF" }
    ));
    ui.exibir(&format!(
        "  [7] Animação de Texto : {}   (digite 'animacao' para alternar)",
        if cfg.animacao_texto { "ON" } else { "OFF" }
    ));
    ui.exibir(&format!(
        "  [8] Imagens em Janela : {}   (digite 'janela' para alternar)",
        if cfg.abrir_janela_externa { "ON" } else { "OFF" }
    ));
    ui.exibir(&format!(
        "  [testar] Testar som do Jumpscare | [voltar] Salvar e retornar"
    ));
    ui.exibir(&format!(
        "{}========================================================{}",
        DOS_CIANO, RESET
    ));
}