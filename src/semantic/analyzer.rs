use std::collections::HashMap;

use crate::ast::{program::Parameter, statement::VariableDeclaration};



pub struct SemanticAnalyzer<'a> {
    context: SemanticContext<'a>,
}

pub struct SemanticContext<'a> {
    scopes: Option<Box<SemanticScope<'a>>>
}

pub struct SemanticScope<'a> {
    parent: Option<Box<SemanticScope<'a>>>,
    symbols: HashMap<&'a [u8], Symbol<'a>>,
}

pub enum Symbol<'a> {
    Parameter(&'a Parameter),
    Variable(&'a VariableDeclaration),
}
