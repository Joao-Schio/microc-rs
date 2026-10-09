use crate::ast::{
    Identifier,
    expression::{BinaryOp, Expression},
    program::{GenericFunction, MainFunction, Parameter, Program},
    statement::{Assignment, Block, LValue, Statement, StatementKind, Type, VariableDeclaration},
};

pub(crate) fn identifier(name: &[u8], line: usize) -> Identifier {
    Identifier::new(name.to_vec(), line)
}

pub(crate) fn parameter(name: &[u8]) -> Parameter {
    Parameter {
        data_type: Type::Int,
        name: identifier(name, 1),
    }
}

pub(crate) fn scalar(name: &[u8], line: usize) -> VariableDeclaration {
    VariableDeclaration::Scalar {
        data_type: Type::Int,
        name: identifier(name, line),
    }
}

pub(crate) fn array(name: &[u8], line: usize) -> VariableDeclaration {
    VariableDeclaration::Array {
        data_type: Type::Int,
        name: identifier(name, line),
        length: 8,
    }
}

pub(crate) fn assignment(line: usize, target: LValue, value: Expression) -> Statement {
    Statement::new(
        line,
        StatementKind::Assignment(Assignment { target, value }),
    )
}

pub(crate) fn program(
    declarations: Vec<VariableDeclaration>,
    statements: Vec<Statement>,
) -> Program {
    Program {
        functions: vec![],
        main: MainFunction {
            body: Block {
                declarations,
                statements,
            },
        },
    }
}

pub(crate) fn generic_function(
    name: &[u8],
    line: usize,
    statements: Vec<Statement>,
) -> GenericFunction {
    GenericFunction {
        return_type: Type::Int,
        name: identifier(name, line),
        parameters: vec![],
        body: Block {
            declarations: vec![],
            statements,
        },
    }
}

pub(crate) fn return_expression(line: usize, value: Expression) -> Statement {
    Statement::new(line, StatementKind::Return { value: Some(value) })
}

pub(crate) fn function_call(name: &[u8], line: usize, arguments: Vec<Expression>) -> Expression {
    Expression::Call {
        callee: identifier(name, line),
        arguments,
    }
}

pub(crate) fn char_scalar(name: &[u8], line: usize) -> VariableDeclaration {
    VariableDeclaration::Scalar {
        data_type: Type::Char,
        name: identifier(name, line),
    }
}

pub(crate) fn binary_add(left: Expression, right: Expression) -> Expression {
    Expression::Binary {
        left: Box::new(left),
        op: BinaryOp::Add,
        right: Box::new(right),
    }
}
