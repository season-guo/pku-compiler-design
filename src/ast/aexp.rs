#[derive(Debug, Clone)]
pub enum AExp {
    ANum(i32),
    AVal(String),
    AUnAry(UnAryOp, Box<AExp>),
    ABinary(ABinaryOp, Box<AExp>, Box<AExp>),
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

impl AExp {
    pub fn calcul_bef_compile(self: Self) -> Self {
        match self {
            Self::ANum(num) => Self::ANum(num),
            Self::AVal(_) => panic!("decl a const, while val in its val"),
            Self::AUnAry(op, aexp) => {
                let simpl_aexp = aexp.calcul_bef_compile();
                if let Self::ANum(num) = simpl_aexp {
                    match op {
                        UnAryOp::Plus => Self::ANum(num),
                        UnAryOp::Minus => Self::ANum(-num),
                        UnAryOp::Not => Self::ANum(if num == 0 { 1 } else { 0 }),
                    }
                } else {
                    panic!("decl a const, while it can't be cacul completely");
                }
            }
            Self::ABinary(op, lhs, rhs) => {
                let simpl_lhs = lhs.calcul_bef_compile();
                let simpl_rhs = rhs.calcul_bef_compile();
                if let (Self::ANum(lhs), Self::ANum(rhs)) = (simpl_lhs, simpl_rhs) {
                    match op {
                        ABinaryOp::Plus => Self::ANum(lhs + rhs),
                        ABinaryOp::Minus => Self::ANum(lhs - rhs),
                        ABinaryOp::Mul => Self::ANum(lhs * rhs),
                        ABinaryOp::Mod => Self::ANum(lhs % rhs),
                        ABinaryOp::Div => Self::ANum(lhs / rhs),
                        ABinaryOp::Or => Self::ANum((lhs != 0 || rhs != 0).into()),
                        ABinaryOp::And => Self::ANum((lhs != 0 && rhs != 0).into()),
                        ABinaryOp::Eq => Self::ANum((lhs == rhs).into()),
                        ABinaryOp::Neq => Self::ANum((lhs != rhs).into()),
                        ABinaryOp::Lt => Self::ANum((lhs < rhs).into()),
                        ABinaryOp::Le => Self::ANum((lhs <= rhs).into()),
                        ABinaryOp::Gt => Self::ANum((lhs > rhs).into()),
                        ABinaryOp::Ge => Self::ANum((lhs >= rhs).into()),
                    }
                } else {
                    panic!("decl a const, while it can't be cacul completely");
                }
            }
        }
    }
}
