# UNIVERSIDADE \[NOME DA INSTITUIÇÃO\]

### CURSO DE CIÊNCIA DA COMPUTAÇÃO

### DISCIPLINA: LINGUAGENS FORMAIS E AUTÔMATOS (CO.N1.16)

### DOCENTE: JOSÉ FONTEBASSO NETO

**PROJETO DE PESQUISA CURRICULARIZADA:** Reconhecedor e Transdutor Léxico para Busca de Padrões em Sequências e Textos (Mascarador de Dados LGPD)

**GRUPO 03:**

* FELIPE MAFRA DOS SANTOS (3859450)  
* GRAZIELA CRISTINA SOARES ANTIORIO (1276217)  
* GUSTAVO FREITAS SAMPAIO (9574999)  
* HELLYEL ROGES DOS PASSOS AMBROZIO PEREIRA (2397495)  
* LUAN FELIX PEREIRA (3299543)

---

## 1\. Definição do Problema e Contexto de Aplicação

O projeto consiste no desenvolvimento de um sistema computacional de linha de comando capaz de atuar como um "Filtro de Privacidade" em arquivos de log ou textos brutos. O objetivo central é identificar e ofuscar (pseudonimizar) Dados Pessoais e Dados Pessoais Sensíveis, em conformidade com as diretrizes da Lei Geral de Proteção de Dados (LGPD).

O sistema processará cadeias de caracteres sequenciais, utilizando um Autômato Finito para reconhecer os padrões, e aplicará o conceito de Máquina de Mealy (Transdutor Finito) para mascarar os dados reconhecidos na saída. O texto lido que não corresponder a informações sensíveis será tratado como ruído comum e repassado integralmente para a saída.

Os dados a serem identificados incluem:

1. **Dados Pessoais Diretos:** Cadastro de Pessoa Física (CPF), E-mails corporativos/pessoais, Números de Telefone, Data de Nascimento e Sexo/Gênero.  
2. **Dados Pessoais Sensíveis (LGPD Art. 5º, II):** Palavras-chave em relatórios textuais que revelem:  
   * **Identidade e Crenças:** Origem racial ou étnica, convicção religiosa e opinião política.  
   * **Associações e Coletivos:** Filiação a sindicato, organização religiosa, filosófica ou política.  
   * **Corpo e Saúde:** Dados referentes à saúde, vida sexual, dados genéticos ou biométricos.

## 2\. Definição Formal dos Alfabetos

Para fins de modelagem formal do autômato e viabilidade da implementação na linguagem C/C++, o vocabulário (alfabeto) do sistema é delimitado da seguinte forma:

* **Vocabulário de Entrada (${V}_{in}$):** ${V}_{in}=L\cup D\cup S$  
  * $L=\{A,B,...,Z,a,b,...,z\}$ (Letras do alfabeto romano)  
  * $D=\{0,1,2,3,4,5,6,7,8,9\}$ (Dígitos arábicos)  
  * $S=\{.,-,@,/,(,),espaço,\_\}$ (Símbolos separadores adotados)  
* **Vocabulário de Saída (${\Gamma }_{out}$):** ${\Gamma }_{out}={V}_{in}\cup \{*\}$  
  * O caractere asterisco ($*$) será emitido exclusivamente pela função de saída da Máquina de Mealy para mascarar os dados sensíveis detectados.

A linguagem analisada será formada por sentenças $w\in {V}_{in}^{*}$.

## 3\. Especificação dos Padrões por Expressões Regulares (ER)

Abaixo estão as descrições algébricas que geram os subconjuntos de sentenças consideradas como falhas de privacidade. Utiliza-se a notação estendida de fecho positivo (⁺) e repetição (ⁿ) para simplificação formal.

* **Padrão 1: CPF**  
  * Formato esperado: `000.000.000-00`  
  * $E{R}_{cpf}={D}^{3}.{D}^{3}.{D}^{3}-{D}^{2}$  
* **Padrão 2: Telefone (Celular Padrão BR)**  
  * Formato esperado: `(00) 00000-0000`  
  * $E{R}_{tel}=({D}^{2})\ {D}^{5}-{D}^{4}$  
* **Padrão 3: Data de Nascimento**  
  * Formato esperado: `00/00/0000`  
  * $E{R}_{data}={D}^{2}/{D}^{2}/{D}^{4}$  
* **Padrão 4: E-mail (Simplificado)**  
  * Formato esperado: `identificador@dominio.let`  
  * $E{R}_{email}={L}^{+}@{L}^{+}.{L}^{+}$  
* **Padrão 5: Dados Sensíveis (Dicionário Finito)**  
  * Reconhecimento de palavras-chave literais que identificam dados protegidos pela LGPD.  
  * ER\_sensivel \= (s·i·n·d·i·c·a·l·i·z·a·d·o) | (h·i·v·\_·p·o·s·i·t·i·v·o) | (c·a·t·o·l·i·c·o)  
