use clap::Parser;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

    /// Salva o relatório em um arquivo (-f <nome_do_arquivo> ou --file <nome_do_arquivo>)
    #[arg(short, long)]
    filename: Option<String>,

    /// Não exibe spinner nem mensagens auxiliares no terminal (-q / --quiet)
    #[arg(short, long, default_value_t = false)]
    quiet: bool,
}

struct SpinnerGuard {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl SpinnerGuard {
    fn start(processed: Arc<AtomicUsize>, phase: Arc<AtomicU8>) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_thread = Arc::clone(&stop);

        let handle = thread::spawn(move || {
            const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
            let mut i = 0usize;

            while !stop_thread.load(Ordering::Relaxed) {
                let n = processed.load(Ordering::Relaxed);
                let phase_str = match phase.load(Ordering::Relaxed) {
                    0 => "Coletando arquivos",
                    1 => "Comparando conteúdo",
                    _ => "Trabalhando",
                };

                eprint!("\r{} {}... ({} arquivos)", FRAMES[i % FRAMES.len()], phase_str, n);
                let _ = io::stderr().flush();

                i = i.wrapping_add(1);
                thread::sleep(Duration::from_millis(120));
            }

            // Limpa a linha do spinner
            eprint!("\r{: <80}\r", "");
            let _ = io::stderr().flush();
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }
}

impl Drop for SpinnerGuard {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
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
    if !args.quiet {
        println!("Analisando diretório: {}", start_path.display());
    }

    // 1. Coleta arquivos agrupados por tamanho
    let mut files_by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    let processed = Arc::new(AtomicUsize::new(0));
    let phase = Arc::new(AtomicU8::new(0));
    let _spinner = if args.quiet {
        None
    } else {
        Some(SpinnerGuard::start(
            Arc::clone(&processed),
            Arc::clone(&phase),
        ))
    };

    collect_files(&start_path, &mut files_by_size, &processed)?;

    // 2. Verifica conteúdo para arquivos com mesmo tamanho
    let mut duplicates: Vec<(u64, Vec<PathBuf>)> = Vec::new();
    phase.store(1, Ordering::Relaxed);

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

    match args.filename {
        Some(ref name) => {
            print_progress_file(&duplicates, size_limit, size_suffix, name, args.quiet)?;
        }
        None => {
            print_progress(&duplicates, size_limit, size_suffix, args.quiet)?;
        }
    }

    Ok(())
}

fn print_progress(
    duplicates: &[(u64, Vec<PathBuf>)],
    size_limit: u64,
    size_suffix: &str,
    quiet: bool,
) -> io::Result<()> {
    // 3. Gera o relatório
    println!("\nRelatório de Arquivos Duplicados");
    println!("================================");

    for (i, (size, group)) in duplicates.iter().enumerate() {
        let size_formatted = *size as f64 / (size_limit as f64);
        println!(           
            "\nGrupo {} (Tamanho: {:.2} {}):",
            i + 1,
            size_formatted,
            size_suffix
        );
        for path in group {
            println!(" - {}", path.display());
        }
    }

    if !quiet {
        println!(
            "Encontrados {} grupos de arquivos repetidos.",
            duplicates.len()
        );
        println!("\n");
    }

    Ok(())
}

fn print_progress_file(
    duplicates: &[(u64, Vec<PathBuf>)],
    size_limit: u64,
    size_suffix: &str,
    filename: &str,
    quiet: bool,
) -> io::Result<()> {
    // 3. Gera o relatório
    let mut file = File::create(filename)?;

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

    if !quiet {
        println!(
            "Encontrados {} grupos de arquivos repetidos.",
            duplicates.len()
        );
        println!("Relatório salvo em: {}", filename);
    }

    Ok(())
}   

fn collect_files(
    dir: &Path,
    map: &mut HashMap<u64, Vec<PathBuf>>,
    processed: &AtomicUsize,
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
                collect_files(&path, map, processed)?;
            } else {
                processed.fetch_add(1, Ordering::Relaxed);
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
