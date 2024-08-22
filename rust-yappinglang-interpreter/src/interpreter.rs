pub mod builtins;
pub mod data;
pub mod run_tree;

use crate::interpreter::data::Data;
use crate::interpreter::run_tree::{Block, Exp, ExpData};
use std::collections::HashMap;
use std::rc::Rc;

type StackFn = fn(
    &mut Vec<Data>,
    &mut Vec<Rc<Block>>,
    &HashMap<String, Data>,
    &mut BlockExec,
) -> Result<(), String>;

pub struct Interpreter {
    root: BlockExec,
    pub blocks: Vec<Rc<Block>>,
    pub stack: Vec<Data>,
}

impl Interpreter {
    pub fn new(builtins: HashMap<String, Data>) -> Self {
        Self {
            root: BlockExec::new(usize::MAX, builtins),
            blocks: vec![],
            stack: vec![],
        }
    }
    pub fn load(&mut self, block: &crate::ast::Block) -> usize {
        run_tree::load(&mut self.blocks, block)
    }

    pub fn run(&mut self, block_id: usize) -> Result<(), String> {
        self.root
            .run_root(&mut self.stack, &mut self.blocks, block_id)
    }
}

struct BlockExec {
    defs: HashMap<String, Data>,
    id: usize,
}

impl BlockExec {
    pub fn new(id: usize, defs: HashMap<String, Data>) -> Self {
        Self { defs, id }
    }

    pub fn new_and_run(
        &mut self,
        stack: &mut Vec<Data>,
        blocks: &mut Vec<Rc<Block>>,
        prev_defs: &HashMap<String, Data>,
        block: &data::Block,
    ) -> Result<(), String> {
        let mut call = BlockExec::new(block.block_id, block.captured_vars.clone());
        let mut defs = prev_defs.clone();
        for (k, v) in &self.defs {
            defs.insert(k.clone(), v.clone());
        }
        call.run_block(stack, blocks, &defs, block.block_id)
    }

    pub fn run_root(
        &mut self,
        stack: &mut Vec<Data>,
        blocks: &mut Vec<Rc<Block>>,
        block_id: usize,
    ) -> Result<(), String> {
        self.run_block(stack, blocks, &HashMap::new(), block_id)
    }
    pub fn run_block(
        &mut self,
        stack: &mut Vec<Data>,
        blocks: &mut Vec<Rc<Block>>,
        prev_defs: &HashMap<String, Data>,
        block_id: usize,
    ) -> Result<(), String> {
        let Some(block) = blocks.get(block_id) else {
            return self.error(format!("no block with id {}", block_id));
        };
        if let Some(exp) = &block.clone().exp {
            self.run_exp(stack, blocks, prev_defs, exp)?;
        }
        Ok(())
    }

    fn run_exp(
        &mut self,
        stack: &mut Vec<Data>,
        blocks: &mut Vec<Rc<Block>>,
        prev_defs: &HashMap<String, Data>,
        exp: &Exp,
    ) -> Result<(), String> {
        if let Some(next_exp) = &exp.next_exp {
            self.run_exp(stack, blocks, prev_defs, next_exp)?;
        }
        match &exp.data {
            ExpData::Var(var) => {
                if let Some(data) = self.get_data(var, prev_defs) {
                    match data {
                        Data::Fn(block) => {
                            self.new_and_run(stack,blocks,prev_defs, &block)?;
                        }
                        Data::BuiltinFunc(func) => {
                            func(stack, blocks, prev_defs, self)?;
                        }
                        _ => stack.push(data),
                    }
                } else {
                    return self.error(format!("variable '{}' not found", var));
                }
            }
            ExpData::Block(id) => {
                let Some(block) = blocks.get(*id) else {
                    return self.error(format!("no block with id {}", id));
                };
                let mut captured_vars = HashMap::new();
                for var in &block.capture_vars {
                    if let Some(data) = self.get_data(var, prev_defs) {
                        captured_vars.insert(var.clone(), data);
                    } else {
                        return self.error(format!("variable '{}' not found for capture", var));
                    }
                }
                stack.push(Data::Block(data::Block {
                    block_id: *id,
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

    fn error(&self, message: String) -> Result<(), String> {
        Err(format!("error in block {}: {}", self.id, message))
    }


}
