use clap::Parser;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};

// Apagar se retirar a verificação de hash ou implementar BD.
//use sha2::{Digest, Sha256};

/// Busca por arquivos duplicados em uma árvore de diretórios
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None,
    long_version = concat!(
        env!("CARGO_PKG_VERSION"),
        "\nAutor: ",
        env!("CARGO_PKG_AUTHORS")
    )
)]
struct Args {
    /// Nome do diretório de entrada (argumento posicional)
    #[arg(default_value = ".")]
    input: String,

    /// Exibe tamanho em Bytes, KB ou MB (-s B ou --size B, ou -s K ou --size K, ou -s M ou --size M)
    #[arg(short, long, default_value = "M")]
    size: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let size_type_arg = args.size.to_lowercase();
    let (size_limit, size_suffix) = match size_type_arg.as_str() {
        "m" => (1024 * 1024, "MB"),
        "k" => (1024, "KB"),
        "b" => (1, "Bytes"),
        _ => {
            println!(
                "Tipo de tamanho inválido. Use 'm' para megabytes, 'k' para kilobytes ou 'b' para bytes."
            );
            return Ok(());
        }
    };

    let start_path = fs::canonicalize(args.input)?;
    println!("Analisando diretório: {}", start_path.display());

    // 1. Coleta arquivos agrupados por tamanho
    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let mut count = 0;
    collect_files(&start_path, &mut files_by_size, &mut count)?;

    // 2. Verifica conteúdo para arquivos com mesmo tamanho
    let mut duplicates: Vec<(u64, Vec<PathBuf>)> = Vec::new();

    for (size, paths) in files_by_size {
        if paths.len() > 1 {
            let groups = group_identical_files(paths)?;
            for group in groups {
                if group.len() > 1 {
                    duplicates.push((size, group));
                }
            }
        }
    }

    if duplicates.is_empty() {
        println!("Nenhum arquivo duplicado encontrado.");
        return Ok(());
    }

    // Ordena os resultados para um relatório consistente e alfabético
    for (_, group) in &mut duplicates {
        group.sort(); // Ordena os caminhos dentro de cada grupo
    }
    // Ordena os grupos com base no primeiro caminho de cada um
    duplicates.sort_by(|a, b| a.1.first().unwrap().cmp(b.1.first().unwrap()));

    // 3. Gera o relatório
    let output_file = "duplicados.txt";
    let mut file = File::create(output_file)?;

    writeln!(file, "Relatório de Arquivos Duplicados")?;
    writeln!(file, "================================")?;

    for (i, (size, group)) in duplicates.iter().enumerate() {
        let size_formatted = *size as f64 / (size_limit as f64);
        writeln!(
            file,
            "\nGrupo {} (Tamanho: {:.2} {}):",
            i + 1,
            size_formatted,
            size_suffix
        )?;
        for path in group {
            writeln!(file, " - {}", path.display())?;
        }
    }

    println!(
        "Encontrados {} grupos de arquivos repetidos.",
        duplicates.len()
    );
    println!("Relatório salvo em: {}", output_file);

    Ok(())
}

fn collect_files(
    dir: &Path,
    map: &mut HashMap<u64, Vec<PathBuf>>,
    count: &mut usize,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            // Ignora links simbólicos para evitar loops ou duplicatas falsas
            if entry.file_type()?.is_symlink() {
                continue;
            }

            if path.is_dir() {
                collect_files(&path, map, count)?;
            } else {
                *count += 1;
                println!("Processando: {} (Total: {})", path.display(), count);
                let metadata = entry.metadata()?;
                map.entry(metadata.len()).or_default().push(path);
            }
        }
    }
    Ok(())
}

fn group_identical_files(paths: Vec<PathBuf>) -> io::Result<Vec<Vec<PathBuf>>> {
    let mut groups: Vec<Vec<PathBuf>> = Vec::new();

    for path in paths {
        let mut found = false;
        for group in &mut groups {
            if let Some(first) = group.first() {
                if files_are_equal(first, &path)? {
                    group.push(path.clone());
                    found = true;
                    break;
                }
            }
        }
        if !found {
            groups.push(vec![path]);
        }
    }
    Ok(groups)
}

fn files_are_equal(p1: &Path, p2: &Path) -> io::Result<bool> {
    let f1 = File::open(p1)?;
    let f2 = File::open(p2)?;

    let mut r1 = BufReader::new(f1);
    let mut r2 = BufReader::new(f2);
    let mut buf1 = [0; 8192];
    let mut buf2 = [0; 8192];

    loop {
        let n1 = r1.read(&mut buf1)?;
        let n2 = r2.read(&mut buf2)?;

        // Se leram quantidades diferentes de bytes, arquivos têm tamanhos diferentes
        // (ou foram alterados durante a leitura) e não são iguais.
        if n1 != n2 {
            return Ok(false);
        }

        // Ambos chegaram ao fim ao mesmo tempo.
        if n1 == 0 {
            return Ok(true);
        }

        if buf1[..n1] != buf2[..n1] {
            return Ok(false);
        }
    }
}
/*
// Verificação por hash
fn files_have_same_hash(p1: &Path, p2: &Path) -> io::Result<bool> {
    let h1 = compute_file_hash(p1)?;
    let h2 = compute_file_hash(p2)?;
    Ok(h1 == h2)
}

fn compute_file_hash(path: &Path) -> io::Result<[u8; 32]> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }

    let result = hasher.finalize();
    let mut hash_bytes = [0u8; 32];
    hash_bytes.copy_from_slice(&result[..]);
    Ok(hash_bytes)
}
*/
