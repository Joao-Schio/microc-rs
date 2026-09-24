use crate::ast::expression::Expression;

#[derive(Debug, PartialEq, Eq)]
pub enum LValue {
    Identifier(Vec<u8>),
    ArrayElement {
        array: Vec<u8>,
        index: Box<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrintContent {
    StringConst(Vec<u8>),
    Expression(Expression),
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataType {
    Int,
    Char,
}

#[derive(Debug, PartialEq, Eq)]
pub enum VariableDeclaration {
    Scalar {
        data_type: DataType,
        name: Vec<u8>,
    },
    Array {
        data_type: DataType,
        name: Vec<u8>,
        length: i64,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    pub declarations: Vec<VariableDeclaration>,
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Assignment {
    pub target: LValue,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Assignment(Assignment),
    Return {
        value: Option<Expression>,
    },
    Print {
        content: PrintContent,
    },
    If {
        condition: Expression,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },
    Block(Block),
    For {
        initialization: Assignment,
        condition: Expression,
        update: Assignment,
        body: Box<Statement>,
    },
    Empty,
}
