use crate::interpreter::builtins::base;
use crate::interpreter::Interpreter;
use crate::parser::{print_ast, Parser};
use crate::scanner::Scanner;
use crate::utils::ResultToString;
use std::collections::HashMap;
use std::env::args;
use std::fs::File;
use std::io::{stdout, Read, Write};
use std::path::Path;

mod ast;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod utils;

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
            Err(err) => {
                println!("Error: {}", err)
            }
        }
    }
}

fn run(source: String) -> Result<(), String> {
    let mut scanner = Scanner::new(&source);
    let tokens = scanner.scan_tokens()?;
    //println!("tokens:");
    //println!("{:?}", tokens);
    let mut parser = Parser::new(tokens);
    let ast = parser.build_tree()?;
    //println!("ast:");
    //println!("{:?}", ast);
    //print_ast(&ast,"test.puml")?;
    let mut intr = Interpreter::new(base(HashMap::new()));
    let id = intr.load(&ast);
    intr.run(id).map_err(|e|println!("{e}"));
    println!("stack after end:");
    intr.stack.iter().rev().for_each(|d| println!("{}", d));
    //println!("blocks:");
    //println!("{:?}",intr.blocks);

    Ok(())
}
