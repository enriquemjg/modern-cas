//! Algebraic core of modern-cas, independent of the interactive interface.

/// Exact rational coefficients backed by arbitrary-precision integers.
pub type Rational = num_rational::BigRational;

pub mod parser;
pub mod repl;

pub mod polynomial;
