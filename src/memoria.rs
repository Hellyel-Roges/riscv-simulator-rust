pub struct Memory {
    // 0x9FFFF (655359) é o último endereço válido. Portanto, a RAM tem 655.360 posições (640 KB).
    pub ram: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        // Aloca exatamente o limite especificado no mapeamento de memória.
        let tamanho_total = 0xA0000;

        Memory {
            ram: vec![0; tamanho_total],
        }
    }

    // Leitura simulada de 32 bits (4 bytes)
    pub fn ler_u32(&self, endereco: usize) -> u32 {
        if endereco + 3 < self.ram.len() {
            u32::from_le_bytes([
                self.ram[endereco],
                self.ram[endereco + 1],
                self.ram[endereco + 2],
                self.ram[endereco + 3],
            ])
        } else {
            0
        }
    }

    // Escrita simulada de 32 bits (4 bytes)
    pub fn escrever_u32(&mut self, endereco: usize, valor: u32) {
        // Proteção física de escrita
        if endereco + 3 < self.ram.len() {
            let bytes = valor.to_le_bytes();
            self.ram[endereco] = bytes[0];
            self.ram[endereco + 1] = bytes[1];
            self.ram[endereco + 2] = bytes[2];
            self.ram[endereco + 3] = bytes[3];
        }
    }
}
