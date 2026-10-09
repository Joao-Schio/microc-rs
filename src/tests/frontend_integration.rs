use crate::{
    frontend::analyze_source, frontend_error::FrontendError, lexer::LexerError,
    parser::ParserError, semantic::SemanticError,
};

#[test]
fn validates_complete_program() {
    let source = b"
        int main() {
            int value;
            value = 42;
            return value;
        }
    ";

    let program = analyze_source(&source[..]).expect("valid MicroC source must pass");

    assert_eq!(program.main.body.declarations.len(), 1);
    assert_eq!(program.main.body.statements.len(), 2);
}

#[test]
fn propagates_lexical_error() {
    let result = analyze_source(b"int main() { return @; }".as_slice());

    assert!(matches!(
        result,
        Err(FrontendError::Lexer(LexerError::UnexpectedCharacter { .. }))
    ));
}

#[test]
fn propagates_parser_error() {
    let result = analyze_source(b"int main() { return 42 }".as_slice());

    assert!(matches!(
        result,
        Err(FrontendError::Parser(ParserError::UnexpectedToken { .. }))
    ));
}

#[test]
fn propagates_semantic_error() {
    let result = analyze_source(b"int main() { return missing; }".as_slice());

    assert!(matches!(
        result,
        Err(FrontendError::Semantic(SemanticError::UndeclaredVariable(
            _
        )))
    ));
}
