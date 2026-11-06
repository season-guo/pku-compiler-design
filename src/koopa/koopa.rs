use std::collections::{HashMap, VecDeque};

use crate::ast::{
    aexp::{ABinaryOp, AExp, UnAryOp},
    decl::{Asgn, Decl},
    func::{BlockItem, CompUnit, Stmt, Block},
};
use crate::{add_bb, add_value, new_bb, new_value};
use koopa::ir::{
    BasicBlock, BinaryOp, FunctionData, Program, Type, Value, ValueKind,
    builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder},
};

#[derive(Debug, Clone)]
enum Symbol {
    Const(AExp),
    Var(Value),
}

pub struct KoopaGenerator {
    comp_unit: CompUnit,
    block_name: i32, // 自增id
}

impl KoopaGenerator {
    pub fn new(comp_unit: CompUnit) -> Self {
        Self {
            comp_unit,
            block_name: 0,
        }
    }

    fn calcul_bef_compile(aexp: AExp, symbol_maps: &mut VecDeque<HashMap<String, Symbol>>) -> AExp {
        match aexp {
            AExp::ANum(num) => AExp::ANum(num),
            AExp::AVal(id) => {
                let symbol = Self::find_symbol(symbol_maps, id);
                if let Some(Symbol::Const(exp)) = symbol {
                    exp.clone()
                } else {
                    panic!("cacul const encounter a var or no such var");
                }
            }
            AExp::AUnAry(op, aexp) => {
                let simpl_aexp = Self::calcul_bef_compile(*aexp, symbol_maps);
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
                let simpl_lhs = Self::calcul_bef_compile(*lhs, symbol_maps);
                let simpl_rhs = Self::calcul_bef_compile(*rhs, symbol_maps);
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
        symbol_maps: &mut VecDeque<HashMap<String, Symbol>>
    ) -> Value {
        match aexp {
            AExp::ANum(num) => new_value!(function_data).integer(num),
            AExp::AUnAry(op, aexp) => {
                let pre_value = self.aexp_to_koopa_insts(function_data, block, *aexp, symbol_maps);
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
                let lhs = self.aexp_to_koopa_insts(function_data, block, *lhs, symbol_maps);
                let rhs = self.aexp_to_koopa_insts(function_data, block, *rhs, symbol_maps);
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
            AExp::AVal(ident) => match Self::find_symbol(symbol_maps, ident) {
                Some(Symbol::Const(exp)) => {
                    self.aexp_to_koopa_insts(function_data, block, exp.clone(), symbol_maps)
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

    // 基本块是否已经有终结指令(ret / jump / br)
    fn is_terminated(function_data: &FunctionData, bb: BasicBlock) -> bool {
        function_data
            .layout()
            .bbs()
            .node(&bb)
            .and_then(|node| node.insts().back_key().copied())
            .map(|val| {
                matches!(
                    function_data.dfg().value(val).kind(),
                    ValueKind::Return(_) | ValueKind::Jump(_) | ValueKind::Branch(_)
                )
            })
            .unwrap_or(false)
    }


    fn find_symbol(symbol_maps: &VecDeque<HashMap<String, Symbol>>, key: String) -> Option<&Symbol> {
        symbol_maps.iter().find_map(|symbol_map| symbol_map.get(&key))
    }

    fn make_block_name(self: &mut Self) -> String {
        self.block_name += 1;
        format!("%block{}", self.block_name.to_string())
    }

    fn handle_block(self:&mut Self, block: Block, function_data: &mut FunctionData, mut bb: BasicBlock, symbol_maps: &mut VecDeque<HashMap<String, Symbol>>, prev_end_block: Option<BasicBlock>) {
        symbol_maps.push_front(HashMap::new());
        let mut loc = 0usize;
        let item_len = block.items.len();
        for item in block.items {
            if Self::is_terminated(function_data, bb) {
                break;
            }
            loc += 1;
            // 当前基本块已经以 ret/jump/br 结尾, 后面的语句不可达, 直接丢弃
            match item {
                BlockItem::Decl(Decl::Const(_, asgns)) => {
                    for Asgn { ident, val } in asgns {
                        let symbol = Symbol::Const(Self::calcul_bef_compile(val.clone(), symbol_maps));
                        symbol_maps.front_mut().unwrap().insert(ident.clone(), symbol);
                    }
                }
                BlockItem::Decl(Decl::Var(_, asgns)) => {
                    for Asgn { ident, val } in asgns {
                        let alloc = new_value!(function_data).alloc(Type::get_i32());
                        symbol_maps.front_mut().unwrap().insert(ident.clone(), Symbol::Var(alloc));
                        let _ = add_value!(function_data, bb, alloc);

                        let store_val = self.aexp_to_koopa_insts(function_data, &bb, val.clone(), symbol_maps);
                        let store = new_value!(function_data).store(store_val, alloc);
                        let _ = add_value!(function_data, bb, store);
                    }
                }
                BlockItem::Decl(Decl::UnInitVar(_, idents)) => {
                    for ident in idents {
                        let alloc = new_value!(function_data).alloc(Type::get_i32());
                        symbol_maps.front_mut().unwrap().insert(ident.clone(), Symbol::Var(alloc));
                        let _ = add_value!(function_data, bb, alloc);
                    }
                }
                BlockItem::Stmt(Stmt::Ret(aexp)) => {
                    let ret = match aexp {
                        Some(aexp) => {
                            let ret_value = self.aexp_to_koopa_insts(function_data, &bb, aexp, symbol_maps);
                            new_value!(function_data).ret(Some(ret_value))
                        }
                        None => new_value!(function_data).ret(None),
                    };

                    let _ = add_value!(function_data, bb, ret);
                }
                BlockItem::Stmt(Stmt::Asgn(Asgn { ident, val })) => {
                    let var = Self::find_symbol(symbol_maps, ident).unwrap().clone();
                    if let Symbol::Var(var) = var {
                        let aexp_val = self.aexp_to_koopa_insts(function_data, &bb, val.clone(), symbol_maps);
                        let store = new_value!(function_data).store(aexp_val, var);

                        let _ = add_value!(function_data, bb, store);
                    }
                }
                BlockItem::Stmt(Stmt::Exp(_exp)) => {
                    // 啥都不做，目前表达式没有副作用
                }
                BlockItem::Stmt(Stmt::If(cond, true_branch, false_branch)) => {
                    let then_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                    let else_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                    let _ = add_bb!(function_data, then_bb);
                    let _ = add_bb!(function_data, else_bb);

                    let cond_val = self.aexp_to_koopa_insts(function_data, &bb, cond, symbol_maps);
                    let br_inst = new_value!(function_data).branch(cond_val, then_bb, else_bb);
                    let _ = add_value!(function_data, bb, br_inst);

                    let end_bb = new_bb!(function_data).basic_block(Some(self.make_block_name())); // end_bb
                    let _ = add_bb!(function_data, end_bb);
                    if let Some(prev) = prev_end_block && loc == item_len {
                        let jump_prev_end = new_value!(function_data).jump(prev);
                        let _ = add_value!(function_data, end_bb, jump_prev_end);
                    }

                    bb = end_bb;
                    self.handle_block(Block{items: vec![BlockItem::Stmt(*true_branch)]}, function_data, then_bb, symbol_maps, Some(bb));
                    if let Some(false_branch) = false_branch {
                        self.handle_block(Block{items: vec![BlockItem::Stmt(*false_branch)]}, function_data, else_bb, symbol_maps, Some(bb));
                    }

                    let jump_end = new_value!(function_data).jump(bb);
                    for next_bb in [then_bb, else_bb] {
                        if !Self::is_terminated(function_data, next_bb) {
                            let _ = add_value!(function_data, next_bb, jump_end);
                        }
                    }
                }
                BlockItem::Stmt(Stmt::Block(inner_block)) => self.handle_block(inner_block, function_data, bb, symbol_maps, prev_end_block),
            }
        }
        symbol_maps.pop_front();
    }

    pub fn build_program_from_compunit(mut self: Self) -> Program {
        let mut program = Program::new();
        let func_def = &self.comp_unit.func_def;
        let func_type = func_def.func_type.clone().into();

        let main = program.new_func_def(format!("@{}", func_def.ident), vec![], func_type);
        let main_ref = program.func_mut(main);
        let entry = new_bb!(main_ref).basic_block(Some("%entry".into()));
        let _ = add_bb!(main_ref, entry);
        let mut symbol_maps = VecDeque::new();

        self.handle_block(func_def.block.clone(), main_ref, entry, &mut symbol_maps, None);

        program
    }
}

#[cfg(test)]
mod tests {
    use crate::mode::Mode;
    use std::fs::read_to_string;
    use crate::sysy;

    #[test]
    fn aexp_sysy_2_ast() {
        let input = read_to_string("test/lv6/simple_if.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn aexp_sysy_2_koopa() {
        let input = read_to_string("test/lv6/5_more_logical.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let builder = crate::koopa::koopa::KoopaGenerator::new(ast);
        let mode = Mode::Koopa;
        mode.handle(builder.build_program_from_compunit(), None);
    }
}
