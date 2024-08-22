use crate::ast::{Block, Exp, ExpData};
use crate::token::{Token, TokenData};
use crate::utils::ResultToString;
use std::any::type_name;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn build_tree(mut self) -> Result<Block, String> {
        return self.build_block(true);
    }

    fn build_block(&mut self, file: bool) -> Result<Block, String> {
        let mut lines: Vec<Vec<ExpData>> = vec![vec![]];
        while !self.is_at_end() {
            match self.advance().typ.clone() {
                TokenData::Identifier(name) => {
                    lines.last_mut().unwrap().push(ExpData::Var(name));
                }
                TokenData::String(str) => {
                    lines.last_mut().unwrap().push(ExpData::String(str));
                }
                TokenData::Integer(int) => {
                    lines.last_mut().unwrap().push(ExpData::Integer(int));
                }
                TokenData::Decimal(dec) => {
                    lines.last_mut().unwrap().push(ExpData::Decimal(dec));
                }
                TokenData::Semicolon => {
                    lines.push(vec![]);
                }
                TokenData::LeftParen => {
                    let block = self.build_block(false)?;
                    lines.last_mut().unwrap().push(ExpData::Block(block));
                }
                TokenData::RightParen => {
                    if file {
                        return self.error("to many )");
                    } else {
                        let exp = flat_lines(lines).map(|e| Box::new(e));
                        return Ok(Block { exp });
                    }
                }
                TokenData::Capture => match self.advance().typ.clone() {
                    TokenData::Identifier(name) => {
                        lines.last_mut().unwrap().push(ExpData::CapturedVar(name));
                    }
                    _ => return self.error("expect identifier for capture"),
                },
            }
        }
        if file {
            let exp = flat_lines(lines).map(|e| Box::new(e));
            return Ok(Block { exp });
        } else {
            return self.error("block not ended by )");
        }
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.tokens.len();
    }
    fn advance(&mut self) -> &Token {
        if self.is_at_end() {
            return self.previous();
        }
        let t = &self.tokens[self.current];
        self.current += 1;
        return t;
    }

    fn previous(&self) -> &Token {
        return &self.tokens[self.current - 1];
    }

    fn error<T>(&self, err: &str) -> Result<T, String> {
        Err(format!("[line {}] Error: {}", self.previous().line, err))
    }
}

fn flat_lines(lines: Vec<Vec<ExpData>>) -> Option<Exp> {
    let mut head = None;
    for line in lines.into_iter() {
        for exp in line.into_iter().rev() {
            if let Some(e) = head {
                head = Some(Exp {
                    data: exp,
                    next_exp: Some(Box::new(e)),
                });
            } else {
                head = Some(Exp {
                    data: exp,
                    next_exp: None,
                });
            }
        }
    }
    head
}

pub fn print_ast(tree: &Block, filepath: &str) -> Result<(), String> {
    let mut file = File::create(Path::new(filepath)).str_res()?;

    file.write_all("@startuml\n".as_ref()).str_res()?;

    file.write_all("(file)as 0\n".as_ref()).str_res()?;

    if let Some(exp) = &tree.exp {
        let idx = print_ast_exp(exp, &mut file)?;
        file.write_all(format!("0-d->{}\n", idx).as_ref())
            .str_res()?;
    }

    file.write_all("@enduml\n".as_ref()).str_res()?;

    Ok(())
}
fn print_ast_exp(exp: &Exp, file: &mut File) -> Result<usize, String> {
    file.write_all("(".as_ref()).str_res()?;
    match &exp.data {
        ExpData::Var(var) => {
            file.write_all(var.as_ref()).str_res()?;
        }
        ExpData::CapturedVar(var) => {
            file.write_all(format!("@{var}").as_ref()).str_res()?;
        }
        ExpData::Block(_) => {
            file.write_all("block".as_ref()).str_res()?;
        }
        ExpData::Integer(int) => {
            file.write_all(format!("{int}").as_ref()).str_res()?;
        }
        ExpData::Decimal(dec) => {
            file.write_all(format!("@{dec}").as_ref()).str_res()?;
        }
        ExpData::String(str) => {
            file.write_all(format!("\"{str}").as_ref()).str_res()?;
        }
    }
    let idx = exp as *const Exp as usize;
    file.write_all(format!(")as {}\n", idx).as_ref())
        .str_res()?;
    if let ExpData::Block(block) = &exp.data {
        if let Some(block_exp) = &block.exp {
            let block_idx = print_ast_exp(block_exp, file)?;
            file.write_all(format!("{}-r->{}\n", idx, block_idx).as_ref())
                .str_res()?;
        }
    }
    if let Some(next) = &exp.next_exp {
        let next_idx = print_ast_exp(next, file)?;
        file.write_all(format!("{}-d->{}\n", idx, next_idx).as_ref())
            .str_res()?;
    }
    Ok(idx)
}
