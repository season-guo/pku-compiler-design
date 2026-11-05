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
    SP, //栈指针
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
    Lw(Register, Register, i32),
    Sw(Register, Register, i32),
    Addi(Register, Register, i32),
    Ret,
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
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
            Self::SP => "sp",
        };
        f.write_str(s)
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
        if !self.is_temp() { *other } else { *self }
    }
}

impl ToString for Inst {
    fn to_string(&self) -> String {
        match self {
            Self::Int(num) => num.to_string(),
            Self::Li(rd, imm) => format!("  li {}, {imm}", rd),
            Self::Mv(rd, rs) => format!("  mv {}, {}", rd, rs),
            Self::Add(rd, rs1, rs2) => format!("  add {}, {}, {}", rd, rs1, rs2),
            Self::Sub(rd, rs1, rs2) => format!("  sub {}, {}, {}", rd, rs1, rs2),
            Self::Mul(rd, rs1, rs2) => format!("  mul {}, {}, {}", rd, rs1, rs2),
            Self::Div(rd, rs1, rs2) => format!("  div {}, {}, {}", rd, rs1, rs2),
            Self::Rem(rd, rs1, rs2) => format!("  rem {}, {}, {}", rd, rs1, rs2),
            Self::Or(rd, rs1, rs2) => format!("  or {}, {}, {}", rd, rs1, rs2),
            Self::And(rd, rs1, rs2) => format!("  and {}, {}, {}", rd, rs1, rs2),
            Self::SLt(rd, rs1, rs2) => format!("  slt {}, {}, {}", rd, rs1, rs2),
            Self::SGt(rd, rs1, rs2) => format!("  sgt {}, {}, {}", rd, rs1, rs2),
            Self::Xor(rd, rs1, rs2) => format!("  xor {}, {}, {}", rd, rs1, rs2),
            Self::Seqz(rd, rs) => format!("  seqz {}, {}", rd, rs),
            Self::SNez(rd, rs) => format!("  snez {}, {}", rd, rs),
            Self::Lw(rs, rd, imm) => format!("  lw {rs}, {imm}({rd})"),
            Self::Sw(rs2, rs1, imm) => format!("  sw {rs2}, {imm}({rs1})"),
            Self::Addi(rd, rs, imm) => format!("  addi {rd}, {rs}, {imm}"),
            Self::Ret => "  ret".to_string(),
        }
    }
}
