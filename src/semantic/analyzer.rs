use std::{collections::HashMap, mem};

use crate::ast::{program::Parameter, statement::VariableDeclaration};

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

#[cfg(test)]
mod tests {
    use super::{Context, SemanticAnalyzer, Symbol};
    use crate::ast::{program::Parameter, statement::Type};

    fn parameter(name: &[u8]) -> Parameter {
        Parameter {
            data_type: Type::Int,
            name: name.to_vec(),
        }
    }

    #[test]
    fn resolves_symbol_from_current_context() {
        let parameter = parameter(b"value");
        let mut context = Context::new();
        context.declare(parameter.name.as_slice(), Symbol::Parameter(&parameter));

        match context.resolve(b"value") {
            Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &parameter)),
            _ => panic!("expected parameter from current context"),
        }
    }

    #[test]
    fn resolves_symbol_from_parent_context() {
        let parameter = parameter(b"value");
        let mut parent = Context::new();
        parent.declare(parameter.name.as_slice(), Symbol::Parameter(&parameter));
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
        parent.declare(outer.name.as_slice(), Symbol::Parameter(&outer));
        let mut context = Context::with_parent(parent);
        context.declare(inner.name.as_slice(), Symbol::Parameter(&inner));

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
            .declare(outer.name.as_slice(), Symbol::Parameter(&outer));

        analyzer.enter_scope();
        analyzer
            .context
            .declare(inner.name.as_slice(), Symbol::Parameter(&inner));

        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_some());
        assert!(analyzer.leave_scope());
        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_none());
        assert!(!analyzer.leave_scope());
    }
}
