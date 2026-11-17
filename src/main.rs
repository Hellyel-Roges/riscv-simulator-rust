mod barramento;
mod cpu;
mod memoria;

use barramento::Barramento;
use cpu::CPU;
use memoria::Memory;

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::fs;
use std::io::{Result, stdout};
use std::panic;
use std::time::Duration;

#[derive(PartialEq)]
enum TelaAtual {
    Menu,
    Simulador,
}

struct MetadadosArte {
    total_words: usize,
    largura_maxima: u16,
    total_linhas: u16,
}

// INICIALIZAÇÃO DA RAM COM O ARQUIVO SELECIONADO
fn carregar_memoria(barramento: &mut Barramento, arquivo_txt: &str) -> MetadadosArte {
    let caminho = format!("assets/{}", arquivo_txt);

    let arte_bruta = fs::read_to_string(&caminho)
        .unwrap_or_else(|_| String::from("ERRO: Arquivo nao encontrado!\n"));

    // Remove '\r' (CRLF -> LF) para evitar corrupção visual de cursor no terminal
    let arte_limpa: String = arte_bruta.chars().filter(|&c| c != '\r').collect();

    let largura_maxima = arte_limpa
        .lines()
        .map(|l| l.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let total_linhas = arte_limpa.lines().count() as u16;

    let mut bytes = arte_limpa.into_bytes();

    // Alinha para 32 bits (múltiplo de 4 bytes)
    while !bytes.len().is_multiple_of(4) {
        bytes.push(b' ');
    }
    let total_words = bytes.len() / 4;

    // Adiciona Null Terminator (palavra 0x00000000 indicando fim da string)
    bytes.extend_from_slice(&[0, 0, 0, 0]);

    // Grava a chave XOR no endereço 252 (0xFC), entre o fim do programa (0x78) e o início da arte (256)
    let chave_xor = 0x5A5A_5A5A;
    barramento.escrever_u32(252, chave_xor);

    // Grava a arte na RAM (Endereço base: 0x0100 = 256) aplicada com máscara XOR
    for (i, chunk) in bytes.chunks(4).enumerate() {
        let mut arr = [0u8; 4];
        arr[..chunk.len()].copy_from_slice(chunk);
        let mut word = u32::from_le_bytes(arr);

        // Aplica XOR em todas as palavras (inclusive no terminador nulo) para descriptografia pela CPU
        word ^= chave_xor;

        barramento.escrever_u32(256 + (i * 4), word);
    }

    // ALGORITMO ASSEMBLY: Exercita as 28 instruções (RV32I + LDINC) e renderiza a arte na VRAM
    let programa: [u32; 31] = [
        // Inicialização de ponteiros e validação das instruções Tipo-I e LW
        0x0080_0113, // 0x00: ADDI  x2, x0, 8       (Prepara base VRAM)
        0x0101_1113, // 0x04: SLLI  x2, x2, 16      (x2 = 0x80000 -> Base VRAM)
        0x0FC0_6093, // 0x08: ORI   x1, x0, 252     (x1 = 252 / 0xFC -> Endereço da chave XOR)
        0x0000_A283, // 0x0C: LW    x5, 0(x1)       (x5 = MEM[252] = 0x5A5A5A5A -> Carrega chave)
        0x00F0_4313, // 0x10: XORI  x6, x0, 15      (x6 = 15)
        0x0073_7393, // 0x14: ANDI  x7, x6, 7       (x7 = 15 & 7 = 7)
        0x00A3_A413, // 0x18: SLTI  x8, x7, 10      (x8 = (7 < 10) = 1)
        0x00A3_B493, // 0x1C: SLTIU x9, x7, 10      (x9 = (7 <u 10) = 1)
        0x0013_5513, // 0x20: SRLI  x10, x6, 1      (x10 = 15 >> 1 = 7)
        0x4013_5593, // 0x24: SRAI  x11, x6, 1      (x11 = 15 >>a 1 = 7)
        // Validação das instruções aritméticas e lógicas Tipo-R na ULA
        0x0094_0633, // 0x28: ADD   x12, x8, x9     (x12 = 1 + 1 = 2)
        0x4073_06B3, // 0x2C: SUB   x13, x6, x7     (x13 = 15 - 7 = 8)
        0x0086_1733, // 0x30: SLL   x14, x12, x8    (x14 = 2 << 1 = 4 -> Offset de 4 bytes)
        0x00D6_27B3, // 0x34: SLT   x15, x12, x13   (x15 = (2 < 8) = 1)
        0x00D6_3833, // 0x38: SLTU  x16, x12, x13   (x16 = (2 <u 8) = 1)
        0x0086_D8B3, // 0x3C: SRL   x17, x13, x8    (x17 = 8 >> 1 = 4)
        0x4086_D933, // 0x40: SRA   x18, x13, x8    (x18 = 8 >>a 1 = 4)
        0x0087_69B3, // 0x44: OR    x19, x14, x8    (x19 = 4 | 1 = 5)
        0x00E3_7A33, // 0x48: AND   x20, x6, x14    (x20 = 15 & 4 = 4)
        // Validação das instruções de desvio condicional Tipo-B e ajuste do ponteiro da arte
        0x00D6_4263, // 0x4C: BLT   x12, x13, +4    (2 < 8: avança para 0x50)
        0x00C6_D263, // 0x50: BGE   x13, x12, +4    (8 >= 2: avança para 0x54)
        0x00D6_6263, // 0x54: BLTU  x12, x13, +4    (2 <u 8: avança para 0x58)
        0x00C6_F263, // 0x58: BGEU  x13, x12, +4    (8 >=u 2: avança para 0x5C)
        0x00E0_80B3, // 0x5C: ADD   x1, x1, x14     (x1 = 252 + 4 = 256 / 0x00100 -> Base RAM da Arte)
        // Loop de leitura com LDINC, descriptografia XOR e escrita na VRAM com SW
        0x0000_818B, // 0x60: LDINC x3, 0(x1)       (x3 = MEM[x1]; x1 += 4)
        0x0051_C1B3, // 0x64: XOR   x3, x3, x5      (Descriptografa: x3 = x3 ^ x5)
        0x0001_8863, // 0x68: BEQ   x3, x0, +16     (Se x3 == 0, salta p/ 0x78 HALT)
        0x0031_2023, // 0x6C: SW    x3, 0(x2)       (VRAM[x2] = x3)
        0x0041_0113, // 0x70: ADDI  x2, x2, 4       (x2 += 4)
        0xFE01_16E3, // 0x74: BNE   x2, x0, -20     (Volta p/ 0x60 LOOP)
        0x0000_0000, // 0x78: HALT                  (Fim da execução)
    ];

    for (i, &instrucao) in programa.iter().enumerate() {
        barramento.escrever_u32(i * 4, instrucao);
    }

    MetadadosArte {
        total_words,
        largura_maxima,
        total_linhas,
    }
}

// IDENTIFICAÇÃO DO MNEMÔNICO DA INSTRUÇÃO ATUAL PARA EXIBIÇÃO NO PAINEL DECODIFICADOR
fn decodificar_nome_instrucao(instrucao: u32) -> &'static str {
    if instrucao == 0 {
        return "HALT (Fim do Programa)";
    }
    let opcode = instrucao & 0x7F;
    let funct3 = (instrucao >> 12) & 0x7;
    let funct7 = (instrucao >> 25) & 0x7F;

    match opcode {
        0x0B => "LDINC (Custom Grupo 5)",
        0x03 => "LW (Tipo-I Load Word)",
        0x23 => "SW (Tipo-S Store Word)",
        0x73 => "MRET (Retorno Interrupção)",
        0x13 => match funct3 {
            0b000 => "ADDI (Tipo-I Soma Imediato)",
            0b010 => "SLTI (Tipo-I Menor c/ Sinal)",
            0b011 => "SLTIU (Tipo-I Menor s/ Sinal)",
            0b100 => "XORI (Tipo-I XOR Imediato)",
            0b110 => "ORI (Tipo-I OR Imediato)",
            0b111 => "ANDI (Tipo-I AND Imediato)",
            0b001 => "SLLI (Tipo-I Shift Esq.)",
            0b101 if funct7 == 0b0100000 => "SRAI (Tipo-I Shift Dir. Arit.)",
            0b101 => "SRLI (Tipo-I Shift Dir. Lóg.)",
            _ => "Tipo-I Desconhecida",
        },
        0x33 => match (funct3, funct7) {
            (0b000, 0b0100000) => "SUB (Tipo-R Subtração)",
            (0b000, _) => "ADD (Tipo-R Soma)",
            (0b001, _) => "SLL (Tipo-R Shift Esq.)",
            (0b010, _) => "SLT (Tipo-R Menor c/ Sinal)",
            (0b011, _) => "SLTU (Tipo-R Menor s/ Sinal)",
            (0b100, _) => "XOR (Tipo-R OU Exclusivo)",
            (0b101, 0b0100000) => "SRA (Tipo-R Shift Dir. Arit.)",
            (0b101, _) => "SRL (Tipo-R Shift Dir. Lóg.)",
            (0b110, _) => "OR (Tipo-R OU Lógico)",
            (0b111, _) => "AND (Tipo-R E Lógico)",
            _ => "Tipo-R Desconhecida",
        },
        0x63 => match funct3 {
            0b000 => "BEQ (Tipo-B Igual)",
            0b001 => "BNE (Tipo-B Diferente)",
            0b100 => "BLT (Tipo-B Menor c/ Sinal)",
            0b101 => "BGE (Tipo-B Maior/Igual c/ Sinal)",
            0b110 => "BLTU (Tipo-B Menor s/ Sinal)",
            0b111 => "BGEU (Tipo-B Maior/Igual s/ Sinal)",
            _ => "Tipo-B Desconhecida",
        },
        _ => "Instrução Desconhecida",
    }
}

