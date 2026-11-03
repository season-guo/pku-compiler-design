use koopa::ir::{BasicBlock, BinaryOp, FunctionData, Program, Value, builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder}};
use crate::ast::{aexp::{AExp, UnAryOp}, func::CompUnit};
use crate::{new_bb, new_value, add_bb, add_value};

pub fn aexp_to_koopa_insts(function_data: &mut FunctionData, block: &BasicBlock, aexp: AExp) -> Value {
    match aexp {
        AExp::ANum(num) => new_value!(function_data).integer(num),
        AExp::AUnAry(op, aexp) => {
            let pre_value= aexp_to_koopa_insts(function_data, block, *aexp);
            match op {
                UnAryOp::APlus => pre_value,
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
    }
}

pub fn build_program_from_compunit(CompUnit { func_def }: CompUnit) -> Program {
    let mut program = Program::new();
    let func_type = func_def.func_type.into();

    let main = program.new_func_def(format!("@{}", func_def.ident), vec![], func_type);
    let main_ref = program.func_mut(main);
    let entry = new_bb!(main_ref).basic_block(Some("%entry".into()));
    let _ = add_bb!(main_ref, entry); 

    let ret_value = aexp_to_koopa_insts(main_ref,& entry, func_def.block.stmt.aexp);
    let ret = new_value!(main_ref).ret(Some(ret_value));

    let _ = add_value!(main_ref, entry, ret);

    program
}

#[cfg(test)] 
mod tests {
    use std::fs::read_to_string;
    use lalrpop_util::lalrpop_mod;
    use crate::{koopa::koopa::build_program_from_compunit, mode::Mode};
    lalrpop_mod!(sysy);

    #[test]
    fn aexp_sysy_2_ast() {
        let input = read_to_string("test/lv3/unary.sysy").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn aexp_sysy_2_koopa() {
        let input = read_to_string("test/lv3/unary.sysy").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let mode = Mode::Koopa;
        mode.handle(build_program_from_compunit(ast), None);
    }
}