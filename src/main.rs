#![forbid(unsafe_code)]
#![allow(dead_code, unused)]

mod ast;
mod lexer;
mod parser;
mod scanner;
mod semantic;
#[cfg(test)]
mod tests;
mod token;



fn main() {
    println!("Hello, world!");
}
