use std::{collections::HashMap, mem};

use crate::{
    ast::{
        Identifier,
        expression::Expression,
        program::{Parameter, Program},
        statement::{
            Assignment, Block, LValue, PrintContent, Statement, StatementKind, VariableDeclaration,
        },
    },
    semantic::{SemanticError, TSemanticAnalyzer},
};

pub struct SemanticAnalyzer<'a> {
    context: Context<'a>,
}

pub struct Context<'a> {
    symbols: HashMap<&'a [u8], Symbol<'a>>,
    parent: Option<Box<Context<'a>>>,
}

pub enum Symbol<'a> {
    Parameter(&'a Parameter),
    Variable(&'a VariableDeclaration),
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Self) -> Self {
        Self {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn declare(&mut self, name: &'a [u8], symbol: Symbol<'a>) -> Option<Symbol<'a>> {
        self.symbols.insert(name, symbol)
    }

    pub fn resolve(&self, name: &[u8]) -> Option<&Symbol<'a>> {
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol);
        }

        self.parent
            .as_deref()
            .and_then(|parent| parent.resolve(name))
    }

    pub fn take_parent(&mut self) -> Option<Self> {
        self.parent.take().map(|parent| *parent)
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    fn enter_scope(&mut self) {
        let parent = mem::take(&mut self.context);
        self.context = Context::with_parent(parent);
    }

    fn leave_scope(&mut self) -> bool {
        let Some(parent) = self.context.take_parent() else {
            return false;
        };

        self.context = parent;
        true
    }

    fn with_scope<T>(
        &mut self,
        analyze: impl FnOnce(&mut Self) -> Result<T, SemanticError>,
    ) -> Result<T, SemanticError> {
        self.enter_scope();
        let result = analyze(self);
        let left_scope = self.leave_scope();
        debug_assert!(left_scope, "entered semantic scope must have a parent");
        result
    }

    fn analyze_block(&mut self, block: &'a Block) -> Result<(), SemanticError> {
        for declaration in &block.declarations {
            self.declare_variable(declaration);
        }

        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }

        Ok(())
    }

    fn declare_variable(&mut self, declaration: &'a VariableDeclaration) {
        let name = match declaration {
            VariableDeclaration::Scalar { name, .. } | VariableDeclaration::Array { name, .. } => {
                name
            }
        };

        self.context
            .declare(name.as_bytes(), Symbol::Variable(declaration));
    }

    fn analyze_statement(&mut self, statement: &'a Statement) -> Result<(), SemanticError> {
        let line = statement.line();

        match &statement.kind {
            StatementKind::Assignment(assignment) => self.analyze_assignment(assignment, line),
            StatementKind::Return { value } => {
                if let Some(expression) = value {
                    self.analyze_expression(expression, line)?;
                }
                Ok(())
            }
            StatementKind::Print { content } => match content {
                PrintContent::StringConst(_) => Ok(()),
                PrintContent::Expression(expression) => self.analyze_expression(expression, line),
            },
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.analyze_expression(condition, line)?;
                self.analyze_statement(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.analyze_statement(else_branch)?;
                }
                Ok(())
            }
            StatementKind::Block(block) => {
                self.with_scope(|analyzer| analyzer.analyze_block(block))
            }
            StatementKind::For {
                initialization,
                condition,
                update,
                body,
            } => {
                self.analyze_assignment(initialization, line)?;
                self.analyze_expression(condition, line)?;
                self.analyze_assignment(update, line)?;
                self.analyze_statement(body)
            }
            StatementKind::Empty => Ok(()),
        }
    }

    fn analyze_assignment(
        &mut self,
        assignment: &'a Assignment,
        line: usize,
    ) -> Result<(), SemanticError> {
        self.analyze_lvalue(&assignment.target, line)?;
        self.analyze_expression(&assignment.value, line)
    }

    fn analyze_lvalue(&mut self, lvalue: &'a LValue, line: usize) -> Result<(), SemanticError> {
        match lvalue {
            LValue::Identifier(identifier) => self.resolve_variable(identifier, line),
            LValue::ArrayElement { array, index } => {
                self.resolve_variable(array, line)?;
                self.analyze_expression(index, line)
            }
        }
    }

    fn analyze_expression(
        &mut self,
        expression: &'a Expression,
        line: usize,
    ) -> Result<(), SemanticError> {
        match expression {
            Expression::Integer(_) | Expression::Char(_) => Ok(()),
            Expression::Identifier(identifier) => self.resolve_variable(identifier, line),
            Expression::Unary { expression, .. } => self.analyze_expression(expression, line),
            Expression::Binary { left, right, .. } => {
                self.analyze_expression(left, line)?;
                self.analyze_expression(right, line)
            }
            Expression::ArrayAccess { array, index } => {
                self.resolve_variable(array, line)?;
                self.analyze_expression(index, line)
            }
            Expression::Call { arguments, .. } => {
                for argument in arguments {
                    self.analyze_expression(argument, line)?;
                }
                Ok(())
            }
        }
    }

    fn resolve_variable(&self, identifier: &Identifier, line: usize) -> Result<(), SemanticError> {
        if self.context.resolve(identifier.as_bytes()).is_some() {
            return Ok(());
        }

        Err(SemanticError::UndeclaredVariable {
            name: identifier.name.clone(),
            line,
        })
    }
}

