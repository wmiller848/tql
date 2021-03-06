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

//! Tests of the methods related to `Query::Select`.

#![feature(proc_macro_hygiene)]

extern crate tql;
#[macro_use]
extern crate tql_macros;

#[macro_use] 
mod connection;
backend_extern_crate!();

use tql::{ForeignKey, PrimaryKey};
use connection::{Connection, get_connection}; 

#[derive(SqlTable)]
struct Table {
    id: PrimaryKey,
    related: ForeignKey<RelatedTable>,
}
#[derive(SqlTable)]
struct RelatedTable {
    field1: String,
}
fn main() {
    let connection = get_connection();
    sql!(Table.all().join(related));
}
