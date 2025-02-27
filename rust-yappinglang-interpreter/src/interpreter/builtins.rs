use std::cell::RefCell;
use crate::interpreter::data::Data;
use crate::interpreter::BlockExec;
use crate::utils::{print_stack, ResultToString};
use std::collections::{HashMap, VecDeque};
use std::io::Read;
use std::mem;
use std::ops::Add;
use crate::interpreter::external::External;
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
    ($defs:ident, $prefix:ident, $name:literal, $func:ident) => {
        $defs.insert(format!("{}{}", $prefix, $name), Data::BuiltinFunc($func));
    };
    ($defs:ident, $prefix:ident, $name:literal, $func:expr) => {
        $defs.insert(format!("{}{}", $prefix, $name), Data::BuiltinFunc($func));
    };
    ($defs:ident, $prefix:ident, $name:expr, $func:ident) => {
        $defs.insert(format!("{}{}", $prefix, $name), Data::BuiltinFunc($func));
    };
    ($defs:ident, $prefix:ident, $name:expr, $func:expr) => {
        $defs.insert(format!("{}{}", $prefix, $name), Data::BuiltinFunc($func));
    };
}

macro_rules! binary_op_int {
    ($defs:ident, $prefix:ident, $op:tt) => {
        func!($defs, $prefix, stringify!($op).to_string(), |a,b,c| integer_binary_op(|i1,i2| i1 $op i2,a,b,c));
    };
    ($defs:ident, $prefix:ident, $op:tt, $name:literal) => {
        func!($defs, $prefix, $name.to_string(), |a,b,c| integer_binary_op(|i1,i2| i1 $op i2,a,b,c));
    };
}

macro_rules! binary_op_num {
    ($defs:ident, $prefix:ident, $op:tt) => {
        func!($defs, $prefix, stringify!($op).to_string(), |a,b,c| number_binary_op(|i1,i2| i1 $op i2,|d1,d2| d1 $op d2,a,b,c));
    };
    ($defs:ident, $prefix:ident, $op:tt, $name:literal) => {
        func!($defs, $prefix, $name.to_string(), |a,b,c| number_binary_op(|i1,i2| i1 $op i2,|d1,d2| d1 $op d2,a,b,c));
    };
}

macro_rules! binary_op_bool {
    ($defs:ident, $prefix:ident, $op:tt) => {
        func!($defs, $prefix, stringify!($op).to_string(), |a,b,c| integer_binary_op(|i1,i2| if i1 $op i2 {1} else {0},a,b,c));
    };
    ($defs:ident, $prefix:ident, $op:tt, $name:literal) => {
        func!($defs, $prefix, $name.to_string(), |a,b,c| integer_binary_op(|i1,i2| if i1 $op i2 {1} else {0},a,b,c));
    }
}

macro_rules! unary_op_num {
    ($defs:ident, $prefix:ident, $op:tt) => {
        func!($defs, $prefix, stringify!($op).to_string(), |a,b,c| number_unary_op(|i| $op i,|d| $op d,a,b,c));
    };
    ($defs:ident, $prefix:ident, $op:tt, $name:literal) => {
        func!($defs, $prefix, $name.to_string(), |a,b,c| number_unary_op(|i| $op i,|d| $op d,a,b,c));
    };
}

macro_rules! type_assert {
    ($defs:ident, $prefix:ident, $typ:ident) => {
        func!(
            $defs,
            $prefix,
            format!("{}!", stringify!($typ)),
            |a, b, c| {
                let d = s!(a.last());
                if let Data::$typ(_) = d {
                } else {
                    error!(
                        type_assert,
                        format!("expect {}, found {}", stringify!($typ), d)
                    );
                }
                Ok(())
            }
        );
    };
}

macro_rules! type_check {
    ($defs:ident, $prefix:ident, $typ:ident) => {
        func!(
            $defs,
            $prefix,
            format!("{}!", stringify!($typ)),
            |a, b, c| {
                let d = s!(a.pop());
                if let Data::$typ(_) = d {
                    a.push(Data::Integer(1));
                } else {
                    a.push(Data::Integer(0));
                }
                Ok(())
            }
        );
    };
}

