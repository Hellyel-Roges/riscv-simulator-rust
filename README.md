# Simulador RISC-V (RV32I) em Rust

Este repositório contém o código-fonte de um Simulador de Computador baseado na arquitetura RISC-V (ISA RV32I). Este é o projeto final da disciplina de Organização de Computadores da Universidade Católica de Santos (UniSantos).

## Objetivo do Projeto
Desenvolver um simulador funcional capaz de carregar e executar instruções de máquina do RISC-V. O simulador não é apenas um emulador de CPU, mas a representação de um sistema computacional completo, englobando processador, memória e barramento de dados.

## Estrutura e Componentes
O simulador adota uma arquitetura modular, dividida nos seguintes componentes principais:

* **CPU:** Implementação cycle-accurate do fluxo de busca, decodificação e execução. Possui suporte para os 32 registradores base e para o conjunto de instruções RV32I (lógicas, aritméticas, desvios e acesso à memória).
* **Memória RAM:** Mapeamento de memória estruturado em faixas de endereços, incluindo RAM principal (0x00000 - 0x7FFFF), VRAM (0x80000 - 0x8FFFF) e periféricos mapeados em memória (0x9FC00 - 0x9FFFF).
* **Barramento:** Interface de interligação de 32 bits responsável por rotear dados, endereços e sinais de controle entre a CPU e o subsistema de memória, garantindo o isolamento arquitetural.
* **Entrada/Saída (E/S):** Módulo de E/S programada projetado para processar e renderizar os dados da VRAM diretamente no terminal.

## Equipe e Instrução Customizada
**Grupo 5:**
* Graziela Cristina Soares Antiorio
* Gustavo Freitas Sampaio
* Hellyel Roges dos Passos Ambrozio Pereira

**Instrução Especial Exclusiva:**
* **LDINC (Load and Increment):** Instrução customizada que realiza a leitura da memória e a atualização do ponteiro no mesmo ciclo de instrução (`rd = MEM[rs1]` seguido de `rs1 = rs1 + 4`).

## Tecnologias Utilizadas
* **Linguagem:** [Rust](https://www.rust-lang.org/)
* **Interface (TUI):** `ratatui` e `crossterm` (para renderização visual interativa no terminal com alta performance)
* **Build System & Package Manager:** Cargo

## Como Executar o Projeto
Para rodar este simulador, certifique-se de ter o ambiente Rust instalado na sua máquina (via `rustup`).

1. Clone o repositório:
```bash
git clone [https://github.com/SEU_USUARIO/NOME_DO_REPOSITORIO.git](https://github.com/SEU_USUARIO/NOME_DO_REPOSITORIO.git)
```
```bash
cargo run --release
```
```bash
cargo run --release
```
