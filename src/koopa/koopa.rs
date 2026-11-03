use koopa::ir::{Program, builder::{BasicBlockBuilder, ValueBuilder, LocalInstBuilder}};
use crate::ast::ast::CompUnit;
use crate::{new_bb, new_value, add_bb, add_value};

pub fn build_program_from_compunit(CompUnit { func_def }: CompUnit) -> Program {
    let mut program = Program::new();
    let func_type = func_def.func_type.into();

    let main = program.new_func_def(format!("@{}", func_def.ident), vec![], func_type);
    let main_ref = program.func_mut(main);
    let entry = new_bb!(main_ref).basic_block(Some("%entry".into()));
    let ret_num = new_value!(main_ref).integer(func_def.block.stmt.num);
    let ret = new_value!(main_ref).ret(Some(ret_num));

    let _ = add_bb!(main_ref, entry); 
    let _ = add_value!(main_ref, entry, ret);

    program
}