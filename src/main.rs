#![forbid(unsafe_code)]

mod ast;
mod frontend;
mod frontend_error;
mod lexer;
mod parser;
mod scanner;
mod semantic;
#[cfg(test)]
mod tests;
mod token;

use std::{env, error::Error, fs::File, path::PathBuf, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("MicroC error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);

    let source_path = match (args.next(), args.next()) {
        (Some(path), None) => PathBuf::from(path),
        _ => return Err("usage: microc-rs <source.mcc>".into()),
    };

    let file = File::open(&source_path)?;

    frontend::analyze_source(file)?;

    println!("Frontend validation successful: {}", source_path.display());

    Ok(())
}
