use super::{Context, SemanticAnalyzer, Symbol};
use crate::{
    ast::{
        Identifier,
        expression::Expression,
        statement::{Block, LValue, Statement, StatementKind, Type, VariableDeclaration},
    },
    semantic::{ExprType, SemanticError, TSemanticAnalyzer},
    tests::semantic::helpers::*,
};

#[test]
fn infers_integer_and_character_literals_separately() {
    let analyzer = SemanticAnalyzer::new();

    assert_eq!(
        analyzer.infer_expression_type(&Expression::Integer(1), 1),
        Ok(ExprType::Scalar(Type::Int))
    );
    assert_eq!(
        analyzer.infer_expression_type(&Expression::Char(b'a'), 1),
        Ok(ExprType::Scalar(Type::Char))
    );
}

#[test]
fn resolves_function_symbol_from_context() {
    let function = generic_function(b"helper", 1, vec![]);
    let mut context = Context::new();
    context
        .declare(&function.name, Symbol::Function(&function))
        .expect("function declaration should succeed");

    match context.resolve(b"helper") {
        Some(Symbol::Function(found)) => assert!(std::ptr::eq(*found, &function)),
        _ => panic!("expected function symbol"),
    }
}

#[test]
fn resolves_symbol_from_current_context() {
    let parameter = parameter(b"value");
    let mut context = Context::new();
    context
        .declare(&parameter.name, Symbol::Parameter(&parameter))
        .expect("unique declaration should succeed");

    match context.resolve(b"value") {
        Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &parameter)),
        _ => panic!("expected parameter from current context"),
    }
}

#[test]
fn resolves_symbol_from_parent_context() {
    let parameter = parameter(b"value");
    let mut parent = Context::new();
    parent
        .declare(&parameter.name, Symbol::Parameter(&parameter))
        .expect("unique declaration should succeed");
    let context = Context::with_parent(parent);

    match context.resolve(b"value") {
        Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &parameter)),
        _ => panic!("expected parameter from parent context"),
    }
}

#[test]
fn current_context_shadows_parent_symbol() {
    let outer = parameter(b"value");
    let inner = parameter(b"value");
    let mut parent = Context::new();
    parent
        .declare(&outer.name, Symbol::Parameter(&outer))
        .expect("unique declaration should succeed");
    let mut context = Context::with_parent(parent);
    context
        .declare(&inner.name, Symbol::Parameter(&inner))
        .expect("unique declaration should succeed");

    match context.resolve(b"value") {
        Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &inner)),
        _ => panic!("expected parameter from current context"),
    }
}

#[test]
fn analyzer_can_enter_and_leave_scope() {
    let outer = parameter(b"outer");
    let inner = parameter(b"inner");
    let mut analyzer = SemanticAnalyzer::new();
    analyzer
        .context
        .declare(&outer.name, Symbol::Parameter(&outer))
        .expect("unique declaration should succeed");

    analyzer.enter_scope();
    analyzer
        .context
        .declare(&inner.name, Symbol::Parameter(&inner))
        .expect("unique declaration should succeed");

    assert!(analyzer.context.resolve(b"outer").is_some());
    assert!(analyzer.context.resolve(b"inner").is_some());
    assert!(analyzer.leave_scope());
    assert!(analyzer.context.resolve(b"outer").is_some());
    assert!(analyzer.context.resolve(b"inner").is_none());
    assert!(!analyzer.leave_scope());
}

#[test]
fn scope_is_restored_when_nested_analysis_fails() {
    let nested = Statement::new(
        2,
        StatementKind::Block(Block {
            declarations: vec![scalar(b"inner", 3)],
            statements: vec![assignment(
                4,
                LValue::Identifier(identifier(b"missing", 4)),
                Expression::Integer(1),
            )],
        }),
    );
    let program = program(vec![scalar(b"outer", 1)], vec![nested]);

    let mut analyzer = SemanticAnalyzer::new();
    assert!(analyzer.analyze_block(&program.main.body).is_err());
    assert!(analyzer.context.resolve(b"outer").is_some());
    assert!(analyzer.context.resolve(b"inner").is_none());
}

#[test]
fn main_scope_is_restored_when_analysis_fails() {
    let program = program(
        vec![scalar(b"outer", 1)],
        vec![assignment(
            2,
            LValue::Identifier(identifier(b"missing", 2)),
            Expression::Integer(1),
        )],
    );

    let mut analyzer = SemanticAnalyzer::new();

    assert_eq!(
        analyzer.analyze(&program),
        Err(SemanticError::UndeclaredVariable(Identifier {
            name: b"missing".to_vec(),
            line: 2,
        }))
    );
    assert!(analyzer.context.resolve(b"outer").is_none());
}

#[test]
fn duplicate_declaration_preserves_original_symbol() {
    let original = scalar(b"value", 1);
    let duplicate = scalar(b"value", 4);
    let VariableDeclaration::Scalar {
        name: original_name,
        ..
    } = &original
    else {
        unreachable!()
    };
    let VariableDeclaration::Scalar {
        name: duplicate_name,
        ..
    } = &duplicate
    else {
        unreachable!()
    };

    let mut context = Context::new();
    context
        .declare(original_name, Symbol::Variable(&original))
        .expect("first declaration should succeed");

    assert_eq!(
        context.declare(duplicate_name, Symbol::Variable(&duplicate)),
        Err(SemanticError::DuplicateDeclaration(identifier(b"value", 4)))
    );

    match context.resolve(b"value") {
        Some(Symbol::Variable(found)) => assert!(std::ptr::eq(*found, &original)),
        _ => panic!("the original declaration should remain in the symbol table"),
    }
}
