use koopa::ir::Type as koopaType;

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Void, // 实际上就是Unit Type, 也就是 rs 的 ()
}

impl From<Type> for koopaType {
    fn from(value: Type) -> Self {
        match value {
            Type::Int => koopaType::get_i32(),
            Type::Void => koopaType::get_unit(),
        }
    }
}
