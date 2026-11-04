use koopa::ir::Type as koopaType;

#[derive(Debug, Clone)]
pub enum Type {
    Int,
}

impl From<Type> for koopaType {
    fn from(value: Type) -> Self {
        match value {
            Type::Int => koopaType::get_i32(),
        }
    }
}
