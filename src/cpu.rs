
use crate::barramento::Barramento;

// Operacoes que a ULA consegue realizar
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
        // Busca a instrucao usando o PC atual
        let instrucao = barramento.ler_u32(self.pc as usize);

        // Incrementa para a proxima instrucao
        self.pc += 4;

        // Extrai o opcode
        let opcode = instrucao & 0x7F;

        match opcode {
            0x33 => self.executar_tipo_r(instrucao),
            0x13 => self.executar_tipo_i_aritmetico(instrucao),
            0x63 => self.executar_branch(instrucao),
            0x03 => self.executar_load(instrucao, barramento),
            0x23 => self.executar_store(instrucao, barramento),
            0x0B => self.executar_ldinc(instrucao, barramento),
            0x00 => println!("Fim da execucao (memoria zerada)."),
            _ => println!("Opcode desconhecido: {:#X}", opcode),
        }
    }

    // ==========================================
    // PARTE DA GRAZIELA - ULA
    // ==========================================

    // Recebe a operacao e dois operandos de 32 bits
    // Retorna o resultado do calculo
    fn executar_ula(operacao: OperacaoUla, a: u32, b: u32) -> u32 {
        // Usa somente os 5 bits inferiores nos deslocamentos
        let deslocamento = b & 0x1F;

        match operacao {
            OperacaoUla::Soma => a.wrapping_add(b),

            OperacaoUla::Subtracao => a.wrapping_sub(b),

            OperacaoUla::DeslocamentoEsquerda => a << deslocamento,

            OperacaoUla::MenorComSinal => {
                ((a as i32) < (b as i32)) as u32
            }

            OperacaoUla::MenorSemSinal => (a < b) as u32,

            OperacaoUla::Xor => a ^ b,

            OperacaoUla::DeslocamentoDireitaLogico => {
                a >> deslocamento
            }

            OperacaoUla::DeslocamentoDireitaAritmetico => {
                ((a as i32) >> deslocamento) as u32
            }

            OperacaoUla::Or => a | b,

            OperacaoUla::And => a & b,
        }
    }

    // ==========================================
    // PARTE DA GRAZIELA - INSTRUCOES TIPO-R
    // ==========================================

    pub fn executar_tipo_r(&mut self, instrucao: u32) {
        // Extrai os campos da instrucao Tipo-R
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let funct3 = (instrucao >> 12) & 0x7;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;
        let rs2 = ((instrucao >> 20) & 0x1F) as usize;
        let funct7 = (instrucao >> 25) & 0x7F;

        // Identifica a operacao pelo funct3 e funct7
        let operacao = match (funct3, funct7) {
            (0b000, 0b0000000) => Some(OperacaoUla::Soma), // ADD

            (0b000, 0b0100000) => Some(OperacaoUla::Subtracao), // SUB

            (0b001, 0b0000000) => {
                Some(OperacaoUla::DeslocamentoEsquerda) // SLL
            }

            (0b010, 0b0000000) => {
                Some(OperacaoUla::MenorComSinal) // SLT
            }

            (0b011, 0b0000000) => {
                Some(OperacaoUla::MenorSemSinal) // SLTU
            }

            (0b100, 0b0000000) => Some(OperacaoUla::Xor), // XOR

            (0b101, 0b0000000) => {
                Some(OperacaoUla::DeslocamentoDireitaLogico) // SRL
            }

            (0b101, 0b0100000) => {
                Some(OperacaoUla::DeslocamentoDireitaAritmetico) // SRA
            }

            (0b110, 0b0000000) => Some(OperacaoUla::Or), // OR

            (0b111, 0b0000000) => Some(OperacaoUla::And), // AND

            _ => None,
        };

        // Executa a operacao na ULA
        if let Some(operacao) = operacao {
            let a = self.registradores[rs1];
            let b = self.registradores[rs2];

            let resultado = Self::executar_ula(operacao, a, b);

            // O registrador x0 nao pode ser alterado
            if rd != 0 {
                self.registradores[rd] = resultado;
            }
        } else {
            eprintln!("Instrucao Tipo-R invalida: {:#010X}", instrucao);
        }
    }

    // ==========================================
    // PARTE DA GRAZIELA - INSTRUCOES TIPO-I
    // ==========================================

    pub fn executar_tipo_i_aritmetico(&mut self, instrucao: u32) {
        // Extrai os campos da instrucao Tipo-I
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let funct3 = (instrucao >> 12) & 0x7;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;

        // Imediato de 12 bits com extensao de sinal
        let imediato = (instrucao as i32) >> 20;

        // Campos usados nos deslocamentos
        let funct7 = (instrucao >> 25) & 0x7F;
        let shamt = (instrucao >> 20) & 0x1F;

        // Identifica a operacao
        let operacao = match funct3 {
            0b000 => Some(OperacaoUla::Soma), // ADDI

            0b010 => Some(OperacaoUla::MenorComSinal), // SLTI

            0b011 => Some(OperacaoUla::MenorSemSinal), // SLTIU

            0b100 => Some(OperacaoUla::Xor), // XORI

            0b110 => Some(OperacaoUla::Or), // ORI

            0b111 => Some(OperacaoUla::And), // ANDI

            0b001 if funct7 == 0b0000000 => {
                Some(OperacaoUla::DeslocamentoEsquerda) // SLLI
            }

            0b101 if funct7 == 0b0000000 => {
                Some(OperacaoUla::DeslocamentoDireitaLogico) // SRLI
            }

            0b101 if funct7 == 0b0100000 => {
                Some(OperacaoUla::DeslocamentoDireitaAritmetico) // SRAI
            }

            _ => None,
        };

        // Executa a operacao na ULA
        if let Some(operacao) = operacao {
            let a = self.registradores[rs1];

            // Deslocamentos usam shamt
            // Demais operacoes usam o imediato
            let b = match operacao {
                OperacaoUla::DeslocamentoEsquerda
                | OperacaoUla::DeslocamentoDireitaLogico
                | OperacaoUla::DeslocamentoDireitaAritmetico => shamt,

                _ => imediato as u32,
            };

            let resultado = Self::executar_ula(operacao, a, b);

            // O registrador x0 nao pode ser alterado
            if rd != 0 {
                self.registradores[rd] = resultado;
            }
        } else {
            eprintln!(
                "Instrucao Tipo-I aritmetica invalida: {:#010X}",
                instrucao
            );
        }
    }

    // ==========================================
    // PARTE DO GUSTAVO - BRANCH
    // ==========================================

    pub fn executar_branch(&mut self, _instrucao: u32) {
        // Gustavo, aqui voce extrai os imediatos fracionados,
        // rs1, rs2 e funct3.
        // Se a condicao for verdadeira, voce altera o self.pc.
        // Cuidado ao juntar os pedacos do imediato Tipo-B.
    }

    // ==========================================
    // PARTE DO HELLYEL - LOAD, STORE E LDINC
    // ==========================================

    pub fn executar_load(
        &mut self,
        instrucao: u32,
        barramento: &mut Barramento
    ) {
        // Extracao dos campos do Tipo-I (LW)
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;

        // Imediato de 12 bits com extensao de sinal
        let imm = (instrucao as i32) >> 20;

        // Calcula o endereco base
        let endereco = self.registradores[rs1]
            .wrapping_add(imm as u32) as usize;

        // rd = MEM[rs1 + imm]
        if rd != 0 {
            self.registradores[rd] = barramento.ler_u32(endereco);
        }
    }

    pub fn executar_store(
        &mut self,
        instrucao: u32,
        barramento: &mut Barramento
    ) {
        // Extracao dos campos do Tipo-S (SW)
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;
        let rs2 = ((instrucao >> 20) & 0x1F) as usize;

        // Imediato dividido em duas partes
        let imm_11_5 = (instrucao >> 25) & 0x7F;
        let imm_4_0 = (instrucao >> 7) & 0x1F;

        let mut imm = (imm_11_5 << 5) | imm_4_0;

        // Extensao de sinal
        if (imm & 0x800) != 0 {
            imm |= 0xFFFFF000;
        }

        let endereco = self.registradores[rs1]
            .wrapping_add(imm) as usize;

        let valor_para_salvar = self.registradores[rs2];

        // Grava o valor na memoria atraves do barramento
        barramento.escrever_u32(endereco, valor_para_salvar);

        println!(
            "Gravado {} no endereco {}",
            valor_para_salvar,
            endereco
        );
    }

    pub fn executar_ldinc(
        &mut self,
        instrucao: u32,
        barramento: &mut Barramento
    ) {
        // Extrai os parametros da instrucao LDINC
        let rd = ((instrucao >> 7) & 0x1F) as usize;
        let rs1 = ((instrucao >> 15) & 0x1F) as usize;

        let endereco = self.registradores[rs1] as usize;

        // Passo 1: rd = MEM[rs1]
        if rd != 0 {
            self.registradores[rd] = barramento.ler_u32(endereco);
        }

        // Passo 2: rs1 = rs1 + 4
        if rs1 != 0 {
            self.registradores[rs1] =
                self.registradores[rs1].wrapping_add(4);
        }
    }
}

