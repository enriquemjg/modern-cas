# Project context

modern-cas is a small Rust computer algebra system focused on algorithms from
*Ideals, Varieties, and Algorithms*. Favor simple, readable implementations
over premature optimization or generic algebra frameworks.

## Scope and design

- Use exact rational coefficients (`num-rational::BigRational`, backed by
  `num-bigint::BigInt`).
- First milestone: multivariate polynomials and a basic REPL supporting
  simplification, addition, subtraction, multiplication, and nonnegative integer
  powers. Polynomial division and Gröbner bases are later milestones.
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

## Conventions and validation

- Write all repository content in English, including documentation, code comments,
  Rust documentation comments, user-facing messages, and package metadata.
- See `README.md` for project decisions and `ROADMAP.md` for milestone details.
- Run `cargo fmt --check`, `cargo test`, and
  `cargo clippy --all-targets -- -D warnings` for implementation changes.
- Test algebraic invariants and meaningful examples when adding algorithms.
