use crate::parser::{
    ast::{Program, Statement},
    expressions::{expression::Expression, expression_statement::ExpressionStatement},
};

use super::{eval_error::EvalError, objects::Object};

pub(crate) trait Evaluable {
    fn eval(&self) -> Result<Object, EvalError>;
}

impl Evaluable for Program {
    fn eval(&self) -> Result<Object, EvalError> {
        let mut object: Option<Object> = None;

        for statement in &self.statements {
            object = Some(statement.eval()?);
        }

        match object {
            Some(object) => Ok(object),
            None => Err(EvalError::EmptyProgram),
        }
    }
}

impl Evaluable for Statement {
    fn eval(&self) -> Result<Object, EvalError> {
        match self {
            Statement::Expression(ExpressionStatement { expression }) => expression.eval(),
            Statement::Assign(_) => todo!(),
            Statement::Return(_) => todo!(),
        }
    }
}

impl Evaluable for Expression {
    fn eval(&self) -> Result<Object, EvalError> {
        use Object::*;

        match self {
            Expression::IntegerLiteral(number) => Ok(Integer(*number)),
            Expression::IdentifierLiteral(_) => todo!(),
            Expression::BooleanLiteral(boolean) => Ok(Boolean(*boolean)),
            Expression::Prefix {
                right: _,
                operator: _,
            } => todo!(),
            Expression::Infix {
                left: _,
                right: _,
                operator: _,
            } => todo!(),
            Expression::If(_) => todo!(),
            Expression::Function(_) => todo!(),
            Expression::Call(_) => todo!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        eval::{self, objects::Object},
        parser::test_util,
    };

    #[test]
    fn eval_integer_expression_test() {
        let input_expected: Vec<(&str, i32)> = vec![("5", 5), ("10", 10)];

        let asserter = |expected: &i32, input: &&str| {
            let object = eval::eval(input).expect("Eval failed");

            match object {
                Object::Integer(number) => assert_eq!(&number, expected),
                Object::Boolean(boolean) => {
                    panic!("Should have returned a number, instead got {boolean}")
                }
            }
        };

        test_util::assert_list(input_expected, asserter);
    }

    #[test]
    fn eval_boolean_expression_test() {
        let input_expected: Vec<(&str, bool)> = vec![("true", true), ("false", false)];

        let asserter = |expected: &bool, input: &&str| {
            let object = eval::eval(input).expect("Eval failed");

            match object {
                Object::Boolean(boolean) => assert_eq!(expected, &boolean),
                something_else => panic!("Expected boolean, got {something_else}"),
            }
        };

        test_util::assert_list(input_expected, asserter);
    }
}
