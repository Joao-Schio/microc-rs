use crate::ast::expression::Expression;

#[derive(Debug, PartialEq, Eq)]
pub enum AssignmentTarget {
    Identifier(Vec<u8>),
    ArrayElement {
        array: Vec<u8>,
        index: Box<Expression>,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Statement {
    Assignment {
        target: AssignmentTarget,
        value: Expression,
    },
}
