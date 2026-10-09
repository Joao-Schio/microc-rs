use crate::ast::{
    Identifier,
    expression::{BinaryOp, Expression},
    program::{GenericFunction, MainFunction, Program},
    statement::{
        Assignment, Block, LValue, PrintContent, Statement, StatementKind, Type,
        VariableDeclaration,
    },
};

use super::{CodegenError, ControlFlow, EmitLlvm, FunctionContext};

fn identifier(name: &str) -> Identifier {
    Identifier::new(name.as_bytes().to_vec(), 1)
}

fn emit_expression(expression: Expression) -> Result<(String, String), CodegenError> {
    let mut context = FunctionContext::new();
    let operand = expression.emit_llvm(&mut context)?;
    Ok((context.into_instructions(), operand.to_string()))
}

fn empty_block() -> Block {
    Block {
        declarations: Vec::new(),
        statements: Vec::new(),
    }
}

fn statement(kind: StatementKind) -> Statement {
    Statement::new(1, kind)
}

#[test]
fn expression_lowers_each_supported_binary_operator() {
    for (operator, mnemonic) in [
        (BinaryOp::Add, "add"),
        (BinaryOp::Subtract, "sub"),
        (BinaryOp::Multiply, "mul"),
    ] {
        let expression = Expression::Binary {
            left: Box::new(Expression::Integer(11)),
            op: operator,
            right: Box::new(Expression::Integer(3)),
        };
        let (instructions, operand) = emit_expression(expression).unwrap();
        assert_eq!(instructions, format!("  %t0 = {mnemonic} i32 11, 3\n"));
        assert_eq!(operand, "%t0");
    }
}

#[test]
fn expression_rejects_unsupported_binary_operators_before_emitting_code() {
    for operator in [
        BinaryOp::Divide,
        BinaryOp::Modulo,
        BinaryOp::Equal,
        BinaryOp::NotEqual,
        BinaryOp::Less,
        BinaryOp::LessEqual,
        BinaryOp::Greater,
        BinaryOp::GreaterEqual,
        BinaryOp::And,
        BinaryOp::Or,
    ] {
        let expression = Expression::Binary {
            left: Box::new(Expression::Integer(1)),
            op: operator,
            right: Box::new(Expression::Integer(2)),
        };
        assert!(matches!(
            emit_expression(expression),
            Err(CodegenError::Unsupported("binary operator"))
        ));
    }
}

#[test]
fn expression_propagates_nested_integer_range_errors() {
    let expression = Expression::Binary {
        left: Box::new(Expression::Integer(10)),
        op: BinaryOp::Add,
        right: Box::new(Expression::Integer(i64::MAX)),
    };
    assert!(matches!(
        emit_expression(expression),
        Err(CodegenError::IntegerOutOfRange(i64::MAX))
    ));
}

#[test]
fn statement_reports_unsupported_forms_without_emitting_ir() {
    let forms = [
        StatementKind::Assignment(Assignment {
            target: LValue::Identifier(identifier("x")),
            value: Expression::Integer(1),
        }),
        StatementKind::Print {
            content: PrintContent::StringConst(b"hello".to_vec()),
        },
        StatementKind::If {
            condition: Expression::Integer(1),
            then_branch: Box::new(statement(StatementKind::Empty)),
            else_branch: None,
        },
        StatementKind::For {
            initialization: Assignment {
                target: LValue::Identifier(identifier("x")),
                value: Expression::Integer(0),
            },
            condition: Expression::Integer(1),
            update: Assignment {
                target: LValue::Identifier(identifier("x")),
                value: Expression::Integer(1),
            },
            body: Box::new(statement(StatementKind::Empty)),
        },
    ];
    for kind in forms {
        let mut ctx = FunctionContext::new();
        assert!(matches!(
            statement(kind).emit_llvm(&mut ctx),
            Err(CodegenError::Unsupported("statement"))
        ));
        assert_eq!(ctx.into_instructions(), "");
    }
}

#[test]
fn empty_statement_and_block_continue_without_instructions() {
    let mut ctx = FunctionContext::new();
    assert_eq!(
        statement(StatementKind::Empty).emit_llvm(&mut ctx).unwrap(),
        ControlFlow::Continues
    );
    assert_eq!(
        empty_block().emit_llvm(&mut ctx).unwrap(),
        ControlFlow::Continues
    );
    assert_eq!(ctx.into_instructions(), "");
}

#[test]
fn nested_block_propagates_return_termination() {
    let block = Block {
        declarations: vec![],
        statements: vec![statement(StatementKind::Block(Block {
            declarations: vec![],
            statements: vec![statement(StatementKind::Return {
                value: Some(Expression::Integer(17)),
            })],
        }))],
    };
    let mut ctx = FunctionContext::new();
    assert_eq!(block.emit_llvm(&mut ctx).unwrap(), ControlFlow::Terminated);
    assert_eq!(ctx.into_instructions(), "  ret i32 17\n");
}

#[test]
fn block_rejects_declarations_before_emitting_statements() {
    let block = Block {
        declarations: vec![VariableDeclaration::Scalar {
            data_type: Type::Int,
            name: identifier("x"),
        }],
        statements: vec![statement(StatementKind::Return {
            value: Some(Expression::Integer(4)),
        })],
    };
    let mut ctx = FunctionContext::new();
    assert!(matches!(
        block.emit_llvm(&mut ctx),
        Err(CodegenError::Unsupported("variable declarations"))
    ));
    assert_eq!(ctx.into_instructions(), "");
}

#[test]
fn block_identifies_unreachable_statement_line() {
    let block = Block {
        declarations: vec![],
        statements: vec![
            statement(StatementKind::Return {
                value: Some(Expression::Integer(1)),
            }),
            Statement::new(27, StatementKind::Empty),
        ],
    };
    let mut ctx = FunctionContext::new();
    assert_eq!(
        block.emit_llvm(&mut ctx),
        Err(CodegenError::UnreachableStatement { line: 27 })
    );
    assert_eq!(ctx.into_instructions(), "  ret i32 1\n");
}

#[test]
fn program_with_no_statements_requires_a_return() {
    let program = Program {
        functions: vec![],
        main: MainFunction {
            body: empty_block(),
        },
    };
    assert_eq!(
        super::generate_ir(&program),
        Err(CodegenError::MissingReturn)
    );
}

#[test]
fn additional_functions_are_rejected_before_main_is_lowered() {
    let program = Program {
        functions: vec![GenericFunction {
            return_type: Type::Int,
            name: identifier("helper"),
            parameters: vec![],
            body: empty_block(),
        }],
        main: MainFunction {
            body: empty_block(),
        },
    };
    assert_eq!(
        super::generate_ir(&program),
        Err(CodegenError::Unsupported("additional functions"))
    );
}

#[test]
fn rejects_value_just_below_i32_minimum() {
    let value = i64::from(i32::MIN) - 1;
    assert!(matches!(
        emit_expression(Expression::Integer(value)),
        Err(CodegenError::IntegerOutOfRange(actual)) if actual == value
    ));
}
