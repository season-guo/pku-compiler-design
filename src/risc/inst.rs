#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Register {
    X0, //恒为0的寄存器
    X1, //返回地址
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6, //临时寄存器
    A0, //函数返回值
}

#[derive(Debug)]
pub enum Inst {
    //这不是个指令，只是用来兼容ValueKind
    Int(i32),
    Li(Register, i32),
    Mv(Register, Register),
    Seqz(Register, Register),
    SNez(Register, Register),
    Add(Register, Register, Register),
    Sub(Register, Register, Register),
    Mul(Register, Register, Register),
    Div(Register, Register, Register),
    Rem(Register, Register, Register),
    Or(Register, Register, Register),
    And(Register, Register, Register),
    SLt(Register, Register, Register),
    SGt(Register, Register, Register),
    Xor(Register, Register, Register),
    Ret,
}

impl ToString for Register {
    fn to_string(&self) -> String {
        match self {
            Self::X0 => "x0",
            Self::X1 => "x1",
            Self::T0 => "t0",
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::T3 => "t3",
            Self::T4 => "t4",
            Self::T5 => "t5",
            Self::T6 => "t6",
            Self::A0 => "a0",
        }
        .into()
    }
}

impl Register {
    pub const TEMP_REGISTER: [Self; 7] = [
        Self::T0,
        Self::T1,
        Self::T2,
        Self::T3,
        Self::T4,
        Self::T5,
        Self::T6,
    ];
    pub fn is_temp(self: &Self) -> bool {
        Self::TEMP_REGISTER.contains(self)
    }

    pub fn which_to_reuse(self: &Self, other: &Self) -> Self {
        if !self.is_temp() {
            other.clone()
        } else {
            self.clone()
        }
    }
}

impl ToString for Inst {
    fn to_string(&self) -> String {
        match self {
            Self::Int(num) => num.to_string(),
            Self::Li(rd, imm) => format!("  li {}, {imm}\n", rd.to_string()),
            Self::Mv(rd, rs) => format!("  mv {}, {}\n", rd.to_string(), rs.to_string()),
            Self::Add(rd, rs1, rs2) => format!(
                "  add {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::Sub(rd, rs1, rs2) => format!(
                "  sub {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::Mul(rd, rs1, rs2) => format!(
                "  mul {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::Div(rd, rs1, rs2) => format!(
                "  div {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::Rem(rd, rs1, rs2) => format!(
                "  rem {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::Or(rd, rs1, rs2) => format!(
                "  or {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::And(rd, rs1, rs2) => format!(
                "  and {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::SLt(rd, rs1, rs2) => format!(
                "  slt {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::SGt(rd, rs1, rs2) => format!(
                "  sgt {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::Xor(rd, rs1, rs2) => format!(
                "  xor {}, {}, {}\n",
                rd.to_string(),
                rs1.to_string(),
                rs2.to_string()
            ),
            Self::Seqz(rd, rs) => format!("  seqz {}, {}\n", rd.to_string(), rs.to_string()),
            Self::SNez(rd, rs) => format!("  snez {}, {}\n", rd.to_string(), rs.to_string()),
            Self::Ret => format!("  ret\n"),
        }
    }
}