// DESCRIÇÃO DO TRÁFEGO DO BARRAMENTO (USADO NO AUTOPLAY E NO STEP MANUAL)
fn descrever_ciclo_barramento(instrucao: u32, cpu: &CPU) -> (String, String, String) {
    let opcode = instrucao & 0x7F;
    match opcode {
        0x0B => (
            format!("{:#010X} (RAM)", cpu.registradores[1]),
            format!("{:#010X}", cpu.registradores[3]),
            String::from("READ [RAM -> CPU] + INC x1"),
        ),
        0x23 => (
            format!("{:#010X} (VRAM)", cpu.registradores[2]),
            format!("{:#010X}", cpu.registradores[3]),
            String::from("WRITE [CPU -> VRAM]"),
        ),
        0x03 => (
            format!("{:#010X} (RAM)", cpu.registradores[1]),
            format!("{:#010X}", cpu.registradores[5]),
            String::from("READ LW [RAM -> CPU]"),
        ),
        0x63 => (
            format!("{:#010X} (PC)", cpu.pc),
            format!("{:#010X}", instrucao),
            String::from("BRANCH CHECK (Comparando ULA)"),
        ),
        0x13 => (
            format!("{:#010X} (PC)", cpu.pc),
            format!("{:#010X}", instrucao),
            String::from("ALU IMM (Calculando Registrador)"),
        ),
        0x33 => (
            format!("{:#010X} (PC)", cpu.pc),
            format!("{:#010X}", instrucao),
            String::from("ALU REG (Operação Tipo-R na ULA)"),
        ),
        0x73 => (
            format!("{:#010X} (EPC)", cpu.epc),
            format!("{:#010X}", instrucao),
            String::from("MRET (Retorno de Interrupção)"),
        ),
        _ => (
            String::from("0x00000078"),
            String::from("0x00000000"),
            String::from("HALT (Barramento em Repouso)"),
        ),
    }
}

