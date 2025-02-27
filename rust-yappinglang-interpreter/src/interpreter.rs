pub mod builtins;
pub mod data;
pub mod run_tree;
mod external;

use std::cell::RefCell;
use crate::interpreter::data::Data;
use crate::interpreter::run_tree::{Block, Exp, ExpData};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::interpreter::external::External;

type StackFn = fn(&mut Vec<Data>, &HashMap<String, Data>, &mut BlockExec) -> Result<(), String>;

pub struct Interpreter {
    pub root: BlockExec,
    pub stack: Vec<Data>,
}

impl Interpreter {
    pub fn new(builtins: HashMap<String, Data>) -> Self {
        Self {
            root: BlockExec::new(builtins),
            stack: vec![],
        }
    }
    pub fn load(&mut self, block: &crate::ast::Block) -> Rc<Block> {
        run_tree::load(block)
    }

    pub fn run(&mut self, block: Rc<Block>) -> Result<(), String> {
        self.root.run_root(&mut self.stack, block)
    }

    pub fn load_and_run(&mut self, block: &crate::ast::Block) -> Result<(), String> {
        let block = self.load(block);
        self.run(block)
    }
}

struct BlockExec {
    defs: HashMap<String, Data>,
}

impl BlockExec {
    pub fn new(defs: HashMap<String, Data>) -> Self {
        Self { defs }
    }

    pub fn new_and_run(
        &mut self,
        stack: &mut Vec<Data>,
        prev_defs: &HashMap<String, Data>,
        block: &data::Block,
    ) -> Result<(), String> {
        let mut call = BlockExec::new(block.captured_vars.clone());
        let mut defs = prev_defs.clone();
        for (k, v) in &self.defs {
            defs.insert(k.clone(), v.clone());
        }
        call.run_block(stack, &defs, block.block.clone())
    }

    pub fn run_root(&mut self, stack: &mut Vec<Data>, block: Rc<Block>) -> Result<(), String> {
        self.run_block(stack, &HashMap::new(), block)
    }
    pub fn run_block(
        &mut self,
        stack: &mut Vec<Data>,
        prev_defs: &HashMap<String, Data>,
        block: Rc<Block>,
    ) -> Result<(), String> {
        if let Some(exp) = &block.exp {
            self.run_exp(stack, prev_defs, exp)?;
        }
        Ok(())
    }

    fn run_exp(
        &mut self,
        stack: &mut Vec<Data>,
        prev_defs: &HashMap<String, Data>,
        exp: &Exp,
    ) -> Result<(), String> {
        if let Some(next_exp) = &exp.next_exp {
            self.run_exp(stack, prev_defs, next_exp)?;
        }
        match &exp.data {
            ExpData::Var(var) => {
                if let Some(data) = self.get_data(var, prev_defs) {
                    match data {
                        Data::Fn(block) => {
                            self.new_and_run(stack, prev_defs, &block)?;
                        }
                        Data::BuiltinFunc(func) => {
                            func(stack, prev_defs, self)?;
                        }
                        _ => stack.push(data),
                    }
                } else {
                    return self.error(format!("variable '{}' not found", var));
                }
            }
            ExpData::Block(block) => {
                let captured_vars = self.capture(&block.capture_vars, prev_defs)?;
                stack.push(Data::Block(data::Block {
                    block: block.clone(),
                    captured_vars,
                }));
            }
            ExpData::Integer(int) => stack.push(Data::Integer(*int)),
            ExpData::Decimal(dec) => stack.push(Data::Decimal(*dec)),
            ExpData::String(str) => stack.push(Data::String(str.clone())),
        }
        Ok(())
    }

    fn get_data(&self, var: &String, defs_find: &HashMap<String, Data>) -> Option<Data> {
        if let Some(data) = self.defs.get(var) {
            Some(data.clone())
        } else {
            if let Some(data) = defs_find.get(var) {
                Some(data.clone())
            } else {
                None
            }
        }
    }

    fn capture(
        &mut self,
        vars: &HashSet<String>,
        prev_defs: &HashMap<String, Data>,
    ) -> Result<HashMap<String, Data>, String> {
        let mut captured_vars = HashMap::new();
        for var in vars {
            if let Some(data) = self.get_data(var, prev_defs) {
                captured_vars.insert(var.clone(), data);
            } else {
                return self.error(format!("variable '{}' not found for capture", var));
            }
        }
        Ok(captured_vars)
    }

    fn error<T>(&self, message: String) -> Result<T, String> {
        Err(format!("error: {}", message))
    }
}
