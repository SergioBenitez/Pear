#![feature(plugin)]
#![plugin(pear_codegen)]

#[macro_use] extern crate pear;

use pear::parsers::*;
use pear::combinators::*;
use pear::ParseResult;

#[derive(Debug)]
enum Op {
    Add, Sub, Mul, Div
}

#[derive(Debug)]
enum Expr {
    Binary(Op, Box<Expr>, Box<Expr>),
    Int(isize)
}

impl Expr {
    fn eval(&self) -> isize {
        match *self {
            Expr::Binary(Op::Add, ref e1, ref e2) => e1.eval() + e2.eval(),
            Expr::Binary(Op::Sub, ref e1, ref e2) => e1.eval() - e2.eval(),
            Expr::Binary(Op::Mul, ref e1, ref e2) => e1.eval() * e2.eval(),
            Expr::Binary(Op::Div, ref e1, ref e2) => e1.eval() / e2.eval(),
            Expr::Int(val) => val
        }
    }
}

#[parser]
fn int<'a>(string: &mut &'a str) -> ParseResult<&'a str, Expr> {
    let num = take_while(|c| c == '-' || c.is_numeric());
    Expr::Int(from!(num.parse()))
}

#[parser]
fn val<'a>(string: &mut &'a str) -> ParseResult<&'a str, Expr> {
    switch! {
        eat('(') => {
            let expr = surrounded(expr, char::is_whitespace);
            eat(')');
            expr
        },
        _ => int()
    }
}

#[parser]
fn term<'a>(string: &mut &'a str) -> ParseResult<&'a str, Expr> {
    let left = surrounded(val, char::is_whitespace);

	switch! {
		any!(peek('*'), peek('/')) => {
			let op = switch! {
				eat('*') => Op::Mul,
				eat('/') => Op::Div
			};

			let right = surrounded(term, char::is_whitespace);
			Expr::Binary(op, Box::new(left), Box::new(right))
		},
		_ => left
	}
}

#[parser]
fn expr<'a>(ini_string: &mut &'a str) -> ParseResult<&'a str, Expr> {
    let left = surrounded(term, char::is_whitespace);

	switch! {
		any!(peek('+'), peek('-')) => {
			let op = switch! {
				eat('+') => Op::Add,
				eat('-') => Op::Sub
			};

			let right = surrounded(expr, char::is_whitespace);
			Expr::Binary(op, Box::new(left), Box::new(right))
		},
		_ => left
	}
}

fn eval_expr(mut string: &str) -> Option<isize> {
    expr(&mut string).map(|e| e.eval()).ok()
}

fn main() {
    println!("Result: {:?}", eval_expr("(4 * (3 + 2)) * 2"));
    println!("Result: {:?}", eval_expr("-4 + -2 - 3"));
    println!("Result: {:?}", eval_expr("-1"));
}
