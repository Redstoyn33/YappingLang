use crate::interpreter::{BlockExec, Interpreter};
use std::collections::HashSet;
use std::rc::Rc;

#[derive(Debug)]
pub struct Block {
    pub exp: Option<Box<Exp>>,
    pub capture_vars: HashSet<String>,
}
#[derive(Debug)]
pub enum ExpData {
    Var(String),
    Block(Rc<Block>),
    Integer(i64),
    Decimal(f64),
    String(String),
}
#[derive(Debug)]
pub struct Exp {
    pub data: ExpData,
    pub next_exp: Option<Box<Exp>>,
}

pub fn load(block: &crate::ast::Block) -> Rc<Block> {
    let mut capture_vars = HashSet::new();
    if let Some(exp) = &block.exp {
        let exp = load_exp(exp, &mut capture_vars);
        return Rc::new(Block {
            exp: Some(Box::new(exp)),
            capture_vars,
        });
    } else {
        return Rc::new(Block {
            exp: None,
            capture_vars,
        });
    }
}

fn load_exp(exp: &crate::ast::Exp, captured_vars: &mut HashSet<String>) -> Exp {
    Exp {
        data: match &exp.data {
            crate::ast::ExpData::Var(var) => ExpData::Var(var.clone()),
            crate::ast::ExpData::CapturedVar(var) => {
                captured_vars.insert(var.clone());
                ExpData::Var(var.clone())
            }
            crate::ast::ExpData::Block(block) => {
                let block = load(block);
                ExpData::Block(block)
            }
            crate::ast::ExpData::Integer(int) => ExpData::Integer(*int),
            crate::ast::ExpData::Decimal(dec) => ExpData::Decimal(*dec),
            crate::ast::ExpData::String(str) => ExpData::String(str.clone()),
        },
        next_exp: if let Some(next_exp) = &exp.next_exp {
            Some(Box::new(load_exp(next_exp, captured_vars)))
        } else {
            None
        },
    }
}
