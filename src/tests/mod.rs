#[macro_use]
mod contracts;

mod codegen;
mod comments;
mod diagnostics;
mod frontend_error;
mod frontend_integration;
mod helpers;
mod lexer;
mod numeric;
mod parser;
mod reserved_words;
pub(crate) mod semantic;
mod token_contract;
mod tokenize;
