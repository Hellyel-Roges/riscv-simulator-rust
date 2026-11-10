
use crate::memoria::Memory;

pub struct Barramento {
    pub memoria: Memory,
}

impl Barramento {
    pub fn new(memoria: Memory) -> Self {
        Barramento { memoria }
    }
    
    // "ajusta as posições de memória para leitura/escrita"
    fn validar_endereco(&self, endereco: usize) -> usize {
        // Usa operação bit-a-bit (Bitwise) para checar o alinhamento.
        // Se os 2 últimos bits do endereço forem 0 (addr & 0x3 == 0), ele é múltiplo de 4.
        if endereco & 0x3 == 0 {
            endereco
        } else {
            panic!(
                "ERRO DE ALINHAMENTO NO BARRAMENTO: O endereço solicitado ({:#010X}) não é múltiplo de 4 (Word Boundary).",
                endereco
            );
        }
    }

    // "permitir o fluxo de entrada/saída dos dados"
    fn formatar_dado_in(&self, dado: u32) -> u32 {
        // Fluxo de dados da CPU para a RAM

        // Aplica a mascara de 32 bits
        let dado_formatado = dado & 0xFFFF_FFFF;

        // Log da transferencia de dados
        println!(
            "Barramento de Dados [CPU -> RAM]: Trafegando valor {} ({:#010X})",
            dado_formatado,
            dado_formatado
        );

        // Retorna o valor tratado
        dado_formatado
    }

    fn formatar_dado_out(&self, dado: u32) -> u32 {
        // Fluxo de dados da RAM para a CPU

        // Aplica a mascara de 32 bits
        let dado_formatado = dado & 0xFFFF_FFFF;

        // Log da transferencia de dados
        println!(
            "Barramento de Dados [RAM -> CPU]: Trafegando valor {} ({:#010X})",
            dado_formatado,
            dado_formatado
        );

        // Retorna o valor tratado
        dado_formatado
    }

    // "ajusta a permissão de leitura/escrita dos blocos"
    fn verificar_controle(&self, endereco: usize, is_write: bool) {
        // Mapeamento oficial de memória do simulador:
        // 0x00000 - 0x7FFFF: RAM Principal
        // 0x80000 - 0x8FFFF: VRAM (Vídeo RAM)
        // 0x90000 - 0x9FBFF: Área Reservada
        // 0x9FC00 - 0x9FFFF: Periféricos de Hardware

        // Limite físico do vetor da memória instanciada
        if endereco >= self.memoria.ram.len() {
            panic!(
                "ERRO DE CONTROLE: Hardware fault. Tentativa de acesso físico fora da RAM instanciada. Endereço: {:#07X}",
                endereco
            );
        }

        // Limite lógico da arquitetura do processador (acima de 0x9FFFF não existe mapa)
        if endereco > 0x9FFFF {
            panic!(
                "ERRO DE CONTROLE: Bus Error. Endereço {:#07X} ultrapassa o mapeamento máximo de periféricos (0x9FFFF).",
                endereco
            );
        }

        // Bloqueio da Área Reservada (SegFault)
        // O barramento intercepta e não permite leitura nem escrita no espaço de expansão futura.
        if endereco >= 0x90000 && endereco <= 0x9FBFF {
            let operacao = if is_write { "escrita" } else { "leitura" };
            panic!(
                "ERRO DE CONTROLE: Violação de permissão. Tentativa de {} na Área Reservada (0x90000 a 0x9FBFF). Endereço bloqueado: {:#07X}",
                operacao, endereco
            );
        }
    }

    // FLUXO PRINCIPAL DE LEITURA E ESCRITA

    pub fn ler_u32(&self, endereco: usize) -> u32 {
        // sa pelo pedágio de Endereço (Gustavo)
        let end_valido = self.validar_endereco(endereco);
        
        // Passa pelo pedágio de Controle (Hellyel) - is_write = false
        self.verificar_controle(end_valido, false);

        // Lê o dado real da memória
        let dado_bruto = self.memoria.ler_u32(end_valido);

        // Passa pelo pedágio de Dados (Graziela) antes de devolver pra CPU
        self.formatar_dado_out(dado_bruto)
    }

    pub fn escrever_u32(&mut self, endereco: usize, valor: u32) {
        // Passa pelo pedágio de Endereço (Gustavo)
        let end_valido = self.validar_endereco(endereco);
        
        // Passa pelo pedágio de Controle (Hellyel) - is_write = true
        self.verificar_controle(end_valido, true);

        // Passa pelo pedágio de Dados (Graziela) antes de ir pra memória
        let dado_tratado = self.formatar_dado_in(valor);

        // Grava efetivamente na memória
        self.memoria.escrever_u32(end_valido, dado_tratado);
    }
}

// ==========================================
// TESTES UNITÁRIOS DO BARRAMENTO
// ==========================================

#[cfg(test)]
mod testes {
    use super::*;
    use crate::memoria::Memory;

    #[test]
    fn testar_acesso_permitido_ram_principal() {
        let memoria = Memory::new(1024 * 1024); // 1 MB
        let mut barramento = Barramento::new(memoria);
        
        // Acesso normal, não deve dar erro
        barramento.escrever_u32(0x100, 42);
        assert_eq!(barramento.ler_u32(0x100), 42);
    }

    #[test]
    #[should_panic(expected = "Violação de permissão")]
    fn testar_bloqueio_area_reservada() {
        let memoria = Memory::new(1024 * 1024); // 1 MB
        let barramento = Barramento::new(memoria);
        
        // Tenta ler do endereço 0x90000 (Início da Área Reservada)
        // O teste vai passar justamente porque o barramento DEVE dar panic!
        barramento.ler_u32(0x90000);
    }

    #[test]
    #[should_panic(expected = "Hardware fault")]
    fn testar_bloqueio_fora_da_ram_fisica() {
        let memoria = Memory::new(1024); // RAM de apenas 1 KB
        let barramento = Barramento::new(memoria);
        
        // Tenta acessar o endereço 2000 num pente de RAM que só tem 1024 bytes
        barramento.ler_u32(2000);
    }
    #[test]
    #[should_panic(expected = "ALINHAMENTO")]
    fn testar_erro_de_alinhamento() {
        let memoria = Memory::new(1024); 
        let barramento = Barramento::new(memoria);
        
        // Tenta ler em um endereço quebrado (não múltiplo de 4)
        barramento.ler_u32(0x103); 
    }
}
