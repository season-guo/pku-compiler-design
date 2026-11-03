// CompUnit  ::= FuncDef;
// FuncDef   ::= FuncType IDENT "(" ")" Block;
// FuncType  ::= "int";

// Block     ::= "{" Stmt "}";
// Stmt        ::= "return" Exp ";";
// Number    ::= INT_CONST;

use koopa::ir::Type;

use crate::ast::aexp::AExp;

#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

#[derive(Debug)]
pub enum FuncType {
    Int,
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

#[derive(Debug)]
pub struct Stmt {
    pub aexp: AExp,
}

impl From<FuncType> for Type {
    fn from(value: FuncType) -> Self {
        match value {
            FuncType::Int => Type::get_i32(),
        }
    }
}
