/*
 * Copyright (c) 2017-2018 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

//! Query arguments extractor.

use syn::{
    Expr,
    Ident,
    parse,
};

use ast::{
    Aggregate,
    AggregateFilterExpression,
    Assignment,
    Expression,
    FilterExpression,
    FilterValue,
    Limit,
    MethodCall,
    Query,
};

/// A Rust expression to be send as a parameter to the SQL query function.
#[derive(Clone, Debug)]
pub struct Arg {
    pub expression: Expression,
    pub field_name: Option<Ident>,
    pub field_name_prefix: Option<String>,
}

/// A collection of `Arg`s.
pub type Args = Vec<Arg>;

/// Create an argument from the parameters and add it to `arguments`.
fn add(arguments: &mut Args, literals: &mut Args, field_name: Option<Ident>, field_name_prefix: Option<String>,
       expr: Expression)
{
    add_expr(arguments, literals, Arg {
        expression: expr,
        field_name_prefix,
        field_name,
    });
}

/// Create arguments from the `assignments` and add them to `arguments`.
fn add_assignments(assignments: Vec<Assignment>, arguments: &mut Args, literals: &mut Args) {
    for assign in assignments {
        let field_name = assign.identifier.expect("Assignment identifier");
        // NOTE: At this stage (code generation), the field exists, hence unwrap().
        add(arguments, literals, Some(field_name), None, assign.value);
    }
}

/// Add an argument to `arguments`.
fn add_expr(arguments: &mut Args, literals: &mut Args, arg: Arg) {
    // Do not add literal.
    if let Expr::Lit(_) = arg.expression {
        literals.push(arg);
        return;
    }
    arguments.push(arg);
}

/// Create arguments from the `filter` and add them to `arguments`.
fn add_filter_arguments(filter: FilterExpression, args: &mut Args, literals: &mut Args) {
    match filter {
        FilterExpression::Filter(filter) => {
            add_filter_value_arguments(&filter.operand1, args, literals, Some(filter.operand2));
        },
        FilterExpression::Filters(filters) => {
            add_filter_arguments(*filters.operand1, args, literals);
            add_filter_arguments(*filters.operand2, args, literals);
        },
        FilterExpression::NegFilter(filter) => {
            add_filter_arguments(*filter, args, literals);
        },
        FilterExpression::NoFilters => (),
        FilterExpression::ParenFilter(filter) => {
            add_filter_arguments(*filter, args, literals);
        },
        FilterExpression::FilterValue(filter_value) => {
            add_filter_value_arguments(&filter_value.node, args, literals, None);
        },
    }
}

/// Create arguments from the `filter` and add them to `arguments`.
fn add_aggregate_filter_arguments(filter: AggregateFilterExpression, args: &mut Args, literals: &mut Args) {
    match filter {
        AggregateFilterExpression::Filter(filter) => {
            add_aggregate_filter_value_arguments(&filter.operand1, args, literals, Some(filter.operand2));
        },
        AggregateFilterExpression::Filters(filters) => {
            add_aggregate_filter_arguments(*filters.operand1, args, literals);
            add_aggregate_filter_arguments(*filters.operand2, args, literals);
        },
        AggregateFilterExpression::NegFilter(filter) => {
            add_aggregate_filter_arguments(*filter, args, literals);
        },
        AggregateFilterExpression::NoFilters => (),
        AggregateFilterExpression::ParenFilter(filter) => {
            add_aggregate_filter_arguments(*filter, args, literals);
        },
        AggregateFilterExpression::FilterValue(filter_value) => {
            add_aggregate_filter_value_arguments(&filter_value.node, args, literals, None);
        },
    }
}

/// Create arguments from the `limit` and add them to `arguments`.
fn add_limit_arguments(limit: Limit, arguments: &mut Args, literals: &mut Args) {
    match limit {
        Limit::EndRange(expression) => add(arguments, literals, None, None, expression),
        Limit::Index(expression) => add(arguments, literals, None, None, expression),
        Limit::LimitOffset(_, _) => (), // NOTE: there are no arguments to add for a `LimitOffset` because it is always using literals.
        Limit::NoLimit => (),
        Limit::Range(expression1, expression2) => {
            let offset = expression1.clone();
            let expression = parse((quote! { #expression2 - #offset }).into())
                .expect("Subtraction quoted expression");
            add_expr(arguments, literals, Arg {
                expression,
                field_name: None,
                field_name_prefix: None,
            });
            add(arguments, literals, None, None, expression1);
        },
        Limit::StartRange(expression) => add(arguments, literals, None, None, expression),
    }
}

/// Construct an argument from the method and add it to `args`.
fn add_with_method(args: &mut Args, literals: &mut Args, expr: Expression)
{
    add_expr(args, literals, Arg {
        expression: expr,
        field_name: None,
        field_name_prefix: None,
    });
}

fn add_aggregate_filter_value_arguments(_aggregate: &Aggregate, args: &mut Args, literals: &mut Args,
                                        expression: Option<Expression>)
{
    if let Some(expr) = expression {
        add(args, literals, None, None, expr);
    }
}

fn add_filter_value_arguments(filter_value: &FilterValue, args: &mut Args, literals: &mut Args,
                              expression: Option<Expression>)
{
    match *filter_value {
        FilterValue::Identifier(ref table, ref identifier) => {
            // It is possible to have an identifier without expression, when the identifier is a
            // boolean field name, hence this condition.
            if let Some(expr) = expression {
                add(args, literals, Some(identifier.clone()), Some(table.clone()), expr);
            }
        },
        FilterValue::MethodCall(MethodCall { ref arguments, .. }) => {
            for arg in arguments {
                add_with_method(args, literals, arg.clone());
            }
        },
        FilterValue::None => unreachable!("FilterValue::None in add_filter_value_arguments()"),
        FilterValue::PrimaryKey(ref table) => {
            if let Some(expr) = expression {
                add(args, literals, None, Some(table.clone()), expr);
            }
        },
    }
}

/// Extract the Rust `Expression`s, the literal arguments and identifiers from the `Query`.
pub fn arguments(query: Query) -> (Args, Args) {
    let mut arguments = vec![];
    let mut literals = vec![];

    match query {
        Query::Aggregate { aggregate_filter, filter, .. } => {
            add_filter_arguments(filter, &mut arguments, &mut literals);
            add_aggregate_filter_arguments(aggregate_filter, &mut arguments, &mut literals);
        },
        Query::CreateTable { .. } => (), // No arguments.
        Query::Delete { filter, .. } => {
            add_filter_arguments(filter, &mut arguments, &mut literals);
        },
        Query::Drop { .. } => (), // No arguments.
        Query::Insert { assignments, .. } => {
            add_assignments(assignments, &mut arguments, &mut literals);
        },
        Query::Select { filter, limit, ..} => {
            add_filter_arguments(filter, &mut arguments, &mut literals);
            add_limit_arguments(limit, &mut arguments, &mut literals);
        },
        Query::Update { assignments, filter, .. } => {
            add_assignments(assignments, &mut arguments, &mut literals);
            add_filter_arguments(filter, &mut arguments, &mut literals);
        },
    }

    (arguments, literals)
}
