# rfdupes 🦀

> Uma implementação veloz e multithreaded do **fdupes** escrita em Rust.

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
