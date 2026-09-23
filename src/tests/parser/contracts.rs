macro_rules! parser_contract {
    ($module:ident, $expr_parser:ty, $statement_parser:ty, { $($test:item)* }) => {
        mod $module {
            use super::*;

            fn make_parser(
                tokens: Vec<crate::token::Token>,
            ) -> crate::parser::Parser<$expr_parser, $statement_parser> {
                crate::parser::Parser::with_parsers(
                    tokens,
                    <$expr_parser as Default>::default(),
                    <$statement_parser as Default>::default(),
                )
            }

            $($test)*
        }
    };
}
