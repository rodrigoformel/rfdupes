//! Lógica principal do rfdupes: CLI, orquestração, coleta, comparação e relatório.

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
use std::time::{Duration, Instant};

/// Argumentos da linha de comando.
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
pub struct Args {
    /// Nome do diretório de entrada (argumento posicional)
    #[arg(default_value = ".")]
    pub input: String,

    /// Exibe tamanho em Bytes, KB ou MB (-s B ou --size B, ou -s K ou --size K, ou -s M ou --size M)
    #[arg(short, long, default_value = "M")]
    pub size: String,

    /// Salva o relatório em um arquivo (-f <nome_do_arquivo> ou --file <nome_do_arquivo>)
    #[arg(short, long = "file")]
    pub filename: Option<String>,

    /// Não exibe spinner nem mensagens auxiliares no terminal (-q / --quiet)
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Ignora arquivos com 0 bytes (-z / --zero)
    #[arg(short, long, default_value_t = false)]
    pub zero: bool,

    /// Modo rápido: compara os primeiros N KB antes da comparação integral (-r [KB] ou --rapid [KB]; padrão 16)
    #[arg(short, long, num_args = 0..=1, default_missing_value = "16", value_name = "KB")]
    pub rapid: Option<u32>,

    /// Exibe o tempo total de processamento (-t / --time)
    #[arg(short, long, default_value_t = false)]
    pub time: bool,

    /// Tamanho máximo dos arquivos a serem comparados (-L ou --max-size [B, KB ou MB de acordo com o parâmetro -s]))
    #[arg(short = 'L', long)]
    pub max_size: Option<u64>,

    /// Tamanho mínimo dos arquivos a serem comparados (-G ou --min-size [B, KB ou MB de acordo com o parâmetro -s]))
    #[arg(short = 'G', long)]
    pub min_size: Option<u64>,
}

/// Exibe o tempo decorrido ao sair do escopo, se habilitado.
pub struct TimeGuard {
    start: Instant,
    show: bool,
}

impl TimeGuard {
    pub fn new(show: bool) -> Self {
        Self {
            start: Instant::now(),
            show,
        }
    }
}

impl Drop for TimeGuard {
    fn drop(&mut self) {
        if self.show {
            let elapsed = self.start.elapsed();
            let s = elapsed.as_secs();
            let h = s / 3600;
            let m = (s % 3600) / 60;
            let sec = s % 60;
            let nanos = elapsed.subsec_nanos();
            println!(
                "Tempo de processamento: {:02}:{:02}:{:02}.{:09}",
                h, m, sec, nanos
            );
        }
    }
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

/// Lógica principal: valida entradas, coleta arquivos, agrupa duplicatas e gera o relatório.
pub fn run(args: &Args) -> io::Result<()> {
    let (size_limit, size_suffix) = match args.size.to_lowercase().as_str() {
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

    let start_path = fs::canonicalize(&args.input)?;
    if !args.quiet {
        println!("Analisando diretório: {}", start_path.display());
    }

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

    let min_bytes = args.min_size.map(|s| s * size_limit);
    let max_bytes = args.max_size.map(|s| s * size_limit);

    collect_files(&start_path, &mut files_by_size, &processed, args.zero, min_bytes, max_bytes)?;

    let mut duplicates: Vec<(u64, Vec<PathBuf>)> = Vec::new();
    phase.store(1, Ordering::Relaxed);

    for (size, paths) in files_by_size {
        if paths.len() > 1 {
            let path_lists: Vec<Vec<PathBuf>> = if let Some(kb) = args.rapid {
                let prefix_bytes = (kb as usize) * 1024;
                let mut by_prefix: HashMap<Vec<u8>, Vec<PathBuf>> = HashMap::new();
                for path in paths {
                    let prefix = read_file_prefix(&path, prefix_bytes).unwrap_or_default();
                    by_prefix.entry(prefix).or_default().push(path);
                }
                by_prefix.into_values().filter(|v| v.len() > 1).collect()
            } else {
                vec![paths]
            };
            for path_list in path_lists {
                let groups = group_identical_files(path_list)?;
                for group in groups {
                    if group.len() > 1 {
                        duplicates.push((size, group));
                    }
                }
            }
        }
    }

    if duplicates.is_empty() {
        println!("Nenhum arquivo duplicado encontrado.");
        return Ok(());
    }

    for (_, group) in &mut duplicates {
        group.sort();
    }
    duplicates.sort_by(|a, b| a.1.first().unwrap().cmp(b.1.first().unwrap()));

    match &args.filename {
        Some(name) => print_progress_file(&duplicates, size_limit, size_suffix, name, args.quiet)?,
        None => print_progress(&duplicates, size_limit, size_suffix, args.quiet)?,
    }

    Ok(())
}

fn print_progress(
    duplicates: &[(u64, Vec<PathBuf>)],
    size_limit: u64,
    size_suffix: &str,
    quiet: bool,
) -> io::Result<()> {
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
    zero: bool,
    min_len: Option<u64>,
    max_len: Option<u64>,
) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if entry.file_type()?.is_symlink() {
                continue;
            }

            if path.is_dir() {
                collect_files(&path, map, processed, zero, min_len, max_len)?;
            } else {
                let metadata = entry.metadata()?;
                let len = metadata.len();
                if zero && len == 0 {
                    continue;
                }
                if let Some(min) = min_len {
                    if len < min { continue; }
                }
                if let Some(max) = max_len {
                    if len > max { continue; }
                }

                processed.fetch_add(1, Ordering::Relaxed);
                map.entry(len).or_default().push(path);
            }
        }
    }
    Ok(())
}

fn read_file_prefix(path: &Path, max_bytes: usize) -> io::Result<Vec<u8>> {
    let f = File::open(path)?;
    let mut r = BufReader::new(f);
    let mut buf = vec![0u8; max_bytes];
    let n = r.read(&mut buf)?;
    buf.truncate(n);
    Ok(buf)
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

        if n1 != n2 {
            return Ok(false);
        }

        if n1 == 0 {
            return Ok(true);
        }

        if buf1[..n1] != buf2[..n1] {
            return Ok(false);
        }
    }
}
