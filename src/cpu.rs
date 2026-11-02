use crate::barramento::Barramento;

// Operações aritméticas e lógicas suportadas pela ULA
#[derive(Clone, Copy)]
enum OperacaoUla {
    Soma,
    Subtracao,
    DeslocamentoEsquerda,
    MenorComSinal,
    MenorSemSinal,
    Xor,
    DeslocamentoDireitaLogico,
    DeslocamentoDireitaAritmetico,
    Or,
    And,
}

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

    // Executa o cálculo matemático ou lógico na ULA com dois operandos de 32 bits
    fn executar_ula(operacao: OperacaoUla, a: u32, b: u32) -> u32 {
        // No RV32I, deslocamentos utilizam apenas os 5 bits menos significativos
        let deslocamento = b & 0x1F;

        match operacao {
            OperacaoUla::Soma => a.wrapping_add(b),
            OperacaoUla::Subtracao => a.wrapping_sub(b),
            OperacaoUla::DeslocamentoEsquerda => a << deslocamento,
            OperacaoUla::MenorComSinal => ((a as i32) < (b as i32)) as u32,
            OperacaoUla::MenorSemSinal => (a < b) as u32,
            OperacaoUla::Xor => a ^ b,
            OperacaoUla::DeslocamentoDireitaLogico => a >> deslocamento,
            OperacaoUla::DeslocamentoDireitaAritmetico => ((a as i32) >> deslocamento) as u32,
            OperacaoUla::Or => a | b,
            OperacaoUla::And => a & b,
        }
    }

    // Instruções aritméticas e lógicas Tipo-R e Tipo-I

    pub fn executar_tipo_r(&mut self, instrucao: u32) {
        // Extração dos campos da instrução Tipo-R
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let funct3 = (instrucao >> 12) & 0x7;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;
        let rs2 = ((instrucao >> 20) & 0x1F) as usize;
        let funct7 = (instrucao >> 25) & 0x7F;

        // Define a operação da ULA combinando funct3 e funct7
        let operacao = match (funct3, funct7) {
            (0b000, 0b0000000) => Some(OperacaoUla::Soma),                          // ADD
            (0b000, 0b0100000) => Some(OperacaoUla::Subtracao),                     // SUB
            (0b001, 0b0000000) => Some(OperacaoUla::DeslocamentoEsquerda),          // SLL
            (0b010, 0b0000000) => Some(OperacaoUla::MenorComSinal),                 // SLT
            (0b011, 0b0000000) => Some(OperacaoUla::MenorSemSinal),                // SLTU
            (0b100, 0b0000000) => Some(OperacaoUla::Xor),                           // XOR
            (0b101, 0b0000000) => Some(OperacaoUla::DeslocamentoDireitaLogico),     // SRL
            (0b101, 0b0100000) => Some(OperacaoUla::DeslocamentoDireitaAritmetico), // SRA
            (0b110, 0b0000000) => Some(OperacaoUla::Or),                            // OR
            (0b111, 0b0000000) => Some(OperacaoUla::And),                           // AND
            _ => None,
        };

        if let Some(operacao) = operacao {
            let a = self.registradores[rs1];
            let b = self.registradores[rs2];
            let resultado = Self::executar_ula(operacao, a, b);

            // Garante que o registrador zero (x0) permaneça sempre 0
            if rd != 0 {
                self.registradores[rd] = resultado;
            }
        } else {
            eprintln!("Instrucao Tipo-R invalida: {:#010X}", instrucao);
        }
    }

    pub fn executar_tipo_i_aritmetico(&mut self, instrucao: u32) {
        // Extração dos campos da instrução Tipo-I
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let funct3 = (instrucao >> 12) & 0x7;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;

        // Imediato de 12 bits com extensão de sinal automática via cast para i32
        let imediato = (instrucao as i32) >> 20;

        // Campos específicos para instruções de deslocamento imediato (shifts)
        let funct7 = (instrucao >> 25) & 0x7F;
        let shamt = (instrucao >> 20) & 0x1F;

        let operacao = match funct3 {
            0b000 => Some(OperacaoUla::Soma),          // ADDI
            0b010 => Some(OperacaoUla::MenorComSinal), // SLTI
            0b011 => Some(OperacaoUla::MenorSemSinal), // SLTIU
            0b100 => Some(OperacaoUla::Xor),           // XORI
            0b110 => Some(OperacaoUla::Or),            // ORI
            0b111 => Some(OperacaoUla::And),           // ANDI
            0b001 if funct7 == 0b0000000 => Some(OperacaoUla::DeslocamentoEsquerda),          // SLLI
            0b101 if funct7 == 0b0000000 => Some(OperacaoUla::DeslocamentoDireitaLogico),     // SRLI
            0b101 if funct7 == 0b0100000 => Some(OperacaoUla::DeslocamentoDireitaAritmetico), // SRAI
            _ => None,
        };

        if let Some(operacao) = operacao {
            let a = self.registradores[rs1];

            // Operações de shift usam os 5 bits do shamt; as demais usam o imediato estendido
            let b = match operacao {
                OperacaoUla::DeslocamentoEsquerda
                | OperacaoUla::DeslocamentoDireitaLogico
                | OperacaoUla::DeslocamentoDireitaAritmetico => shamt,
                _ => imediato as u32,
            };

            let resultado = Self::executar_ula(operacao, a, b);

            if rd != 0 {
                self.registradores[rd] = resultado;
            }
        } else {
            eprintln!("Instrucao Tipo-I aritmetica invalida: {:#010X}", instrucao);
        }
    }

    // Instruções de desvio condicional Tipo-B

    pub fn executar_branch(&mut self, instrucao: u32) {
        // Extração dos registradores e do funct3
        let funct3 = (instrucao >> 12) & 0x7;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;
        let rs2 = ((instrucao >> 20) & 0x1F) as usize;

        // Reconstrução do imediato de 13 bits (vêm embaralhados nos bits da instrução Tipo-B)
        let bit_11 = (instrucao >> 7) & 0x1;
        let bits_4_1 = (instrucao >> 8) & 0xF;
        let bits_10_5 = (instrucao >> 25) & 0x3F;
        let bit_12 = (instrucao >> 31) & 0x1;

        let mut imm = (bit_12 << 12) | (bit_11 << 11) | (bits_10_5 << 5) | (bits_4_1 << 1);

        // Extensão de sinal manual baseada no bit 12
        if (imm & 0x1000) != 0 {
            imm |= 0xFFFF_E000;
        }

        let val1 = self.registradores[rs1];
        let val2 = self.registradores[rs2];

        // Verifica se a condição de salto é atendida
        let desviar = match funct3 {
            0b000 => val1 == val2,                   // BEQ (Igual)
            0b001 => val1 != val2,                   // BNE (Diferente)
            0b100 => (val1 as i32) < (val2 as i32),  // BLT (Menor com sinal)
            0b101 => (val1 as i32) >= (val2 as i32), // BGE (Maior ou igual com sinal)
            0b110 => val1 < val2,                    // BLTU (Menor sem sinal)
            0b111 => val1 >= val2,                   // BGEU (Maior ou igual sem sinal)
            _ => {
                eprintln!("Instrucao de branch invalida: {:#010X}", instrucao);
                false
            }
        };

        if desviar {
            // Desconta os 4 bytes já somados no ciclo de busca para calcular o salto relativo ao endereço da instrução
            let pc_instrucao = self.pc.wrapping_sub(4);
            self.pc = pc_instrucao.wrapping_add(imm);
        }
    }

    // Instruções de acesso à memória Load, Store e customizada LDINC

    pub fn executar_load(&mut self, instrucao: u32, barramento: &mut Barramento) {
        // Extração dos campos do Tipo-I (LW)
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;

        // Imediato de 12 bits com extensão de sinal (converte pra i32 e empurra os bits)
        let imm = (instrucao as i32) >> 20;

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
            imm |= 0xFFFF_F000;
        }

        let endereco = self.registradores[rs1].wrapping_add(imm) as usize;
        let valor_para_salvar = self.registradores[rs2];

        // Grava o valor na memória através do barramento
        barramento.escrever_u32(endereco, valor_para_salvar);
        println!("Gravado {} no endereço {}", valor_para_salvar, endereco);
    }

    pub fn executar_ldinc(&mut self, instrucao: u32, barramento: &mut Barramento) {
        // Extrai os registradores da instrução customizada LDINC
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;

        let endereco = self.registradores[rs1] as usize;

        // Carrega o dado da memória no registrador de destino (rd = MEM[rs1])
        if rd != 0 {
            self.registradores[rd] = barramento.ler_u32(endereco);
        }

        // Atualiza o ponteiro base somando 4 bytes (rs1 = rs1 + 4)
        if rs1 != 0 {
            self.registradores[rs1] = self.registradores[rs1].wrapping_add(4);
        }
    }
}

