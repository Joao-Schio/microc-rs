use std::collections::HashSet;

use crate::ast::program::Parameter;





pub struct SemanticScope<'a> {
    context : Option<Box<SemanticScope<'a>>>,
    variables: HashSet<&'a Parameter>
}


