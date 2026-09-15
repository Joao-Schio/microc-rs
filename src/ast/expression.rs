#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UnaryOp {
    Not,
    Negate,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    And,
    Or,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expression {
    Integer(i64),
    Char(u8),
    Identifier(Vec<u8>),
    Unary {
        op: UnaryOp,
        expression: Box<Expression>,
    },
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
    },
}
