/*
 * Copyright (c) 2017 Boucher, Antoni <bouanto@zoho.com>
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

//! Abstract syntax tree for SQL generation.

use std::fmt::{Display, Error, Formatter};

use proc_macro2::Span;
use syn::{Expr, Ident};

use state::tables_singleton;
use types::Type;

pub type Expression = Expr;
pub type FieldList = Vec<Identifier>;
pub type Groups = Vec<Identifier>;
pub type Identifier = String;

/// `Aggregate` for une in SQL Aggregate `Query`.
#[derive(Clone, Debug, Default)]
pub struct Aggregate {
    pub field: Option<Ident>,
    pub function: Identifier,
    pub result_name: Option<Ident>,
}

/// `AggregateFilter` for SQL `Query` (HAVING clause).
#[derive(Debug)]
pub struct AggregateFilter {
    /// The filter value to be compared to `operand2`.
    pub operand1: Aggregate,
    /// The `operator` used to compare `operand1` to `operand2`.
    pub operator: RelationalOperator,
    /// The expression to be compared to `operand1`.
    pub operand2: Expression,
}

/// Aggregate filter expression.
#[derive(Debug)]
pub enum AggregateFilterExpression {
    Filter(AggregateFilter),
    Filters(AggregateFilters),
    NegFilter(Box<AggregateFilterExpression>),
    NoFilters,
    ParenFilter(Box<AggregateFilterExpression>),
    FilterValue(WithSpan<Aggregate>),
}

impl Default for AggregateFilterExpression {
    fn default() -> Self {
        AggregateFilterExpression::NoFilters
    }
}

/// A `Filters` is used to combine `AggregateFilterExpression`s with a `LogicalOperator`.
#[derive(Debug)]
pub struct AggregateFilters {
    /// The `T` to be combined with `operand2`.
    pub operand1: Box<AggregateFilterExpression>,
    /// The `LogicalOperator` used to combine the `FilterExpression`s.
    pub operator: LogicalOperator,
    /// The `T` to be combined with `operand1`.
    pub operand2: Box<AggregateFilterExpression>,
}

/// `Assignment` for use in SQL Insert and Update `Query`.
#[derive(Debug)]
pub struct Assignment {
    pub identifier: Option<Ident>,
    pub operator: WithSpan<AssignementOperator>,
    pub value: Expression,
}

/// `AssignementOperator` for use in SQL Insert and Update `Query`.
#[derive(Debug, PartialEq)]
pub enum AssignementOperator {
    Add,
    Divide,
    Equal,
    Modulo,
    Mul,
    Sub,
}

impl Display for AssignementOperator {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let op =
            match *self {
                AssignementOperator::Add => "+=",
                AssignementOperator::Divide => "/=",
                AssignementOperator::Equal => "=",
                AssignementOperator::Modulo => "%=",
                AssignementOperator::Mul => "*=",
                AssignementOperator::Sub => "-=",
            };
        write!(formatter, "{}", op).unwrap();
        Ok(())
    }
}

/// `Filter` for SQL `Query` (WHERE clause).
#[derive(Debug)]
pub struct Filter {
    /// The filter value to be compared to `operand2`.
    pub operand1: FilterValue,
    /// The `operator` used to compare `operand1` to `operand2`.
    pub operator: RelationalOperator,
    /// The expression to be compared to `operand1`.
    pub operand2: Expression,
}

/// Either a single `Filter`, `Filters`, `NegFilter`, `NoFilters`, `ParenFilter` or a `FilterValue`.
#[derive(Debug)]
pub enum FilterExpression {
    Filter(Filter),
    Filters(Filters),
    NegFilter(Box<FilterExpression>),
    NoFilters,
    ParenFilter(Box<FilterExpression>),
    FilterValue(WithSpan<FilterValue>),
}

impl Default for FilterExpression {
    fn default() -> Self {
        FilterExpression::NoFilters
    }
}

/// A `Filters` is used to combine `FilterExpression`s with a `LogicalOperator`.
#[derive(Debug)]
pub struct Filters {
    /// The `T` to be combined with `operand2`.
    pub operand1: Box<FilterExpression>,
    /// The `LogicalOperator` used to combine the `FilterExpression`s.
    pub operator: LogicalOperator,
    /// The `T` to be combined with `operand1`.
    pub operand2: Box<FilterExpression>,
}

/// Either an identifier or a method call.
#[derive(Debug)]
pub enum FilterValue {
    None,
    Identifier(Ident),
    MethodCall(MethodCall),
}

/// A `Join` with another `joined_table` via a specific `joined_field`.
#[derive(Clone, Debug, Default)]
pub struct Join {
    pub base_field: Identifier,
    pub base_table: Identifier,
    pub joined_field: Identifier,
    pub joined_table: Identifier,
}

/// An SQL LIMIT clause.
#[derive(Clone, Debug)]
pub enum Limit {
    /// [..end]
    EndRange(Expression),
    /// [index]
    Index(Expression),
    /// Not created from a query. It is converted from a `Range`.
    LimitOffset(Expression, Expression),
    /// No limit was specified.
    NoLimit,
    /// [start..end]
    Range(Expression, Expression),
    /// [start..]
    StartRange(Expression),
}

impl Default for Limit {
    fn default() -> Limit {
        Limit::NoLimit
    }
}

/// `LogicalOperator` to combine `Filter`s.
#[derive(Debug, PartialEq)]
pub enum LogicalOperator {
    And,
    Not,
    Or,
}

/// A method call is an abstraction of SQL function call.
#[derive(Debug)]
pub struct MethodCall {
    pub arguments: Vec<Expression>,
    pub method_name: Identifier,
    pub object_name: Ident,
    pub template: String,
}

/// An SQL ORDER BY clause.
#[derive(Debug)]
pub enum Order {
    /// Comes from `sort(field)`.
    Ascending(Identifier),
    /// Comes from `sort(-field)`.
    Descending(Identifier),
}

/// `RelationalOperator` to be used in a `Filter`.
#[derive(Debug)]
pub enum RelationalOperator {
    Equal,
    LesserThan,
    LesserThanEqual,
    NotEqual,
    GreaterThan,
    GreaterThanEqual,
}

/// An SQL `Query`.
#[derive(Debug)]
pub enum Query {
    Aggregate {
        aggregates: Vec<Aggregate>,
        aggregate_filter: AggregateFilterExpression,
        filter: FilterExpression,
        groups: Groups,
        joins: Vec<Join>,
        table: Identifier,
    },
    CreateTable {
        fields: Vec<TypedField>,
        table: Identifier,
    },
    Delete {
        filter: FilterExpression,
        table: Identifier,
    },
    Drop {
        table: Identifier,
    },
    Insert {
        assignments: Vec<Assignment>,
        table: Identifier,
    },
    Select {
        fields: FieldList,
        filter: FilterExpression,
        joins: Vec<Join>,
        limit: Limit,
        order: Vec<Order>,
        table: Identifier,
    },
    Update {
        assignments: Vec<Assignment>,
        filter: FilterExpression,
        table: Identifier,
    },
}

/// The type of the query.
pub enum QueryType {
    AggregateMulti,
    AggregateOne,
    Exec,
    InsertOne,
    SelectMulti,
    SelectOne,
}

/// An SQL field with its type.
#[derive(Debug)]
pub struct TypedField {
    pub identifier: Identifier,
    pub typ: String,
}

/// Get the query table name.
pub fn query_table(query: &Query) -> Identifier {
    let table_name =
        match *query {
            Query::Aggregate { ref table, .. } => table,
            Query::CreateTable { ref table, .. } => table,
            Query::Delete { ref table, .. } => table,
            Query::Drop { ref table, .. } => table,
            Query::Insert { ref table, .. } => table,
            Query::Select { ref table, .. } => table,
            Query::Update { ref table, .. } => table,
        };
    table_name.clone()
}

/// Get the query type.
pub fn query_type(query: &Query) -> QueryType {
    match *query {
        Query::Aggregate { ref groups, .. } => {
            if !groups.is_empty() {
                QueryType::AggregateMulti
            }
            else {
                QueryType::AggregateOne
            }
        },
        Query::Insert { .. } => QueryType::InsertOne,
        Query::Select { ref filter, ref limit, ref table, .. } => {
            let mut typ = QueryType::SelectMulti;
            if let FilterExpression::Filter(ref filter) = *filter {
                let tables = tables_singleton();
                // NOTE: At this stage (code generation), the table and the field exist, hence unwrap().
                let table = tables.get(table).unwrap();
                if let FilterValue::Identifier(ref identifier) = filter.operand1 {
                    if table.fields.get(identifier).unwrap().ty.node == Type::Serial {
                        typ = QueryType::SelectOne;
                    }
                }
            }
            if let Limit::Index(_) = *limit {
                typ = QueryType::SelectOne;
            }
            typ
        },
        Query::CreateTable { .. } | Query::Delete { .. } | Query::Drop { .. } | Query::Update { .. } => QueryType::Exec,
    }
}

#[derive(Debug)]
pub struct WithSpan<T> {
    pub node: T,
    pub span: Span,
}
