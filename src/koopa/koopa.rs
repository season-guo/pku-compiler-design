use std::collections::HashMap;

use crate::ast::{
    aexp::{ABinaryOp, AExp, UnAryOp},
    decl::{Asgn, Decl},
    func::{BlockItem, CompUnit, Stmt},
};
use crate::{add_bb, add_value, new_bb, new_value};
use koopa::ir::{
    BasicBlock, BinaryOp, FunctionData, Program, Value,
    builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
};

pub struct KoopaGenerator {
    pub comp_unit: CompUnit,
    pub symbol_map: HashMap<String, AExp>,
}

impl KoopaGenerator {
    pub fn new(comp_unit: CompUnit) -> Self {
        Self {
            comp_unit,
            symbol_map: HashMap::new(),
        }
    }
    pub fn aexp_to_koopa_insts(
        self: &mut Self,
        function_data: &mut FunctionData,
        block: &BasicBlock,
        aexp: AExp,
    ) -> Value {
        match aexp {
            AExp::ANum(num) => new_value!(function_data).integer(num),
            AExp::AUnAry(op, aexp) => {
                let pre_value = self.aexp_to_koopa_insts(function_data, block, *aexp);
                match op {
                    UnAryOp::Plus => pre_value,
                    UnAryOp::Minus => {
                        let lhs = new_value!(function_data).integer(0);
                        let val = new_value!(function_data).binary(BinaryOp::Sub, lhs, pre_value);
                        let _ = add_value!(function_data, *block, val);
                        val
                    }
                    UnAryOp::Not => {
                        let rhs = new_value!(function_data).integer(0);
                        let val = new_value!(function_data).binary(BinaryOp::Eq, pre_value, rhs);
                        let _ = add_value!(function_data, *block, val);
                        val
                    }
                }
            }
            AExp::ABinary(op, lhs, rhs) => {
                let lhs = self.aexp_to_koopa_insts(function_data, block, *lhs);
                let rhs = self.aexp_to_koopa_insts(function_data, block, *rhs);
                let bin_op = match op {
                    ABinaryOp::Plus => BinaryOp::Add,
                    ABinaryOp::Minus => BinaryOp::Sub,
                    ABinaryOp::Mul => BinaryOp::Mul,
                    ABinaryOp::Mod => BinaryOp::Mod,
                    ABinaryOp::Div => BinaryOp::Div,
                    ABinaryOp::And => {
                        let zero = new_value!(function_data).integer(0);
                        let lhs =
                            new_value!(function_data).binary(BinaryOp::NotEq, lhs, zero.clone());
                        let rhs = new_value!(function_data).binary(BinaryOp::NotEq, rhs, zero);
                        let _ = add_value!(function_data, *block, lhs);
                        let _ = add_value!(function_data, *block, rhs);
                        let expr = new_value!(function_data).binary(BinaryOp::And, lhs, rhs);
                        let _ = add_value!(function_data, *block, expr);
                        return expr;
                    }
                    ABinaryOp::Or => {
                        let zero = new_value!(function_data).integer(0);
                        let lhs =
                            new_value!(function_data).binary(BinaryOp::NotEq, lhs, zero.clone());
                        let rhs = new_value!(function_data).binary(BinaryOp::NotEq, rhs, zero);
                        let _ = add_value!(function_data, *block, lhs);
                        let _ = add_value!(function_data, *block, rhs);
                        let expr = new_value!(function_data).binary(BinaryOp::Or, lhs, rhs);
                        let _ = add_value!(function_data, *block, expr);
                        return expr;
                    }
                    ABinaryOp::Eq => BinaryOp::Eq,
                    ABinaryOp::Neq => BinaryOp::NotEq,
                    ABinaryOp::Lt => BinaryOp::Lt,
                    ABinaryOp::Le => BinaryOp::Le,
                    ABinaryOp::Gt => BinaryOp::Gt,
                    ABinaryOp::Ge => BinaryOp::Ge,
                };

                let expr = new_value!(function_data).binary(bin_op, lhs, rhs);
                let _ = add_value!(function_data, *block, expr);
                expr
            }
            AExp::AVal(val) => {
                if let Some(exp) = self.symbol_map.get(&val) {
                    self.aexp_to_koopa_insts(function_data, block, exp.clone())
                } else {
                    panic!("no such value");
                }
            }
        }
    }

    pub fn build_program_from_compunit(self: &mut Self) -> Program {
        let mut program = Program::new();
        let func_def = &self.comp_unit.func_def;
        let func_type = func_def.func_type.clone().into();

        let main = program.new_func_def(format!("@{}", func_def.ident), vec![], func_type);
        let main_ref = program.func_mut(main);
        let entry = new_bb!(main_ref).basic_block(Some("%entry".into()));
        let _ = add_bb!(main_ref, entry);

        for item in func_def.clone().block.items {
            match item {
                BlockItem::Decl(Decl::Const(_, asgns) | Decl::Var(_, asgns)) => {
                    for Asgn { ident, val } in asgns {
                        self.symbol_map.insert(ident.clone(), val.clone());
                    }
                }
                BlockItem::Stmt(Stmt { aexp }) => {
                    let ret_value = self.aexp_to_koopa_insts(main_ref, &entry, aexp.clone());
                    let ret = new_value!(main_ref).ret(Some(ret_value));

                    let _ = add_value!(main_ref, entry, ret);
                }
            }
        }

        program
    }
}

#[cfg(test)]
mod tests {
    use crate::mode::Mode;
    use koopa::ir::ValueKind;
    use lalrpop_util::lalrpop_mod;
    use std::fs::read_to_string;
    lalrpop_mod!(sysy);

    #[test]
    fn aexp_sysy_2_ast() {
        let input = read_to_string("test/lv4/asgn.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn aexp_sysy_2_koopa() {
        let input = read_to_string("test/lv4/asgn.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let mut builder = crate::koopa::koopa::KoopaGenerator::new(ast);
        let mode = Mode::Koopa;
        mode.handle(&builder.build_program_from_compunit(), None);
    }

    #[test]
    fn koopa_binary_lrhs_value_kind() {
        let input = read_to_string("test/lv3/unary.sysy").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let mode = Mode::Koopa;
        let mut builder = crate::koopa::koopa::KoopaGenerator::new(ast);
        let program = builder.build_program_from_compunit();
        mode.handle(&program, None);
        for func in program.func_layout() {
            let funcion_data = program.func(*func);
            for (_, node) in funcion_data.layout().bbs() {
                for (val, _) in node.insts() {
                    let val_data = funcion_data.dfg().value(*val);
                    //println!("{:#?}", val_data);
                    if let ValueKind::Binary(bin) = val_data.kind() {
                        println!("{:#?}", funcion_data.dfg().value(bin.rhs()));
                    }
                }
            }
        }
    }
}
