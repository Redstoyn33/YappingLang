use std::cell::{Cell, RefCell, RefMut, UnsafeCell};
use std::io::BufRead;
use std::ops::{Add, AddAssign};
use std::rc::Rc;
use crate::interpreter::data::Data;

pub trait External {
    fn apply(&mut self, func: String, stack: &mut Vec<Data>) -> Result<(),String>;
    fn name(&mut self) -> String;
}

pub fn new_external(name: String) -> Result<Rc<RefCell<dyn External>>,String> {
    todo!()
}
