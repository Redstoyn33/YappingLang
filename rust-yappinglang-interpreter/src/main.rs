use crate::interpreter::builtins::{base, math_and_logic};
use crate::interpreter::data::Data;
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
    let mut scanner = Scanner::new(&source);
    let tokens = scanner.scan_tokens()?;
    //println!("tokens:");
    //println!("{:?}", tokens);
    let mut parser = Parser::new(tokens);
    let ast = parser.build_tree()?;
    //println!("ast:");
    //println!("{:?}", ast);
    //print_ast(&ast,"test.puml")?;
    let mut intr = Interpreter::new(get_std_defs());
    let id = intr.load(&ast);
    intr.run(id).map_err(|e| println!("error:\n{e}"));
    println!("stack after end:");
    println!("-----");
    intr.stack.iter().rev().for_each(|d| println!("{}", d));
    println!("-----");
    //println!("blocks:");
    //println!("{:?}",intr.blocks);

    Ok(())
}

fn run_repl() -> Result<(), String> {
    let inp = std::io::stdin();
    let mut intr = Interpreter::new(get_std_defs());
    loop {
        let mut source = String::new();
        print!("> ");
        stdout().flush().str_res()?;
        inp.read_line(&mut source).str_res()?;
        if source.starts_with("!!!") {
            source = String::new();
            loop {
                print!("> ");
                stdout().flush().str_res()?;
                let mut source_add = String::new();
                inp.read_line(&mut source_add).str_res()?;
                if source_add.starts_with("!!!") {
                    break;
                }
                source.push_str(&source_add);
            }
        }

        let mut scanner = Scanner::new(&source);
        let tokens = scanner.scan_tokens()?;
        //println!("tokens:");
        //println!("{:?}", tokens);
        let mut parser = Parser::new(tokens);
        let ast = parser.build_tree()?;
        //println!("ast:");
        //println!("{:?}", ast);
        //print_ast(&ast,"test.puml")?;
        let id = intr.load(&ast);
        intr.run(id).map_err(|e| println!("error:\n{e}"));
    }
    Ok(())
}

fn get_std_defs() -> HashMap<String, Data> {
    math_and_logic(base(HashMap::new(), ""), "")
}
