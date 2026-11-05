use std::collections::HashMap;

use crate::ast::{
    aexp::{ABinaryOp, AExp, UnAryOp},
    decl::{Asgn, Decl},
    func::{BlockItem, CompUnit, Stmt},
};
use crate::{add_bb, add_value, new_bb, new_value};
use koopa::ir::{
    BasicBlock, BinaryOp, FunctionData, Program, Type, Value,
    builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
};

#[derive(Debug, Clone)]
enum Symbol {
    Const(AExp),
    Var(Value),
}

pub struct KoopaGenerator {
    comp_unit: CompUnit,
}

impl KoopaGenerator {
    pub fn new(comp_unit: CompUnit) -> Self {
        Self {
            comp_unit,
        }
    }

    fn calcul_bef_compile(aexp: AExp, symbol_map: &mut HashMap<String, Symbol>) -> AExp {
        match aexp {
            AExp::ANum(num) => AExp::ANum(num),
            AExp::AVal(id) => {
                if let Some(Symbol::Const(exp)) = symbol_map.get(&id) {
                    exp.clone()
                } else {
                    panic!("cacul const encounter a var");
                }
            }
            AExp::AUnAry(op, aexp) => {
                let simpl_aexp = Self::calcul_bef_compile(*aexp, symbol_map);
                if let AExp::ANum(num) = simpl_aexp {
                    match op {
                        UnAryOp::Plus => AExp::ANum(num),
                        UnAryOp::Minus => AExp::ANum(-num),
                        UnAryOp::Not => AExp::ANum(if num == 0 { 1 } else { 0 }),
                    }
                } else {
                    panic!("decl a const, while it can't be cacul completely");
                }
            }
            AExp::ABinary(op, lhs, rhs) => {
                let simpl_lhs = Self::calcul_bef_compile(*lhs, symbol_map);
                let simpl_rhs = Self::calcul_bef_compile(*rhs, symbol_map);
                if let (AExp::ANum(lhs), AExp::ANum(rhs)) = (simpl_lhs, simpl_rhs) {
                    match op {
                        ABinaryOp::Plus => AExp::ANum(lhs + rhs),
                        ABinaryOp::Minus => AExp::ANum(lhs - rhs),
                        ABinaryOp::Mul => AExp::ANum(lhs * rhs),
                        ABinaryOp::Mod => AExp::ANum(lhs % rhs),
                        ABinaryOp::Div => AExp::ANum(lhs / rhs),
                        ABinaryOp::Or => AExp::ANum((lhs != 0 || rhs != 0).into()),
                        ABinaryOp::And => AExp::ANum((lhs != 0 && rhs != 0).into()),
                        ABinaryOp::Eq => AExp::ANum((lhs == rhs).into()),
                        ABinaryOp::Neq => AExp::ANum((lhs != rhs).into()),
                        ABinaryOp::Lt => AExp::ANum((lhs < rhs).into()),
                        ABinaryOp::Le => AExp::ANum((lhs <= rhs).into()),
                        ABinaryOp::Gt => AExp::ANum((lhs > rhs).into()),
                        ABinaryOp::Ge => AExp::ANum((lhs >= rhs).into()),
                    }
                } else {
                    panic!("decl a const, while it can't be cacul completely");
                }
            }
        }
    }

