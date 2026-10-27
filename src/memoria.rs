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
}
