# Roadmap

## Milestone 0 — Project foundation

- [x] Initialize the Git repository and Cargo project (Rust 2024).
- [x] Separate the algebraic library and executable.
- [x] Choose exact rational coefficients and document the session context.
- [x] Define milestones and acceptance criteria.

## Milestone 1 — Polynomials and a basic REPL

### Delivery order: REPL from the start

1. **Complete:** input loop and basic commands, tested through the actual executable.
2. **Complete:** the first rational expression end to end: input, parsing,
   exact evaluation, and output.
3. **Complete:** variables and polynomial operations, extending E2E tests with each delivery.
4. **Complete:** all three orders and context commands, tested through complete sessions.

- [x] Read from `stdin` interactively or through pipes, without requiring a TTY.
- [x] Show prompts only interactively; send results to `stdout` and errors to `stderr`.
- [x] E2E tests launch the binary, send multiple lines, and verify results,
  errors, recovery, EOF, and exit codes.
- [x] Track source positions in the parser to locate syntax errors.

### Core

- [x] Session context with incremental variable registration.
- [x] Monomials: `u32` exponents, normalization, degree, and multiplication.
- [x] Detect exponent overflow and compute degrees without truncation.
- [x] Comparators for `lex`, `grlex`, and `grevlex`, with tests distinguishing them.
- [x] Normalized polynomials and leading-term access (none for zero).
- [x] Addition, subtraction, negation, multiplication, and nonnegative integer powers.
- [x] Readable, deterministic formatting, omitting unit coefficients and exponents.

### Parser and interface

- [x] Small parser for integers, rational literals `p/q`, variable names,
  parentheses, `+`, `-`, `*`, and `^` with a nonnegative integer exponent.
- [x] Usual precedence: `-x^2` means `-(x^2)`.
- [x] Explicit multiplication initially (`2*x`, no implicit multiplication).
- [x] `/` only forms rational literals; expression division is not supported.
- [x] REPL with one expression per line and expanded, simplified output.
- [x] Commands: `:help`, `:vars`, `:reset`, `:order lex|grlex|grevlex`, `:quit`.
- [x] Changing the order resets the context; adding variables preserves it.
- [x] Readable errors, zero denominators rejected, and a usable session after errors.
- [x] Invalid input does not change the variable registry.
- [x] EOF exits cleanly; blank lines are ignored.

Assignments, persistent history, and advanced line editing are outside this milestone.
The REPL can start with standard I/O and no additional dependencies.

### Acceptance criteria

- `(x + y)*(x - y)` produces `x^2 - y^2`.
- `1/2*x + 1/3*x` produces `5/6*x`.
- `x - x` produces `0`.
- `(x + 1)^3` produces `x^3 + 3*x^2 + 3*x + 1`.
- Introducing `z` after `x` and `y` preserves their identifiers and precedence.
- Tests cover exact arithmetic, normalization, all three orders,
  parser precedence, and REPL error recovery.

## Milestone 2 — Multivariate division

### Compound values foundation

- [x] Ordered heterogeneous tuples, including empty, singleton, and nested tuples.
- [x] Rectangular polynomial matrix literals and row/column vectors.
- [x] Recursive formatting, type validation, and transactional variable registration.
- [x] Preserve the polynomial API and add general value evaluation for the REPL.
- [x] Parser and E2E coverage for literals, errors, recovery, and formatting round trips.

### Division algorithm and interface

- [x] Monomial divisibility and exact quotients.
- [x] Division by an ordered list of polynomials.
- [x] Return all quotients and the remainder; reject zero divisors.
- [x] Verify `f = Σ(qᵢ*fᵢ) + r` and that no remainder monomial is divisible
  by any divisor's leading monomial.
- [x] Examples showing dependence on divisor order.
- [x] Add function calls and `div(f, [g1, g2])`, returning `([q1, q2], r)`.

## Milestone 3 — Gröbner bases

- [x] Least common multiples of monomials and S-polynomials.
- [x] A direct implementation of Buchberger's algorithm.
- [x] Monic, minimal, and reduced bases.
- [x] Verify that all pairwise S-polynomials reduce to zero.
- [x] Compare results under all three orders using small examples.
- [x] Expose reduced bases as `groebner([f1, f2])`, composable with `div`.

Evaluate Buchberger optimizations after establishing a correct implementation
that is easy to follow.

## Later

- Assignments and REPL improvements.
- Matrix operations and exact linear algebra (literal representation is available).
- Polynomial differentiation and evaluation.
- Symbolic expressions and functions with a dedicated representation.
- Plotting, separating exact computation from numerical evaluation for visualization.

These extensions should not burden the initial architecture with abstractions
we do not yet need.