    fn aexp_to_koopa_insts(
        self: &mut Self,
        function_data: &mut FunctionData,
        block: &BasicBlock,
        aexp: AExp,
        symbol_map: &mut HashMap<String, Symbol>
    ) -> Value {
        match aexp {
            AExp::ANum(num) => new_value!(function_data).integer(num),
            AExp::AUnAry(op, aexp) => {
                let pre_value = self.aexp_to_koopa_insts(function_data, block, *aexp, symbol_map);
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
                let lhs = self.aexp_to_koopa_insts(function_data, block, *lhs, symbol_map);
                let rhs = self.aexp_to_koopa_insts(function_data, block, *rhs, symbol_map);
                let bin_op = match op {
                    ABinaryOp::Plus => BinaryOp::Add,
                    ABinaryOp::Minus => BinaryOp::Sub,
                    ABinaryOp::Mul => BinaryOp::Mul,
                    ABinaryOp::Mod => BinaryOp::Mod,
                    ABinaryOp::Div => BinaryOp::Div,
                    ABinaryOp::And => {
                        let zero = new_value!(function_data).integer(0);
                        let lhs = new_value!(function_data).binary(BinaryOp::NotEq, lhs, zero);
                        let rhs = new_value!(function_data).binary(BinaryOp::NotEq, rhs, zero);
                        let _ = add_value!(function_data, *block, lhs);
                        let _ = add_value!(function_data, *block, rhs);
                        let expr = new_value!(function_data).binary(BinaryOp::And, lhs, rhs);
                        let _ = add_value!(function_data, *block, expr);
                        return expr;
                    }
                    ABinaryOp::Or => {
                        let zero = new_value!(function_data).integer(0);
                        let lhs = new_value!(function_data).binary(BinaryOp::NotEq, lhs, zero);
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
            AExp::AVal(ident) => match symbol_map.get(&ident) {
                Some(Symbol::Const(exp)) => {
                    self.aexp_to_koopa_insts(function_data, block, exp.clone(), symbol_map)
                }
                Some(Symbol::Var(val)) => {
                    let expr = new_value!(function_data).load(*val);
                    let _ = add_value!(function_data, *block, expr);
                    expr
                }
                None => panic!("no such value"),
            },
        }
    }

    pub fn build_program_from_compunit(mut self: Self) -> Program {
        let mut program = Program::new();
        let func_def = &self.comp_unit.func_def;
        let func_type = func_def.func_type.clone().into();

        let main = program.new_func_def(format!("@{}", func_def.ident), vec![], func_type);
        let main_ref = program.func_mut(main);
        let entry = new_bb!(main_ref).basic_block(Some("%entry".into()));
        let _ = add_bb!(main_ref, entry);
        let mut symbol_map = HashMap::new();

        for item in func_def.clone().block.items {
            match item {
                BlockItem::Decl(Decl::Const(_, asgns)) => {
                    for Asgn { ident, val } in asgns {
                        let symbol = Symbol::Const(Self::calcul_bef_compile(val.clone(), &mut symbol_map));
                        symbol_map.insert(ident.clone(), symbol);
                    }
                }
                BlockItem::Decl(Decl::Var(_, asgns)) => {
                    for Asgn { ident, val } in asgns {
                        let alloc = new_value!(main_ref).alloc(Type::get_i32());
                        symbol_map.insert(ident.clone(), Symbol::Var(alloc));
                        let _ = add_value!(main_ref, entry, alloc);

                        let store_val = self.aexp_to_koopa_insts(main_ref, &entry, val.clone(), &mut symbol_map);
                        let store = new_value!(main_ref).store(store_val, alloc);
                        let _ = add_value!(main_ref, entry, store);
                    }
                }
                BlockItem::Decl(Decl::UnInitVar(_, idents)) => {
                    for ident in idents {
                        let alloc = new_value!(main_ref).alloc(Type::get_i32());
                        symbol_map.insert(ident.clone(), Symbol::Var(alloc));
                        let _ = add_value!(main_ref, entry, alloc);
                    }
                }
                BlockItem::Stmt(Stmt::Ret(aexp)) => {
                    let ret_value = self.aexp_to_koopa_insts(main_ref, &entry, aexp.clone(), &mut symbol_map);
                    let ret = new_value!(main_ref).ret(Some(ret_value));

                    let _ = add_value!(main_ref, entry, ret);
                }
                BlockItem::Stmt(Stmt::Asgn(Asgn { ident, val })) => {
                    let var = symbol_map.get(&ident).unwrap().clone();
                    if let Symbol::Var(var) = var {
                        let aexp_val = self.aexp_to_koopa_insts(main_ref, &entry, val.clone(), &mut symbol_map);
                        let store = new_value!(main_ref).store(aexp_val, var);

                        let _ = add_value!(main_ref, entry, store);
                    }
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
    use std::fs::read_to_string;
    use crate::sysy;

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
        let builder = crate::koopa::koopa::KoopaGenerator::new(ast);
        let mode = Mode::Koopa;
        mode.handle(&builder.build_program_from_compunit(), None);
    }

    #[test]
    fn koopa_binary_lrhs_value_kind() {
        let input = read_to_string("test/lv3/unary.sysy").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let mode = Mode::Koopa;
        let builder = crate::koopa::koopa::KoopaGenerator::new(ast);
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
