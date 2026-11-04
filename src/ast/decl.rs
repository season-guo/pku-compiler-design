use crate::ast::aexp::AExp;
use crate::ast::r#type::Type;

#[derive(Debug, Clone)]
pub enum Decl {
    Const(Type, Vec<Asgn>),
    Var(Type, Vec<Asgn>),
}

#[derive(Debug, Clone)]
pub struct Asgn {
    pub ident: String,
    pub val: Option<AExp>,
}