// ==========================================
// TESTES UNITARIOS - PARTE DA GRAZIELA
// ==========================================

#[cfg(test)]
mod testes {
    use super::CPU;

    // Funcoes auxiliares para montar instrucoes de 32 bits
    fn codificar_tipo_r(
        funct7: u32,
        funct3: u32,
        rs1: u32,
        rs2: u32,
        rd: u32
    ) -> u32 {
        (funct7 << 25)
            | (rs2 << 20)
            | (rs1 << 15)
            | (funct3 << 12)
            | (rd << 7)
            | 0x33
    }

    fn codificar_tipo_i(
        imm: i32,
        funct3: u32,
        rs1: u32,
        rd: u32
    ) -> u32 {
        (((imm as u32) & 0xFFF) << 20)
            | (rs1 << 15)
            | (funct3 << 12)
            | (rd << 7)
            | 0x13
    }

    // TESTE 1 - ADD E ADDI
    #[test]
    fn testar_add_e_addi() {
        let mut cpu = CPU::new();

        cpu.registradores[1] = 10;
        cpu.registradores[2] = 20;

        // ADD: 10 + 20 = 30
        let add = codificar_tipo_r(0, 0b000, 1, 2, 3);

        cpu.executar_tipo_r(add);
        assert_eq!(cpu.registradores[3], 30);

        // ADDI: 30 + 5 = 35
        let addi = codificar_tipo_i(5, 0b000, 3, 4);

        cpu.executar_tipo_i_aritmetico(addi);
        assert_eq!(cpu.registradores[4], 35);
    }

