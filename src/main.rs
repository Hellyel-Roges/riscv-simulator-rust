mod cpu;
mod memoria;
mod barramento;

use cpu::CPU;
use memoria::Memory;
use barramento::Barramento;

fn main() {
    // 1MB de memória instanciado
    let mut memoria = Memory::new(1024 * 1024);
    
    // Inserimos valores fake no endereço 0x100 (256 em decimal) para o teste
    memoria.ram[256] = 0xAA;
    memoria.ram[257] = 0xBB;
    memoria.ram[258] = 0xCC;
    memoria.ram[259] = 0xDD;

    // barramento entregando a memória para ele gerenciar
    let barramento = Barramento::new(memoria);
    
    // CPU criada
    let mut cpu = CPU::new();

    // Configurando a instrução
    cpu.registradores[10] = 256; // Registrador rs1 aponta para o endereço 256
    
    println!("--- Simulador RISC-V Iniciado ---");
    println!("Estado antes do LDINC:");
    println!("rs1 (Reg 10): {}", cpu.registradores[10]);
    println!("rd  (Reg 11): {:#X}\n", cpu.registradores[11]);

    // A CPU pede para executar o LDINC (rd = 11, rs1 = 10) usando o barramento
    cpu.executar_ldinc(11, 10, &barramento);

    println!("Estado depois do LDINC:");
    println!("rd  (Reg 11): {:#X} (Valor lido do Barramento)", cpu.registradores[11]);
    println!("rs1 (Reg 10): {} (Incrementado em +4)", cpu.registradores[10]);
}
