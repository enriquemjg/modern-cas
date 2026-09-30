# Project context

modern-cas is a small Rust computer algebra system focused on algorithms from
*Ideals, Varieties, and Algorithms*. Favor simple, readable implementations
over premature optimization or generic algebra frameworks.

## Scope and design

- Use exact rational coefficients (`num-rational::BigRational`, backed by
  `num-bigint::BigInt`).
- First milestone: multivariate polynomials and a basic REPL supporting
  simplification, addition, subtraction, multiplication, and nonnegative integer
  powers. Ordered multivariate division and reduced Gröbner bases are implemented.
- Keep the algebraic library independent of parsing and terminal I/O.
- Deliver the REPL from the start and extend executable-level E2E tests alongside
  each feature. Support piped stdin without a TTY; show prompts only interactively,
  write results to stdout, and diagnostics to stderr.
- One explicit session context owns variable precedence and the monomial order:
  `lex`, `grlex`, or `grevlex`. Do not store the order in each polynomial or use
  mutable global state.
- Append new variables at the end of the precedence without clearing the session.
  Missing exponents are zero; omit trailing zeros from monomial representations.
- Changing the monomial order or existing variable precedence resets the session.
- Normalize polynomials: sorted terms, no duplicate monomials, no zero coefficients.
- Interpreter values are polynomials, heterogeneous ordered tuples `(a, b)`, and
  rectangular polynomial matrices `[a, b; c, d]`. Vectors are one-row or one-column
  matrices. `(x)` groups; `(x,)` is a singleton tuple; braces are reserved for sets.
- Collection literals do not imply collection arithmetic. Commit new variables only
  after the entire expression and its required result type have been validated.
- Scalar arithmetic includes normalized rational functions: cancel polynomial GCDs,
  make denominators monic, and demote denominator-one results to polynomials.
  `/` is ordinary left-associative division at multiplication precedence; powers
  accept signed integer literals. Equality is fraction-field equality, without
  retaining excluded input points. Matrices still require polynomial entries.
- `div(f, [g1, g2])` returns `([q1, q2], r)`. Accept row/column divisor vectors
  and preserve their shape; reject zero divisors. Empty divisors return `([], f)`.
- `groebner([f1, f2])` uses the active order and returns a reduced monic basis as
  a row vector sorted by descending leading monomial. Ignore zero generators;
  represent the zero ideal by `[]` and the unit ideal by `[1]`.

## Conventions and validation

- `gcd(f, g)` computes a monic multivariate GCD, with `gcd(0, 0) = 0`.
  Use Euclid for univariate inputs and recursive primitive pseudo-remainder
  sequences for multivariate inputs.

- Future expansion follows `ROADMAP.md`: reusable REPL values, symbolic
  expressions, and Q(i) coefficients.
  Keep irrational/transcendental values exact and symbolic; do not add implicit
  floating-point approximations. These are planned capabilities, not current ones.
- Preserve specialized polynomial and rational-function invariants separately from
  general symbolic expressions. Branch-sensitive rewrites require assumptions.

- Write all repository content in English, including documentation, code comments,
  Rust documentation comments, user-facing messages, and package metadata.
- See `README.md` for project decisions and `ROADMAP.md` for milestone details.
- Run `cargo fmt --check`, `cargo test`, and
  `cargo clippy --all-targets -- -D warnings` for implementation changes.
- Test algebraic invariants and meaningful examples when adding algorithms.
