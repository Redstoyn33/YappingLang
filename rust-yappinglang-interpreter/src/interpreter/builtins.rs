use crate::interpreter::data::Data;
use crate::interpreter::run_tree::Block;
use crate::interpreter::BlockExec;
use crate::utils::{print_stack, OptionToString};
use std::collections::HashMap;
use std::rc::Rc;

macro_rules! error {
    ($func:ident, $msg:expr) => {
        return Err(format!(
            "builtin error func '{}' line {}: {}",
            stringify!($func),
            line!(),
            $msg
        ));
    };
    ($msg:expr) => {
        return Err(format!("builtin error line {}: {}", line!(), $msg));
    };
}

macro_rules! s {
    ($opt:expr) => {
        if let Some(v) = $opt {
            v
        } else {
            return error!("empty stack when pop");
        }
    };
}

macro_rules! func {
    ($defs:ident, $name:literal, $func:ident) => {
        $defs.insert($name.to_string(), Data::BuiltinFunc($func));
    };
}

macro_rules! binary_op_int {
    ($defs:ident, $op:tt) => {
        $defs.insert(stringify!($op).to_string(), Data::BuiltinFunc(|a,b,c,d| integer_binary_op(|i1,i2| i1 $op i2,a,b,c,d)));
    };
    ($defs:ident, $op:tt, $name:literal) => {
        $defs.insert($name.to_string(), Data::BuiltinFunc(|a,b,c,d| integer_binary_op(|i1,i2| i1 $op i2,a,b,c,d)));
    };
}

macro_rules! binary_op_num {
    ($defs:ident, $op:tt) => {
        $defs.insert(stringify!($op).to_string(), Data::BuiltinFunc(|a,b,c,d| number_binary_op(|i1,i2| i1 $op i2,|d1,d2| d1 $op d2,a,b,c,d)));
    };
    ($defs:ident, $op:tt, $name:literal) => {
        $defs.insert($name.to_string(), Data::BuiltinFunc(|a,b,c,d| integer_binary_op(|i1,i2| if i1 $op i2 {1} else {0},a,b,c,d)));
    }
}

macro_rules! binary_op_bool {
    ($defs:ident, $op:tt) => {
        $defs.insert(stringify!($op).to_string(), Data::BuiltinFunc(|a,b,c,d| integer_binary_op(|i1,i2| if i1 $op i2 {1} else {0},a,b,c,d)));
    };
    ($defs:ident, $op:tt, $name:literal) => {
        $defs.insert($name.to_string(), Data::BuiltinFunc(|a,b,c,d| integer_binary_op(|i1,i2| if i1 $op i2 {1} else {0},a,b,c,d)));
    }
}

macro_rules! unary_op_num {
    ($defs:ident, $op:tt) => {
        $defs.insert(stringify!($op).to_string(), Data::BuiltinFunc(|a,b,c,d| number_unary_op(|i| $op i,|d| $op d,a,b,c,d)));
    };
    ($defs:ident, $op:tt, $name:literal) => {
        $defs.insert($name.to_string(), Data::BuiltinFunc(|a,b,c,d| number_unary_op(|i| $op i,|d| $op d,a,b,c,d)));
    };
}

pub fn base(mut defs: HashMap<String, Data>) -> HashMap<String, Data> {
    func!(defs, "Def-fn", def_fn);
    func!(defs, "Def", def);
    func!(defs, "If", _if);
    func!(defs, "Do", _do);
    func!(defs, "Print", print);
    binary_op_num!(defs, +);
    binary_op_num!(defs, -);
    binary_op_num!(defs, *);
    binary_op_num!(defs, /);
    binary_op_num!(defs, %);
    binary_op_int!(defs, &, "And");
    binary_op_int!(defs, ^, "Xor");
    binary_op_int!(defs, |, "Or");
    binary_op_bool!(defs, ==);
    binary_op_bool!(defs, !=);
    binary_op_bool!(defs, <);
    binary_op_bool!(defs, <=);
    binary_op_bool!(defs, >);
    binary_op_bool!(defs, >=);
    binary_op_bool!(defs, >=);
    unary_op_num!(defs, -, "Neg");
    defs
}

