# rfdupes 🦀
> A fast and multithreaded implementation of **fdupes** written in Rust.

> Uma implementação veloz e multithreaded do **fdupes** escrita em Rust.

<div align="justify">

<details title="English" align='left'>
<summary align='left'>:uk: English</summary>
<br>

**rfdupes** is a command-line utility for locating and identifying
duplicate files within a directory tree. It provides size filters, fast
comparison modes, and detailed reports, leveraging the safety and
performance of the Rust language.

## ✨ Features

-   **🚀 High Performance:** Written in Rust, with efficient processing.
-   **🧵 Multithreading:** Uses threads to display visual feedback
    (spinner) while processing.
-   **🔍 Smart Comparison:**
    -   Initial comparison by file size.
    -   **Rapid Mode (`--rapid`):** Compares prefixes (e.g., first 16KB)
        before reading the entire file.
    -   Full byte-by-byte comparison for final confirmation.
-   **🎚️ Search Filters:**
    -   Filter by maximum (`-L`) and minimum (`-G`) file size.
    -   Ignore empty files (`-z`).
-   **📊 Flexible Reports:**
    -   Display in Bytes, KB, or MB.
    -   Direct export to a text file (`--file`).
    -   Shows total execution time.

## 📦 Installation

### Prerequisites

You need to have Rust and Cargo installed.

``` bash
# Clone the repository
git clone https://github.com/seu-usuario/rfdupes.git

# Enter the directory
cd rfdupes

# Build and install
cargo install --path .
```

## 🛠️ How to Use

The simplest way to run it is by pointing to a directory:

``` bash
rfdupes /path/to/analyze
```

### Common Examples

**1. Filter large files (between 100MB and 1GB):** The `-s` parameter
defines the unit (MB by default) used by the `-G` and `-L` filters.

``` bash
rfdupes -s M -G 100 -L 1000 /my/files
```

**2. Save the report to a file (no terminal output):** Useful for logs
or later analysis.

``` bash
rfdupes --quiet --file duplicates.txt .
```

**3. Rapid Mode:** Groups files by comparing only the first 4KB (default
is 16KB) before full verification, speeding up searches when files are
large and differ early.

``` bash
rfdupes --rapid 4 .
```

### ⚙️ Options (Flags)

| Short flag | Long flag | Description |
| :--- | :--- | :--- |
| `-s` | `--size` | Defines the size unit: `B` (Bytes), `K` (KB), `M` (MB). Default: `M`. |
| `-L` | `--max-size` | **Maximum** file size (respects the `-s` unit). |
| `-G` | `--min-size` | **Minimum** file size (respects the `-s` unit). |
| `-z` | `--zero` | Ignores 0-byte (empty) files. |
| `-r` | `--rapid` | Rapid mode: compares the first N KB (default 16KB). |
| `-R` | `--recursive` | Recursively descends into directories. |
| `-f` | `--file` | Saves the result to a specified text file. |
| `-q` | `--quiet` | Quiet mode: does not display spinner or progress in the terminal. |
| `-t` | `--time` | Displays the total processing time at the end. |
| `-h` | `--help` | Displays help information. |


### 🤝 Contributing

Contributions are welcome! Feel free to open **issues** or submit **pull
requests**.

1.  Fork the project
2.  Create your Feature Branch (`git checkout -b feature/MyFeature`)
3.  Commit your changes (`git commit -m 'Add MyFeature'`)
4.  Push to the Branch (`git push origin feature/MyFeature`)
5.  Open a Pull Request

<hr>

</details>

<details title="Português" align='left'>
<summary align='left'>:brazil: Português</summary>
<br>

O **rfdupes** é um utilitário de linha de comando para localizar e identificar arquivos duplicados dentro de uma árvore de diretórios. Ele oferece filtros de tamanho, modos de comparação rápida e relatórios detalhados, aproveitando a segurança e performance da linguagem Rust.

## ✨ Funcionalidades

- **🚀 Alta Performance:** Escrito em Rust, com processamento eficiente.
- **🧵 Multithreading:** Utiliza threads para exibir feedback visual (spinner) enquanto processa.
- **🔍 Comparação Inteligente:**
  - Comparação inicial por tamanho.
  - **Modo Rápido (`--rapid`):** Compara prefixos (ex: primeiros 16KB) antes de ler o arquivo inteiro.
  - Comparação byte-a-byte completa para confirmação final.
- **🎚️ Filtros de Busca:**
  - Filtrar por tamanho máximo (`-L`) e mínimo (`-G`).
  - Ignorar arquivos vazios (`-z`).
- **📊 Relatórios Flexíveis:**
  - Visualização em Bytes, KB ou MB.
  - Exportação direta para arquivo de texto (`--file`).
  - Exibição do tempo total de execução.

## 📦 Instalação

### Pré-requisitos
Você precisa ter o Rust e o Cargo instalados.

```bash
# Clone o repositório
git clone https://github.com/seu-usuario/rfdupes.git

# Entre no diretório
cd rfdupes

# Compile e instale
cargo install --path .
```

## 🛠️ Como Usar

A forma mais simples de rodar é apontar para um diretório:

```bash
rfdupes /caminho/para/analisar
```

### Exemplos Comuns

**1. Filtrar arquivos grandes (entre 100MB e 1GB):**
O parâmetro `-s` define a unidade (MB padrão) que será usada pelos filtros `-G` e `-L`.

```bash
rfdupes -s M -G 100 -L 1000 /meus/arquivos
```

**2. Salvar o relatório em um arquivo (sem output no terminal):**
Útil para logs ou análise posterior.

```bash
rfdupes --quiet --file duplicados.txt .
```

**3. Modo Rápido (Rapid Mode):**
Agrupa arquivos comparando apenas os primeiros 4KB (padrão é 16KB) antes da verificação total, acelerando a busca em arquivos grandes e distintos no início.

```bash
rfdupes --rapid 4 .
```

### ⚙️ Opções (Flags)

| Flag curta | Flag longa | Descrição |
| :--- | :--- | :--- |
| `-s` | `--size` | Define a unidade de tamanho: `B` (Bytes), `K` (KB), `M` (MB). Padrão: `M`. |
| `-L` | `--max-size` | Tamanho **máximo** do arquivo (respeita a unidade de `-s`). |
| `-G` | `--min-size` | Tamanho **mínimo** do arquivo (respeita a unidade de `-s`). |
| `-z` | `--zero` | Ignora arquivos de 0 bytes (vazios). |
| `-r` | `--rapid` | Modo rápido: compara os primeiros N KB (padrão 16KB). |
| `-R` | `--recursive` | Desce recursivamente em diretórios. |
| `-f` | `--file` | Salva o resultado em um arquivo de texto especificado. |
| `-q` | `--quiet` | Modo silencioso: não exibe spinner ou progresso no terminal. |
| `-t` | `--time` | Exibe o tempo total de processamento ao final. |
| `-h` | `--help` | Exibe a ajuda. |

### 🤝 Contribuição

Contribuições são bem-vindas! Sinta-se à vontade para abrir **issues** ou enviar **pull requests**.

1. Faça um Fork do projeto
2. Crie sua Feature Branch (`git checkout -b feature/MinhaFeature`)
3. Commit suas mudanças (`git commit -m 'Adiciona MinhaFeature'`)
4. Push para a Branch (`git push origin feature/MinhaFeature`)
5. Abra um Pull Request

</details>

</div>