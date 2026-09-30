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

## Direction: exact algebra and gradual symbolic expansion

Milestones 0–3 above describe completed work. The following milestones are planned,
in delivery order. Each adds a usable REPL operation and executable-level tests.
Proposed function names are interface sketches to finalize during implementation.

Keep arithmetic exact. Irrational and transcendental values may remain symbolic;
do not introduce floating-point evaluation or a general approximate real type.
Polynomial, rational-function, and symbolic-expression representations have
different invariants: preserve specialized algebraic types and introduce each
abstraction when an operation needs it. Tuples and matrices remain containers.

## Milestone 4 — Polynomial GCD

### 4a: Univariate foundation

- [x] Exact polynomial quotient helper that rejects a nonzero remainder.
- [x] Euclidean GCD over rational coefficients for polynomials involving at most
  one common variable; reject multivariate inputs until 4b is implemented.
- [x] Expose `gcd(f, g)` in the REPL and return a monic result.
- [x] Define `gcd(0, 0) = 0`, `gcd(f, 0) = monic(f)`, and nonzero constant GCDs as 1.
- [x] Verify common-factor examples, coprime inputs, rational coefficients,
  symmetry, and divisibility of both operands.

### 4b: Multivariate extension

- [x] Recursive coefficient decomposition in a chosen variable, coefficient
  content, and primitive parts.
- [x] Primitive polynomial remainder sequences with pseudo-division over the
  remaining polynomial coefficient ring; recursively compute coefficient GCDs.
- [x] Extend `gcd` to multivariate polynomials, with monic normalization in the
  active order and no dependency on a rational-function type.
- [x] Verify shared factors such as `gcd((x+y)*(x+1), (x+y)*(y+1)) = x+y`,
  content factors, zero inputs, and equivalent results up to units across orders.

Multivariate division with remainder alone is not a multivariate Euclidean GCD
algorithm. Full polynomial factorization is not required for this milestone.

## Milestone 5 — Rational functions

Depends on multivariate GCD and exact polynomial quotient.

- [ ] Represent a rational function as polynomial numerator and nonzero denominator.
- [ ] Cancel their polynomial GCD, make the denominator monic, normalize zero,
  and return a polynomial when the normalized denominator is 1.
- [ ] Implement exact addition, subtraction, multiplication, division, equality,
  and integer powers, including negative powers of nonzero values.
- [ ] Extend `/` to general expression division. Preserve exact rational numeric
  input and existing arithmetic precedence; document how this extends literals.
- [ ] Define equality in the fraction field: cancellation does not retain excluded
  points of the original expression. Pointwise domains require separate metadata.
- [ ] Verify `(x^2-1)/(x-1) = x+1`, `1/x + 1/y = (x+y)/(x*y)`, zero denominators,
  arithmetic identities, and format/parse round trips.

## Milestone 6 — Reusable REPL values

- [ ] Assignments for scalar values, matrices, and tuples, plus tuple destructuring.
- [ ] Define binding names versus polynomial indeterminates explicitly so assigning
  a name cannot silently change previously constructed algebraic objects.
- [ ] Indexing for tuples and matrices, with documented indexing and shape rules.
- [ ] Extend session reset to bindings and preserve atomic evaluation on failures.
- [ ] Exercise workflows that save generators, compute a basis, and reuse results.

## Milestone 7 — Symbolic expressions and exact functions

- [ ] Add a persistent symbolic expression tree, distinct from the parser's syntax
  tree, for sums, products, quotients, powers, and named function applications.
- [ ] Reuse polynomial and rational-function arithmetic whenever operands belong
  to those domains; retain unevaluated expressions otherwise.
- [ ] Support `sqrt`, rational powers, `sin`, `cos`, and generic symbolic calls
  incrementally. Define known-function arity checks and unknown-function behavior.
- [ ] Establish exact constants and naming rules before adding constants such as pi.
- [ ] Specify principal-root/branch conventions and a minimal assumptions model.
  Never simplify `sqrt(x^2)` to `x` without sufficient assumptions.
- [ ] Start with bounded, local simplification: `sqrt(4) = 2`, `sin(0) = 0`,
  `cos(0) = 1`, zero/one identities, and exact numeric radical extraction.
- [ ] Keep `sqrt(2)` symbolic. Distinguish structural equality from algebraic
  equality; do not promise a universal canonical form for arbitrary expressions.
- [ ] Add substitution and differentiation, starting with polynomials and rational
  functions, then supported symbolic operations and the chain rule.
- [ ] Test exact identities, unevaluated cases, assumptions, and expression growth.

## Milestone 8 — Gaussian rational coefficients

- [ ] Represent Q(i) coefficients as pairs of arbitrary-precision rationals.
- [ ] Implement exact field arithmetic, conjugation, and zero checks.
- [ ] Make the coefficient domain an explicit session choice; changing it resets
  the session. Define how the imaginary unit differs from a variable named `i`.
- [ ] Adapt polynomial GCD, division, rational functions, and Gröbner bases to Q(i)
  through the smallest shared coefficient interface needed by these two fields.
- [ ] Verify `(1+i)*(1-i) = 2`, `1/(1+i) = (1-i)/2`, and polynomial algorithms
  with nonreal coefficients. Q(i) does not contain every algebraic number.

## Milestone 9 — Exact matrix operations and linear algebra

Basic matrix arithmetic may be delivered earlier: polynomial entries already suffice.

- [ ] Matrix addition, subtraction, negation, scalar multiplication, matrix product,
  transpose, and identity matrices, with explicit dimension checks.
- [ ] Extend matrix scalar entries to rational functions when available. General
  symbolic entries require explicit assumptions for pivot zero/nonzero decisions.
- [ ] Gaussian elimination first over Q, then Q(i): reduced row echelon form, rank,
  and linear systems with unique, inconsistent, or parametrized solutions.
- [ ] Matrix inversion with singularity detection; determinants and characteristic
  polynomials using exact arithmetic.
- [ ] Treat elimination over rational functions as a fraction-field computation:
  generic rank and solutions may change after specializing variables.
- [ ] Verify reconstruction, inverse identities, rank, and solution families.

## Later — Exact algebraic roots and further algorithms

- Exact algebraic-number representation using defining polynomials and root
  identification, if needed beyond symbolic radicals.
- Exact eigenvalues expressed as algebraic roots; no assumption that radicals can
  express every root. Eigenvectors depend on arithmetic in the required extensions.
- Factorization, elimination applications, and measured Gröbner optimizations.
- REPL history and editing improvements.
- Plotting only as a separate future rendering concern, with any numerical
  sampling explicitly separated from the exact algebraic core.

Keep these extensions out of the current implementation until their milestone
requires them. Each delivery must preserve the existing exact polynomial workflows.
