use std::{
    collections::{HashMap, hash_map::Entry},
    mem,
};

use crate::{
    ast::{
        Identifier,
        expression::Expression,
        program::{GenericFunction, Parameter, Program},
        statement::{
            Assignment, Block, LValue, PrintContent, Statement, StatementKind, Type,
            VariableDeclaration,
        },
    },
    semantic::{ExprType, SemanticError, TSemanticAnalyzer},
};

pub struct SemanticAnalyzer<'a> {
    context: Context<'a>,
    current_return_type: Type,
}

pub struct Context<'a> {
    symbols: HashMap<&'a [u8], Symbol<'a>>,
    parent: Option<Box<Context<'a>>>,
}

pub enum Symbol<'a> {
    Function(&'a GenericFunction),
    Parameter(&'a Parameter),
    Variable(&'a VariableDeclaration),
}

impl<'a> Context<'a> {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Self) -> Self {
        Self {
            symbols: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn declare(
        &mut self,
        identifier: &'a Identifier,
        symbol: Symbol<'a>,
    ) -> Result<(), SemanticError> {
        match self.symbols.entry(identifier.as_bytes()) {
            Entry::Vacant(entry) => {
                entry.insert(symbol);
                Ok(())
            }
            Entry::Occupied(_) => Err(SemanticError::DuplicateDeclaration(identifier.clone())),
        }
    }

    pub fn resolve(&self, name: &[u8]) -> Option<&Symbol<'a>> {
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol);
        }

        self.parent
            .as_deref()
            .and_then(|parent| parent.resolve(name))
    }

    pub fn take_parent(&mut self) -> Option<Self> {
        self.parent.take().map(|parent| *parent)
    }
}

