use crate::barramento::Barramento;

pub struct CPU {
    pub registradores: [u32; 32],
    pub pc: u32,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registradores: [0; 32],
            pc: 0,
        }
    }

    pub fn executar_ciclo(&mut self, barramento: &mut Barramento) {
        // Busca a instrução usando o PC atual
        let instrucao = barramento.ler_u32(self.pc as usize);
        
        // Incrementa para a próxima instrução
        self.pc += 4;
        
        // Extrai o opcode (7 bits menos significativos)
        let opcode = instrucao & 0x7F;

        match opcode {
            0x33 => self.executar_tipo_r(instrucao),
            0x13 => self.executar_tipo_i_aritmetico(instrucao),
            0x63 => self.executar_branch(instrucao),
            0x03 => self.executar_load(instrucao, barramento),
            0x23 => self.executar_store(instrucao, barramento),
            0x0B => self.executar_ldinc(instrucao, barramento), // Opcode customizado para o Grupo 5
            0x00 => println!("Fim da execucao (memoria zerada)."),
            _ => println!("Opcode desconhecido: {:#X}", opcode),
        }
    }

    pub fn executar_tipo_r(&mut self, _instrucao: u32) {
        // Grazi, aqui você extrai o rs1, rs2, rd, funct3 e funct7.
        // Dica de extração de bits. let funct3 = (_instrucao >> 12) & 0x7;
        // Lembra que o registrador zero (x0) sempre vale 0 e não pode ser alterado.
    }

    pub fn executar_tipo_i_aritmetico(&mut self, _instrucao: u32) {
        // Grazi, aqui é a lógica do ADDI, SLLI, etc.
        // O imediato tem 12 bits, você vai precisar fazer extensão de sinal nele.
    }

    pub fn executar_branch(&mut self, _instrucao: u32) {
        // Gustavo, aqui você extrai os imediatos fracionados, rs1, rs2 e funct3.
        // Se a condição (ex: rs1 == rs2 no BEQ) for verdadeira, você altera o self.pc
        // Cuidado na hora de juntar os pedaços do imediato (eles vêm embaralhados na instrução Tipo-B).
    }


    
    // Parte do Hellyel, Implementação das instruções de load e store, incluindo a customizada LDINC.

    pub fn executar_load(&mut self, instrucao: u32, barramento: &mut Barramento) {
        // Extração dos campos do Tipo-I (LW)
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;
        
        // Imediato de 12 bits com extensão de sinal (converte pra i32 e empurra os bits)
        let imm = ((instrucao as i32) >> 20) as i32; 
        
        // Calcula o endereço base (rs1 + imediato)
        let endereco = self.registradores[rs1].wrapping_add(imm as u32) as usize;
        
        // rd = MEM[rs1 + imm]
        if rd != 0 {
            self.registradores[rd] = barramento.ler_u32(endereco);
        }
    }

pub fn executar_store(&mut self, instrucao: u32, barramento: &mut Barramento) {
        // Extração dos campos do Tipo-S (SW)
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;
        let rs2 = ((instrucao >> 20) & 0x1F) as usize;
        
        // Imediato do Tipo-S é fatiado em dois pedaços (bits 11:5 e 4:0)
        let imm_11_5 = (instrucao >> 25) & 0x7F;
        let imm_4_0 = (instrucao >> 7) & 0x1F;
        let mut imm = (imm_11_5 << 5) | imm_4_0;
        
        // Extensão de sinal manual se o número for negativo
        if (imm & 0x800) != 0 {
            imm |= 0xFFFFF000;
        }
        
        let endereco = self.registradores[rs1].wrapping_add(imm) as usize;
        let valor_para_salvar = self.registradores[rs2];
        
        // Grava o valor na memória através do barramento
        barramento.escrever_u32(endereco, valor_para_salvar);
        println!("Gravado {} no endereço {}", valor_para_salvar, endereco);
    }

    pub fn executar_ldinc(&mut self, instrucao: u32, barramento: &mut Barramento) {
        // A LDINC agora lê seus próprios parâmetros de dentro dos 32 bits
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;
        
        let endereco = self.registradores[rs1] as usize;
        
        // Passo 1: rd = MEM[rs1]
        if rd != 0 {
            self.registradores[rd] = barramento.ler_u32(endereco);
        }

        // Passo 2: rs1 = rs1 + 4
        if rs1 != 0 {
            self.registradores[rs1] = self.registradores[rs1].wrapping_add(4); 
        }
    }
}