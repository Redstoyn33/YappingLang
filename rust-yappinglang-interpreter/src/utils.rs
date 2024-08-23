use crate::interpreter::data::Data;
use std::error::Error;

pub trait ResultToString<T, E: ToString> {
    fn str_res(self) -> Result<T, String>;
}
impl<T, E: Error> ResultToString<T, E> for Result<T, E> {
    fn str_res(self) -> Result<T, String> {
        match self {
            Ok(val) => Ok(val),
            Err(err) => Err(err.to_string()),
        }
    }
}

pub trait OptionToString<T> {
    fn str_res(self) -> Result<T, String>;
}

impl<T> OptionToString<T> for Option<T> {
    fn str_res(self) -> Result<T, String> {
        match self {
            None => Err("Option: expect Some, found None".to_string()),
            Some(v) => Ok(v),
        }
    }
}

pub fn print_stack(stack: &Vec<Data>) {
    stack.iter().rev().for_each(|d| println!("{d}"));
}