impl Default for Context<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            current_return_type: Type::Int,
        }
    }

    fn enter_scope(&mut self) {
        let parent = mem::take(&mut self.context);
        self.context = Context::with_parent(parent);
    }

    fn leave_scope(&mut self) -> bool {
        let Some(parent) = self.context.take_parent() else {
            return false;
        };

        self.context = parent;
        true
    }

    fn with_scope<T>(
        &mut self,
        analyze: impl FnOnce(&mut Self) -> Result<T, SemanticError>,
    ) -> Result<T, SemanticError> {
        self.enter_scope();
        let result = analyze(self);
        let left_scope = self.leave_scope();
        debug_assert!(left_scope, "entered semantic scope must have a parent");
        result
    }

    fn analyze_block(&mut self, block: &'a Block) -> Result<(), SemanticError> {
        for declaration in &block.declarations {
            self.declare_variable(declaration)?;
        }

        for statement in &block.statements {
            self.analyze_statement(statement)?;
        }

        Ok(())
    }

    fn declare_variable(
        &mut self,
        declaration: &'a VariableDeclaration,
    ) -> Result<(), SemanticError> {
        let identifier = match declaration {
            VariableDeclaration::Scalar { name, .. } | VariableDeclaration::Array { name, .. } => {
                name
            }
        };

        self.context
            .declare(identifier, Symbol::Variable(declaration))
    }

    fn analyze_statement(&mut self, statement: &'a Statement) -> Result<(), SemanticError> {
        let line = statement.line();

        match &statement.kind {
            StatementKind::Assignment(assignment) => self.analyze_assignment(assignment, line),
            StatementKind::Return { value } => {
                let Some(expression) = value else {
                    return Err(SemanticError::MissingReturnValue { line });
                };
                let actual = self.infer_expression_type(expression, line)?;
                Self::require_type(
                    ExprType::Scalar(self.current_return_type),
                    actual,
                    line,
                )
            }
            StatementKind::Print { content } => match content {
                PrintContent::StringConst(_) => Ok(()),
                PrintContent::Expression(expression) => {
                    let actual = self.infer_expression_type(expression, line)?;
                    match actual {
                        ExprType::Scalar(Type::Int | Type::Char)
                        | ExprType::Array(Type::Char) => Ok(()),
                        _ => Err(SemanticError::InvalidPrintType { actual, line }),
                    }
                }
            },
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let condition_type = self.infer_expression_type(condition, line)?;
                Self::require_type(ExprType::Scalar(Type::Int), condition_type, line)?;
                self.analyze_statement(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.analyze_statement(else_branch)?;
                }
                Ok(())
            }
            StatementKind::Block(block) => {
                self.with_scope(|analyzer| analyzer.analyze_block(block))
            }
            StatementKind::For {
                initialization,
                condition,
                update,
                body,
            } => {
                self.analyze_assignment(initialization, line)?;
                let condition_type = self.infer_expression_type(condition, line)?;
                Self::require_type(ExprType::Scalar(Type::Int), condition_type, line)?;
                self.analyze_assignment(update, line)?;
                self.analyze_statement(body)
            }
            StatementKind::Empty => Ok(()),
        }
    }

    fn require_type(
        expected: ExprType,
        actual: ExprType,
        line: usize,
    ) -> Result<(), SemanticError> {
        if actual == expected {
            Ok(())
        } else {
            Err(SemanticError::TypeMismatch {
                expected,
                actual,
                line,
            })
        }
    }

    fn analyze_assignment(
        &self,
        assignment: &Assignment,
        line: usize,
    ) -> Result<(), SemanticError> {
        let expected = self.infer_lvalue_type(&assignment.target, line)?;
        let actual = self.infer_expression_type(&assignment.value, line)?;
        Self::require_type(expected, actual, line)
    }

    fn infer_lvalue_type(
        &self,
        lvalue: &LValue,
        line: usize,
    ) -> Result<ExprType, SemanticError> {
        match lvalue {
            LValue::Identifier(identifier) => {
                let value_type = self.resolve_value_type(identifier, line)?;
                match value_type {
                    ExprType::Scalar(_) => Ok(value_type),
                    ExprType::Array(_) => {
                        Err(SemanticError::NotAssignable(identifier.clone()))
                    }
                }
            }
            LValue::ArrayElement { array, index } => {
                self.infer_array_access_type(array, index, line)
            }
        }
    }

    fn infer_array_access_type(
        &self,
        array: &Identifier,
        index: &Expression,
        line: usize,
    ) -> Result<ExprType, SemanticError> {
        let array_type = self.resolve_value_type(array, line)?;
        let ExprType::Array(element_type) = array_type else {
            return Err(SemanticError::NotAnArray(array.clone()));
        };
        let index_type = self.infer_expression_type(index, line)?;
        Self::require_type(ExprType::Scalar(Type::Int), index_type, line)?;
        Ok(ExprType::Scalar(element_type))
    }

    /// Infer an expression's type without mutating the syntax tree or symbol table.
    /// MicroC deliberately performs no implicit int/char conversions.
    fn infer_expression_type(
        &self,
        expression: &Expression,
        line: usize,
    ) -> Result<ExprType, SemanticError> {
        let int_type = ExprType::Scalar(Type::Int);
        match expression {
            Expression::Integer(_) => Ok(int_type),
            Expression::Char(_) => Ok(ExprType::Scalar(Type::Char)),
            Expression::Identifier(identifier) => self.resolve_value_type(identifier, line),
            Expression::Unary { op, expression } => {
                let operand_type = self.infer_expression_type(expression, line)?;
                if operand_type == int_type {
                    Ok(int_type)
                } else {
                    Err(SemanticError::InvalidUnaryOperand {
                        operator: *op,
                        actual: operand_type,
                        line,
                    })
                }
            }
            Expression::Binary { left, op, right } => {
                let left_type = self.infer_expression_type(left, line)?;
                let right_type = self.infer_expression_type(right, line)?;
                if left_type == int_type && right_type == int_type {
                    Ok(int_type)
                } else {
                    Err(SemanticError::InvalidBinaryOperands {
                        operator: *op,
                        left: left_type,
                        right: right_type,
                        line,
                    })
                }
            }
            Expression::ArrayAccess { array, index } => {
                self.infer_array_access_type(array, index, line)
            }
            Expression::Call { callee, arguments } => {
                let function = match self.context.resolve(callee.as_bytes()) {
                    Some(Symbol::Function(function)) => function,
                    Some(_) => return Err(SemanticError::NotCallable(callee.clone())),
                    None => return Err(SemanticError::UndeclaredFunction(callee.clone())),
                };

                // Validate nested expressions before arity: an undeclared variable
                // in an argument must still be diagnosed precisely.
                let argument_types = arguments
                    .iter()
                    .map(|argument| self.infer_expression_type(argument, line))
                    .collect::<Result<Vec<_>, _>>()?;

                if arguments.len() != function.parameters.len() {
                    return Err(SemanticError::ArgumentCountMismatch {
                        callee: callee.clone(),
                        expected: function.parameters.len(),
                        actual: arguments.len(),
                    });
                }

                for (argument_type, parameter) in
                    argument_types.into_iter().zip(&function.parameters)
                {
                    Self::require_type(
                        ExprType::Scalar(parameter.data_type),
                        argument_type,
                        callee.line,
                    )?;
                }
                Ok(ExprType::Scalar(function.return_type))
            }
        }
    }

    fn resolve_value_type(
        &self,
        identifier: &Identifier,
        line: usize,
    ) -> Result<ExprType, SemanticError> {
        match self.context.resolve(identifier.as_bytes()) {
            Some(Symbol::Parameter(parameter)) => Ok(ExprType::Scalar(parameter.data_type)),
            Some(Symbol::Variable(VariableDeclaration::Scalar { data_type, .. })) => {
                Ok(ExprType::Scalar(*data_type))
            }
            Some(Symbol::Variable(VariableDeclaration::Array { data_type, .. })) => {
                Ok(ExprType::Array(*data_type))
            }
            Some(Symbol::Function(_)) => Err(SemanticError::NotAVariable(identifier.clone())),
            None => Err(SemanticError::UndeclaredVariable(Identifier {
                name: identifier.name.clone(),
                line,
            })),
        }
    }

    fn analyze_function(&mut self, function: &'a GenericFunction) -> Result<(), SemanticError> {
        // Function return types form a nested semantic context, just like names.
        // Restore it even when a declaration or statement fails.
        let previous_return_type = mem::replace(&mut self.current_return_type, function.return_type);
        let result = self.with_scope(|analyzer| {
            for parameter in &function.parameters {
                analyzer
                    .context
                    .declare(&parameter.name, Symbol::Parameter(parameter))?;
            }

            analyzer.analyze_block(&function.body)
        });
        self.current_return_type = previous_return_type;
        result
    }
}

