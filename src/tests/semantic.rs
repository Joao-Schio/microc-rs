use crate::{
    ast::{
        Identifier,
        expression::{BinaryOp, Expression},
        program::{GenericFunction, MainFunction, Parameter, Program},
        statement::{Block, LValue, Statement, StatementKind, Type},
    },
    semantic::{ExprType, SemanticError},
};

pub(crate) mod helpers;
use helpers::*;

semantic_analyzer_contract!(
    default_analyzer,
    crate::semantic::analyzer::SemanticAnalyzer::new,
    {
        #[test]
        fn rejects_mixed_integer_and_character_addition() {
            let expression = binary_add(Expression::Integer(1), Expression::Char(b'a'));
            let program = program(vec![], vec![return_expression(4, expression)]);
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::InvalidBinaryOperands {
                    operator: BinaryOp::Add,
                    left: ExprType::Scalar(Type::Int),
                    right: ExprType::Scalar(Type::Char),
                    line: 4,
                })
            );
        }

        #[test]
        fn rejects_character_arithmetic_without_implicit_promotions() {
            let expression = binary_add(Expression::Char(b'a'), Expression::Char(b'b'));
            let program = program(vec![], vec![return_expression(4, expression)]);
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::InvalidBinaryOperands {
                    operator: BinaryOp::Add,
                    left: ExprType::Scalar(Type::Char),
                    right: ExprType::Scalar(Type::Char),
                    line: 4,
                })
            );
        }

        #[test]
        fn accepts_integer_arithmetic() {
            let expression = binary_add(Expression::Integer(1), Expression::Integer(2));
            let program = program(vec![], vec![return_expression(4, expression)]);
            let mut analyzer = make_analyzer();

            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn rejects_character_value_assigned_to_int() {
            let program = program(
                vec![scalar(b"value", 1)],
                vec![assignment(
                    2,
                    LValue::Identifier(identifier(b"value", 2)),
                    Expression::Char(b'a'),
                )],
            );
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Int),
                    actual: ExprType::Scalar(Type::Char),
                    line: 2,
                })
            );
        }

        #[test]
        fn rejects_integer_value_assigned_to_char() {
            let program = program(
                vec![char_scalar(b"value", 1)],
                vec![assignment(
                    2,
                    LValue::Identifier(identifier(b"value", 2)),
                    Expression::Integer(97),
                )],
            );
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Char),
                    actual: ExprType::Scalar(Type::Int),
                    line: 2,
                })
            );
        }

        #[test]
        fn accepts_matching_character_assignment() {
            let program = program(
                vec![char_scalar(b"value", 1)],
                vec![assignment(
                    2,
                    LValue::Identifier(identifier(b"value", 2)),
                    Expression::Char(b'a'),
                )],
            );
            let mut analyzer = make_analyzer();

            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn rejects_int_argument_to_char_parameter() {
            let mut p = program(
                vec![],
                vec![return_expression(
                    5,
                    function_call(b"accept", 5, vec![Expression::Integer(97)]),
                )],
            );
            let mut function = generic_function(
                b"accept",
                1,
                vec![return_expression(2, Expression::Integer(1))],
            );
            function.parameters = vec![Parameter {
                data_type: Type::Char,
                name: identifier(b"value", 1),
            }];
            p.functions.push(function);
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Char),
                    actual: ExprType::Scalar(Type::Int),
                    line: 5,
                })
            );
        }

        #[test]
        fn rejects_char_argument_to_int_parameter() {
            let mut p = program(
                vec![],
                vec![return_expression(
                    5,
                    function_call(b"accept", 5, vec![Expression::Char(b'a')]),
                )],
            );
            let mut function = generic_function(
                b"accept",
                1,
                vec![return_expression(2, Expression::Integer(1))],
            );
            function.parameters.push(parameter(b"value"));
            p.functions.push(function);
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Int),
                    actual: ExprType::Scalar(Type::Char),
                    line: 5,
                })
            );
        }

        #[test]
        fn accepts_char_argument_to_char_parameter() {
            let mut p = program(
                vec![],
                vec![return_expression(
                    5,
                    function_call(b"accept", 5, vec![Expression::Char(b'a')]),
                )],
            );
            let mut function = generic_function(
                b"accept",
                1,
                vec![return_expression(2, Expression::Integer(1))],
            );
            function.parameters.push(Parameter {
                data_type: Type::Char,
                name: identifier(b"value", 1),
            });
            p.functions.push(function);
            let mut analyzer = make_analyzer();

            assert_eq!(analyzer.analyze(&p), Ok(()));
        }

        #[test]
        fn rejects_function_call_with_wrong_argument_count() {
            let mut p = program(
                vec![],
                vec![return_expression(5, function_call(b"accept", 5, vec![]))],
            );
            let mut function = generic_function(b"accept", 1, vec![]);
            function.parameters.push(parameter(b"value"));
            p.functions.push(function);
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::ArgumentCountMismatch {
                    callee: identifier(b"accept", 5),
                    expected: 1,
                    actual: 0,
                })
            );
        }

        #[test]
        fn rejects_char_returned_from_int_function() {
            let mut p = program(vec![], vec![]);
            p.functions.push(generic_function(
                b"wrong",
                1,
                vec![return_expression(3, Expression::Char(b'a'))],
            ));
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Int),
                    actual: ExprType::Scalar(Type::Char),
                    line: 3,
                })
            );
        }

        #[test]
        fn accepts_char_returned_from_char_function() {
            let mut p = program(vec![], vec![]);
            let mut f = generic_function(
                b"character",
                1,
                vec![return_expression(3, Expression::Char(b'a'))],
            );
            f.return_type = Type::Char;
            p.functions.push(f);
            let mut analyzer = make_analyzer();

            assert_eq!(analyzer.analyze(&p), Ok(()));
        }

        #[test]
        fn restores_main_return_type_after_checking_char_function() {
            let mut p = program(vec![], vec![return_expression(8, Expression::Char(b'a'))]);
            let mut f = generic_function(
                b"character",
                1,
                vec![return_expression(3, Expression::Char(b'b'))],
            );
            f.return_type = Type::Char;
            p.functions.push(f);
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Int),
                    actual: ExprType::Scalar(Type::Char),
                    line: 8,
                })
            );
        }

        #[test]
        fn rejects_character_array_index() {
            let p = program(
                vec![array(b"items", 1)],
                vec![return_expression(
                    3,
                    Expression::ArrayAccess {
                        array: identifier(b"items", 3),
                        index: Box::new(Expression::Char(b'a')),
                    },
                )],
            );
            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Int),
                    actual: ExprType::Scalar(Type::Char),
                    line: 3,
                })
            );
        }

        #[test]
        fn rejects_indexing_scalar_variable() {
            let p = program(
                vec![scalar(b"item", 1)],
                vec![return_expression(
                    3,
                    Expression::ArrayAccess {
                        array: identifier(b"item", 3),
                        index: Box::new(Expression::Integer(0)),
                    },
                )],
            );
            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::NotAnArray(identifier(b"item", 3)))
            );
        }

        #[test]
        fn rejects_array_in_arithmetic() {
            let p = program(
                vec![array(b"items", 1)],
                vec![return_expression(
                    3,
                    binary_add(
                        Expression::Identifier(identifier(b"items", 3)),
                        Expression::Integer(2),
                    ),
                )],
            );
            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::InvalidBinaryOperands {
                    operator: BinaryOp::Add,
                    left: ExprType::Array(Type::Int),
                    right: ExprType::Scalar(Type::Int),
                    line: 3,
                })
            );
        }

        #[test]
        fn rejects_assignment_to_an_entire_array() {
            let p = program(
                vec![array(b"items", 1)],
                vec![assignment(
                    3,
                    LValue::Identifier(identifier(b"items", 3)),
                    Expression::Integer(42),
                )],
            );
            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::NotAssignable(identifier(b"items", 3)))
            );
        }

        #[test]
        fn rejects_char_condition_without_conversion() {
            let p = program(
                vec![],
                vec![Statement::new(
                    3,
                    StatementKind::If {
                        condition: Expression::Char(b'a'),
                        then_branch: Box::new(Statement::new(4, StatementKind::Empty)),
                        else_branch: None,
                    },
                )],
            );
            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::TypeMismatch {
                    expected: ExprType::Scalar(Type::Int),
                    actual: ExprType::Scalar(Type::Char),
                    line: 3,
                })
            );
        }

        #[test]
        fn rejects_missing_return_value() {
            let p = program(
                vec![],
                vec![Statement::new(3, StatementKind::Return { value: None })],
            );
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&p),
                Err(SemanticError::MissingReturnValue { line: 3 })
            );
        }

        #[test]
        fn calls_declared_function_from_main() {
            let mut program = program(
                vec![],
                vec![return_expression(5, function_call(b"helper", 5, vec![]))],
            );
            program.functions.push(generic_function(
                b"helper",
                1,
                vec![return_expression(2, Expression::Integer(42))],
            ));

            let mut analyzer = make_analyzer();
            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn resolves_forward_function_reference() {
            let mut program = program(vec![], vec![]);
            program.functions = vec![
                generic_function(
                    b"first",
                    1,
                    vec![return_expression(2, function_call(b"second", 2, vec![]))],
                ),
                generic_function(
                    b"second",
                    4,
                    vec![return_expression(5, Expression::Integer(42))],
                ),
            ];

            let mut analyzer = make_analyzer();
            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn resolves_recursive_function_reference() {
            let mut program = program(vec![], vec![]);
            program.functions.push(generic_function(
                b"recurse",
                1,
                vec![return_expression(2, function_call(b"recurse", 2, vec![]))],
            ));

            let mut analyzer = make_analyzer();
            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn rejects_duplicate_function_names() {
            let mut program = program(vec![], vec![]);
            program.functions = vec![
                generic_function(b"helper", 1, vec![]),
                generic_function(b"helper", 5, vec![]),
            ];

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::DuplicateDeclaration(identifier(
                    b"helper", 5
                )))
            );
        }

        #[test]
        fn rejects_call_to_local_variable() {
            let program = program(
                vec![scalar(b"value", 1)],
                vec![return_expression(2, function_call(b"value", 2, vec![]))],
            );

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::NotCallable(identifier(b"value", 2)))
            );
        }

        #[test]
        fn rejects_call_to_parameter() {
            let mut program = program(vec![], vec![]);
            let mut function = generic_function(
                b"helper",
                1,
                vec![return_expression(2, function_call(b"value", 2, vec![]))],
            );
            function.parameters.push(parameter(b"value"));
            program.functions.push(function);

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::NotCallable(identifier(b"value", 2)))
            );
        }

        #[test]
        fn rejects_function_used_as_variable() {
            let mut program = program(
                vec![],
                vec![return_expression(
                    5,
                    Expression::Identifier(identifier(b"helper", 5)),
                )],
            );
            program
                .functions
                .push(generic_function(b"helper", 1, vec![]));

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::NotAVariable(identifier(b"helper", 5)))
            );
        }

        #[test]
        fn local_variable_shadows_function_and_is_not_callable() {
            let mut program = program(
                vec![scalar(b"helper", 3)],
                vec![return_expression(4, function_call(b"helper", 4, vec![]))],
            );
            program
                .functions
                .push(generic_function(b"helper", 1, vec![]));

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::NotCallable(identifier(b"helper", 4)))
            );
        }

        #[test]
        fn checks_arguments_of_a_valid_function_call() {
            let mut program = program(
                vec![],
                vec![return_expression(
                    7,
                    function_call(
                        b"helper",
                        7,
                        vec![Expression::Identifier(identifier(b"missing", 7))],
                    ),
                )],
            );
            program
                .functions
                .push(generic_function(b"helper", 1, vec![]));

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::UndeclaredVariable(identifier(b"missing", 7)))
            );
        }

        #[test]
        fn analyzer_can_resolve_simple_main() {
            let program = program(
                vec![scalar(b"x", 1)],
                vec![assignment(
                    2,
                    LValue::Identifier(identifier(b"x", 2)),
                    Expression::Integer(20),
                )],
            );

            let mut analyzer = make_analyzer();
            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn undeclared_assignment_target_uses_statement_line() {
            let program = program(
                vec![],
                vec![assignment(
                    7,
                    LValue::Identifier(identifier(b"missing", 99)),
                    Expression::Integer(20),
                )],
            );

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::UndeclaredVariable(Identifier {
                    name: b"missing".to_vec(),
                    line: 7,
                }))
            );
        }

        #[test]
        fn undeclared_identifier_in_assignment_value_is_rejected() {
            let program = program(
                vec![scalar(b"x", 1)],
                vec![assignment(
                    11,
                    LValue::Identifier(identifier(b"x", 11)),
                    Expression::Identifier(identifier(b"missing", 99)),
                )],
            );

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::UndeclaredVariable(Identifier {
                    name: b"missing".to_vec(),
                    line: 11,
                }))
            );
        }

        #[test]
        fn analyzer_recurses_through_binary_expressions() {
            let program = program(
                vec![scalar(b"x", 1)],
                vec![assignment(
                    5,
                    LValue::Identifier(identifier(b"x", 5)),
                    Expression::Binary {
                        left: Box::new(Expression::Integer(1)),
                        op: BinaryOp::Add,
                        right: Box::new(Expression::Identifier(identifier(b"missing", 5))),
                    },
                )],
            );

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::UndeclaredVariable(Identifier {
                    name: b"missing".to_vec(),
                    line: 5,
                }))
            );
        }

        #[test]
        fn nested_block_can_resolve_parent_variable() {
            let nested = Statement::new(
                2,
                StatementKind::Block(Block {
                    declarations: vec![],
                    statements: vec![assignment(
                        3,
                        LValue::Identifier(identifier(b"x", 3)),
                        Expression::Integer(1),
                    )],
                }),
            );
            let program = program(vec![scalar(b"x", 1)], vec![nested]);

            let mut analyzer = make_analyzer();
            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn nested_block_scope_does_not_leak() {
            let nested = Statement::new(
                2,
                StatementKind::Block(Block {
                    declarations: vec![scalar(b"inner", 3)],
                    statements: vec![assignment(
                        4,
                        LValue::Identifier(identifier(b"inner", 4)),
                        Expression::Integer(1),
                    )],
                }),
            );
            let program = program(
                vec![],
                vec![
                    nested,
                    assignment(
                        9,
                        LValue::Identifier(identifier(b"inner", 9)),
                        Expression::Integer(2),
                    ),
                ],
            );

            let mut analyzer = make_analyzer();
            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::UndeclaredVariable(Identifier {
                    name: b"inner".to_vec(),
                    line: 9,
                }))
            );
        }

        #[test]
        fn analyzer_resolves_array_target_and_index_expression() {
            let program = program(
                vec![array(b"values", 1), scalar(b"index", 2)],
                vec![assignment(
                    3,
                    LValue::ArrayElement {
                        array: identifier(b"values", 3),
                        index: Box::new(Expression::Identifier(identifier(b"index", 3))),
                    },
                    Expression::Integer(42),
                )],
            );

            let mut analyzer = make_analyzer();
            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn function_parameters_are_visible_inside_function_body() {
            let program = Program {
                functions: vec![GenericFunction {
                    return_type: Type::Int,
                    name: identifier(b"add", 1),
                    parameters: vec![parameter(b"a"), parameter(b"b")],
                    body: Block {
                        declarations: vec![],
                        statements: vec![Statement::new(
                            2,
                            StatementKind::Return {
                                value: Some(Expression::Binary {
                                    left: Box::new(Expression::Identifier(identifier(b"a", 2))),
                                    op: BinaryOp::Add,
                                    right: Box::new(Expression::Identifier(identifier(b"b", 2))),
                                }),
                            },
                        )],
                    },
                }],
                main: MainFunction {
                    body: Block {
                        declarations: vec![],
                        statements: vec![],
                    },
                },
            };

            let mut analyzer = make_analyzer();

            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn rejects_undeclared_variable_inside_generic_function() {
            let program = Program {
                functions: vec![GenericFunction {
                    return_type: Type::Int,
                    name: identifier(b"calculate", 1),
                    parameters: vec![parameter(b"x")],
                    body: Block {
                        declarations: vec![],
                        statements: vec![Statement::new(
                            3,
                            StatementKind::Return {
                                value: Some(Expression::Identifier(identifier(b"missing", 3))),
                            },
                        )],
                    },
                }],
                main: MainFunction {
                    body: Block {
                        declarations: vec![],
                        statements: vec![],
                    },
                },
            };

            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::UndeclaredVariable(Identifier {
                    name: b"missing".to_vec(),
                    line: 3,
                }))
            );
        }

        #[test]
        fn rejects_duplicate_variable_declaration_in_same_scope() {
            let program = program(vec![scalar(b"value", 1), scalar(b"value", 3)], vec![]);

            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::DuplicateDeclaration(Identifier {
                    name: b"value".to_vec(),
                    line: 3,
                }))
            );
        }
        #[test]
        fn rejects_scalar_and_array_with_same_name_in_one_scope() {
            let program = program(vec![scalar(b"value", 1), array(b"value", 5)], vec![]);
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::DuplicateDeclaration(identifier(b"value", 5)))
            );
        }

        #[test]
        fn rejects_duplicate_function_parameters() {
            let program = Program {
                functions: vec![GenericFunction {
                    return_type: Type::Int,
                    name: identifier(b"f", 1),
                    parameters: vec![
                        parameter(b"value"),
                        Parameter {
                            data_type: Type::Char,
                            name: identifier(b"value", 2),
                        },
                    ],
                    body: Block {
                        declarations: vec![],
                        statements: vec![],
                    },
                }],
                main: MainFunction {
                    body: Block {
                        declarations: vec![],
                        statements: vec![],
                    },
                },
            };
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::DuplicateDeclaration(identifier(b"value", 2)))
            );
        }

        #[test]
        fn rejects_local_variable_colliding_with_function_parameter() {
            let program = Program {
                functions: vec![GenericFunction {
                    return_type: Type::Int,
                    name: identifier(b"f", 1),
                    parameters: vec![parameter(b"value")],
                    body: Block {
                        declarations: vec![scalar(b"value", 4)],
                        statements: vec![],
                    },
                }],
                main: MainFunction {
                    body: Block {
                        declarations: vec![],
                        statements: vec![],
                    },
                },
            };
            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::DuplicateDeclaration(identifier(b"value", 4)))
            );
        }

        #[test]
        fn permits_shadowing_variable_in_nested_block() {
            let nested = Statement::new(
                2,
                StatementKind::Block(Block {
                    declarations: vec![scalar(b"value", 3)],
                    statements: vec![assignment(
                        4,
                        LValue::Identifier(identifier(b"value", 4)),
                        Expression::Integer(1),
                    )],
                }),
            );
            let program = program(vec![scalar(b"value", 1)], vec![nested]);
            let mut analyzer = make_analyzer();

            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn permits_same_parameter_name_in_different_functions() {
            let function = |name: &[u8]| GenericFunction {
                return_type: Type::Int,
                name: identifier(name, 1),
                parameters: vec![parameter(b"value")],
                body: Block {
                    declarations: vec![],
                    statements: vec![Statement::new(
                        2,
                        StatementKind::Return {
                            value: Some(Expression::Identifier(identifier(b"value", 2))),
                        },
                    )],
                },
            };
            let program = Program {
                functions: vec![function(b"first"), function(b"second")],
                main: MainFunction {
                    body: Block {
                        declarations: vec![],
                        statements: vec![],
                    },
                },
            };
            let mut analyzer = make_analyzer();

            assert_eq!(analyzer.analyze(&program), Ok(()));
        }

        #[test]
        fn rejects_call_to_undeclared_function() {
            let program = program(
                vec![],
                vec![Statement::new(
                    7,
                    StatementKind::Return {
                        value: Some(Expression::Call {
                            callee: identifier(b"missing", 7),
                            arguments: vec![Expression::Integer(42)],
                        }),
                    },
                )],
            );

            let mut analyzer = make_analyzer();

            assert_eq!(
                analyzer.analyze(&program),
                Err(SemanticError::UndeclaredFunction(identifier(b"missing", 7)))
            );
        }
    }
);
