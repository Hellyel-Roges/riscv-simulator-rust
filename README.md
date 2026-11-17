# Simulador RISC-V (RV32I) em Rust

Este repositório contém o código-fonte de um Simulador de Computador baseado na arquitetura RISC-V (ISA RV32I), desenvolvido como projeto final da disciplina de Organização de Computadores da Universidade Católica de Santos (UniSantos).

## Informações Acadêmicas

* **Instituição:** Universidade Católica de Santos (UniSantos)
* **Curso:** Ciência da Computação
* **Disciplina:** Organização de Computadores
* **Professor Avaliador:** Walter S. Oliveira
* **Equipe Desenvolvedora (Grupo 5):**
  * Graziela Cristina Soares Antiorio
  * Gustavo Freitas Sampaio
  * Hellyel Roges dos Passos Ambrozio Pereira

## Arquitetura do Sistema

O simulador adota uma estrutura modular focada em precisão de ciclo (*cycle-accurate*), dividida nos seguintes componentes de hardware simulado:

* **CPU:** Implementa o fluxo de busca, decodificação e execução. Suporta o conjunto base de instruções RV32I (32 bits) com 32 registradores de propósito geral (sendo `x0` *hardwired* em zero), registrador de programa (`PC`) e registrador de contexto de exceção (`EPC`).
* **Barramento (Bus):** Roteador central de 32 bits. Valida o alinhamento de memória (*Word Boundary*), roteia acessos físicos e lógicos, e protege áreas restritas contra violações de segmento (*SegFault*).
* **Memória:** Mapeamento MMIO (Memory-Mapped I/O) contíguo totalizando 640 KB.
* **Interface (TUI):** E/S programada interativa no terminal, construída sobre `ratatui` e `crossterm`, realizando varredura e renderização visual do endereçamento da VRAM em tempo real.

## Mapeamento de Memória

A memória está dividida logicamente nas seguintes regiões arquiteturais:

| Faixa de Endereço (Hex) | Região | Descrição |
| --- | --- | --- |
| `0x00000` - `0x7FFFF` | RAM Principal | Armazena o código executável, dados gerais, chave criptográfica e o vetor de interrupção (em `0x7F000`). |
| `0x80000` - `0x8FFFF` | VRAM | Memória de Vídeo mapeada em tempo real pela Interface Gráfica. |
| `0x90000` - `0x9FBFF` | Área Reservada | Região bloqueada por controle de barramento. Qualquer acesso dispara interrupção de *hardware fault*. |
| `0x9FC00` - `0x9FFFF` | Periféricos | Área reservada para controladores de hardware genéricos. |

## Conjunto de Instruções (ISA Supported)

O núcleo suporta rigorosamente os formatos Tipo-R, Tipo-I, Tipo-S e Tipo-B, validando opcodes e códigos `funct3`/`funct7` contra execuções ilegais.

* **Aritméticas e Lógicas (Tipo-R):** `ADD`, `SUB`, `SLL`, `SLT`, `SLTU`, `XOR`, `SRL`, `SRA`, `OR`, `AND`.
* **Aritméticas e Lógicas com Imediato (Tipo-I):** `ADDI`, `SLTI`, `SLTIU`, `XORI`, `ORI`, `ANDI`, `SLLI`, `SRLI`, `SRAI`.
* **Acesso à Memória:** `LW` (Load Word) e `SW` (Store Word). Filtros implementados rejeitam extensões de byte/halfword não suportadas pelo escopo.
* **Desvio Condicional (Branch Tipo-B):** `BEQ`, `BNE`, `BLT`, `BGE`, `BLTU`, `BGEU`.
* **Controle:** `MRET` (Retorno de Exceção) e `HALT` (Convenção arquitetural associada à palavra nula `0x00000000`).

### Instrução Exclusiva (Grupo 5): LDINC
Instrução customizada de *Load and Increment* (Opcode `0x0B`). A instrução realiza `rd = MEM[rs1]` seguido de `rs1 = rs1 + 4` em um único ciclo de clock. A arquitetura exige obrigatoriamente que destino e ponteiro sejam distintos (`rd != rs1`), possuindo trava de prevenção contra *Data Hazards*.

## Recursos Avançados de Auditoria

Para validação plena dos requisitos sem corromper o fluxo da aplicação final, o sistema emprega técnicas avançadas no algoritmo de execução:

1. **Tratamento de Interrupções de Hardware (IRQ):** Permite disparos assíncronos via dispositivo de entrada (teclado). O desvio preserva o contexto em `EPC`, isola a rotina de tratamento para evitar corrupção dos registradores da aplicação base e retoma o fluxo de instrução originário.
2. **Motor de Descriptografia Simétrica:** O arquivo de renderização transferido para a RAM está sob encriptação (XOR). O programa Assembly simulado é forçado a atuar como um motor de descriptografia. Ao fazê-lo, exercita organicamente toda a Unidade Lógica e Aritmética (ULA) — incluindo comparadores (`SLT`), máscaras lógicas (`AND`, `OR`), operações com imediatos (`ADDI`) e instruções base de arquitetura customizada (`LDINC`) — validando todo o core RISC-V em ambiente real.

## Construção e Execução

Necessário possuir a *toolchain* da linguagem Rust devidamente configurada.

1. Clone o repositório:
```bash
git clone [https://github.com/Hellyel-Roges/riscv-simulator-rust.git](https://github.com/Hellyel-Roges/riscv-simulator-rust.git)

```

2. Acesse o diretório:

```bash
cd riscv-simulator-rust

```

3. (Opcional) Execute a suíte de testes unitários da arquitetura e barramento:

```bash
cargo test

```

4. Inicie o simulador interativo:

```bash
cargo run --release

```

## Controles da Interface (TUI)

| Comando | Descrição da Ação |
| --- | --- |
| `[UP/DOWN]` | Navega pelo seletor de arquivos fonte no Menu Inicial. |
| `[ENTER]` | Confirma seleção / Avança a CPU em um ciclo estrito (*Step Mode*). |
| `[ESPAÇO]` | Inicia ou pausa a execução do *Clock Automático*. |
| `[ I ]` | Injeta solicitação de Interrupção Externa (IRQ) no barramento. |
| `[ + / - ]` | Modifica o multiplicador de velocidade de execução de ciclos (Autoplay). |
| `[SETAS]` | Movimenta o eixo visual (*Scroll*) dentro do Monitor da VRAM. |
| `[ Q ]` | Aborta a execução atual e retorna ao Menu Inicial. |
| `[ ESC ]` | Finaliza o processo e encerra o emulador. |