impl Default for SemanticAnalyzer<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> TSemanticAnalyzer<'a> for SemanticAnalyzer<'a> {
    fn analyze(&mut self, program: &'a Program) -> Result<(), SemanticError> {
        self.context = Context::new();
        self.current_return_type = Type::Int;

        // Register every function before analyzing any body so forward calls and
        // recursion resolve through the enclosing scope.
        for function in &program.functions {
            self.context
                .declare(&function.name, Symbol::Function(function))?;
        }

        for function in &program.functions {
            self.analyze_function(function)?;
        }

        self.with_scope(|analyzer| analyzer.analyze_block(&program.main.body))
    }
}

#[cfg(test)]
mod tests {
    use super::{Context, SemanticAnalyzer, Symbol};
    use crate::{
        ast::{
            Identifier,
            expression::{BinaryOp, Expression},
            program::{GenericFunction, MainFunction, Parameter, Program},
            statement::{
                Assignment, Block, LValue, Statement, StatementKind, Type, VariableDeclaration,
            },
        },
        semantic::{ExprType, SemanticError, TSemanticAnalyzer},
    };

    fn identifier(name: &[u8], line: usize) -> Identifier {
        Identifier::new(name.to_vec(), line)
    }

    fn parameter(name: &[u8]) -> Parameter {
        Parameter {
            data_type: Type::Int,
            name: identifier(name, 1),
        }
    }

    fn scalar(name: &[u8], line: usize) -> VariableDeclaration {
        VariableDeclaration::Scalar {
            data_type: Type::Int,
            name: identifier(name, line),
        }
    }

    fn array(name: &[u8], line: usize) -> VariableDeclaration {
        VariableDeclaration::Array {
            data_type: Type::Int,
            name: identifier(name, line),
            length: 8,
        }
    }

    fn assignment(line: usize, target: LValue, value: Expression) -> Statement {
        Statement::new(
            line,
            StatementKind::Assignment(Assignment { target, value }),
        )
    }

    fn program(declarations: Vec<VariableDeclaration>, statements: Vec<Statement>) -> Program {
        Program {
            functions: vec![],
            main: MainFunction {
                body: Block {
                    declarations,
                    statements,
                },
            },
        }
    }

    fn generic_function(
        name: &[u8],
        line: usize,
        statements: Vec<Statement>,
    ) -> GenericFunction {
        GenericFunction {
            return_type: Type::Int,
            name: identifier(name, line),
            parameters: vec![],
            body: Block {
                declarations: vec![],
                statements,
            },
        }
    }

    fn return_expression(line: usize, value: Expression) -> Statement {
        Statement::new(line, StatementKind::Return { value: Some(value) })
    }

    fn function_call(name: &[u8], line: usize, arguments: Vec<Expression>) -> Expression {
        Expression::Call {
            callee: identifier(name, line),
            arguments,
        }
    }

    fn char_scalar(name: &[u8], line: usize) -> VariableDeclaration {
        VariableDeclaration::Scalar {
            data_type: Type::Char,
            name: identifier(name, line),
        }
    }

    fn binary_add(left: Expression, right: Expression) -> Expression {
        Expression::Binary {
            left: Box::new(left),
            op: BinaryOp::Add,
            right: Box::new(right),
        }
    }