    // TESTE 2 - SUB
    #[test]
    fn testar_sub() {
        let mut cpu = CPU::new();

        cpu.registradores[1] = 30;
        cpu.registradores[2] = 10;

        // SUB: 30 - 10 = 20
        let sub = codificar_tipo_r(
            0b0100000, 0b000, 1, 2, 3
        );

        cpu.executar_tipo_r(sub);
        assert_eq!(cpu.registradores[3], 20);
    }

    // TESTE 3 - XOR, OR E AND
    #[test]
    fn testar_xor_or_and() {
        let mut cpu = CPU::new();

        cpu.registradores[1] = 0b1100;
        cpu.registradores[2] = 0b1010;

        // XOR
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b100, 1, 2, 3)
        );
        assert_eq!(cpu.registradores[3], 6);

        // OR
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b110, 1, 2, 4)
        );
        assert_eq!(cpu.registradores[4], 14);

        // AND
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b111, 1, 2, 5)
        );
        assert_eq!(cpu.registradores[5], 8);
    }

    // TESTE 4 - DESLOCAMENTOS E COMPARACOES TIPO-R
    #[test]
    fn testar_tipo_r_deslocamentos_e_comparacoes() {
        let mut cpu = CPU::new();

        cpu.registradores[1] = 0x8000_0000;
        cpu.registradores[2] = 33;

        // SLL
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b001, 1, 2, 3)
        );
        assert_eq!(cpu.registradores[3], 0);

        // SRL
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b101, 1, 2, 4)
        );
        assert_eq!(cpu.registradores[4], 0x4000_0000);

        // SRA
        cpu.executar_tipo_r(
            codificar_tipo_r(0b0100000, 0b101, 1, 2, 5)
        );
        assert_eq!(cpu.registradores[5], 0xC000_0000);

        // SLT com sinal
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b010, 1, 2, 6)
        );
        assert_eq!(cpu.registradores[6], 1);

        // SLTU sem sinal
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b011, 1, 2, 7)
        );
        assert_eq!(cpu.registradores[7], 0);
    }

    // TESTE 5 - OPERACOES TIPO-I E NEGATIVOS
    #[test]
    fn testar_tipo_i_logicas_comparacoes_e_negativos() {
        let mut cpu = CPU::new();

        cpu.registradores[1] = 0b1100;

        // XORI
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(0b1010, 0b100, 1, 3)
        );
        assert_eq!(cpu.registradores[3], 6);

        // ORI
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(0b1010, 0b110, 1, 4)
        );
        assert_eq!(cpu.registradores[4], 14);

        // ANDI
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(0b1010, 0b111, 1, 5)
        );
        assert_eq!(cpu.registradores[5], 8);

        // ADDI com numero negativo
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(-5, 0b000, 1, 6)
        );
        assert_eq!(cpu.registradores[6], 7);

        cpu.registradores[1] = u32::MAX;

        // SLTI
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(1, 0b010, 1, 7)
        );
        assert_eq!(cpu.registradores[7], 1);

        // SLTIU
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(1, 0b011, 1, 8)
        );
        assert_eq!(cpu.registradores[8], 0);

        // SLTIU com imediato negativo
        cpu.registradores[1] = 0;

        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(-1, 0b011, 1, 9)
        );
        assert_eq!(cpu.registradores[9], 1);
    }

    // TESTE 6 - DESLOCAMENTOS TIPO-I
    #[test]
    fn testar_tipo_i_deslocamentos() {
        let mut cpu = CPU::new();

        cpu.registradores[1] = 0x8000_0000;

        // SLLI
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(1, 0b001, 1, 3)
        );
        assert_eq!(cpu.registradores[3], 0);

        // SRLI
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(1, 0b101, 1, 4)
        );
        assert_eq!(cpu.registradores[4], 0x4000_0000);

        // SRAI
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(0x401, 0b101, 1, 5)
        );
        assert_eq!(cpu.registradores[5], 0xC000_0000);
    }

    // TESTE 7 - REGISTRADOR ZERO, OVERFLOW E INVALIDAS
    #[test]
    fn testar_x0_overflow_e_instrucoes_invalidas() {
        let mut cpu = CPU::new();

        cpu.registradores[1] = u32::MAX;
        cpu.registradores[2] = 1;

        // ADD com overflow
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b000, 1, 2, 3)
        );
        assert_eq!(cpu.registradores[3], 0);

        // Nao pode escrever no registrador x0
        cpu.executar_tipo_r(
            codificar_tipo_r(0, 0b000, 1, 2, 0)
        );

        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(10, 0b000, 1, 0)
        );

        assert_eq!(cpu.registradores[0], 0);

        // Instrucao Tipo-R invalida
        cpu.registradores[3] = 77;

        cpu.executar_tipo_r(
            codificar_tipo_r(0b1111111, 0b100, 1, 2, 3)
        );

        assert_eq!(cpu.registradores[3], 77);

        // Instrucao Tipo-I invalida
        cpu.executar_tipo_i_aritmetico(
            codificar_tipo_i(0x201, 0b001, 1, 3)
        );

        assert_eq!(cpu.registradores[3], 77);
    }
}
