use crate::interpreter::StackFn;
use std::any::type_name_of_val;
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Pointer};
use std::rc::Rc;

#[derive(Clone)]
pub enum Data {
    String(String),
    Integer(i64),
    Decimal(f64),
    List(Vec<Data>),
    Dict(HashMap<Data, Data>),
    Box(Rc<Cell<Data>>),
    Block(Block),
    Fn(Block),
    External(usize),
    BuiltinFunc(StackFn),
}

#[derive(Clone)]
pub struct Block {
    pub block_id: usize,
    pub captured_vars: HashMap<String, Data>,
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::String(str) => f.write_fmt(format_args!("'{str}'")),
            Data::Integer(int) => f.write_fmt(format_args!("{int}")),
            Data::Decimal(dec) => f.write_fmt(format_args!("{dec}")),
            Data::List(l) => f.write_fmt(format_args!("vec of {}", l.len())),
            Data::Dict(d) => f.write_fmt(format_args!("map of {}", d.len())),
            Data::Box(b) => f.write_fmt(format_args!("box")),
            Data::Block(b) => f.write_fmt(format_args!("block {}", b.block_id)),
            Data::Fn(b) => f.write_fmt(format_args!("fn {}", b.block_id)),
            Data::External(e) => f.write_fmt(format_args!("external {e}")),
            Data::BuiltinFunc(b) => {
                f.write_fmt(format_args!("built-in func {}", type_name_of_val(b)))
            }
        }
    }
}
