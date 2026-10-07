# Simulador RISC-V (RV32I) em Rust 🦀

Este repositório contém o código-fonte de um Simulador de Computador baseado na arquitetura RISC-V (ISA RV32I). Este é o projeto final da disciplina de Organização de Computadores da UniSantos.

## 🎯 Objetivo do Projeto
Desenvolver um simulador funcional capaz de carregar e executar instruções de máquina do RISC-V. O simulador não é apenas um emulador de CPU, mas a representação de um sistema completo, englobando processador, memória e barramento de dados.

## 🏗️ Estrutura e Componentes
O simulador é dividido nos seguintes módulos principais:

* **CPU:** Implementação do ciclo de busca, decodificação e execução. Suporte para registradores de 32 bits e instruções lógicas, aritméticas, de desvio e de acesso à memória.
* **Memória RAM:** Mapeamento de faixas de endereços para a RAM principal (0x00000 - 0x7FFFF), VRAM (0x80000 - 0x8FFFF) e periféricos.
* **Barramento:** Lógica de interligação de 32 bits (dados, endereços e controle) entre a CPU e a memória.
* **Entrada/Saída (E/S):** Controle de E/S programada para renderizar os dados da VRAM no terminal.

## 🚀 Instrução Customizada (A Definir)
**Grupo:**
* GRAZIELA CRISTINA SOARES ANTIORIO
* GUSTAVO FREITAS SAMPAIO
* HELLYEL ROGES DOS PASSOS AMBROZIO PEREIRA 

**Instrução Especial:** 
* [AINDA SERÁ DECIDIDA]

## 🛠️ Tecnologias Utilizadas
* **Linguagem:** [Rust](https://www.rust-lang.org/)
* **Gerenciador de pacotes:** Cargo

## 🏃 Como rodar o projeto
Certifique-se de ter o Rust e o Cargo instalados na sua máquina.

1. Clone o repositório:
```bash
git clone https://github.com/SEU_USUARIO/NOME_DO_REPOSITORIO.git
```

2. Entre no diretório do projeto:
```bash
cd NOME_DO_REPOSITORIO
```

3. Compile e execute o simulador:
```bash
cargo run
```

---
*Projeto desenvolvido para a disciplina de Organização de Computadores - Universidade Católica de Santos (UniSantos).*
