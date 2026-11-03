use std::io::Write;

use koopa::ir::{
    FunctionData, Program, ValueKind, entities::ValueData,
};

pub fn koopa_to_risc(program: &Program, mut writer: impl Write) {
    let mut prefix: String = " .text\n .global main\n".into();
    for func in program.func_layout() {
        let func_data = program.func(*func);
        prefix += &koopa_function_to_risc(func_data);
    }

    let _ = writer.write(&prefix.into_bytes());
}

fn koopa_function_to_risc(function_data: &FunctionData) -> String {
    let mut prefix = "main:\n".to_owned();
    for (_, node) in function_data.layout().bbs() {
        for (val, _) in node.insts() {
            let value_data = function_data.dfg().value(*val);
            prefix += &koopa_inst_to_risc(function_data, value_data);
        }
    }

    prefix
}

fn koopa_inst_to_risc(function_data: &FunctionData, val_data: &ValueData) -> String {
    match val_data.kind() {
        ValueKind::Integer(int) => int.value().to_string(),
        ValueKind::Return(ret) => {
            let val = ret
                .value()
                .map(|val| {
                    let val = function_data.dfg().value(val);
                    // 这里只会是 integer, 这个递归只会走一步
                    koopa_inst_to_risc(function_data, val)
                })
                .unwrap_or_else(|| "0".to_owned());
            format!("  li a0, {val}\n  ret")
        }
        _ => unreachable!(),
    }
}
