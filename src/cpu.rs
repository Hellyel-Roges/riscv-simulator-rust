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

    
pub fn executar_tipo_r(&mut self, instrucao: u32) {
    // Extrai os campos da instrução Tipo-R
    let rd = ((instrucao >> 7) & 0x1F) as usize;
    let funct3 = (instrucao >> 12) & 0x7;
    let rs1 = ((instrucao >> 15) & 0x1F) as usize;
    let rs2 = ((instrucao >> 20) & 0x1F) as usize;
    let funct7 = (instrucao >> 25) & 0x7F;

    // Busca os valores dos registradores
    let a = self.registradores[rs1];
    let b = self.registradores[rs2];

    // Deslocamento limitado aos 5 bits inferiores
    let deslocamento = b & 0x1F;

    // Identifica a operação usando funct3 e funct7
    let resultado = match (funct3, funct7) {
        // Soma
        (0b000, 0b0000000) => Some(a.wrapping_add(b)), // ADD
        // Subtração
        (0b000, 0b0100000) => Some(a.wrapping_sub(b)), // SUB
        // Deslocamento lógico para esquerda
        (0b001, 0b0000000) => Some(a << deslocamento), // SLL
        // Comparação com sinal
        (0b010, 0b0000000) => Some(((a as i32) < (b as i32)) as u32), // SLT
        // Comparação sem sinal
        (0b011, 0b0000000) => Some((a < b) as u32), // SLTU
        // XOR bit a bit
        (0b100, 0b0000000) => Some(a ^ b), // XOR
        // Deslocamento lógico para direita
        (0b101, 0b0000000) => Some(a >> deslocamento), // SRL
        // Deslocamento aritmético para direita
        (0b101, 0b0100000) => Some(((a as i32) >> deslocamento) as u32), // SRA
        // OR bit a bit
        (0b110, 0b0000000) => Some(a | b), // OR
        // AND bit a bit
        (0b111, 0b0000000) => Some(a & b), // AND
        // Instrução não reconhecida
        _ => None,
    };

    // Salva o resultado no registrador de destino
    // O registrador x0 não pode ser alterado
    match resultado {
        Some(valor) if rd != 0 => {
            self.registradores[rd] = valor;
        }
        Some(_) => {}
        None => {
            eprintln!("Instrucao Tipo-R invalida");
        }
    }
}


    
pub fn executar_tipo_i_aritmetico(&mut self, instrucao: u32) {
    // Extrai os campos da instrução Tipo-I
    let rd = ((instrucao >> 7) & 0x1F) as usize;
    let funct3 = (instrucao >> 12) & 0x7;
    let rs1 = ((instrucao >> 15) & 0x1F) as usize;

    // Extrai imediato de 12 bits com extensão de sinal
    let imediato = (instrucao as i32) >> 20;
    // Extrai campos usados nos deslocamentos
    let funct7 = (instrucao >> 25) & 0x7F;
    let shamt = (instrucao >> 20) & 0x1F;
    
    // Busca o valor do registrador de origem
    let a = self.registradores[rs1];

    // Identifica e executa a operação
    let resultado = match funct3 {
        // Soma com imediato
        0b000 => Some(a.wrapping_add(imediato as u32)), // ADDI
        // Comparação com sinal
        0b010 => Some(((a as i32) < imediato) as u32), // SLTI
        // Comparação sem sinal
        0b011 => Some((a < imediato as u32) as u32), // SLTIU
        // XOR com imediato
        0b100 => Some(a ^ (imediato as u32)), // XORI
        // OR com imediato
        0b110 => Some(a | (imediato as u32)), // ORI
        // AND com imediato
        0b111 => Some(a & (imediato as u32)), // ANDI
        // Deslocamento lógico para esquerda
        0b001 if funct7 == 0b0000000 => {
            Some(a << shamt) // SLLI
        }

        // Deslocamento lógico para direita
        0b101 if funct7 == 0b0000000 => {
            Some(a >> shamt) // SRLI
        }

        // Deslocamento aritmético para direita
        0b101 if funct7 == 0b0100000 => {
            Some(((a as i32) >> shamt) as u32) // SRAI
        }

        // Instrução não reconhecida
        _ => None,
    };

    // Salva o resultado no registrador de destino
    // O registrador x0 permanece sempre zero
    match resultado {
        Some(valor) if rd != 0 => {
            self.registradores[rd] = valor;
        }
        Some(_) => {}
        None => {
            eprintln!("Instrucao Tipo-I invalida");
        }
    }
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
