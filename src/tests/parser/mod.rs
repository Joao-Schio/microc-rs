#[macro_use]
mod contracts;

macro_rules! identifier {
    ($bytes:expr) => {
        crate::ast::Identifier::from($bytes.to_vec())
    };
}

mod block;
mod declaration;
mod declaration_array;
mod error;
mod expression;
mod for_statement;
mod if_statement;
mod program;
mod statement;
