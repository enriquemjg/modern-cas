//! Tokenization, Pratt parsing, and exact evaluation of polynomial expressions.

use std::fmt;

use num_bigint::BigInt;

use crate::Rational;
use crate::polynomial::{Polynomial, PolynomialRing};
use crate::rational_function::RationalFunction;
use crate::value::{Matrix, Value};

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub column: usize,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "column {}: {}", self.column, self.message)
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind {
    Integer(String),
    Identifier(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Open,
    Close,
    Comma,
    Semicolon,
    OpenBracket,
    CloseBracket,
    End,
}

struct Token {
    kind: Kind,
    column: usize,
}

fn error(column: usize, message: impl Into<String>) -> Error {
    Error {
        column,
        message: message.into(),
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().enumerate().peekable();
    while let Some((offset, ch)) = chars.next() {
        let column = offset + 1;
        let kind = match ch {
            ch if ch.is_whitespace() => continue,
            '0'..='9' => {
                let mut digits = String::from(ch);
                while let Some(&(_, digit)) = chars.peek() {
                    if !digit.is_ascii_digit() {
                        break;
                    }
                    digits.push(digit);
                    chars.next();
                }
                Kind::Integer(digits)
            }
            ch if ch.is_ascii_alphabetic() || ch == '_' => {
                let mut name = String::from(ch);
                while let Some(&(_, next)) = chars.peek() {
                    if !next.is_ascii_alphanumeric() && next != '_' {
                        break;
                    }
                    name.push(next);
                    chars.next();
                }
                Kind::Identifier(name)
            }
            '+' => Kind::Plus,
            '-' => Kind::Minus,
            '*' => Kind::Star,
            '/' => Kind::Slash,
            '^' => Kind::Caret,
            '(' => Kind::Open,
            ')' => Kind::Close,
            ',' => Kind::Comma,
            ';' => Kind::Semicolon,
            '[' => Kind::OpenBracket,
            ']' => Kind::CloseBracket,
            _ => {
                return Err(error(column, format!("unexpected character '{ch}'")));
            }
        };
        tokens.push(Token { kind, column });
        if tokens.len() > 1024 {
            return Err(error(column, "expression token limit exceeded (1024)"));
        }
    }
    tokens.push(Token {
        kind: Kind::End,
        column: input.chars().count() + 1,
    });
    Ok(tokens)
}

enum Expr {
    Number(Rational),
    Variable(String),
    Call(String, Vec<(Expr, usize)>, usize),
    Unary(bool, Box<Expr>, usize),
    Tuple(Vec<Expr>),
    Matrix(Vec<Vec<(Expr, usize)>>),
    Binary(Kind, Box<Expr>, Box<Expr>, usize),
    Power(Box<Expr>, u32, bool, usize),
}

fn polynomial(value: Value, column: usize) -> Result<Polynomial, Error> {
    match value {
        Value::Polynomial(value) => Ok(value),
        Value::RationalFunction(_) => Err(error(
            column,
            "expected a polynomial, found a rational function",
        )),
        Value::Tuple(_) => Err(error(column, "expected a polynomial, found a tuple")),
        Value::Matrix(_) => Err(error(column, "expected a polynomial, found a matrix")),
    }
}

fn scalar(value: Value, column: usize) -> Result<RationalFunction, Error> {
    match value {
        Value::Polynomial(value) => Ok(RationalFunction::from_polynomial(value)),
        Value::RationalFunction(value) => Ok(value),
        Value::Matrix(_) => Err(error(
            column,
            "expected a polynomial or rational function, found a matrix",
        )),
        Value::Tuple(_) => Err(error(
            column,
            "expected a polynomial or rational function, found a tuple",
        )),
    }
}

impl Expr {
    fn evaluate(self, ring: &mut PolynomialRing) -> Result<Value, Error> {
        let value = match self {
            Self::Call(name, arguments, column) => {
                if name == "gcd" {
                    if arguments.len() != 2 {
                        return Err(error(column, "gcd expects two polynomials"));
                    }
                    let mut arguments = arguments.into_iter();
                    let (left, left_column) = arguments.next().unwrap();
                    let left = polynomial(left.evaluate(ring)?, left_column)?;
                    let (right, right_column) = arguments.next().unwrap();
                    let right = polynomial(right.evaluate(ring)?, right_column)?;
                    return ring
                        .gcd(&left, &right)
                        .map(Value::Polynomial)
                        .map_err(|message| error(column, message));
                }
                if name == "groebner" {
                    if arguments.len() != 1 {
                        return Err(error(column, "groebner expects one polynomial vector"));
                    }
                    let (argument, argument_column) = arguments.into_iter().next().unwrap();
                    let Value::Matrix(generators) = argument.evaluate(ring)? else {
                        return Err(error(
                            argument_column,
                            "groebner expects a polynomial vector",
                        ));
                    };
                    if generators.rows() > 1 && generators.columns() > 1 {
                        return Err(error(
                            argument_column,
                            "groebner expects a row or column vector, not a matrix",
                        ));
                    }
                    let basis = ring
                        .groebner_basis(generators.entries())
                        .map_err(|message| error(column, message))?;
                    let columns = basis.len();
                    return Matrix::new(usize::from(columns != 0), columns, basis)
                        .map(Value::Matrix)
                        .map_err(|message| error(column, message));
                }
                if name != "div" {
                    return Err(error(column, format!("unknown function '{name}'")));
                }
                if arguments.len() != 2 {
                    return Err(error(
                        column,
                        "div expects two arguments: a polynomial and a divisor vector",
                    ));
                }
                let mut arguments = arguments.into_iter();
                let (dividend, dividend_column) = arguments.next().unwrap();
                let dividend = polynomial(dividend.evaluate(ring)?, dividend_column)?;
                let (divisors, divisor_column) = arguments.next().unwrap();
                let Value::Matrix(divisors) = divisors.evaluate(ring)? else {
                    return Err(error(
                        divisor_column,
                        "div expects a vector of polynomial divisors",
                    ));
                };
                if divisors.rows() > 1 && divisors.columns() > 1 {
                    return Err(error(
                        divisor_column,
                        "div expects a row or column vector, not a matrix",
                    ));
                }
                if let Some(index) = divisors.entries().iter().position(Polynomial::is_zero) {
                    return Err(error(
                        divisor_column,
                        format!("divisor {} is the zero polynomial", index + 1),
                    ));
                }
                let result = ring
                    .divide(&dividend, divisors.entries())
                    .map_err(|message| error(column, message))?;
                let quotients = Matrix::new(divisors.rows(), divisors.columns(), result.quotients)
                    .map_err(|message| error(column, message))?;
                return Ok(Value::Tuple(vec![
                    Value::Matrix(quotients),
                    Value::Polynomial(result.remainder),
                ]));
            }
            Self::Tuple(elements) => {
                return elements
                    .into_iter()
                    .map(|e| e.evaluate(ring))
                    .collect::<Result<Vec<_>, _>>()
                    .map(Value::Tuple);
            }
            Self::Matrix(rows) => {
                let row_count = rows.len();
                let columns = rows.first().map_or(0, Vec::len);
                let mut entries = Vec::new();
                for row in rows {
                    for (entry, column) in row {
                        entries.push(polynomial(entry.evaluate(ring)?, column)?);
                    }
                }
                return Matrix::new(row_count, columns, entries)
                    .map(Value::Matrix)
                    .map_err(|message| error(1, message));
            }
            Self::Number(value) => Polynomial::constant(value),
            Self::Variable(name) => ring.variable(&name),
            Self::Unary(negative, expression, column) => {
                let value = scalar(expression.evaluate(ring)?, column)?;
                return Ok(Value::from_rational_function(if negative {
                    value.negate()
                } else {
                    value
                }));
            }
            Self::Binary(op, left, right, column) => {
                let left = scalar(left.evaluate(ring)?, column)?;
                let right = scalar(right.evaluate(ring)?, column)?;
                let result = match op {
                    Kind::Plus => left.add(&right, ring),
                    Kind::Minus => left.subtract(&right, ring),
                    Kind::Star => left.multiply(&right, ring),
                    Kind::Slash => left.divide(&right, ring),
                    _ => unreachable!("only arithmetic operators produce binary nodes"),
                }
                .map_err(|message| error(column, message))?;
                return Ok(Value::from_rational_function(result));
            }
            Self::Power(base, exponent, negative, column) => {
                let base = scalar(base.evaluate(ring)?, column)?;
                return base
                    .power(exponent, negative, ring)
                    .map(Value::from_rational_function)
                    .map_err(|message| error(column, message));
            }
        };
        Ok(Value::Polynomial(value))
    }
}

struct Parser {
    tokens: Vec<Token>,
    next: usize,
}

impl Parser {
    fn current(&self) -> &Token {
        &self.tokens[self.next]
    }

    fn fail(&self, message: &str) -> Error {
        error(self.current().column, message)
    }

    fn integer(&mut self) -> Result<BigInt, Error> {
        let Kind::Integer(digits) = &self.current().kind else {
            return Err(self.fail("expected an unsigned integer"));
        };
        let value = digits.parse().expect("lexer only emits decimal digits");
        self.next += 1;
        Ok(value)
    }

    fn matrix(&mut self, depth: usize) -> Result<Expr, Error> {
        self.next += 1;
        if self.current().kind == Kind::CloseBracket {
            self.next += 1;
            return Ok(Expr::Matrix(vec![]));
        }
        let mut rows: Vec<Vec<(Expr, usize)>> = Vec::new();
        loop {
            let row_column = self.current().column;
            let mut row = Vec::new();
            loop {
                let column = self.current().column;
                row.push((self.expression(0, depth + 1)?, column));
                if self.current().kind != Kind::Comma {
                    break;
                }
                self.next += 1;
            }
            if rows.first().is_some_and(|first| first.len() != row.len()) {
                return Err(error(row_column, "matrix rows must have equal lengths"));
            }
            rows.push(row);
            match self.current().kind {
                Kind::Semicolon => self.next += 1,
                Kind::CloseBracket => {
                    self.next += 1;
                    break;
                }
                _ => return Err(self.fail("expected ',', ';' or ']'")),
            }
        }
        Ok(Expr::Matrix(rows))
    }

    fn expression(&mut self, minimum: u8, depth: usize) -> Result<Expr, Error> {
        if depth >= 128 {
            return Err(self.fail("expression nesting limit exceeded"));
        }
        let mut left = match self.current().kind.clone() {
            Kind::Integer(_) => {
                let numerator = self.integer()?;
                Expr::Number(Rational::from_integer(numerator))
            }
            Kind::Identifier(name) => {
                let column = self.current().column;
                self.next += 1;
                if self.current().kind == Kind::Open {
                    self.next += 1;
                    let mut arguments = Vec::new();
                    if self.current().kind != Kind::Close {
                        loop {
                            let argument_column = self.current().column;
                            arguments.push((self.expression(0, depth + 1)?, argument_column));
                            if self.current().kind != Kind::Comma {
                                break;
                            }
                            self.next += 1;
                        }
                    }
                    if self.current().kind != Kind::Close {
                        return Err(self.fail("expected ')' or ',' in function call"));
                    }
                    self.next += 1;
                    Expr::Call(name, arguments, column)
                } else {
                    Expr::Variable(name)
                }
            }
            Kind::Plus | Kind::Minus => {
                let negative = self.current().kind == Kind::Minus;
                let column = self.current().column;
                self.next += 1;
                let value = self.expression(3, depth + 1)?;
                Expr::Unary(negative, Box::new(value), column)
            }
            Kind::Open => {
                self.next += 1;
                if self.current().kind == Kind::Close {
                    self.next += 1;
                    Expr::Tuple(vec![])
                } else {
                    let first = self.expression(0, depth + 1)?;
                    if self.current().kind == Kind::Close {
                        self.next += 1;
                        first
                    } else {
                        if self.current().kind != Kind::Comma {
                            return Err(self.fail("expected ')' or ','"));
                        }
                        let mut elements = vec![first];
                        while self.current().kind == Kind::Comma {
                            self.next += 1;
                            if self.current().kind == Kind::Close {
                                break;
                            }
                            elements.push(self.expression(0, depth + 1)?);
                        }
                        if self.current().kind != Kind::Close {
                            return Err(self.fail("expected ')' or ','"));
                        }
                        self.next += 1;
                        Expr::Tuple(elements)
                    }
                }
            }
            Kind::OpenBracket => self.matrix(depth)?,
            _ => return Err(self.fail("expected a number, variable, unary sign, '(' or '['")),
        };
        loop {
            let op = self.current().kind.clone();
            let binding = match op {
                Kind::Plus | Kind::Minus => 1,
                Kind::Star | Kind::Slash => 2,
                Kind::Caret => 4,
                _ => break,
            };
            if binding < minimum {
                break;
            }
            let operator_column = self.current().column;
            self.next += 1;
            if op == Kind::Caret {
                let negative = self.current().kind == Kind::Minus;
                if negative || self.current().kind == Kind::Plus {
                    self.next += 1;
                }
                let column = self.current().column;
                let exponent = self
                    .integer()?
                    .try_into()
                    .map_err(|_| error(column, "exponent must fit in u32"))?;
                left = Expr::Power(Box::new(left), exponent, negative, operator_column);
                if self.current().kind == Kind::Caret {
                    return Err(self
                        .fail("parenthesize chained powers; exponents must be integer literals"));
                }
            } else {
                let right = self.expression(binding + 1, depth + 1)?;
                left = Expr::Binary(op, Box::new(left), Box::new(right), operator_column);
            }
        }
        Ok(left)
    }
}

fn parse(input: &str) -> Result<Expr, Error> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        next: 0,
    };
    let expression = parser.expression(0, 0)?;
    if parser.current().kind != Kind::End {
        return Err(parser.fail("unexpected token; use explicit '*' for multiplication"));
    }
    Ok(expression)
}

/// Evaluate any value, committing new variables only after complete success.
pub fn evaluate_value(input: &str, ring: &mut PolynomialRing) -> Result<Value, Error> {
    let expression = parse(input)?;
    let mut candidate = ring.clone();
    let value = expression.evaluate(&mut candidate)?;
    *ring = candidate;
    Ok(value)
}

/// Evaluate only a polynomial, rejecting compound results without changing the ring.
pub fn evaluate(input: &str, ring: &mut PolynomialRing) -> Result<Polynomial, Error> {
    let expression = parse(input)?;
    let mut candidate = ring.clone();
    let column = input.chars().position(|c| !c.is_whitespace()).unwrap_or(0) + 1;
    let value = polynomial(expression.evaluate(&mut candidate)?, column)?;
    *ring = candidate;
    Ok(value)
}
