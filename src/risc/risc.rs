use std::{
    collections::{HashMap, VecDeque},
    io::Write,
};

use koopa::ir::{BinaryOp, FunctionData, Program, Value, ValueKind, entities::ValueData};

use crate::risc::inst::{Inst, Register};

pub struct RiscVGenerator<'a> {
    pub program: &'a Program, // 本来这里最好直接move但是test不方便...或者换 rc 可能好点，反正写了这个了都...
    pub value_map: HashMap<Value, Register>,
}

impl<'a> RiscVGenerator<'a> {
    pub fn new(program: &'a Program) -> Self {
        RiscVGenerator {
            program,
            value_map: HashMap::new(),
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
        let mut space_need = 0;

        for (_, node) in function_data.layout().bbs() {
            for (val, _) in node.insts() {
                let val_data = function_data.dfg().value(*val);
                if let ValueKind::Alloc(_) = val_data.kind() {
                    space_need += 4;
                } else if !val_data.ty().is_unit() {
                    space_need += 4;
                }
            }
        }

        let space_mod = space_need % 16;
        if space_mod != 0 {
            space_need = space_need + 16 - space_mod;
        }
        prefix += &Inst::Addi(Register::SP, Register::SP, -space_need).to_string();
        let mut stack_offset = 0;
        let mut stack_map = HashMap::new();

        for (_, node) in function_data.layout().bbs() {
            for (val, _) in node.insts() {
                prefix += self
                    .koopa_inst_to_risc(function_data, val, &mut stack_offset, &mut stack_map)
                    .into_iter()
                    .map(|inst| inst.to_string())
                    .reduce(|s1, s2| s1 + &s2)
                    .unwrap_or_else(|| "".into())
                    .as_str();
                prefix += "\n"
            }
        }

        prefix
    }

    fn imm_to_register(
        self: &mut Self,
        val: &Value,
        val_data: &ValueData,
        can_use_x0: bool,
        other_register: Option<Register>,
        stack_map: &mut HashMap<Value, i32>,
    ) -> (Vec<Inst>, Register) {
        if let ValueKind::Integer(num) = val_data.kind() {
            if num.value() == 0 && can_use_x0 {
                return (vec![], Register::X0);
            }

            let register = if other_register == Some(Register::T0) {
                Register::T1
            } else {
                Register::T0
            };
            self.value_map.insert(*val, register);
            (vec![Inst::Li(register, num.value())], register)
        } else if let Some(offset) = stack_map.get(val) {
            let register = if other_register == Some(Register::T0) {
                Register::T1
            } else {
                Register::T0
            };
            (vec![Inst::Lw(register, Register::SP, *offset)], register)
        } else {
            (vec![], *self.value_map.get(val).unwrap())
        }
    }

    fn koopa_inst_to_risc(
        self: &mut Self,
        function_data: &FunctionData,
        val: &Value,
        stack_offset: &mut i32,
        stack_map: &mut HashMap<Value, i32>,
    ) -> Vec<Inst> {
        let val_data = function_data.dfg().value(*val);
        match val_data.kind() {
            ValueKind::Integer(num) => vec![Inst::Int(num.value())],
            ValueKind::Return(ret) => {
                let asgn_inst = ret
                    .value()
                    .map(|val| {
                        let val_data = function_data.dfg().value(val);
                        if let ValueKind::Integer(num) = val_data.kind() {
                            Inst::Li(Register::A0, num.value())
                        } else if let Some(offset) = stack_map.get(&val) {
                            Inst::Lw(Register::A0, Register::SP, *offset)
                        } else {
                            Inst::Mv(Register::A0, *self.value_map.get(&val).unwrap())
                        }
                    })
                    .unwrap_or_else(|| Inst::Li(Register::A0, 0));
                if *stack_offset == 0 {
                    vec![asgn_inst, Inst::Ret]
                } else {
                    let space_mod = *stack_offset % 16;
                    if space_mod != 0 {
                        *stack_offset += 16 - space_mod;
                    }
                    vec![
                        asgn_inst,
                        Inst::Addi(Register::SP, Register::SP, *stack_offset),
                        Inst::Ret,
                    ]
                }
            }
            ValueKind::Binary(bin) => {
                let lhs = bin.lhs();
                let lhs_data = function_data.dfg().value(lhs);
                let rhs = bin.rhs();
                let rhs_data = function_data.dfg().value(rhs);
                let (mut asgn_inst1, rs1) =
                    self.imm_to_register(&lhs, lhs_data, true, None, stack_map);
                let (asgn_inst2, rs2) = self.imm_to_register(
                    &rhs,
                    rhs_data,
                    !(rs1 == Register::X0),
                    Some(rs1),
                    stack_map,
                );
                let reuse_register = rs1.which_to_reuse(&rs2);
                self.value_map.insert(*val, reuse_register);
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
                if !val_data.ty().is_unit() {
                    asgn_inst1.push(Inst::Sw(reuse_register, Register::SP, *stack_offset));
                    stack_map.insert(*val, *stack_offset);
                    *stack_offset += 4;
                }
                asgn_inst1
            }
            ValueKind::Alloc(_) => vec![],
            ValueKind::Load(load) => {
                let src = load.src();
                println!("{:#?}", function_data.dfg().value(src));
                let load_offset = *stack_map.get(&src).unwrap();
                let load_to_rg_inst = Inst::Lw(Register::T0, Register::SP, load_offset);
                let load_to_st_inst = Inst::Sw(Register::T0, Register::SP, *stack_offset);
                stack_map.insert(*val, *stack_offset);
                *stack_offset += 4;
                vec![load_to_rg_inst, load_to_st_inst]
            }
            ValueKind::Store(store) => {
                let dest = store.dest();
                let store_val = store.value();
                let store_val_data = function_data.dfg().value(store_val);
                let (mut store_insts, store_register) =
                    self.imm_to_register(&store_val, store_val_data, false, None, stack_map);
                let store_in_stack_inst = if let Some(offset) = stack_map.get(&dest) {
                    Inst::Sw(store_register, Register::SP, *offset)
                } else {
                    stack_map.insert(dest, *stack_offset);
                    *stack_offset += 4;
                    Inst::Sw(store_register, Register::SP, *stack_offset - 4)
                };

                store_insts.push(store_in_stack_inst);
                store_insts
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Mode, koopa::koopa::KoopaGenerator, sysy};
    use std::fs::read_to_string;
    #[test]
    fn aexp_sysy_2_riscv() {
        let input = read_to_string("test/lv4/asgn.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let mode = Mode::RiscV;
        mode.handle(
            &KoopaGenerator::new(ast).build_program_from_compunit(),
            None,
        );
    }
}
