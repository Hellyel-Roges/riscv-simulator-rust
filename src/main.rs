mod barramento;
mod cpu;
mod memoria;

use barramento::Barramento;
use cpu::CPU;
use memoria::Memory;

fn main() {
    let memoria = Memory::new(1024 * 1024);
    let mut barramento = Barramento::new(memoria);
    let mut cpu = CPU::new();

    // Programa em código de máquina RV32I carregado na RAM (a partir do endereço 0x00):
    // 0x00: ADDI x1, x0, 100   -> x1 = 100 (ponteiro base na RAM)
    // 0x04: ADDI x2, x0, 10    -> x2 = 10
    // 0x08: ADDI x3, x0, 20    -> x3 = 20
    // 0x0C: SW   x2, 0(x1)     -> MEM[100] = 10
    // 0x10: SW   x3, 4(x1)     -> MEM[104] = 20
    // 0x14: ADDI x4, x0, 108   -> x4 = 108 (endereço limite para encerrar o laço)
    // 0x18: LDINC x5, 0(x1)    -> x5 = MEM[x1] e x1 = x1 + 4 (instrução do Grupo 5)
    // 0x1C: ADD  x6, x6, x5    -> x6 = x6 + x5 (acumula a soma na ULA)
    // 0x20: BNE  x1, x4, -8    -> se x1 != 108, volta 8 bytes no PC (para 0x18)
    // 0x24: SW   x6, 0(x1)     -> MEM[108] = x6 (salva o resultado 30 na RAM)
    // 0x28: LW   x7, 0(x1)     -> x7 = MEM[108] (carrega o resultado final em x7)
    let programa: [u32; 11] = [
        0x0640_0093, // ADDI x1, x0, 100
        0x00A0_0113, // ADDI x2, x0, 10
        0x0140_0193, // ADDI x3, x0, 20
        0x0020_A023, // SW   x2, 0(x1)
        0x0030_A223, // SW   x3, 4(x1)
        0x06C0_0213, // ADDI x4, x0, 108
        0x0000_828B, // LDINC x5, 0(x1)
        0x0053_0333, // ADD  x6, x6, x5
        0xFE40_9CE3, // BNE  x1, x4, -8
        0x0060_A023, // SW   x6, 0(x1)
        0x0000_A383, // LW   x7, 0(x1)
    ];

    // Carrega as instruções na memória principal a partir do endereço 0
    for (i, &instrucao) in programa.iter().enumerate() {
        barramento.escrever_u32(i * 4, instrucao);
    }

    println!("=== Simulador RISC-V (RV32I) - Execução de Ciclo Único ===");

    let mut ciclo = 1;
    loop {
        let pc_atual = cpu.pc;
        let instrucao = barramento.ler_u32(pc_atual as usize);

        // Encerra quando encontra a memória vazia (0x00000000) após o fim do programa
        if instrucao == 0 {
            println!(
                "\n[Ciclo {:02}] PC: {:#04X} | Fim da execução (memória zerada).",
                ciclo, pc_atual
            );
            break;
        }

        println!(
            "[Ciclo {:02}] PC: {:#04X} | Instrução: {:#010X}",
            ciclo, pc_atual, instrucao
        );

        cpu.executar_ciclo(&mut barramento);
        ciclo += 1;
    }

    println!("\n=== Estado Final dos Registradores Utilizados ===");
    println!("x1 (Ponteiro após LDINC) : {}", cpu.registradores[1]);
    println!("x2 (Valor 1)             : {}", cpu.registradores[2]);
    println!("x3 (Valor 2)             : {}", cpu.registradores[3]);
    println!("x4 (Limite do laço)      : {}", cpu.registradores[4]);
    println!("x5 (Último lido LDINC)   : {}", cpu.registradores[5]);
    println!("x6 (Acumulador ADD)      : {}", cpu.registradores[6]);
    println!("x7 (Resultado lido LW)   : {}", cpu.registradores[7]);
}