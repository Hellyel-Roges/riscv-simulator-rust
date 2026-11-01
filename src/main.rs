mod cpu;
mod memoria;
mod barramento;

use cpu::CPU;
use memoria::Memory;
use barramento::Barramento;

fn main() {
 let mut memoria = Memory::new(1024 * 1024);
    
    // Instrução real em linguagem de máquina para "SW x6, 4(x5)"
    // O opcode dela é 0x23 (cai no match do Hellyel)
    let instrucao_sw: u32 = 0x0062A223; 
    let bytes = instrucao_sw.to_le_bytes(); 
    
    memoria.ram[0] = bytes[0];
    memoria.ram[1] = bytes[1];
    memoria.ram[2] = bytes[2];
    memoria.ram[3] = bytes[3];

    let mut barramento = Barramento::new(memoria);
    let mut cpu = CPU::new();

    // Injetando valores fakes para a sua função testar a matemática de ponteiros
    cpu.registradores[5] = 100; // rs1 (endereço base)
    cpu.registradores[6] = 999; // rs2 (valor a ser salvo na memória)

    println!("--- Simulador RISC-V: Teste de Ciclo ---");
    println!("PC atual: {}", cpu.pc);
    
    // Roda a CPU
    cpu.executar_ciclo(&mut barramento); 

    println!("PC após o ciclo: {}", cpu.pc);
}