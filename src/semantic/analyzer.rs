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
                Self::require_type(ExprType::Scalar(self.current_return_type), actual, line)
            }
            StatementKind::Print { content } => match content {
                PrintContent::StringConst(_) => Ok(()),
                PrintContent::Expression(expression) => {
                    let actual = self.infer_expression_type(expression, line)?;
                    match actual {
                        ExprType::Scalar(Type::Int | Type::Char) | ExprType::Array(Type::Char) => {
                            Ok(())
                        }
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

    fn infer_lvalue_type(&self, lvalue: &LValue, line: usize) -> Result<ExprType, SemanticError> {
        match lvalue {
            LValue::Identifier(identifier) => {
                let value_type = self.resolve_value_type(identifier, line)?;
                match value_type {
                    ExprType::Scalar(_) => Ok(value_type),
                    ExprType::Array(_) => Err(SemanticError::NotAssignable(identifier.clone())),
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
        let previous_return_type =
            mem::replace(&mut self.current_return_type, function.return_type);
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
#[path = "../tests/semantic/internal.rs"]
mod tests;
