#[derive(Debug, Clone)]
pub enum AExp {
    ANum(i32),
    AVal(String),
    AUnAry(UnAryOp, Box<AExp>),
    ABinary(ABinaryOp, Box<AExp>, Box<AExp>),
    AFunc(String, Vec<AExp>),
}

#[derive(Debug, Clone)]
pub enum UnAryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug, Clone)]
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
