use std::{
    collections::{HashMap, VecDeque},
    io::Write,
};

use koopa::ir::{BinaryOp, FunctionData, Program, Value, ValueKind, entities::ValueData};

use crate::risc::inst::{Inst, Register};

pub struct RiscVGenerator<'a> {
    pub program: &'a Program, // 本来这里最好直接move但是test不方便...或者换 rc 可能好点，反正写了这个了都...
    pub value_map: HashMap<Value, Register>,
    pub spare_registers: VecDeque<Register>,
}

impl<'a> RiscVGenerator<'a> {
    pub fn new(program: &'a Program) -> Self {
        RiscVGenerator {
            program,
            value_map: HashMap::new(),
            spare_registers: VecDeque::from(Register::TEMP_REGISTER),
        }
    }
    pub fn koopa_to_risc(self: &mut Self, mut writer: impl Write) {
        let mut prefix: String = " .text\n .global main\n".into();
        let program = self.program;
        for func in program.func_layout() {
            let func_data = program.func(*func);
            prefix += &self.koopa_function_to_risc(func_data);
        }

        let _ = writer.write(&prefix.into_bytes());
    }

    fn koopa_function_to_risc(self: &mut Self, function_data: &FunctionData) -> String {
        let mut prefix = "main:\n".to_owned();
        for (_, node) in function_data.layout().bbs() {
            for (val, _) in node.insts() {
                prefix += &self
                    .koopa_inst_to_risc(function_data, val)
                    .into_iter()
                    .map(|inst| inst.to_string())
                    .reduce(|s1, s2| s1 + &s2)
                    .unwrap()
                    .as_str();
            }
        }

        prefix
    }

    fn imm_to_register(
        self: &mut Self,
        val: &Value,
        val_data: &ValueData,
        can_use_x0: bool,
    ) -> (Vec<Inst>, Register) {
        if let ValueKind::Integer(num) = val_data.kind() {
            if num.value() == 0 && can_use_x0 {
                return (vec![], Register::X0);
            }

            let register = self.spare_registers.pop_front().unwrap();
            self.value_map.insert(val.clone(), register.clone());
            (vec![Inst::Li(register.clone(), num.value())], register)
        } else {
            (vec![], self.value_map.get(val).unwrap().clone())
        }
    }

    fn koopa_inst_to_risc(self: &mut Self, function_data: &FunctionData, val: &Value) -> Vec<Inst> {
        let val_data = function_data.dfg().value(*val);
        // println!("{:#?} \n {:#?} \n\n {:#?}\n\n\n\n\n", val, val_data, self.value_map);
        match val_data.kind() {
            ValueKind::Integer(num) => vec![Inst::Int(num.value())],
            ValueKind::Return(ret) => {
                let asgn_inst = ret
                    .value()
                    .map(|val| {
                        let val_data = function_data.dfg().value(val);
                        if let ValueKind::Integer(num) = val_data.kind() {
                            Inst::Li(Register::A0, num.value())
                        } else {
                            Inst::Mv(Register::A0, self.value_map.get(&val).unwrap().clone())
                        }
                    })
                    .unwrap_or_else(|| Inst::Li(Register::A0, 0));
                vec![asgn_inst, Inst::Ret]
            }
            ValueKind::Binary(bin) => {
                let lhs = bin.lhs();
                let lhs_data = function_data.dfg().value(lhs.clone());
                let rhs = bin.rhs();
                let rhs_data = function_data.dfg().value(rhs.clone());
                let (mut asgn_inst1, rs1) = self.imm_to_register(&lhs, lhs_data, true);
                let (asgn_inst2, rs2) =
                    self.imm_to_register(&rhs, rhs_data, !(rs1 == Register::X0));
                let reuse_register = rs1.which_to_reuse(&rs2);
                self.value_map.insert(val.clone(), reuse_register);
                asgn_inst1.extend(asgn_inst2);
                let cacul_insts = match bin.op() {
                    BinaryOp::Eq => {
                        vec![
                            Inst::Xor(reuse_register, rs1, rs2),
                            Inst::Seqz(reuse_register, reuse_register),
                        ]
                    }
                    BinaryOp::Add => {
                        vec![Inst::Add(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::Sub => {
                        vec![Inst::Sub(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::Mul => {
                        vec![Inst::Mul(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::Div => {
                        vec![Inst::Div(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::Mod => {
                        vec![Inst::Rem(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::Or => {
                        vec![Inst::Or(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::And => {
                        vec![Inst::And(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::NotEq => {
                        vec![
                            Inst::Xor(reuse_register, rs1, rs2),
                            Inst::SNez(reuse_register, reuse_register),
                        ]
                    }
                    BinaryOp::Lt => {
                        vec![Inst::SLt(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::Le => {
                        vec![
                            Inst::SGt(reuse_register, rs1, rs2),
                            Inst::Seqz(reuse_register, reuse_register),
                        ]
                    }
                    BinaryOp::Gt => {
                        vec![Inst::SGt(reuse_register, rs1, rs2)]
                    }
                    BinaryOp::Ge => {
                        vec![
                            Inst::SLt(reuse_register, rs1, rs2),
                            Inst::Seqz(reuse_register, reuse_register),
                        ]
                    }
                    _ => unreachable!(),
                };
                asgn_inst1.extend(cacul_insts);
                asgn_inst1
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Mode, koopa::koopa::build_program_from_compunit, sysy};
    use std::fs::read_to_string;
    #[test]
    fn aexp_sysy_2_riscv() {
        let input = read_to_string("test/lv3/24_landq.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let mode = Mode::RiscV;
        mode.handle(&build_program_from_compunit(ast), None);
    }
}