//fn def_fn(stack: &mut Vec<Data>, blocks: &mut Vec<Rc<Block>>, prev_defs: &HashMap<String,Data>, block_exec: &mut BlockExec) -> Result<(), String> {}
fn def_fn(
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let name = s!(stack.pop());
    if let Data::String(name) = (name) {
        let block = s!(stack.pop());
        if let Data::Block(block) = (block) {
            block_exec.defs.insert(name, Data::Fn(block));
        } else {
            error!(def_fn, format!("expect Block, found {}", block));
        }
    } else {
        error!(def_fn, format!("expect String, found {}", name));
    }
    Ok(())
}
fn def(
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let name = s!(stack.pop());
    if let Data::String(name) = (name) {
        let data = s!(stack.pop());
        block_exec.defs.insert(name, data);
    } else {
        error!(def, format!("expect String, found {}", name));
    }
    Ok(())
}

fn _if(
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let bool = s!(stack.pop());
    let true_data = s!(stack.pop());
    let false_data = s!(stack.pop());
    match bool {
        Data::Integer(bool) => {
            if bool != 0 {
                stack.push(true_data);
            } else {
                stack.push(false_data);
            }
        }
        Data::Block(block) => {
            block_exec.new_and_run(stack, blocks, prev_defs, &block)?;
            let bool = s!(stack.pop());
            if let Data::Integer(bool) = (bool) {
                if bool != 0 {
                    stack.push(true_data);
                } else {
                    stack.push(false_data);
                }
            } else {
                error!(_if, format!("expect Integer from Block, found {}", bool));
            }
        }
        _ => {
            error!(_if, format!("expect Integer or Block, found {}", bool));
        }
    }
    Ok(())
}

fn number_binary_op(
    int_op: fn(i64, i64) -> i64,
    dec_op: fn(f64, f64) -> f64,
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let num1 = s!(stack.pop());
    let num2 = s!(stack.pop());
    match (num1, num2) {
        (Data::Integer(i1), Data::Integer(i2)) => stack.push(Data::Integer(int_op(i1, i2))),
        (Data::Decimal(d1), Data::Integer(i2)) => stack.push(Data::Decimal(dec_op(d1, i2 as f64))),
        (Data::Integer(i1), Data::Decimal(d2)) => stack.push(Data::Decimal(dec_op(i1 as f64, d2))),
        (Data::Decimal(d1), Data::Decimal(d2)) => stack.push(Data::Decimal(dec_op(d1, d2))),
        (n1, n2) => {
            return error!(
                number_binary_op,
                format!("expected 2 Integer or Decimal, found {} {}", n1, n2)
            );
        }
    }
    Ok(())
}

fn integer_binary_op(
    int_op: fn(i64, i64) -> i64,
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let num1 = s!(stack.pop());
    let num2 = s!(stack.pop());
    match (num1, num2) {
        (Data::Integer(i1), Data::Integer(i2)) => stack.push(Data::Integer(int_op(i1, i2))),
        (n1, n2) => {
            return error!(
                integer_binary_op,
                format!("expected 2 Integer or Decimal, found {} {}", n1, n2)
            );
        }
    }
    Ok(())
}

fn number_unary_op(
    int_op: fn(i64) -> i64,
    dec_op: fn(f64) -> f64,
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let num1 = s!(stack.pop());
    match num1 {
        Data::Integer(i1) => stack.push(Data::Integer(int_op(i1))),
        Data::Decimal(d1) => stack.push(Data::Decimal(dec_op(d1))),
        n1 => {
            return error!(
                number_unary_op,
                format!("expected Integer or Decimal, found {}", n1)
            );
        }
    }
    Ok(())
}

fn _do(
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let block = s!(stack.pop());
    if let Data::Block(block) = (block) {
        block_exec.new_and_run(stack,blocks,prev_defs,&block)?;
    } else {
        error!(_do, format!("expect Block, found {}", block));
    }
    Ok(())
}

fn print(
    stack: &mut Vec<Data>,
    blocks: &mut Vec<Rc<Block>>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let data = s!(stack.pop());
    println!("{data}");
    Ok(())
}