pub fn base(mut defs: HashMap<String, Data>, prefix: &str) -> HashMap<String, Data> {
    func!(defs, prefix, "Def-fn", def_fn);
    func!(defs, prefix, "Def", def);
    func!(defs, prefix, "If", _if);
    func!(defs, prefix, "Do", _do);
    func!(defs, prefix, "Do-local", _do_local);
    func!(defs, prefix, "Get-fn", get_fn);
    func!(defs, prefix, "Get-def", get_def);
    func!(defs, prefix, "Has-def", has_def);
    func!(defs, prefix, "Print", print);
    func!(defs, prefix, "Debug", debug);
    func!(defs, prefix, "Timer", timer);
    func!(defs, prefix, "Assert", assert);
    func!(defs, prefix, "Use-file", use_file);
    func!(defs, prefix, "Try-use-file", try_use_file);
    func!(defs, prefix, "Any!", any_assert);
    func!(defs, prefix, "Any?", any_check);
    func!(defs, prefix, "Drop", drop);
    func!(defs, prefix, "New-ext", new_external);
    func!(defs, prefix, "Use-ext", use_external);

    type_assert!(defs, prefix, String);
    type_assert!(defs, prefix, Integer);
    type_assert!(defs, prefix, Decimal);
    type_assert!(defs, prefix, List);
    type_assert!(defs, prefix, Dict);
    type_assert!(defs, prefix, Box);
    type_assert!(defs, prefix, Block);
    type_assert!(defs, prefix, Fn);
    type_assert!(defs, prefix, External);

    type_check!(defs, prefix, String);
    type_check!(defs, prefix, Integer);
    type_check!(defs, prefix, Decimal);
    type_check!(defs, prefix, List);
    type_check!(defs, prefix, Dict);
    type_check!(defs, prefix, Box);
    type_check!(defs, prefix, Block);
    type_check!(defs, prefix, Fn);
    type_check!(defs, prefix, External);

    func!(defs, prefix, "To-integer", to_integer);
    func!(defs, prefix, "To-decimal", to_decimal);
    func!(defs, prefix, "To-string", to_string);
    func!(defs, prefix, "To-list", to_list);

    func!(defs, prefix, "String-join", string_join);
    func!(defs, prefix, "String-chars", string_chars);

    func!(defs, prefix, "List", list);
    func!(defs, prefix, "List-reverse", list_reverse);
    func!(defs, prefix, "List-stack", list_stack);

    defs
}
pub fn math_and_logic(mut defs: HashMap<String, Data>, prefix: &str) -> HashMap<String, Data> {
    binary_op_num!(defs, prefix, +);
    binary_op_num!(defs, prefix, -);
    binary_op_num!(defs, prefix, *);
    binary_op_num!(defs, prefix, /);
    binary_op_num!(defs, prefix, %);
    binary_op_int!(defs, prefix, &, "And");
    binary_op_int!(defs, prefix, ^, "Xor");
    binary_op_int!(defs, prefix, |, "Or");
    binary_op_bool!(defs, prefix, ==);
    binary_op_bool!(defs, prefix, !=);
    binary_op_bool!(defs, prefix, <);
    binary_op_bool!(defs, prefix, <=);
    binary_op_bool!(defs, prefix, >);
    binary_op_bool!(defs, prefix, >=);
    binary_op_bool!(defs, prefix, >=);
    unary_op_num!(defs, prefix, -, "Neg");
    func!(defs, prefix, "To-bool", |a,b,c| integer_unary_op(|n| if n==0 {0} else {1},a,b,c));
    func!(defs, prefix, "Not", |a,b,c| integer_unary_op(|n| if n==0 {1} else {0},a,b,c));
    defs.insert(format!("{}True", prefix), Data::Integer(1));
    defs.insert(format!("{}False", prefix), Data::Integer(0));
    defs
}