    #[test]
    fn infers_integer_and_character_literals_separately() {
        let analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.infer_expression_type(&Expression::Integer(1), 1),
            Ok(ExprType::Scalar(Type::Int))
        );
        assert_eq!(
            analyzer.infer_expression_type(&Expression::Char(b'a'), 1),
            Ok(ExprType::Scalar(Type::Char))
        );
    }

    #[test]
    fn rejects_mixed_integer_and_character_addition() {
        let expression = binary_add(Expression::Integer(1), Expression::Char(b'a'));
        let program = program(vec![], vec![return_expression(4, expression)]);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::InvalidBinaryOperands {
                operator: BinaryOp::Add,
                left: ExprType::Scalar(Type::Int),
                right: ExprType::Scalar(Type::Char),
                line: 4,
            })
        );
    }

    #[test]
    fn rejects_character_arithmetic_without_implicit_promotions() {
        let expression = binary_add(Expression::Char(b'a'), Expression::Char(b'b'));
        let program = program(vec![], vec![return_expression(4, expression)]);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::InvalidBinaryOperands {
                operator: BinaryOp::Add,
                left: ExprType::Scalar(Type::Char),
                right: ExprType::Scalar(Type::Char),
                line: 4,
            })
        );
    }

    #[test]
    fn accepts_integer_arithmetic() {
        let expression = binary_add(Expression::Integer(1), Expression::Integer(2));
        let program = program(vec![], vec![return_expression(4, expression)]);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn rejects_character_value_assigned_to_int() {
        let program = program(
            vec![scalar(b"value", 1)],
            vec![assignment(
                2,
                LValue::Identifier(identifier(b"value", 2)),
                Expression::Char(b'a'),
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Int),
                actual: ExprType::Scalar(Type::Char),
                line: 2,
            })
        );
    }

    #[test]
    fn rejects_integer_value_assigned_to_char() {
        let program = program(
            vec![char_scalar(b"value", 1)],
            vec![assignment(
                2,
                LValue::Identifier(identifier(b"value", 2)),
                Expression::Integer(97),
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Char),
                actual: ExprType::Scalar(Type::Int),
                line: 2,
            })
        );
    }

    #[test]
    fn accepts_matching_character_assignment() {
        let program = program(
            vec![char_scalar(b"value", 1)],
            vec![assignment(
                2,
                LValue::Identifier(identifier(b"value", 2)),
                Expression::Char(b'a'),
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn rejects_int_argument_to_char_parameter() {
        let mut p = program(
            vec![],
            vec![return_expression(5, function_call(b"accept", 5, vec![Expression::Integer(97)]))],
        );
        let mut function = generic_function(b"accept", 1, vec![
            return_expression(2, Expression::Integer(1))
        ]);
        function.parameters = vec![Parameter {
            data_type: Type::Char,
            name: identifier(b"value", 1),
        }];
        p.functions.push(function);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Char),
                actual: ExprType::Scalar(Type::Int),
                line: 5,
            })
        );
    }

    #[test]
    fn rejects_char_argument_to_int_parameter() {
        let mut p = program(
            vec![],
            vec![return_expression(5, function_call(b"accept", 5, vec![Expression::Char(b'a')]))],
        );
        let mut function = generic_function(b"accept", 1, vec![
            return_expression(2, Expression::Integer(1))
        ]);
        function.parameters.push(parameter(b"value"));
        p.functions.push(function);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Int),
                actual: ExprType::Scalar(Type::Char),
                line: 5,
            })
        );
    }

    #[test]
    fn accepts_char_argument_to_char_parameter() {
        let mut p = program(
            vec![],
            vec![return_expression(5, function_call(b"accept", 5, vec![Expression::Char(b'a')]))],
        );
        let mut function = generic_function(b"accept", 1, vec![
            return_expression(2, Expression::Integer(1))
        ]);
        function.parameters.push(Parameter {
            data_type: Type::Char,
            name: identifier(b"value", 1),
        });
        p.functions.push(function);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(analyzer.analyze(&p), Ok(()));
    }

    #[test]
    fn rejects_function_call_with_wrong_argument_count() {
        let mut p = program(
            vec![],
            vec![return_expression(5, function_call(b"accept", 5, vec![]))],
        );
        let mut function = generic_function(b"accept", 1, vec![]);
        function.parameters.push(parameter(b"value"));
        p.functions.push(function);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::ArgumentCountMismatch {
                callee: identifier(b"accept", 5),
                expected: 1,
                actual: 0,
            })
        );
    }

    #[test]
    fn rejects_char_returned_from_int_function() {
        let mut p = program(vec![], vec![]);
        p.functions.push(generic_function(
            b"wrong",
            1,
            vec![return_expression(3, Expression::Char(b'a'))],
        ));
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Int),
                actual: ExprType::Scalar(Type::Char),
                line: 3,
            })
        );
    }

    #[test]
    fn accepts_char_returned_from_char_function() {
        let mut p = program(vec![], vec![]);
        let mut f = generic_function(
            b"character",
            1,
            vec![return_expression(3, Expression::Char(b'a'))],
        );
        f.return_type = Type::Char;
        p.functions.push(f);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(analyzer.analyze(&p), Ok(()));
    }

    #[test]
    fn restores_main_return_type_after_checking_char_function() {
        let mut p = program(vec![], vec![return_expression(8, Expression::Char(b'a'))]);
        let mut f = generic_function(
            b"character",
            1,
            vec![return_expression(3, Expression::Char(b'b'))],
        );
        f.return_type = Type::Char;
        p.functions.push(f);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Int),
                actual: ExprType::Scalar(Type::Char),
                line: 8,
            })
        );
    }

    #[test]
    fn rejects_character_array_index() {
        let p = program(
            vec![array(b"items", 1)],
            vec![return_expression(
                3,
                Expression::ArrayAccess {
                    array: identifier(b"items", 3),
                    index: Box::new(Expression::Char(b'a')),
                },
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Int),
                actual: ExprType::Scalar(Type::Char),
                line: 3,
            })
        );
    }

    #[test]
    fn rejects_indexing_scalar_variable() {
        let p = program(
            vec![scalar(b"item", 1)],
            vec![return_expression(
                3,
                Expression::ArrayAccess {
                    array: identifier(b"item", 3),
                    index: Box::new(Expression::Integer(0)),
                },
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::NotAnArray(identifier(b"item", 3)))
        );
    }

    #[test]
    fn rejects_array_in_arithmetic() {
        let p = program(
            vec![array(b"items", 1)],
            vec![return_expression(
                3,
                binary_add(
                    Expression::Identifier(identifier(b"items", 3)),
                    Expression::Integer(2),
                ),
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::InvalidBinaryOperands {
                operator: BinaryOp::Add,
                left: ExprType::Array(Type::Int),
                right: ExprType::Scalar(Type::Int),
                line: 3,
            })
        );
    }

    #[test]
    fn rejects_assignment_to_an_entire_array() {
        let p = program(
            vec![array(b"items", 1)],
            vec![assignment(
                3,
                LValue::Identifier(identifier(b"items", 3)),
                Expression::Integer(42),
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::NotAssignable(identifier(b"items", 3)))
        );
    }

    #[test]
    fn rejects_char_condition_without_conversion() {
        let p = program(
            vec![],
            vec![Statement::new(
                3,
                StatementKind::If {
                    condition: Expression::Char(b'a'),
                    then_branch: Box::new(Statement::new(4, StatementKind::Empty)),
                    else_branch: None,
                },
            )],
        );
        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::TypeMismatch {
                expected: ExprType::Scalar(Type::Int),
                actual: ExprType::Scalar(Type::Char),
                line: 3,
            })
        );
    }

    #[test]
    fn rejects_missing_return_value() {
        let p = program(
            vec![],
            vec![Statement::new(3, StatementKind::Return { value: None })],
        );
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&p),
            Err(SemanticError::MissingReturnValue { line: 3 })
        );
    }

    #[test]
    fn resolves_function_symbol_from_context() {
        let function = generic_function(b"helper", 1, vec![]);
        let mut context = Context::new();
        context
            .declare(&function.name, Symbol::Function(&function))
            .expect("function declaration should succeed");

        match context.resolve(b"helper") {
            Some(Symbol::Function(found)) => assert!(std::ptr::eq(*found, &function)),
            _ => panic!("expected function symbol"),
        }
    }

    #[test]
    fn calls_declared_function_from_main() {
        let mut program = program(
            vec![],
            vec![return_expression(5, function_call(b"helper", 5, vec![]))],
        );
        program.functions.push(generic_function(
            b"helper",
            1,
            vec![return_expression(2, Expression::Integer(42))],
        ));

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn resolves_forward_function_reference() {
        let mut program = program(vec![], vec![]);
        program.functions = vec![
            generic_function(
                b"first",
                1,
                vec![return_expression(2, function_call(b"second", 2, vec![]))],
            ),
            generic_function(
                b"second",
                4,
                vec![return_expression(5, Expression::Integer(42))],
            ),
        ];

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn resolves_recursive_function_reference() {
        let mut program = program(vec![], vec![]);
        program.functions.push(generic_function(
            b"recurse",
            1,
            vec![return_expression(2, function_call(b"recurse", 2, vec![]))],
        ));

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn rejects_duplicate_function_names() {
        let mut program = program(vec![], vec![]);
        program.functions = vec![
            generic_function(b"helper", 1, vec![]),
            generic_function(b"helper", 5, vec![]),
        ];

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::DuplicateDeclaration(identifier(b"helper", 5)))
        );
    }

    #[test]
    fn rejects_call_to_local_variable() {
        let program = program(
            vec![scalar(b"value", 1)],
            vec![return_expression(2, function_call(b"value", 2, vec![]))],
        );

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::NotCallable(identifier(b"value", 2)))
        );
    }

    #[test]
    fn rejects_call_to_parameter() {
        let mut program = program(vec![], vec![]);
        let mut function = generic_function(
            b"helper",
            1,
            vec![return_expression(2, function_call(b"value", 2, vec![]))],
        );
        function.parameters.push(parameter(b"value"));
        program.functions.push(function);

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::NotCallable(identifier(b"value", 2)))
        );
    }

    #[test]
    fn rejects_function_used_as_variable() {
        let mut program = program(
            vec![],
            vec![return_expression(
                5,
                Expression::Identifier(identifier(b"helper", 5)),
            )],
        );
        program.functions.push(generic_function(b"helper", 1, vec![]));

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::NotAVariable(identifier(b"helper", 5)))
        );
    }

    #[test]
    fn local_variable_shadows_function_and_is_not_callable() {
        let mut program = program(
            vec![scalar(b"helper", 3)],
            vec![return_expression(4, function_call(b"helper", 4, vec![]))],
        );
        program.functions.push(generic_function(b"helper", 1, vec![]));

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::NotCallable(identifier(b"helper", 4)))
        );
    }

    #[test]
    fn checks_arguments_of_a_valid_function_call() {
        let mut program = program(
            vec![],
            vec![return_expression(
                7,
                function_call(
                    b"helper",
                    7,
                    vec![Expression::Identifier(identifier(b"missing", 7))],
                ),
            )],
        );
        program.functions.push(generic_function(b"helper", 1, vec![]));

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable(identifier(b"missing", 7)))
        );
    }

    #[test]
    fn resolves_symbol_from_current_context() {
        let parameter = parameter(b"value");
        let mut context = Context::new();
        context
            .declare(&parameter.name, Symbol::Parameter(&parameter))
            .expect("unique declaration should succeed");

        match context.resolve(b"value") {
            Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &parameter)),
            _ => panic!("expected parameter from current context"),
        }
    }

    #[test]
    fn resolves_symbol_from_parent_context() {
        let parameter = parameter(b"value");
        let mut parent = Context::new();
        parent
            .declare(&parameter.name, Symbol::Parameter(&parameter))
            .expect("unique declaration should succeed");
        let context = Context::with_parent(parent);

        match context.resolve(b"value") {
            Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &parameter)),
            _ => panic!("expected parameter from parent context"),
        }
    }

    #[test]
    fn current_context_shadows_parent_symbol() {
        let outer = parameter(b"value");
        let inner = parameter(b"value");
        let mut parent = Context::new();
        parent
            .declare(&outer.name, Symbol::Parameter(&outer))
            .expect("unique declaration should succeed");
        let mut context = Context::with_parent(parent);
        context
            .declare(&inner.name, Symbol::Parameter(&inner))
            .expect("unique declaration should succeed");

        match context.resolve(b"value") {
            Some(Symbol::Parameter(found)) => assert!(std::ptr::eq(*found, &inner)),
            _ => panic!("expected parameter from current context"),
        }
    }

    #[test]
    fn analyzer_can_enter_and_leave_scope() {
        let outer = parameter(b"outer");
        let inner = parameter(b"inner");
        let mut analyzer = SemanticAnalyzer::new();
        analyzer
            .context
            .declare(&outer.name, Symbol::Parameter(&outer))
            .expect("unique declaration should succeed");

        analyzer.enter_scope();
        analyzer
            .context
            .declare(&inner.name, Symbol::Parameter(&inner))
            .expect("unique declaration should succeed");

        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_some());
        assert!(analyzer.leave_scope());
        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_none());
        assert!(!analyzer.leave_scope());
    }

    #[test]
    fn analyzer_can_resolve_simple_main() {
        let program = program(
            vec![scalar(b"x", 1)],
            vec![assignment(
                2,
                LValue::Identifier(identifier(b"x", 2)),
                Expression::Integer(20),
            )],
        );

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn undeclared_assignment_target_uses_statement_line() {
        let program = program(
            vec![],
            vec![assignment(
                7,
                LValue::Identifier(identifier(b"missing", 99)),
                Expression::Integer(20),
            )],
        );

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable(Identifier {
                name: b"missing".to_vec(),
                line: 7,
            }))
        );
    }

    #[test]
    fn undeclared_identifier_in_assignment_value_is_rejected() {
        let program = program(
            vec![scalar(b"x", 1)],
            vec![assignment(
                11,
                LValue::Identifier(identifier(b"x", 11)),
                Expression::Identifier(identifier(b"missing", 99)),
            )],
        );

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable(Identifier {
                name: b"missing".to_vec(),
                line: 11,
            }))
        );
    }

    #[test]
    fn analyzer_recurses_through_binary_expressions() {
        let program = program(
            vec![scalar(b"x", 1)],
            vec![assignment(
                5,
                LValue::Identifier(identifier(b"x", 5)),
                Expression::Binary {
                    left: Box::new(Expression::Integer(1)),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Identifier(identifier(b"missing", 5))),
                },
            )],
        );

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable(Identifier {
                name: b"missing".to_vec(),
                line: 5,
            }))
        );
    }

    #[test]
    fn nested_block_can_resolve_parent_variable() {
        let nested = Statement::new(
            2,
            StatementKind::Block(Block {
                declarations: vec![],
                statements: vec![assignment(
                    3,
                    LValue::Identifier(identifier(b"x", 3)),
                    Expression::Integer(1),
                )],
            }),
        );
        let program = program(vec![scalar(b"x", 1)], vec![nested]);

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn nested_block_scope_does_not_leak() {
        let nested = Statement::new(
            2,
            StatementKind::Block(Block {
                declarations: vec![scalar(b"inner", 3)],
                statements: vec![assignment(
                    4,
                    LValue::Identifier(identifier(b"inner", 4)),
                    Expression::Integer(1),
                )],
            }),
        );
        let program = program(
            vec![],
            vec![
                nested,
                assignment(
                    9,
                    LValue::Identifier(identifier(b"inner", 9)),
                    Expression::Integer(2),
                ),
            ],
        );

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable(Identifier {
                name: b"inner".to_vec(),
                line: 9,
            }))
        );
    }

    #[test]
    fn scope_is_restored_when_nested_analysis_fails() {
        let nested = Statement::new(
            2,
            StatementKind::Block(Block {
                declarations: vec![scalar(b"inner", 3)],
                statements: vec![assignment(
                    4,
                    LValue::Identifier(identifier(b"missing", 4)),
                    Expression::Integer(1),
                )],
            }),
        );
        let program = program(vec![scalar(b"outer", 1)], vec![nested]);

        let mut analyzer = SemanticAnalyzer::new();
        assert!(analyzer.analyze_block(&program.main.body).is_err());
        assert!(analyzer.context.resolve(b"outer").is_some());
        assert!(analyzer.context.resolve(b"inner").is_none());
    }

    #[test]
    fn main_scope_is_restored_when_analysis_fails() {
        let program = program(
            vec![scalar(b"outer", 1)],
            vec![assignment(
                2,
                LValue::Identifier(identifier(b"missing", 2)),
                Expression::Integer(1),
            )],
        );

        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable(Identifier {
                name: b"missing".to_vec(),
                line: 2,
            }))
        );
        assert!(analyzer.context.resolve(b"outer").is_none());
    }

    #[test]
    fn analyzer_resolves_array_target_and_index_expression() {
        let program = program(
            vec![array(b"values", 1), scalar(b"index", 2)],
            vec![assignment(
                3,
                LValue::ArrayElement {
                    array: identifier(b"values", 3),
                    index: Box::new(Expression::Identifier(identifier(b"index", 3))),
                },
                Expression::Integer(42),
            )],
        );

        let mut analyzer = SemanticAnalyzer::new();
        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn function_parameters_are_visible_inside_function_body() {
        let program = Program {
            functions: vec![GenericFunction {
                return_type: Type::Int,
                name: identifier(b"add", 1),
                parameters: vec![parameter(b"a"), parameter(b"b")],
                body: Block {
                    declarations: vec![],
                    statements: vec![Statement::new(
                        2,
                        StatementKind::Return {
                            value: Some(Expression::Binary {
                                left: Box::new(Expression::Identifier(identifier(b"a", 2))),
                                op: BinaryOp::Add,
                                right: Box::new(Expression::Identifier(identifier(b"b", 2))),
                            }),
                        },
                    )],
                },
            }],
            main: MainFunction {
                body: Block {
                    declarations: vec![],
                    statements: vec![],
                },
            },
        };

        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn rejects_undeclared_variable_inside_generic_function() {
        let program = Program {
            functions: vec![GenericFunction {
                return_type: Type::Int,
                name: identifier(b"calculate", 1),
                parameters: vec![parameter(b"x")],
                body: Block {
                    declarations: vec![],
                    statements: vec![Statement::new(
                        3,
                        StatementKind::Return {
                            value: Some(Expression::Identifier(identifier(b"missing", 3))),
                        },
                    )],
                },
            }],
            main: MainFunction {
                body: Block {
                    declarations: vec![],
                    statements: vec![],
                },
            },
        };

        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredVariable(Identifier {
                name: b"missing".to_vec(),
                line: 3,
            }))
        );
    }

    #[test]
    fn rejects_duplicate_variable_declaration_in_same_scope() {
        let program = program(vec![scalar(b"value", 1), scalar(b"value", 3)], vec![]);

        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::DuplicateDeclaration(Identifier {
                name: b"value".to_vec(),
                line: 3,
            }))
        );
    }
    #[test]
    fn duplicate_declaration_preserves_original_symbol() {
        let original = scalar(b"value", 1);
        let duplicate = scalar(b"value", 4);
        let VariableDeclaration::Scalar {
            name: original_name,
            ..
        } = &original
        else {
            unreachable!()
        };
        let VariableDeclaration::Scalar {
            name: duplicate_name,
            ..
        } = &duplicate
        else {
            unreachable!()
        };

        let mut context = Context::new();
        context
            .declare(original_name, Symbol::Variable(&original))
            .expect("first declaration should succeed");

        assert_eq!(
            context.declare(duplicate_name, Symbol::Variable(&duplicate)),
            Err(SemanticError::DuplicateDeclaration(identifier(b"value", 4)))
        );

        match context.resolve(b"value") {
            Some(Symbol::Variable(found)) => assert!(std::ptr::eq(*found, &original)),
            _ => panic!("the original declaration should remain in the symbol table"),
        }
    }

    #[test]
    fn rejects_scalar_and_array_with_same_name_in_one_scope() {
        let program = program(vec![scalar(b"value", 1), array(b"value", 5)], vec![]);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::DuplicateDeclaration(identifier(b"value", 5)))
        );
    }

    #[test]
    fn rejects_duplicate_function_parameters() {
        let program = Program {
            functions: vec![GenericFunction {
                return_type: Type::Int,
                name: identifier(b"f", 1),
                parameters: vec![
                    parameter(b"value"),
                    Parameter {
                        data_type: Type::Char,
                        name: identifier(b"value", 2),
                    },
                ],
                body: Block {
                    declarations: vec![],
                    statements: vec![],
                },
            }],
            main: MainFunction {
                body: Block {
                    declarations: vec![],
                    statements: vec![],
                },
            },
        };
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::DuplicateDeclaration(identifier(b"value", 2)))
        );
    }

    #[test]
    fn rejects_local_variable_colliding_with_function_parameter() {
        let program = Program {
            functions: vec![GenericFunction {
                return_type: Type::Int,
                name: identifier(b"f", 1),
                parameters: vec![parameter(b"value")],
                body: Block {
                    declarations: vec![scalar(b"value", 4)],
                    statements: vec![],
                },
            }],
            main: MainFunction {
                body: Block {
                    declarations: vec![],
                    statements: vec![],
                },
            },
        };
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::DuplicateDeclaration(identifier(b"value", 4)))
        );
    }

    #[test]
    fn permits_shadowing_variable_in_nested_block() {
        let nested = Statement::new(
            2,
            StatementKind::Block(Block {
                declarations: vec![scalar(b"value", 3)],
                statements: vec![assignment(
                    4,
                    LValue::Identifier(identifier(b"value", 4)),
                    Expression::Integer(1),
                )],
            }),
        );
        let program = program(vec![scalar(b"value", 1)], vec![nested]);
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn permits_same_parameter_name_in_different_functions() {
        let function = |name: &[u8]| GenericFunction {
            return_type: Type::Int,
            name: identifier(name, 1),
            parameters: vec![parameter(b"value")],
            body: Block {
                declarations: vec![],
                statements: vec![Statement::new(
                    2,
                    StatementKind::Return {
                        value: Some(Expression::Identifier(identifier(b"value", 2))),
                    },
                )],
            },
        };
        let program = Program {
            functions: vec![function(b"first"), function(b"second")],
            main: MainFunction {
                body: Block {
                    declarations: vec![],
                    statements: vec![],
                },
            },
        };
        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(analyzer.analyze(&program), Ok(()));
    }

    #[test]
    fn rejects_call_to_undeclared_function() {
        let program = program(
            vec![],
            vec![Statement::new(
                7,
                StatementKind::Return {
                    value: Some(Expression::Call {
                        callee: identifier(b"missing", 7),
                        arguments: vec![Expression::Integer(42)],
                    }),
                },
            )],
        );

        let mut analyzer = SemanticAnalyzer::new();

        assert_eq!(
            analyzer.analyze(&program),
            Err(SemanticError::UndeclaredFunction(identifier(b"missing", 7)))
        );
    }
}
