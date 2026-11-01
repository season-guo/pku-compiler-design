use koopa::ir::{
    Program, Type,
    builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
};

use crate::ast::*;

pub fn build_program_from_compunit(CompUnit { func_def }: CompUnit) -> Program {
    let mut program = Program::new();
    let func_type = func_def.func_type.into();

    let main = program.new_func_def(format!("@{}", func_def.ident), vec![], func_type);
    let main_ref = program.func_mut(main);
    let entry = main_ref
        .dfg_mut()
        .new_bb()
        .basic_block(Some("%entry".into()));
    main_ref
        .layout_mut()
        .bbs_mut()
        .push_key_back(entry)
        .unwrap();
    let ret_num = main_ref
        .dfg_mut()
        .new_value()
        .integer(func_def.block.stmt.num);
    let ret = main_ref.dfg_mut().new_value().ret(Some(ret_num));
    main_ref
        .layout_mut()
        .bb_mut(entry)
        .insts_mut()
        .push_key_back(ret)
        .unwrap();

    program
}

pub fn koopa_example_volume_1() -> Program {
    let mut program = Program::new();
    let main = program.new_func_def("@main".into(), vec![], Type::get_i32());
    let main_ref = program.func_mut(main);
    let entry = main_ref
        .dfg_mut()
        .new_bb()
        .basic_block(Some("%entry".into()));
    main_ref
        .layout_mut()
        .bbs_mut()
        .push_key_back(entry)
        .unwrap();
    let zero = main_ref.dfg_mut().new_value().integer(0);
    let ret = main_ref.dfg_mut().new_value().ret(Some(zero));
    main_ref
        .layout_mut()
        .bb_mut(entry)
        .insts_mut()
        .push_key_back(ret)
        .unwrap();
    program
}
