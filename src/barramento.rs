use crate::memoria::Memory;

pub struct Barramento {
    pub memoria: Memory,
}

impl Barramento {
    // O Barramento recebe a memória quando é criado
    pub fn new(memoria: Memory) -> Self {
        Barramento { memoria }
    }

    // O Barramento recebe o pedido de leitura da CPU
    pub fn ler_u32(&self, endereco: usize) -> u32 {

        
        self.memoria.ler_u32(endereco)
    }
    pub fn escrever_u32(&mut self, endereco: usize, valor: u32) {
        // Aqui, no futuro, você também vai poder rotear para a VRAM (0x80000)
        self.memoria.escrever_u32(endereco, valor);
    }
}
