pub struct Memory {
    pub ram: Vec<u8>,
}

impl Memory {
    pub fn new(tamanho: usize) -> Self {
        Memory {
            ram: vec![0; tamanho], 
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
    pub fn escrever_u32(&mut self, endereco: usize, valor: u32) {
        let bytes = valor.to_le_bytes();
        self.ram[endereco] = bytes[0];
        self.ram[endereco + 1] = bytes[1];
        self.ram[endereco + 2] = bytes[2];
        self.ram[endereco + 3] = bytes[3];
    }
}
