//! Ponto de entrada: parse dos argumentos e delegação para o módulo app.

mod app;

use clap::Parser;
use std::io;

fn main() -> io::Result<()> {
    let args = app::Args::parse();
    let _time_guard = app::TimeGuard::new(args.time);
    app::run(&args)
}
