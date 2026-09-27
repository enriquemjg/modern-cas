//! Tokenization, Pratt parsing, and exact evaluation of rational expressions.

use std::fmt;

use num_bigint::BigInt;
use num_traits::{One, Zero};

use crate::Rational;

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
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Open,
    Close,
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
            '+' => Kind::Plus,
            '-' => Kind::Minus,
            '*' => Kind::Star,
            '/' => Kind::Slash,
            '^' => Kind::Caret,
            '(' => Kind::Open,
            ')' => Kind::Close,
            _ => {
                return Err(error(
                    column,
                    format!("unexpected character '{ch}' (variables are not supported yet)"),
                ));
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
    Neg(Box<Expr>),
    Binary(Kind, Box<Expr>, Box<Expr>),
    Power(Box<Expr>, u32),
}

impl Expr {
    fn evaluate(self) -> Rational {
        match self {
            Self::Number(value) => value,
            Self::Neg(value) => -value.evaluate(),
            Self::Binary(op, left, right) => {
                let (left, right) = (left.evaluate(), right.evaluate());
                match op {
                    Kind::Plus => left + right,
                    Kind::Minus => left - right,
                    Kind::Star => left * right,
                    _ => unreachable!("only arithmetic operators produce binary nodes"),
                }
            }
            Self::Power(base, mut exponent) => {
                let mut base = base.evaluate();
                let mut result = Rational::one();
                while exponent > 0 {
                    if exponent % 2 == 1 {
                        result *= &base;
                    }
                    exponent /= 2;
                    if exponent > 0 {
                        base = &base * &base;
                    }
                }
                result
            }
        }
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

    fn expression(&mut self, minimum: u8, depth: usize) -> Result<Expr, Error> {
        if depth >= 128 {
            return Err(self.fail("expression nesting limit exceeded"));
        }
        let mut left = match self.current().kind.clone() {
            Kind::Integer(_) => {
                let numerator = self.integer()?;
                let denominator = if self.current().kind == Kind::Slash {
                    self.next += 1;
                    let column = self.current().column;
                    let denominator = self.integer()?;
                    if denominator.is_zero() {
                        return Err(error(column, "denominator cannot be zero"));
                    }
                    denominator
                } else {
                    BigInt::one()
                };
                Expr::Number(Rational::new(numerator, denominator))
            }
            Kind::Plus | Kind::Minus => {
                let negative = self.current().kind == Kind::Minus;
                self.next += 1;
                let value = self.expression(3, depth + 1)?;
                if negative {
                    Expr::Neg(Box::new(value))
                } else {
                    value
                }
            }
            Kind::Open => {
                self.next += 1;
                let value = self.expression(0, depth + 1)?;
                if self.current().kind != Kind::Close {
                    return Err(self.fail("expected ')'"));
                }
                self.next += 1;
                value
            }
            _ => return Err(self.fail("expected a number, unary sign, or '('")),
        };
        loop {
            let op = self.current().kind.clone();
            let binding = match op {
                Kind::Plus | Kind::Minus => 1,
                Kind::Star => 2,
                Kind::Caret => 4,
                _ => break,
            };
            if binding < minimum {
                break;
            }
            self.next += 1;
            if op == Kind::Caret {
                let column = self.current().column;
                let exponent = self
                    .integer()?
                    .try_into()
                    .map_err(|_| error(column, "exponent must fit in u32"))?;
                left = Expr::Power(Box::new(left), exponent);
                if self.current().kind == Kind::Caret {
                    return Err(self
                        .fail("parenthesize chained powers; exponents must be integer literals"));
                }
            } else {
                let right = self.expression(binding + 1, depth + 1)?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            }
        }
        Ok(left)
    }
}

/// Parse the entire input before evaluating it. Slash is only a fraction separator.
pub fn evaluate(input: &str) -> Result<Rational, Error> {
    let mut parser = Parser {
        tokens: tokenize(input)?,
        next: 0,
    };
    let expression = parser.expression(0, 0)?;
    if parser.current().kind != Kind::End {
        return Err(
            parser.fail("unexpected token; use explicit '*' and '/' only in rational literals")
        );
    }
    Ok(expression.evaluate())
}
