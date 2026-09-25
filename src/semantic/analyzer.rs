use std::{collections::HashMap, mem};

use crate::{
    ast::{
        program::Parameter,
        statement::{LValue, StatementKind, VariableDeclaration},
    },
    semantic::TSemanticAnalyzer,
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
}

impl Default for SemanticAnalyzer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TSemanticAnalyzer<'a> for SemanticAnalyzer<'a> {
    fn analyze(
        &mut self,
        program: &'a crate::ast::program::Program,
    ) -> Result<(), super::SemanticError> {
        for declaration in &program.main.body.declarations {
            match declaration {
                VariableDeclaration::Scalar { name, .. } => {
                    self.context
                        .declare(&name.name, Symbol::Variable(declaration));
                }
                _ => todo!(),
            }
        }

        for statement in &program.main.body.statements {
            match &statement.kind {
                StatementKind::Assignment(assignment) => {
                    if let LValue::Identifier(ref id) = assignment.target {
                        match self.context.resolve(&id.name) {
                            None => {
                                return Err(super::SemanticError::UndeclaredVariable {
                                    name: id.name.to_owned(),
                                    line: statement.line(),
                                });
                            }
                            Some(_) => continue,
                        }
                    }
                }
                _ => todo!(),
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Context, SemanticAnalyzer, Symbol};
    use crate::{
        ast::{
            Identifier,
            program::{Parameter, Program},
            statement::{
                Assignment, Block, LValue, Statement, StatementKind, Type, VariableDeclaration,
            },
        },
        semantic::TSemanticAnalyzer,
    };

    fn parameter(name: &[u8]) -> Parameter {
        Parameter {
            data_type: Type::Int,
            name: Identifier::new(name.to_vec(), 1),
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
        let program = Program {
            functions: vec![],
            main: crate::ast::program::MainFunction {
                body: Block {
                    declarations: vec![VariableDeclaration::Scalar {
                        data_type: Type::Int,
                        name: Identifier {
                            name: b"x".to_vec(),
                            line: 1,
                        },
                    }],
                    statements: vec![Statement::new(
                        1,
                        StatementKind::Assignment(Assignment {
                            target: LValue::Identifier(Identifier {
                                name: b"x".to_vec(),
                                line: 1,
                            }),
                            value: crate::ast::expression::Expression::Integer(20),
                        }),
                    )],
                },
            },
        };

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }
}
