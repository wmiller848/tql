//! Rust parsing.

use syntax::ast::Expr;
use syntax::ast::Expr_::{ExprIndex, ExprMethodCall, ExprPath};
use syntax::codemap::{Span, Spanned};
use syntax::ptr::P;

use ast::Expression;
use error::{Error, SqlResult, res};

/// A method call.
#[derive(Debug)]
pub struct MethodCall {
    pub arguments: Vec<P<Expr>>,
    pub name: String,
    pub position: Span,
}

/// A collection of method calls.
#[derive(Debug)]
pub struct MethodCalls {
    pub calls: Vec<MethodCall>,
    /// The identifier at the start of the calls chain.
    pub name: String,
    pub position: Span,
}

impl MethodCalls {
    /// Add a call to the method calls `Vec`.
    fn push(&mut self, call: MethodCall) {
        self.calls.push(call);
    }
}

/// Convert a method call expression to a simpler vector-based structure.
pub fn parse<'a>(expression: Expression) -> SqlResult<'a, MethodCalls> {
    let mut errors = vec![];
    let mut calls = MethodCalls {
        calls: vec![],
        name:  "".to_owned(),
        position: expression.span,
    };

    /// Add the calls from the `expression` into the `calls` `Vec`.
    fn add_calls<'a>(expression: &Expression, calls: &mut MethodCalls, errors: &mut Vec<Error>) {
        match expression.node {
            ExprMethodCall(Spanned { node: object, span: method_span}, _, ref arguments) => {
                add_calls(&arguments[0], calls, errors);

                let mut arguments = arguments.clone();
                arguments.remove(0);

                calls.push(MethodCall {
                    name: object.to_string(),
                    arguments: arguments,
                    position: method_span,
                });
            }
            ExprPath(_, ref path) => {
                if path.segments.len() == 1 {
                    calls.name = path.segments[0].identifier.to_string();
                }
            }
            ExprIndex(ref expr1, ref expr2) => {
                add_calls(expr1, calls, errors);
                calls.push(MethodCall {
                    name: "limit".to_owned(),
                    arguments: vec![expr2.clone()],
                    position: expr2.span,
                });
            }
            _ => {
                errors.push(Error::new(
                    "Expected method call".to_owned(),
                    expression.span,
                ));
            }
        }
    }

    add_calls(&expression, &mut calls, &mut errors);
    res(calls, errors)
}
