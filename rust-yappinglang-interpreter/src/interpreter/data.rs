use crate::interpreter::run_tree;
use crate::interpreter::StackFn;
use std::any::type_name_of_val;
use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Pointer};
use std::rc::Rc;
use crate::interpreter::external::External;

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
    External(Rc<RefCell<dyn External>>),
    BuiltinFunc(StackFn),
}

#[derive(Clone)]
pub struct Block {
    pub block: Rc<run_tree::Block>,
    pub captured_vars: HashMap<String, Data>,
}

impl Display for Data {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Data::String(str) => f.write_fmt(format_args!("'{str}'")),
            Data::Integer(int) => f.write_fmt(format_args!("{int}")),
            Data::Decimal(dec) => f.write_fmt(format_args!("{dec}f")),
            Data::List(l) => {
                f.write_fmt(format_args!("[ "))?;
                for d in l.iter().rev() {
                    f.write_fmt(format_args!("{d} "))?;
                }
                f.write_fmt(format_args!("]"))
            }
            Data::Dict(d) => {
                f.write_fmt(format_args!("{{ "))?;
                for (k, v) in d {
                    f.write_fmt(format_args!("({k}):({v}) "))?;
                }
                f.write_fmt(format_args!("}}"))
            }
            Data::Box(b) => f.write_fmt(format_args!("box {:p}", b.as_ptr())),
            Data::Block(b) => f.write_fmt(format_args!("block {:p}", b.block.as_ref())),
            Data::Fn(b) => f.write_fmt(format_args!("fn {:p}", b.block.as_ref())),
            Data::External(e) => f.write_fmt(format_args!("external {:p}",e.as_ref())),
            Data::BuiltinFunc(b) => f.write_fmt(format_args!("built-in {:p}", b)),
        }
    }
}
