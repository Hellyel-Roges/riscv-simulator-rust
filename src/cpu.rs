use crate::barramento::Barramento;

pub struct CPU {
    pub registradores: [u32; 32],
//  pub pc: u32,
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            registradores: [0; 32],
            // pc: 0,
        }
    }

    // A instrução (LDINC).
    pub fn executar_ldinc(&mut self, rd: usize, rs1: usize, barramento: &Barramento) {
        
        let endereco = self.registradores[rs1] as usize;
        
        if rd != 0 {
            // A CPU pede a leitura ao barramento, e não à memória!
            self.registradores[rd] = barramento.ler_u32(endereco);
        }

        // Incrementa simultaneamente o ponteiro base
        if rs1 != 0 {
            self.registradores[rs1] = self.registradores[rs1].wrapping_add(4); 
        }
    }
}
