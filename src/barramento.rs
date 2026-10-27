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
}
