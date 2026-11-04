use crate::ast::aexp::AExp;
use crate::ast::decl::Decl;
use crate::ast::r#type::Type;

#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug, Clone)]
pub struct FuncDef {
    pub func_type: Type,
    pub ident: String,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug, Clone)]
pub enum BlockItem {
    Stmt(Stmt),
    Decl(Decl),
}

#[derive(Debug, Clone)]
pub struct Stmt {
    pub aexp: AExp,
}
