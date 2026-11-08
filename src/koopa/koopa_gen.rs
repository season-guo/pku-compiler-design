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

/// 一个 block 当前的 context, 有除了 symbol_map 外的足够信息
#[derive(Debug)]
struct KoopaBlockContext {
    /// 如果该 block 是 if或while 产生的, 应该知道在该 block 结束时要跳到哪
    /// 对于 if, 应当是生成 end_block, 对于 while, 应当是生成的 cond_block
    if_end_jump_tar: Option<BasicBlock>,
    while_end_jump_tar: Option<BasicBlock>,
    /// 当遇到 break 时, 要 jump 到当前 while的 end_block
    while_break_jump_tar: Option<BasicBlock>,
}

pub struct KoopaGenerator {
    comp_unit: CompUnit,
    block_name: i32, // 自增id, 用于给 block 起名
    symbol_maps: VecDeque<HashMap<String, Symbol>>,
}

impl KoopaGenerator {
    pub fn new(comp_unit: CompUnit) -> Self {
        Self {
            comp_unit,
            block_name: 0,
            symbol_maps: VecDeque::new(),
        }
    }

    fn calcul_bef_compile(self: &Self, aexp: AExp) -> AExp {
        match aexp {
            AExp::ANum(num) => AExp::ANum(num),
            AExp::AVal(ident) => {
                let symbol = self.find_symbol(ident);
                if let Some(Symbol::Const(exp)) = symbol {
                    exp.clone()
                } else {
                    panic!("cacul const encounter a var or no such var");
                }
            }
            AExp::AUnAry(op, aexp) => {
                let simpl_aexp = self.calcul_bef_compile(*aexp);
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
                let simpl_lhs = self.calcul_bef_compile(*lhs);
                let simpl_rhs = self.calcul_bef_compile(*rhs);
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
        block: &mut BasicBlock,
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
                        let one = new_value!(function_data).integer(1);

                        let result_alloc = new_value!(function_data).alloc(Type::get_i32());
                        let store_result = new_value!(function_data).store(zero, result_alloc);
                        let _ = add_value!(function_data, *block, result_alloc);
                        let _ = add_value!(function_data, *block, store_result);
                        
                        let cond = new_value!(function_data).binary(BinaryOp::Eq, lhs, zero);
                        let true_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                        let false_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                        let end_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                        let branch = new_value!(function_data).branch(cond, true_bb, false_bb);
                        let false_val = new_value!(function_data).binary(BinaryOp::Eq, rhs, one);
                        let store_result_if_false = new_value!(function_data).store(false_val, result_alloc);
                        let load_result = new_value!(function_data).load(result_alloc);
                        let _ = add_bb!(function_data, true_bb);
                        let _ = add_bb!(function_data, false_bb);
                        let _ = add_bb!(function_data, end_bb);
                        let _ = add_value!(function_data, *block, cond);
                        let _ = add_value!(function_data, *block, branch);
                        let _ = add_value!(function_data, end_bb, load_result);
                        let _ = add_value!(function_data, false_bb, false_val);
                        let _ = add_value!(function_data, false_bb, store_result_if_false);
                        
                        let jump_val = new_value!(function_data).jump(end_bb);
                        let _ = add_value!(function_data, true_bb, jump_val);
                        let _ = add_value!(function_data, false_bb, jump_val);

                        *block = end_bb;
                        return load_result;                
                    }
                    ABinaryOp::Or => {
                        let zero = new_value!(function_data).integer(0);

                        let result_alloc = new_value!(function_data).alloc(Type::get_i32());
                        let result_val = new_value!(function_data).integer(1);
                        let store_result = new_value!(function_data).store(result_val, result_alloc);
                        let _ = add_value!(function_data, *block, result_alloc);
                        let _ = add_value!(function_data, *block, store_result);
                        
                        let cond = new_value!(function_data).binary(BinaryOp::Eq, lhs, zero);
                        let true_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                        let both_false_end_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                        let branch = new_value!(function_data).branch(cond, true_bb, both_false_end_bb);
                        let true_val = new_value!(function_data).binary(BinaryOp::NotEq, rhs, zero);
                        let store_result_if_true = new_value!(function_data).store(true_val, result_alloc);
                        let load_result = new_value!(function_data).load(result_alloc);
                        let _ = add_bb!(function_data, true_bb);
                        let _ = add_bb!(function_data, both_false_end_bb);
                        let _ = add_value!(function_data, *block, cond);
                        let _ = add_value!(function_data, *block, branch);
                        let _ = add_value!(function_data, both_false_end_bb, load_result);
                        let _ = add_value!(function_data, true_bb, true_val);
                        let _ = add_value!(function_data, true_bb, store_result_if_true);
                        
                        let jump_val = new_value!(function_data).jump(both_false_end_bb);
                        let _ = add_value!(function_data, true_bb, jump_val);

                        *block = both_false_end_bb;
                        return load_result;
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
            AExp::AVal(ident) => match self.find_symbol(ident) {
                Some(Symbol::Const(exp)) => {
                    self.aexp_to_koopa_insts(function_data, block, exp.clone())
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
    fn is_terminated(function_data: &FunctionData, bb: &BasicBlock) -> bool {
        function_data
            .layout()
            .bbs()
            .node(bb)
            .and_then(|node| node.insts().back_key().copied())
            .map(|val| {
                match function_data.dfg().value(val).kind() {
                    ValueKind::Return(_) | ValueKind::Jump(_) => true,
                    ValueKind::Branch(branch) => *bb != branch.true_bb() && *bb != branch.false_bb(),
                    _ => false,
                }
            })
            .unwrap_or(false)
    }


    fn find_symbol(self: &Self, key: String) -> Option<&Symbol> {
        self.symbol_maps.iter().find_map(|symbol_map| symbol_map.get(&key))
    }

    fn make_block_name(self: &mut Self) -> String {
        self.block_name += 1;
        format!("%block{}", self.block_name)
    }

    fn handle_block(self:&mut Self, block: Block, function_data: &mut FunctionData, mut bb: BasicBlock, ctx: &KoopaBlockContext) -> BasicBlock {
        self.symbol_maps.push_front(HashMap::new());
        let mut loc = 0usize;
        let item_len = block.items.len();
        for item in block.items {
            if Self::is_terminated(function_data, &bb) {
                break;
            }
            loc += 1;
            // 当前基本块已经以 ret/jump/br 结尾, 后面的语句不可达, 直接丢弃
            match item {
                BlockItem::Decl(Decl::Const(_, asgns)) => {
                    for Asgn { ident, val } in asgns {
                        let symbol = Symbol::Const(self.calcul_bef_compile(val.clone(), ));
                        self.symbol_maps.front_mut().unwrap().insert(ident.clone(), symbol);
                    }
                }
                BlockItem::Decl(Decl::Var(_, asgns)) => {
                    for Asgn { ident, val } in asgns {
                        let alloc = new_value!(function_data).alloc(Type::get_i32());
                        self.symbol_maps.front_mut().unwrap().insert(ident.clone(), Symbol::Var(alloc));
                        let _ = add_value!(function_data, bb, alloc);

                        let store_val = self.aexp_to_koopa_insts(function_data, &mut bb, val.clone());
                        let store = new_value!(function_data).store(store_val, alloc);
                        let _ = add_value!(function_data, bb, store);
                    }
                }
                BlockItem::Decl(Decl::UnInitVar(_, idents)) => {
                    for ident in idents {
                        let alloc = new_value!(function_data).alloc(Type::get_i32());
                        self.symbol_maps.front_mut().unwrap().insert(ident.clone(), Symbol::Var(alloc));
                        let _ = add_value!(function_data, bb, alloc);
                    }
                }
                BlockItem::Stmt(Stmt::Ret(aexp)) => {
                    let ret = match aexp {
                        Some(aexp) => {
                            let ret_value = self.aexp_to_koopa_insts(function_data, &mut bb, aexp );
                            new_value!(function_data).ret(Some(ret_value))
                        }
                        None => new_value!(function_data).ret(None),
                    };

                    let _ = add_value!(function_data, bb, ret);
                }
                BlockItem::Stmt(Stmt::Asgn(Asgn { ident, val })) => {
                    let var = self.find_symbol(ident).unwrap().clone();
                    if let Symbol::Var(var) = var {
                        let aexp_val = self.aexp_to_koopa_insts(function_data, &mut bb, val.clone());
                        let store = new_value!(function_data).store(aexp_val, var);

                        let _ = add_value!(function_data, bb, store);
                    }
                }
                BlockItem::Stmt(Stmt::Exp(_exp)) => {
                    // 啥都不做，目前表达式没有副作用
                }
                BlockItem::Stmt(Stmt::If(cond, true_branch, false_branch)) => {
                    if let Stmt::Exp(None) = *true_branch {
                        if false_branch.is_none() {
                            continue;
                        }
                        if let Some(inner) = false_branch.as_ref() && let Stmt::Exp(None) = **inner {
                                continue;
                        }
                    }

                    let then_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                    let _ = add_bb!(function_data, then_bb);

                    let else_bb = false_branch.is_some().then(|| new_bb!(function_data).basic_block(Some(self.make_block_name())));

                    let end_bb = if loc == item_len {
                        if let Some(prev) = ctx.if_end_jump_tar {
                            let end_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                            let _ = add_bb!(function_data, end_bb);
                            let jump_prev_end = new_value!(function_data).jump(prev);
                            let _ = add_value!(function_data, end_bb, jump_prev_end);
                            end_bb
                        } else {
                            bb
                        }
                    } else {
                        let end_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                        let _ = add_bb!(function_data, end_bb);
                        end_bb
                    };

                    let cond_val = self.aexp_to_koopa_insts(function_data, &mut bb, cond);
                    let br_inst = new_value!(function_data).branch(cond_val, then_bb, else_bb.unwrap_or( end_bb));
                    let _ = add_value!(function_data, bb, br_inst);


                    let jump_end = new_value!(function_data).jump(end_bb);
                    if let Some(else_bb) = else_bb {
                        let _ = add_bb!(function_data, else_bb);
                        let else_ctx = KoopaBlockContext {
                            if_end_jump_tar: Some(end_bb),
                            while_end_jump_tar: ctx.while_end_jump_tar,
                            while_break_jump_tar: ctx.while_break_jump_tar,
                        };
                        let else_end_bb = self.handle_block(Block{items: vec![BlockItem::Stmt(*false_branch.unwrap())]}, function_data, else_bb, &else_ctx);
                        if !Self::is_terminated(function_data, &else_end_bb) {
                            let _ = add_value!(function_data, else_end_bb, jump_end);
                        }
                    }

                    let then_ctx = KoopaBlockContext {
                        if_end_jump_tar: Some(end_bb),
                        while_end_jump_tar: ctx.while_end_jump_tar,
                        while_break_jump_tar: ctx.while_break_jump_tar,
                    };
                    let then_end_bb = self.handle_block(Block{items: vec![BlockItem::Stmt(*true_branch)]}, function_data, then_bb, &then_ctx);
                    if !Self::is_terminated(function_data, &then_end_bb) {
                        let _ = add_value!(function_data, then_end_bb, jump_end);
                    }

                    bb = end_bb;
                }
                BlockItem::Stmt(Stmt::While(cond, do_stmt)) => {
                    let mut cond_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                    let cond_bb_clone = cond_bb;
                    let _ = add_bb!(function_data, cond_bb);
                    let jump_cond_block = new_value!(function_data).jump(cond_bb);
                    let _ = add_value!(function_data, bb, jump_cond_block);
                    let cond_val = self.aexp_to_koopa_insts(function_data, &mut cond_bb, cond);

                    let while_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                    let end_bb = new_bb!(function_data).basic_block(Some(self.make_block_name()));
                    let _ = add_bb!(function_data, end_bb);
                    let _ = add_bb!(function_data, while_bb);
                    let cond_branch = new_value!(function_data).branch(cond_val, while_bb, end_bb);
                    let _ = add_value!(function_data, cond_bb, cond_branch); 

                    bb = end_bb;
                    let while_ctx = KoopaBlockContext {
                        if_end_jump_tar: ctx.if_end_jump_tar,
                        while_end_jump_tar: Some(cond_bb_clone),
                        while_break_jump_tar: Some(end_bb),
                    };
                    let while_end_bb = self.handle_block(Block{items: vec![BlockItem::Stmt(*do_stmt)]}, function_data, while_bb, &while_ctx);
                    if !Self::is_terminated(function_data, &while_end_bb) {
                        let jump_while_begin = new_value!(function_data).jump(cond_bb_clone);
                        let _ = add_value!(function_data, while_end_bb, jump_while_begin);
                    }

                }
                BlockItem::Stmt(Stmt::Break) => {
                    let jump_out = new_value!(function_data).jump(ctx.while_break_jump_tar.unwrap());
                    let _ = add_value!(function_data, bb, jump_out);
                }
                BlockItem::Stmt(Stmt::Continue) => {
                    let jump_begin = new_value!(function_data).jump(ctx.while_end_jump_tar.unwrap());
                    let _ = add_value!(function_data, bb, jump_begin);
                }
                BlockItem::Stmt(Stmt::Block(inner_block)) => {
                    bb = self.handle_block(inner_block, function_data, bb, ctx);
                },
            }
        }
        self.symbol_maps.pop_front();
        bb
    }

    pub fn build_program_from_compunit(mut self: Self) -> Program {
        let mut program = Program::new();
        let func_def = &self.comp_unit.func_def;
        let func_type = func_def.func_type.clone().into();

        let main = program.new_func_def(format!("@{}", func_def.ident), vec![], func_type);
        let main_ref = program.func_mut(main);
        let entry = new_bb!(main_ref).basic_block(Some("%entry".into()));
        let _ = add_bb!(main_ref, entry);
        let ctx = KoopaBlockContext {
            if_end_jump_tar: None, 
            while_end_jump_tar: None, 
            while_break_jump_tar: None,
        };
        self.handle_block(func_def.block.clone(), main_ref, entry, &ctx);

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
        let input = read_to_string("test/lv7/while.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        println!("{:#?}", ast);
    }

    #[test]
    fn aexp_sysy_2_koopa() {
        let input = read_to_string("test/lv5/block.c").unwrap();
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let builder = crate::koopa::koopa_gen::KoopaGenerator::new(ast);
        let mode = Mode::Koopa;
        mode.handle(builder.build_program_from_compunit(), None);
    }
}
