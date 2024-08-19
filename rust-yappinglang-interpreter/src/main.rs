use std::env::args;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use crate::scanner::Scanner;
use crate::utils::ResultToString;

mod scanner;
mod utils;
mod token;

fn main() {
    let mut args = args();
    match args.len() {
        1 => {
            run_repl();
        }
        2 => {
            run_file(&args.skip(1).next().unwrap()).unwrap();
        }
        _ => {
            println!("Usage: {} [file.yp]", args.next().unwrap());
        }
    }
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
        println!("> ");
        inp.read_line(&mut source).str_res()?;
        match run(source) {
            Ok(_) => {}
            Err(err) => {println!("Error: {}", err)}
        }
    }
}

fn run(source: String) -> Result<(), String> {
    let scanner = Scanner::new(source);

}