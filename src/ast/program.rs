use crate::ast::statement::{Block, Type};

#[derive(Debug, PartialEq, Eq)]
pub struct Program {
    pub functions: Vec<GenericFunction>,
    pub main: MainFunction,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GenericFunction {
    pub return_type: Type,
    pub name: Vec<u8>,
    pub parameters: Vec<Parameter>,
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MainFunction {
    pub body: Block,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Parameter {
    pub data_type: Type,
    pub name: Vec<u8>,
}
