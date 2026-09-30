//! Algebraic core of modern-cas, independent of the interactive interface.

/// Exact rational coefficients backed by arbitrary-precision integers.
pub type Rational = num_rational::BigRational;

pub mod parser;
pub mod repl;

pub mod gcd;
pub mod groebner;
pub mod polynomial;
pub mod rational_function;
pub mod value;