#[cfg(test)]
mod testes {
    use super::CPU;
    use crate::barramento::Barramento;
    use crate::memoria::Memory;

    // Funções auxiliares para montar instruções de máquina nos testes
    fn codificar_tipo_r(funct7: u32, funct3: u32, rs1: u32, rs2: u32, rd: u32) -> u32 {
        (funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | 0x33
    }

    fn codificar_tipo_i(imm: i32, funct3: u32, rs1: u32, rd: u32) -> u32 {
        (((imm as u32) & 0xFFF) << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | 0x13
    }

    fn codificar_branch(imm: i32, funct3: u32, rs1: u32, rs2: u32) -> u32 {
        let imm_u32 = (imm as u32) & 0x1FFE;
        let bit_12 = (imm_u32 >> 12) & 0x1;
        let bit_11 = (imm_u32 >> 11) & 0x1;
        let bits_10_5 = (imm_u32 >> 5) & 0x3F;
        let bits_4_1 = (imm_u32 >> 1) & 0xF;

        (bit_12 << 31)
            | (bits_10_5 << 25)
            | (rs2 << 20)
            | (rs1 << 15)
            | (funct3 << 12)
            | (bits_4_1 << 8)
            | (bit_11 << 7)
            | 0x63
    }

    fn codificar_lw(imm: i32, rs1: u32, rd: u32) -> u32 {
        (((imm as u32) & 0xFFF) << 20) | (rs1 << 15) | (0b010 << 12) | (rd << 7) | 0x03
    }

    fn codificar_sw(imm: i32, rs1: u32, rs2: u32) -> u32 {
        let imm_u32 = (imm as u32) & 0xFFF;
        let imm_11_5 = (imm_u32 >> 5) & 0x7F;
        let imm_4_0 = imm_u32 & 0x1F;

        (imm_11_5 << 25)
            | (rs2 << 20)
            | (rs1 << 15)
            | (0b010 << 12)
            | (imm_4_0 << 7)
            | 0x23
    }

    fn codificar_ldinc(rs1: u32, rd: u32) -> u32 {
        (rs1 << 15) | (rd << 7) | 0x0B
    }

    // Testes unitários das instruções Tipo-R e Tipo-I

    #[test]
    fn testar_add_e_addi() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = 10;
        cpu.registradores[2] = 20;

        let add = codificar_tipo_r(0, 0b000, 1, 2, 3);
        cpu.executar_tipo_r(add);
        assert_eq!(cpu.registradores[3], 30);

        let addi = codificar_tipo_i(5, 0b000, 3, 4);
        cpu.executar_tipo_i_aritmetico(addi);
        assert_eq!(cpu.registradores[4], 35);
    }

    #[test]
    fn testar_sub() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = 30;
        cpu.registradores[2] = 10;

        let sub = codificar_tipo_r(0b0100000, 0b000, 1, 2, 3);
        cpu.executar_tipo_r(sub);
        assert_eq!(cpu.registradores[3], 20);
    }

