#[derive(Debug)]
pub enum AExp {
    ANum(i32),
    AUnAry(UnAryOp, Box<AExp>),
    ABinary(ABinaryOp, Box<AExp>, Box<AExp>),
}

#[derive(Debug)]
pub enum UnAryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub enum ABinaryOp {
    Plus,
    Minus,
    Mul,
    Mod,
    Div,
    Or,
    And,
    Eq,
    Neq,
    Lt,
    Le,
    Gt,
    Ge,
}
