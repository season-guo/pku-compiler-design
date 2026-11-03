// Exp         ::= UnaryExp;
// PrimaryExp  ::= "(" Exp ")" | Number;
// Number      ::= INT_CONST;
// UnaryExp    ::= PrimaryExp | UnaryOp UnaryExp;
// UnaryOp     ::= "+" | "-" | "!";

#[derive(Debug)]
pub enum AExp {
    ANum(i32),
    AUnAry(UnAryOp, Box<AExp>),
}

#[derive(Debug)]
pub enum UnAryOp {
    APlus,
    Minus,
    Not,
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