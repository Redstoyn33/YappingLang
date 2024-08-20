use std::env::args;
use std::fs::File;
use std::io::{Read, stdout, Write};
use std::path::Path;
use crate::scanner::Scanner;
use crate::utils::ResultToString;

mod scanner;
mod utils;
mod token;
mod parser;

fn main() -> Result<(), String> {
    let mut args = args();
    match args.len() {
        1 => {
            return run_repl();
        }
        2 => {
            return run_file(&args.skip(1).next().unwrap());
        }
        _ => {
            println!("Usage: {} [file.yp]", args.next().unwrap());
        }
    }
    Ok(())
}

fn run_file(path: &str) -> Result<(), String> {
    let mut file = File::open(Path::new(path)).str_res()?;
    let mut source = String::new();
    file.read_to_string(&mut source).str_res()?;
    run(source)?;
    Ok(())
}


fn run_repl() -> Result<(), String> {
    let inp = std::io::stdin();
    loop {
        let mut source = String::new();
        print!("> ");
        stdout().flush().str_res()?;
        inp.read_line(&mut source).str_res()?;
        match run(source) {
            Ok(_) => {}
            Err(err) => {println!("Error: {}", err)}
        }
    }
}

fn run(source: String) -> Result<(), String> {
    let mut scanner = Scanner::new(&source);
    println!("{:?}",scanner.scan_tokens()?);
    Ok(())
}