* **Texto Comum (Ruído)**  
  * Representado por ${V}_{in}^{*}$, processado quando as transições para os padrões acima falham.

## 4\. Construção e Modelagem dos Autômatos

A construção do modelo computacional obedecerá às seguintes etapas algorítmicas, conforme a teoria da computação abordada na disciplina:

### 4.1. Autômato Finito Não-Determinístico (AFND) Inicial

Inicialmente, os padrões descritos pelas ERs são agrupados em um AFND, representado formalmente por $M=(K,{V}_{in},T,i,F)$, onde a função de transição $T:K\times {V}_{in}\rightarrow {2}^{K}$ permite o não-determinismo na transição do estado inicial ${q}_{0}$ para as diferentes ramificações dos padrões léxicos (CPF, E-mail, Saúde, etc.).

*\[INSERIR AQUI A IMAGEM DO DIAGRAMA DO AFND (GERADO NO JFLAP)\]*

### 4.2. Conversão para Autômato Finito Determinístico (AFD)

Como o não-determinismo facilita a representação mas não aumenta o poder computacional, o algoritmo de conversão será aplicado. Para cada subconjunto de estados do AFND atingível pela leitura de um símbolo, criar-se-á um novo estado único no AFD equivalente, definindo uma função de transição rigorosa $T':K'\times {V}_{in}\rightarrow K'$.

*\[INSERIR AQUI A TABELA DE TRANSIÇÃO DO AFD CONVERTIDO\]*

### 4.3. Minimização do AFD

Para otimizar o processamento e a implementação em C/C++, o Algoritmo de Classes de Equivalência será aplicado. O conjunto $K'$ será inicialmente particionado nas classes $[F]$ (estados finais) e $[K-F]$ (estados não finais). Estados inacessíveis ou mortos gerados na conversão anterior serão eliminados, resultando no AFD Mínimo.

### 4.4. O Transdutor Finito (Máquina de Mealy)

A extensão aplicada deste projeto transforma o AFD Mínimo construído em um Transdutor. Optou-se pela **Máquina de Mealy**, cuja sêxtupla é dada por ${M}_{mealy}=({K}_{min},{V}_{in},{\Gamma }_{out},{T}_{min},\lambda ,{q}_{0})$.

A função de saída $\lambda :{K}_{min}\times {V}_{in}\rightarrow {\Gamma }_{out}$ opera de forma síncrona:

1. Enquanto o autômato transita por estados correspondentes ao "Ruído", $\lambda$ ecoa o caractere lido: $\lambda ({q}_{ruido},a)=a$.  
2. Quando ocorre uma transição bem-sucedida dentro das ramificações de um padrão de PII (ex: lendo os dígitos de um CPF ou dado de saúde), $\lambda$ emite a máscara: $\lambda ({q}_{cpf},D)=*$.  
3. Se um padrão falha antes de atingir o estado final (ex: lê `123.456` e a próxima entrada é uma letra), o buffer interno do sistema em C/C++ injetará a string não aceita de volta na saída padrão, retornando o controle ao estado ${q}_{0}$.

## 5\. Casos de Teste Iniciais

| Cadeia de Entrada Bruta (Log) | Caminho de Reconhecimento | Saída Esperada do Transdutor |
| :---- | :---- | :---- |
| `Processo 123.456.789-00 aberto` | Ruído $\rightarrow$ **ER\_cpf** $\rightarrow$ Ruído | `Processo ***.***.***-** aberto` |
| `Contato: (11) 98765-4321` | Ruído $\rightarrow$ **ER\_tel** | `Contato: (**) *****-****` |
| `Paciente hiv_positivo dia 10/05/1990` | Ruído $\rightarrow$ **ER\_saude** $\rightarrow$ Ruído $\rightarrow$ **ER\_data** | `Paciente ************ dia **/**/****` |
| `Funcionario sindicalizado demitido` | Ruído $\rightarrow$ **ER\_filiacao** $\rightarrow$ Ruído | `Funcionario ************* demitido` |
| `Falso padrão: 123.45X` | Ruído *(falha de transição no AFD)* | `Falso padrão: 123.45X` |

&nbsp;

## 6\. Planejamento e Divisão de Tarefas

Para a construção eficiente deste projeto, as responsabilidades foram divididas entre os integrantes do grupo:

* **Hellyel e Felipe:** Modelagem formal dos autômatos. Construção dos diagramas no software JFLAP (AFND e AFD), validação teórica das Expressões Regulares e elaboração das tabelas de transição.  
* **Gustavo e Luan:** Projeto da Arquitetura do Software em C/C++. Definição da estrutura de dados para o grafo do autômato (matrizes de transição ou estruturas `switch/case`), lógica do transdutor e gerenciamento de buffers para tratamentos de rejeição.  
* **Graziela e Grupo Conjunto:** Documentação do projeto, compilação dos relatórios, união dos módulos em C/C++, e execução da bateria de testes experimentais validando a corretude do modelo matemático frente à implementação.

&nbsp;