impl Default for SemanticAnalyzer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TSemanticAnalyzer<'a> for SemanticAnalyzer<'a> {
    fn analyze(&mut self, program: &'a Program) -> Result<(), SemanticError> {
        self.context = Context::new();
        self.analyze_block(&program.main.body)
    }
}

#[cfg(test)]
mod tests {
    use super::{Context, SemanticAnalyzer, Symbol};
    use crate::{
        ast::{
            Identifier,
            expression::{BinaryOp, Expression},
            program::{MainFunction, Parameter, Program},
            statement::{
                Assignment, Block, LValue, Statement, StatementKind, Type, VariableDeclaration,
            },
        },
        semantic::{SemanticError, TSemanticAnalyzer},
    };

    fn identifier(name: &[u8], line: usize) -> Identifier {
        Identifier::new(name.to_vec(), line)
    }

    fn parameter(name: &[u8]) -> Parameter {
        Parameter {
            data_type: Type::Int,
            name: identifier(name, 1),
        }
    }

    fn scalar(name: &[u8], line: usize) -> VariableDeclaration {
        VariableDeclaration::Scalar {
            data_type: Type::Int,
            name: identifier(name, line),
        }
    }

    fn array(name: &[u8], line: usize) -> VariableDeclaration {
        VariableDeclaration::Array {
            data_type: Type::Int,
            name: identifier(name, line),
            length: 8,
        }
    }

    fn assignment(line: usize, target: LValue, value: Expression) -> Statement {
        Statement::new(
            line,
            StatementKind::Assignment(Assignment { target, value }),
        )
    }

    fn program(declarations: Vec<VariableDeclaration>, statements: Vec<Statement>) -> Program {
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

    #[test]
    fn resolves_symbol_from_current_context() {
        let parameter = parameter(b"value");
        let mut context = Context::new();
        context.declare(parameter.name.as_bytes(), Symbol::Parameter(&parameter));

        match context.resolve(b"value") {
            Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &parameter)),
            _ => panic!("expected parameter from current context"),
        }
    }

    #[test]
    fn resolves_symbol_from_parent_context() {
        let parameter = parameter(b"value");
        let mut parent = Context::new();
        parent.declare(parameter.name.as_bytes(), Symbol::Parameter(&parameter));
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
        parent.declare(outer.name.as_bytes(), Symbol::Parameter(&outer));
        let mut context = Context::with_parent(parent);
        context.declare(inner.name.as_bytes(), Symbol::Parameter(&inner));

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
            .declare(outer.name.as_bytes(), Symbol::Parameter(&outer));

        analyzer.enter_scope();
        analyzer
            .context
            .declare(inner.name.as_bytes(), Symbol::Parameter(&inner));

        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_some());
        assert!(analyzer.leave_scope());
        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_none());
        assert!(!analyzer.leave_scope());
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

        let mut analyzer = SemanticAnalyzer::new();
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

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable {
                name: b"missing".to_vec(),
                line: 7,
            })
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

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable {
                name: b"missing".to_vec(),
                line: 11,
            })
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

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable {
                name: b"missing".to_vec(),
                line: 5,
            })
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

        let mut analyzer = SemanticAnalyzer::new();
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

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable {
                name: b"inner".to_vec(),
                line: 9,
            })
        );
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
        assert!(analyzer.analyze(&program).is_err());
        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_none());
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

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }
}