    #[test]
    fn testar_xor_or_and() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = 0b1100;
        cpu.registradores[2] = 0b1010;

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b100, 1, 2, 3));
        assert_eq!(cpu.registradores[3], 6);

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b110, 1, 2, 4));
        assert_eq!(cpu.registradores[4], 14);

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b111, 1, 2, 5));
        assert_eq!(cpu.registradores[5], 8);
    }

    #[test]
    fn testar_tipo_r_deslocamentos_e_comparacoes() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = 0x8000_0000;
        cpu.registradores[2] = 33;

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b001, 1, 2, 3));
        assert_eq!(cpu.registradores[3], 0);

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b101, 1, 2, 4));
        assert_eq!(cpu.registradores[4], 0x4000_0000);

        cpu.executar_tipo_r(codificar_tipo_r(0b0100000, 0b101, 1, 2, 5));
        assert_eq!(cpu.registradores[5], 0xC000_0000);

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b010, 1, 2, 6));
        assert_eq!(cpu.registradores[6], 1);

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b011, 1, 2, 7));
        assert_eq!(cpu.registradores[7], 0);
    }

    #[test]
    fn testar_tipo_i_logicas_comparacoes_e_negativos() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = 0b1100;

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(0b1010, 0b100, 1, 3));
        assert_eq!(cpu.registradores[3], 6);

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(0b1010, 0b110, 1, 4));
        assert_eq!(cpu.registradores[4], 14);

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(0b1010, 0b111, 1, 5));
        assert_eq!(cpu.registradores[5], 8);

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(-5, 0b000, 1, 6));
        assert_eq!(cpu.registradores[6], 7);

        cpu.registradores[1] = u32::MAX;
        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(1, 0b010, 1, 7));
        assert_eq!(cpu.registradores[7], 1);

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(1, 0b011, 1, 8));
        assert_eq!(cpu.registradores[8], 0);

        cpu.registradores[1] = 0;
        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(-1, 0b011, 1, 9));
        assert_eq!(cpu.registradores[9], 1);
    }

    #[test]
    fn testar_tipo_i_deslocamentos() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = 0x8000_0000;

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(1, 0b001, 1, 3));
        assert_eq!(cpu.registradores[3], 0);

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(1, 0b101, 1, 4));
        assert_eq!(cpu.registradores[4], 0x4000_0000);

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(0x401, 0b101, 1, 5));
        assert_eq!(cpu.registradores[5], 0xC000_0000);
    }

    #[test]
    fn testar_x0_overflow_e_instrucoes_invalidas() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = u32::MAX;
        cpu.registradores[2] = 1;

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b000, 1, 2, 3));
        assert_eq!(cpu.registradores[3], 0);

        cpu.executar_tipo_r(codificar_tipo_r(0, 0b000, 1, 2, 0));
        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(10, 0b000, 1, 0));
        assert_eq!(cpu.registradores[0], 0);

        cpu.registradores[3] = 77;
        cpu.executar_tipo_r(codificar_tipo_r(0b1111111, 0b100, 1, 2, 3));
        assert_eq!(cpu.registradores[3], 77);

        cpu.executar_tipo_i_aritmetico(codificar_tipo_i(0x201, 0b001, 1, 3));
        assert_eq!(cpu.registradores[3], 77);
    }

    // Testes unitários das instruções de desvio Tipo-B

    #[test]
    fn testar_branches_beq_e_bne() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = 42;
        cpu.registradores[2] = 42;

        // Simula que o ciclo de busca leu a instrução no endereço 100 e avançou o PC para 104
        cpu.pc = 104;
        let beq_salto = codificar_branch(16, 0b000, 1, 2);
        cpu.executar_branch(beq_salto);
        assert_eq!(cpu.pc, 116); // 100 + 16 = 116

        // BNE não deve pular pois os valores são iguais
        cpu.pc = 104;
        let bne_sem_salto = codificar_branch(16, 0b001, 1, 2);
        cpu.executar_branch(bne_sem_salto);
        assert_eq!(cpu.pc, 104);
    }

    #[test]
    fn testar_branches_comparacoes_e_salto_negativo() {
        let mut cpu = CPU::new();
        cpu.registradores[1] = -10i32 as u32;
        cpu.registradores[2] = 5;

        // BLT com sinal (-10 < 5 é verdadeiro) e salto para trás (-12 bytes)
        cpu.pc = 204; // Instrução estava no endereço 200
        let blt_atras = codificar_branch(-12, 0b100, 1, 2);
        cpu.executar_branch(blt_atras);
        assert_eq!(cpu.pc, 188); // 200 - 12 = 188

        // BGEU sem sinal (-10 em u32 é 0xFFFFFFF6, que é maior que 5)
        cpu.pc = 204;
        let bgeu_frente = codificar_branch(8, 0b111, 1, 2);
        cpu.executar_branch(bgeu_frente);
        assert_eq!(cpu.pc, 208); // 200 + 8 = 208
    }

    // Testes unitários de memória, LDINC e ciclo de busca

    #[test]
    fn testar_store_e_load_com_offset_negativo() {
        let memoria = Memory::new(1024);
        let mut barramento = Barramento::new(memoria);
        let mut cpu = CPU::new();

        cpu.registradores[5] = 200; // Endereço base
        cpu.registradores[6] = 0xDEAD_BEEF; // Valor para gravar

        // Grava no endereço 200 - 4 = 196
        let sw = codificar_sw(-4, 5, 6);
        cpu.executar_store(sw, &mut barramento);
        assert_eq!(barramento.ler_u32(196), 0xDEAD_BEEF);

        // Lê de volta do endereço 196 para o registrador x7
        let lw = codificar_lw(-4, 5, 7);
        cpu.executar_load(lw, &mut barramento);
        assert_eq!(cpu.registradores[7], 0xDEAD_BEEF);
    }

    #[test]
    fn testar_instrucao_customizada_ldinc() {
        let memoria = Memory::new(1024);
        let mut barramento = Barramento::new(memoria);
        let mut cpu = CPU::new();

        barramento.escrever_u32(300, 777);
        cpu.registradores[10] = 300; // Ponteiro de endereço

        let ldinc = codificar_ldinc(10, 11);
        cpu.executar_ldinc(ldinc, &mut barramento);

        assert_eq!(cpu.registradores[11], 777); // Carregou o dado da memória
        assert_eq!(cpu.registradores[10], 304); // Incrementou o ponteiro em 4 bytes
    }

    #[test]
    fn testar_ciclo_completo_cpu() {
        let memoria = Memory::new(1024);
        let mut barramento = Barramento::new(memoria);
        let mut cpu = CPU::new();

        // Monta um pequeno programa na RAM a partir do endereço 0:
        // 0x00: ADDI x1, x0, 100  (x1 = 100)
        // 0x04: ADDI x2, x0, 55   (x2 = 55)
        // 0x08: SW   x2, 8(x1)    (MEM[108] = 55)
        // 0x0C: LW   x3, 8(x1)    (x3 = MEM[108])
        barramento.escrever_u32(0, codificar_tipo_i(100, 0b000, 0, 1));
        barramento.escrever_u32(4, codificar_tipo_i(55, 0b000, 0, 2));
        barramento.escrever_u32(8, codificar_sw(8, 1, 2));
        barramento.escrever_u32(12, codificar_lw(8, 1, 3));

        cpu.executar_ciclo(&mut barramento);
        cpu.executar_ciclo(&mut barramento);
        cpu.executar_ciclo(&mut barramento);
        cpu.executar_ciclo(&mut barramento);

        assert_eq!(cpu.pc, 16);
        assert_eq!(cpu.registradores[1], 100);
        assert_eq!(cpu.registradores[2], 55);
        assert_eq!(cpu.registradores[3], 55);
    }
}