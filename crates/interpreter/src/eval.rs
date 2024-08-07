use eval_error::EvalError;
use objects::{EnvReference, Object};

use crate::parser::{ParsedProgram, Parser, StatementError};

mod array_evaluator;
pub mod builtin;
pub mod eval_error;
mod expression_evaluator;
pub mod function_evaluator;
pub mod objects;
mod statement_evaluator;

pub enum EvaledProgram {
    ParseError(Vec<StatementError>),
    EvalError(EvalError),
    Valid(Object),
}

pub fn eval(input: &str, env: &mut EnvReference) -> EvaledProgram {
    let mut parser = Parser::new(input);
    let program = parser.parse_program();
    match program {
        ParsedProgram::InvalidProgram(parse_errors) => EvaledProgram::ParseError(parse_errors),
        ParsedProgram::ValidProgram(valid_program) => {
            let evaled = statement_evaluator::eval_statements(&valid_program, env);

            match evaled {
                Ok(evaled) => EvaledProgram::Valid(evaled),
                Err(errored) => EvaledProgram::EvalError(errored),
            }
        }
    }
}