//fn def_fn(stack: &mut Vec<Data>, blocks: &mut Vec<Rc<Block>>, prev_defs: &HashMap<String,Data>, block_exec: &mut BlockExec) -> Result<(), String> {}
fn def_fn(
    stack: &mut Vec<Data>,
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
            block_exec.new_and_run(stack, prev_defs, &block)?;
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
            error!(
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
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let num1 = s!(stack.pop());
    let num2 = s!(stack.pop());
    match (num1, num2) {
        (Data::Integer(i1), Data::Integer(i2)) => stack.push(Data::Integer(int_op(i1, i2))),
        (n1, n2) => {
            error!(
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
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let num1 = s!(stack.pop());
    match num1 {
        Data::Integer(i1) => stack.push(Data::Integer(int_op(i1))),
        Data::Decimal(d1) => stack.push(Data::Decimal(dec_op(d1))),
        n1 => {
            error!(
                number_unary_op,
                format!("expected Integer or Decimal, found {}", n1)
            );
        }
    }
    Ok(())
}
fn integer_unary_op(
    int_op: fn(i64) -> i64,
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let num1 = s!(stack.pop());
    match num1 {
        Data::Integer(i1) => stack.push(Data::Integer(int_op(i1))),
        n1 => {
            error!(
                integer_unary_op,
                format!("expected Integer or Decimal, found {}", n1)
            );
        }
    }
    Ok(())
}

fn _do(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let block = s!(stack.pop());
    if let Data::Block(block) = (block) {
        block_exec.new_and_run(stack, prev_defs, &block)?;
    } else {
        error!(_do, format!("expect Block, found {}", block));
    }
    Ok(())
}

fn _do_local(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let block = s!(stack.pop());
    if let Data::Block(block) = (block) {
        block_exec.run_block(stack, prev_defs, block.block)?;
    } else {
        error!(_do_local, format!("expect Block, found {}", block));
    }
    Ok(())
}
fn get_fn(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let fn_name = s!(stack.pop());
    if let Data::String(fn_name) = (fn_name) {
        let func = s!(block_exec.get_data(&fn_name, prev_defs));
        if let Data::Fn(func) = func {
            stack.push(Data::Block(func));
        } else {
            error!(
                get_fn,
                format!("expect def-fn {} as Fn, found {}", fn_name, func)
            );
        }
    } else {
        error!(get_fn, format!("expect String, found {}", fn_name));
    }
    Ok(())
}

fn get_def(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let def_name = s!(stack.pop());
    if let Data::String(def_name) = (def_name) {
        let data = s!(block_exec.get_data(&def_name, prev_defs));
        stack.push(data);
    } else {
        error!(get_def, format!("expect String, found {}", def_name));
    }
    Ok(())
}

fn has_def(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let def_name = s!(stack.pop());
    if let Data::String(def_name) = (def_name) {
        if block_exec.get_data(&def_name, prev_defs).is_some() {
            stack.push(Data::Integer(1));
        } else {
            stack.push(Data::Integer(0));
        }
    } else {
        error!(get_def, format!("expect String, found {}", def_name));
    }
    Ok(())
}

fn timer(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let block = s!(stack.pop());
    if let Data::Block(block) = (block) {
        let start = std::time::Instant::now();
        block_exec.new_and_run(stack, prev_defs, &block)?;
        let duration = std::time::Instant::now().duration_since(start);
        println!("timer: {}s", duration.as_secs_f64());
    } else {
        error!(timer, format!("expect Block, found {}", block));
    }
    Ok(())
}

fn print(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let data = s!(stack.pop());
    println!("{data}");
    Ok(())
}

fn debug(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    print_stack(stack);
    Ok(())
}

fn assert(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let bool = s!(stack.pop());
    match bool {
        Data::Integer(bool) => {
            if bool == 0 {
                error!(assert, "assert");
            }
        }
        _ => {
            error!(assert, format!("expect Integer, found {}", bool));
        }
    }
    Ok(())
}

fn use_file(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let file = s!(stack.pop());
    if let Data::String(file) = (file) {
        let mut file = std::fs::File::open(std::path::Path::new(&file)).str_res()?;
        let mut source = String::new();
        file.read_to_string(&mut source).str_res()?;
        let mut scanner = crate::scanner::Scanner::new(&source);
        let tokens = scanner.scan_tokens()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let ast = parser.build_tree()?;
        let block = crate::interpreter::run_tree::load(&ast);
        let captured_vars = block_exec.capture(&block.capture_vars, prev_defs)?;
        stack.push(Data::Block(crate::interpreter::data::Block {
            block,
            captured_vars,
        }));
    } else {
        error!(use_file, format!("expect String, found {}", file));
    }
    Ok(())
}
fn try_use_file(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let file = s!(stack.pop());
    if let Data::String(file) = (file) {
        if let Ok(mut file) = std::fs::File::open(std::path::Path::new(&file)).str_res() {
            let mut source = String::new();
            if file.read_to_string(&mut source).str_res().is_ok() {
                let mut scanner = crate::scanner::Scanner::new(&source);
                if let Ok(tokens) = scanner.scan_tokens() {
                    let mut parser = crate::parser::Parser::new(tokens);
                    if let Ok(ast) = parser.build_tree() {
                        let block = crate::interpreter::run_tree::load(&ast);
                        if let Ok(captured_vars) =
                            block_exec.capture(&block.capture_vars, prev_defs)
                        {
                            stack.push(Data::Block(crate::interpreter::data::Block {
                                block,
                                captured_vars,
                            }));
                            stack.push(Data::Integer(1));
                            return Ok(());
                        }
                    }
                }
            }
        }
        stack.push(Data::Integer(0));
    } else {
        error!(try_use_file, format!("expect String, found {}", file));
    }
    Ok(())
}

fn any_assert(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    if stack.len() == 0 {
        error!(any_assert, "No data on stack");
    }
    Ok(())
}
fn any_check(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    if stack.len() == 0 {
        stack.push(Data::Integer(0));
    } else {
        stack.push(Data::Integer(1));
    }
    Ok(())
}
fn drop(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    s!(stack.pop());
    Ok(())
}
fn to_integer(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let d = s!(stack.pop());
    match d {
        Data::String(d) => {
            stack.push(Data::Integer(d.parse::<i64>().str_res()?));
        }
        Data::Integer(d) => {stack.push(Data::Integer(d))}
        Data::Decimal(d) => {stack.push(Data::Integer(d as i64))}
        _ => {error!(to_integer, format!("expect String or Integer or Decimal, found {}", d));}
    }
    Ok(())
}
fn to_decimal(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let d = s!(stack.pop());
    match d {
        Data::String(d) => {
            stack.push(Data::Decimal(d.parse::<f64>().str_res()?));
        }
        Data::Integer(d) => {stack.push(Data::Decimal(d as f64))}
        Data::Decimal(d) => {stack.push(Data::Decimal(d))}
        _ => {error!(to_decimal, format!("expect String or Integer or Decimal, found {}", d));}
    }
    Ok(())
}

fn to_string(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let d = s!(stack.pop());
    stack.push(Data::String(d.to_string()));
    Ok(())
}

fn to_list(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let d = s!(stack.pop());
    stack.push(Data::List(vec![d]));
    Ok(())
}
fn new_external(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let name = s!(stack.pop());
    if let Data::String(name) = (name) {
        let ext = crate::interpreter::external::new_external(name)?;
        stack.push(Data::External(ext));
    } else {
        error!(new_external, format!("expect String, found {}", name));
    }
    Ok(())
}
fn use_external(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let func = s!(stack.pop());
    if let Data::String(func) = (func) {
        let ext = s!(stack.pop());
        if let Data::External(mut ext) = ext {
            ext.borrow_mut().apply(func,stack)?;
        } else {
            error!(use_external, format!("expect External, found {}", ext));
        }
    } else {
        error!(use_external, format!("expect String, found {}", func));
    }
    Ok(())
}
fn string_join(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let s1 = s!(stack.pop());
    if let Data::String(s1) = (s1) {
        let s2 = s!(stack.pop());
        if let Data::String(s2) = (s2) {
            stack.push(Data::String(s1.add(&s2)));
        } else {
            error!(string_join, format!("expect String, found {}", s2));
        }
    } else {
        error!(string_join, format!("expect String, found {}", s1));
    }
    Ok(())
}
fn string_chars(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let s = s!(stack.pop());
    if let Data::String(s) = (s) {
        stack.push(Data::List(s.chars().rev().map(|c|Data::String(c.to_string())).collect()))
    } else {
        error!(string_chars, format!("expect String, found {}", s));
    }
    Ok(())
}

fn list(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let block = s!(stack.pop());
    if let Data::Block(block) = (block) {
        let mut new_stack = Vec::new();
        block_exec.new_and_run(&mut new_stack, prev_defs, &block)?;
        stack.push(Data::List(new_stack));
    } else {
        error!(list, format!("expect Block, found {}", block));
    }
    Ok(())
}


fn list_reverse(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let list = s!(stack.pop());
    if let Data::List(mut list) = (list) {
        list.reverse();
        stack.push(Data::List(list));
    } else {
        error!(list_reverse, format!("expect List, found {}", list));
    }
    Ok(())
}


fn list_stack(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let mut list = Vec::new();
    mem::swap(stack,&mut list);
    stack.push(Data::List(list));
    Ok(())
}

fn list_swap_stack(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let list = s!(stack.pop());
    if let Data::List(mut list) = list {
        mem::swap(stack,&mut list);
        stack.push(Data::List(list));
    } else {
        error!(list_to_stack, format!("expect List, found {}", list));
    }
    Ok(())
}


fn list_to_stack(
    stack: &mut Vec<Data>,
    prev_defs: &HashMap<String, Data>,
    block_exec: &mut BlockExec,
) -> Result<(), String> {
    let list = s!(stack.pop());
    if let Data::List(mut list) = list {
        for data in list.into_iter().rev() {
            stack.push(data);
        }
    } else {
        error!(list_to_stack, format!("expect List, found {}", list));
    }
    Ok(())
}
