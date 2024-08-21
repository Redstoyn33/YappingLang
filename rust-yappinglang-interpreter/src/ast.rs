#[derive(Debug)]
pub struct Block {
    pub exp: Option<Box<Exp>>,
}
#[derive(Debug)]
pub enum ExpData {
    Var(String),
    CapturedVar(String),
    Block(Block),
    Integer(i64),
    Decimal(f64),
    String(String),
}
#[derive(Debug)]
pub struct Exp {
    pub data: ExpData,
    pub next_exp: Option<Box<Exp>>,
}
