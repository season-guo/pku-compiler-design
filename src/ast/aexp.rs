// Exp         ::= UnaryExp;
// PrimaryExp  ::= "(" Exp ")" | Number;
// Number      ::= INT_CONST;
// UnaryExp    ::= PrimaryExp | UnaryOp UnaryExp;
// UnaryOp     ::= "+" | "-" | "!";

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

// impl AExp {
//     pub fn calc_aexp(self: Self) -> i32 {
//         match self {
//             Self::ANum(int) => int,
//             Self::AUnAry(op, aexp) => 
//         }
//     }
// }

// impl UnAry {
//     pub fn calc_unary(self: Self) -> i32 {
//         match self {
//             Self::APlus(int) => int,
//         }
//     }
// }