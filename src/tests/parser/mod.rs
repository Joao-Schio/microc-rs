#[macro_use]
mod contracts;

macro_rules! identifier {
    ($bytes:expr) => {
        crate::ast::Identifier::new($bytes.to_vec(), 1)
    };
    ($bytes:expr, $line:expr) => {
        crate::ast::Identifier::new($bytes.to_vec(), $line)
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
