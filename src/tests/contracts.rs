macro_rules! lexer_contract {
    ($module:ident, $make_lexer:path, { $($test:item)* }) => {
        mod $module {
            use super::*;

            fn make_lexer(input: &str) -> impl crate::lexer::TLexer {
                ($make_lexer)(input)
            }

            $($test)*
        }
    };
    ($module:ident, $make_lexer:path, $make_failing_lexer:path, { $($test:item)* }) => {
        mod $module {
            use super::*;

            fn make_lexer(input: &str) -> impl crate::lexer::TLexer {
                ($make_lexer)(input)
            }

            fn make_failing_lexer() -> impl crate::lexer::TLexer {
                ($make_failing_lexer)()
            }

            $($test)*
        }
    };
}