// RENDERIZAÇÃO DA VRAM COM MARGEM FIXA ESTÁVEL
fn construir_tela_vram(memoria: &Memory, term_width: u16, largura_arte: u16) -> (String, u16) {
    let mut conteudo_bruto = String::new();

    for i in (0x80000..0x90000).step_by(4) {
        let word = memoria.ler_u32(i);
        if word == 0 {
            break;
        }

        for &byte in &word.to_le_bytes() {
            if byte == 0 {
                break;
            }
            if byte == b'\n' || (32..=126).contains(&byte) {
                conteudo_bruto.push(byte as char);
            }
        }
    }

    let largura_interna = term_width.saturating_sub(2);
    let margem_esquerda = if largura_interna > largura_arte {
        (largura_interna - largura_arte) / 2
    } else {
        0
    };
    let margem_str = " ".repeat(margem_esquerda as usize);

    let mut tela_centralizada = String::new();
    let mut linhas_renderizadas: u16 = 0;

    for linha in conteudo_bruto.split('\n') {
        tela_centralizada.push_str(&margem_str);
        tela_centralizada.push_str(linha);
        tela_centralizada.push('\n');
        linhas_renderizadas += 1;
    }

    (tela_centralizada, linhas_renderizadas)
}

fn main() -> Result<()> {
    let hook_original = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = stdout().execute(LeaveAlternateScreen);
        hook_original(panic_info);
    }));

    let mut arquivos_txt: Vec<String> = fs::read_dir("assets")
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "txt"))
        .map(|e| e.file_name().into_string().unwrap())
        .collect();

    arquivos_txt.sort();

    if arquivos_txt.is_empty() {
        arquivos_txt.push(String::from("Nenhum arquivo .txt encontrado"));
    }

    let mut tela_atual = TelaAtual::Menu;
    let mut arquivo_selecionado = 0;

    let memoria = Memory::new();
    let mut barramento = Barramento::new(memoria);
    let mut cpu = CPU::new();

    let mut executando = false;
    let mut ciclo_count: u64 = 0;
    let mut inst_atual: u32 = 0;

    let mut bus_addr = String::from("0x00000000");
    let mut bus_data = String::from("0x00000000");
    let mut bus_ctrl = String::from("IDLE (Aguardando Clock)");

    let mut scroll_y: u16 = 0;
    let mut scroll_x: u16 = 0;
    let mut cycles_per_tick: u32 = 1;
    let mut meta_arte = MetadadosArte {
        total_words: 1,
        largura_maxima: 80,
        total_linhas: 1,
    };
    let mut linhas_atuais_vram: u16 = 0;
    let mut altura_janela_vram: u16 = 20;

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|f| {
            let area_total = f.area();

            if tela_atual == TelaAtual::Menu {
                let menu_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(10),
                        Constraint::Length(5),
                        Constraint::Min(10),
                    ])
                    .margin(3)
                    .split(area_total);

                let logo_ascii = "\
 ██████╗ ██╗███████╗ ██████╗    ██╗   ██╗
 ██╔══██╗██║██╔════╝██╔════╝    ██║   ██║
 ██████╔╝██║███████╗██║         ██║   ██║
 ██╔══██╗██║╚════██║██║         ╚██╗ ██╔╝
 ██║  ██║██║███████║╚██████╗     ╚████╔╝ 
 ╚═╝  ╚═╝╚═╝╚══════╝ ╚═════╝      ╚═══╝  
 \n      S I M U L A D O R   R V 3 2 I   -   S H A D O W   E D I T I O N";

                let logo = Paragraph::new(logo_ascii)
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                    .alignment(Alignment::Center);
                f.render_widget(logo, menu_layout[0]);

                let resumo = "Arquitetura de Ciclo Único RV32I | Mapeamento MMIO RAM (0x00000) & VRAM (0x80000)\nInstrução Especial Exclusiva (Grupo 5): LDINC rd, 0(rs1) -> Lê 32-bits e incrementa ponteiro em +4.";
                let widget_resumo = Paragraph::new(resumo)
                    .style(Style::default().fg(Color::LightMagenta))
                    .alignment(Alignment::Center)
                    .block(Block::default().borders(Borders::ALL).title(" Sobre o Projeto "));
                f.render_widget(widget_resumo, menu_layout[1]);

                let mut lista_txt = String::from(
                    "Use as SETAS [UP/DOWN] para escolher o arquivo ASCII, [ENTER] para carregar na RAM e [ESC] para sair:\n\n",
                );
                for (i, nome) in arquivos_txt.iter().enumerate() {
                    if i == arquivo_selecionado {
                        lista_txt.push_str(&format!("  ▶ [ {} ] ◀ (Selecionado)\n", nome));
                    } else {
                        lista_txt.push_str(&format!("    - {}\n", nome));
                    }
                }

                let widget_lista = Paragraph::new(lista_txt)
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().borders(Borders::ALL).title(" Seletor de Arquivos (.txt) "));
                f.render_widget(widget_lista, menu_layout[2]);
            } else {
                let layout_principal = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(3),
                        Constraint::Length(14),
                        Constraint::Min(10),
                    ])
                    .split(area_total);

                altura_janela_vram = layout_principal[2].height.saturating_sub(2);

                let status_str = if inst_atual == 0 {
                    "HALT (CONCLUÍDO)"
                } else if executando {
                    "RUNNING"
                } else {
                    "PAUSED"
                };
                let status_color = if inst_atual == 0 {
                    Color::Cyan
                } else if executando {
                    Color::Green
                } else {
                    Color::Red
                };

                let header = Paragraph::new(Line::from(vec![
                    Span::styled(
                        format!(" ⚒ ARQUIVO: {} ", arquivos_txt[arquivo_selecionado]),
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("| Status: "),
                    Span::styled(status_str, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                    Span::raw(format!(
                        " | Ciclos: {} | Vel: {}x | [SPC] Play/Pause | [ENTER] Step | [I] IRQ | [+/-] Vel | [Setas] Scroll | [Q] Menu",
                        ciclo_count, cycles_per_tick
                    )),
                ]))
                .block(Block::default().borders(Borders::ALL));
                f.render_widget(header, layout_principal[0]);

                let layout_hardware = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(28),
                        Constraint::Percentage(24),
                        Constraint::Percentage(23),
                        Constraint::Percentage(25),
                    ])
                    .split(layout_principal[1]);

                // PROGRAMA EM MEMÓRIA (MOSTRA O LOOP, A ULA, A INTERRUPÇÃO E O PC AO VIVO)
                let linhas_asm = [
                    (0x00, "0x00: ADDI  x2, x0, 8"),
                    (0x04, "0x04: SLLI  x2, x2, 16    [VRAM]"),
                    (0x08, "0x08: ORI   x1, x0, 252   [0xFC]"),
                    (0x0C, "0x0C: LW    x5, 0(x1)     [Chave]"),
                    (0x10, "0x10: XORI  x6, x0, 15    [x6=15]"),
                    (0x14, "0x14: ANDI  x7, x6, 7     [x7=7]"),
                    (0x18, "0x18: SLTI  x8, x7, 10    [x8=1]"),
                    (0x1C, "0x1C: SLTIU x9, x7, 10    [x9=1]"),
                    (0x20, "0x20: SRLI  x10, x6, 1    [x10=7]"),
                    (0x24, "0x24: SRAI  x11, x6, 1    [x11=7]"),
                    (0x28, "0x28: ADD   x12, x8, x9   [x12=2]"),
                    (0x2C, "0x2C: SUB   x13, x6, x7   [x13=8]"),
                    (0x30, "0x30: SLL   x14, x12, x8  [x14=4]"),
                    (0x34, "0x34: SLT   x15, x12, x13 [x15=1]"),
                    (0x38, "0x38: SLTU  x16, x12, x13 [x16=1]"),
                    (0x3C, "0x3C: SRL   x17, x13, x8  [x17=4]"),
                    (0x40, "0x40: SRA   x18, x13, x8  [x18=4]"),
                    (0x44, "0x44: OR    x19, x14, x8  [x19=5]"),
                    (0x48, "0x48: AND   x20, x6, x14  [x20=4]"),
                    (0x4C, "0x4C: BLT   x12, x13, +4  [Salta]"),
                    (0x50, "0x50: BGE   x13, x12, +4  [Salta]"),
                    (0x54, "0x54: BLTU  x12, x13, +4  [Salta]"),
                    (0x58, "0x58: BGEU  x13, x12, +4  [Salta]"),
                    (0x5C, "0x5C: ADD   x1, x1, x14   [0x100]"),
                    (0x60, "0x60: LDINC x3, 0(x1)     [Lê+Inc]"),
                    (0x64, "0x64: XOR   x3, x3, x5    [Decrip]"),
                    (0x68, "0x68: BEQ   x3, x0, +16   [Fim?]"),
                    (0x6C, "0x6C: SW    x3, 0(x2)     [VRAM]"),
                    (0x70, "0x70: ADDI  x2, x2, 4     [+4]"),
                    (0x74, "0x74: BNE   x2, x0, -20   [Loop]"),
                    (0x78, "0x78: HALT  (0x00000000)"),
                    (0x0007_F000, "7F000: ADDI x31, x31, 1   [IRQ]"),
                    (0x0007_F004, "7F004: MRET               [EPC]"),
                ];

                let mut texto_prog = Vec::new();
                for &(addr, desc) in &linhas_asm {
                    if cpu.pc == addr {
                        texto_prog.push(Line::from(Span::styled(
                            format!(" ▶ {}", desc),
                            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        texto_prog.push(Line::from(Span::styled(
                            format!("   {}", desc),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }
                }

                // Ajusta a rolagem do painel de programa automaticamente conforme a posição do PC
                let idx_pc = linhas_asm
                    .iter()
                    .position(|&(addr, _)| addr == cpu.pc)
                    .unwrap_or(0) as u16;
                let scroll_prog = idx_pc.saturating_sub(4).min(21);

                let cpu_widget = Paragraph::new(texto_prog)
                    .block(
                        Block::default()
                            .title(format!(" Programa na RAM (PC: {:#04X}) ", cpu.pc))
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::Cyan)),
                    )
                    .scroll((scroll_prog, 0));
                f.render_widget(cpu_widget, layout_hardware[0]);

                // DECODIFICADOR DE INSTRUÇÃO
                let opcode = inst_atual & 0x7F;
                let rd = (inst_atual >> 7) & 0x1F;
                let funct3 = (inst_atual >> 12) & 0x7;
                let rs1 = (inst_atual >> 15) & 0x1F;
                let rs2 = (inst_atual >> 20) & 0x1F;
                let funct7 = (inst_atual >> 25) & 0x7F;

                let nome_inst = decodificar_nome_instrucao(inst_atual);

                let decoder_info = format!(
                    " Instrução (Fetch): {:#010X}\n Operação : {}\n ├─ Opcode: {:#04X} | funct3: {:#X}\n ├─ funct7: {:#04X} (bits 31:25)\n ├─ rs1   : x{:<2}  | rs2: x{:<2}\n └─ rd    : x{:<2}",
                    inst_atual, nome_inst, opcode, funct3, funct7, rs1, rs2, rd
                );
                let decoder_widget = Paragraph::new(decoder_info).block(
                    Block::default()
                        .title(" Decodificador & ULA ")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::LightYellow)),
                );
                f.render_widget(decoder_widget, layout_hardware[1]);

                // BANCO DE REGISTRADORES
                let bytes_x3 = cpu.registradores[3].to_le_bytes();
                let chars_x3: String = bytes_x3
                    .iter()
                    .map(|&b| if (32..=126).contains(&b) { b as char } else { '·' })
                    .collect();

                let prox_ram = if (cpu.registradores[1] as usize) < 0x90000 {
                    barramento.memoria.ler_u32(cpu.registradores[1] as usize)
                } else {
                    0
                };

                let reg_info = format!(
                    " x1 (Ptr RAM): {:#010X}\n x2 (Ptr VRA): {:#010X}\n x3 (Buffer) : {:#010X}\n x5 (Chave)  : {:#010X}\n MEM[x1] Próx: {:#010X}\n x31 (IRQs)  : {}\n\n ✎ ASCII(x3): [{}]",
                    cpu.registradores[1],
                    cpu.registradores[2],
                    cpu.registradores[3],
                    cpu.registradores[5],
                    prox_ram,
                    cpu.registradores[31],
                    chars_x3
                );

                let reg_widget = Paragraph::new(reg_info).block(
                    Block::default()
                        .title(" Registradores & Memória ")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White)),
                );
                f.render_widget(reg_widget, layout_hardware[2]);

                // BARRAMENTO & PROGRESSO DA VRAM
                let words_gravadas = (cpu.registradores[2].saturating_sub(0x80000) / 4) as usize;
                let progresso_pct = ((words_gravadas as f64 / meta_arte.total_words.max(1) as f64) * 100.0).min(100.0);

                let bus_info = format!(
                    " ⛖ Endereço : {}\n ❒ Dados    : {}\n ◉ Controle : {}\n\n ▤ Transferência RAM -> VRAM:\n    {} / {} words ({:.1}%)\n    Linhas na VRAM: {} / {}",
                    bus_addr,
                    bus_data,
                    bus_ctrl,
                    words_gravadas,
                    meta_arte.total_words,
                    progresso_pct,
                    linhas_atuais_vram,
                    meta_arte.total_linhas
                );
                let bus_widget = Paragraph::new(bus_info).block(
                    Block::default()
                        .title(" Barramento (32-bits) ")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::LightMagenta)),
                );
                f.render_widget(bus_widget, layout_hardware[3]);

                // MONITOR VRAM CENTRALIZADO E COM SCROLL TRAVADO NA BORDA
                let (vram_tela, linhas) = construir_tela_vram(
                    &barramento.memoria,
                    layout_principal[2].width,
                    meta_arte.largura_maxima,
                );
                linhas_atuais_vram = linhas;

                let max_scroll_y = linhas_atuais_vram.saturating_sub(altura_janela_vram);
                if scroll_y > max_scroll_y {
                    scroll_y = max_scroll_y;
                }

                let vram_widget = Paragraph::new(vram_tela)
                    .block(
                        Block::default()
                            .title(format!(
                                " Monitor VRAM (0x80000) | Scroll Y: {}/{} | Use as SETAS para mover ",
                                scroll_y, max_scroll_y
                            ))
                            .borders(Borders::ALL)
                            .style(Style::default().fg(Color::Cyan)),
                    )
                    .scroll((scroll_y, scroll_x));
                f.render_widget(vram_widget, layout_principal[2]);
            }
        })?;

        // DRENA TODOS OS EVENTOS DE TECLADO PENDENTES PARA EVITAR LAG AO SEGURAR TECLAS
        if event::poll(Duration::from_millis(15))? {
            while event::poll(Duration::ZERO)? {
                if let Event::Key(key) = event::read()? {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }

                    if tela_atual == TelaAtual::Menu {
                        match key.code {
                            KeyCode::Esc => {
                                disable_raw_mode()?;
                                stdout().execute(LeaveAlternateScreen)?;
                                return Ok(());
                            }
                            KeyCode::Up => {
                                arquivo_selecionado = arquivo_selecionado.saturating_sub(1);
                            }
                            KeyCode::Down => {
                                if arquivo_selecionado < arquivos_txt.len() - 1 {
                                    arquivo_selecionado += 1;
                                }
                            }
                            KeyCode::Enter
                                if !arquivos_txt.is_empty()
                                    && arquivos_txt[0] != "Nenhum arquivo .txt encontrado" =>
                            {
                                let nova_mem = Memory::new();
                                barramento = Barramento::new(nova_mem);
                                cpu = CPU::new();

                                executando = false;
                                ciclo_count = 0;
                                scroll_y = 0;
                                scroll_x = 0;

                                let selecao = &arquivos_txt[arquivo_selecionado];

                                meta_arte = carregar_memoria(&mut barramento, selecao);

                                // Grava a rotina de tratamento de interrupção em 0x0007_F000 (incrementa x31 e retorna via MRET)
                                barramento.escrever_u32(0x0007_F000, 0x001F_8F93); // ADDI x31, x31, 1
                                barramento.escrever_u32(0x0007_F004, 0x3020_0073); // MRET

                                inst_atual = barramento.ler_u32(cpu.pc as usize);
                                bus_addr = String::from("0x00000000 (PC)");
                                bus_data = format!("{:#010X}", inst_atual);
                                bus_ctrl = String::from("PRONTO (Pressione SPACE ou ENTER)");

                                tela_atual = TelaAtual::Simulador;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Esc => {
                                disable_raw_mode()?;
                                stdout().execute(LeaveAlternateScreen)?;
                                return Ok(());
                            }
                            KeyCode::Char('q') | KeyCode::Backspace => {
                                executando = false;
                                tela_atual = TelaAtual::Menu;
                            }
                            // Dispara uma solicitação de interrupção externa via teclado (IRQ)
                            KeyCode::Char('i') | KeyCode::Char('I') => {
                                if !cpu.parado {
                                    cpu.interrupcao_pendente = true;
                                    bus_ctrl =
                                        String::from("IRQ PENDENTE (Aguardando próximo ciclo)");
                                }
                            }
                            KeyCode::Char(' ') => executando = !executando,
                            KeyCode::Char('+') | KeyCode::Char('=') => {
                                cycles_per_tick = (cycles_per_tick * 2).min(50000)
                            }
                            KeyCode::Char('-') => cycles_per_tick = (cycles_per_tick / 2).max(1),
                            KeyCode::Up => scroll_y = scroll_y.saturating_sub(2),
                            KeyCode::Down => {
                                let max_scroll_y =
                                    linhas_atuais_vram.saturating_sub(altura_janela_vram);
                                scroll_y = (scroll_y + 2).min(max_scroll_y);
                            }
                            KeyCode::Left => scroll_x = scroll_x.saturating_sub(4),
                            KeyCode::Right => scroll_x = scroll_x.saturating_add(4),
                            KeyCode::Enter => {
                                if inst_atual != 0 {
                                    let (addr, data, ctrl) =
                                        descrever_ciclo_barramento(inst_atual, &cpu);
                                    bus_addr = addr;
                                    bus_data = data;
                                    bus_ctrl = ctrl;

                                    cpu.executar_ciclo(&mut barramento);
                                    inst_atual = barramento.ler_u32(cpu.pc as usize);
                                    ciclo_count += 1;
                                } else {
                                    bus_ctrl = String::from("HALT (Execução Finalizada)");
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // EXECUÇÃO CONTÍNUA (AUTOPLAY)
        if executando && tela_atual == TelaAtual::Simulador {
            for _ in 0..cycles_per_tick {
                if inst_atual != 0 {
                    let (addr, data, ctrl) = descrever_ciclo_barramento(inst_atual, &cpu);
                    bus_addr = addr;
                    bus_data = data;
                    bus_ctrl = ctrl;

                    cpu.executar_ciclo(&mut barramento);
                    inst_atual = barramento.ler_u32(cpu.pc as usize);
                    ciclo_count += 1;
                } else {
                    bus_addr = String::from("0x00000078");
                    bus_data = String::from("0x00000000");
                    bus_ctrl = String::from("HALT (Renderização 100% Concluída)");
                    executando = false;
                    break;
                }
            }
        }
    }
}