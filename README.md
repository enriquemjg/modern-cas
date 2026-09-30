# modern-cas

A small Computer Algebra System in Rust, focused on algorithms from
_Ideals, Varieties, and Algorithms_ by D. Cox, J. Little and D. O'Shea.
We prioritize simple code, exact arithmetic, and recognizable algorithms
over premature optimization.

## Status

A working REPL for multivariate polynomials over exact rationals, with a lexer and Pratt parser
requiring no additional dependencies. Coefficients use
`num-rational::BigRational` and `num-bigint::BigInt`. Expressions are expanded
and normalized automatically. Ordered multivariate division is available through `div`.

```sh
cargo run
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

The REPL accepts `+`, `-`, `*`, unary signs, parentheses, and powers with
nonnegative integer literal exponents that fit in `u32`. `/` is only allowed
in rational literals such as `1/2`; it does not divide expressions. `-2^2`
evaluates to `-4`. Chained powers require parentheses: `(2^3)^2`.
We define `0^0 = 1`. Inputs are currently limited to 1024 tokens and a parser
depth of 128.

Available commands: `:help`, `:vars`, `:reset`, `:order lex|grlex|grevlex`,
and `:quit`. `:vars` shows variable precedence. `:reset` clears the variable
registry and preserves the current monomial order. Changing to a different order
clears the registry; selecting the current order preserves it. Invalid commands
and failed expressions leave the context unchanged.

Variable names match `[A-Za-z_][A-Za-z0-9_]*`. Multiplication must be explicit:
`2*x`, not `2x`. Variables are registered on first appearance in successful
expressions, including expressions that simplify to zero. Exponents use `u32`;
monomial multiplication reports overflow rather than wrapping.

```text
cas> (x+y)*(x-y)
x^2 - y^2
cas> 1/2*x + 1/3*x
5/6*x
cas> (x+1)^3
x^3 + 3*x^2 + 3*x + 1
cas> :vars
x > y
```

```sh
printf '1/2 + 1/3\n(2 + 3)*4\n' | cargo run --quiet
# Output: 5/6, then 20 (one result per line).
```

Piped sessions show no prompts. Results go to `stdout`; errors go to `stderr`
with a column number and position marker. Invalid input does not end the session:
EOF and `:quit` exit successfully even after expression errors; an I/O failure
produces a nonzero exit code. E2E tests launch the actual executable and verify
complete sessions.

## Compound values

Every expression returns one value: a polynomial, tuple, or polynomial matrix.

| Syntax         | Meaning                                    |
| -------------- | ------------------------------------------ |
| `(x)`          | Grouping, equivalent to `x`                |
| `()`           | Empty tuple                                |
| `(x,)`         | Singleton tuple                            |
| `(x, y, z)`    | Ordered, possibly heterogeneous tuple      |
| `[x, y]`       | Row vector (1 by 2 matrix)                 |
| `[x; y]`       | Column vector (2 by 1 matrix)              |
| `[a, b; c, d]` | Rectangular matrix                         |
| `[x]`          | 1 by 1 matrix, distinct from scalar `x`    |
| `[]`           | Empty 0 by 0 matrix                        |
| `([1, 0], 1)`  | Tuple containing a vector and a polynomial |

Tuples preserve order and repetition, allow nesting, and accept a trailing comma.
Matrix entries must evaluate to polynomials. Commas separate columns and
semicolons separate rows; rows must have equal lengths. Spaces are not separators,
and empty rows, trailing separators, and block concatenation are not supported.
Entries simplify automatically: `[x+x, (x+1)^2]` becomes `[2*x, x^2 + 2*x + 1]`.

Arithmetic, including unary signs, currently accepts polynomials only. Collection
arithmetic, indexing, assignments, and destructuring are future features.
Braces are reserved for sets. Named function calls support `div` and `groebner`.

## Polynomial division

`div(f, [g1, g2])` returns `([q1, q2], r)` satisfying
`f = q1*g1 + q2*g2 + r`. The algorithm uses the session's monomial order and
the first applicable divisor at each step. No monomial of the remainder is
divisible by a leading monomial of any divisor. Divisors need not form a Gröbner
basis, and changing their order can change the remainder:

```text
cas> div(x*y, [x*y-1, y-1])
([1, 0], 1)
cas> div(x*y, [y-1, x*y-1])
([x, 0], x)
```

The divisor argument must be a row or column vector of polynomials, or `[]`.
The quotient vector preserves its shape and element order. `div(f, [])` returns
`([], f)`. Zero divisors are rejected, including expressions that simplify to zero,
even when the dividend is zero. Arithmetic overflow also produces an error.
All failures leave the session unchanged.

Calls take comma-separated arguments without a trailing comma. Their results can
appear inside tuples, but polynomial operators do not accept the tuple returned by
`div`. A bare name such as `div` remains a polynomial variable; only `div(...)`
is a call. `/` remains restricted to rational literals.

The core API is `PolynomialRing::divide(&dividend, &divisors)`, returning a
`DivisionResult` with `quotients` and `remainder`, independently of REPL values.

Elements are evaluated left to right, with matrices traversed by rows. Failed
entries or type checks leave the entire session context unchanged. Parser limits
also apply to collections. Library callers can use `parser::evaluate_value` for
all values or `parser::evaluate` to require a polynomial transactionally.

## Gröbner bases

`groebner([f1, f2])` computes the reduced Gröbner basis for the active monomial
order using direct Buchberger completion, removal of redundant leading monomials,
and interreduction. The result is always a row vector, sorted by descending leading
monomial, even when the input is a column vector.

```text
cas> groebner([x*y-1, y^2-x])
[x - y^2, y^3 - 1]
cas> div(x*y-1, groebner([x*y-1, y^2-x]))
([y, 1], 0)
cas> :order grevlex
Order: grevlex. Session reset.
cas> groebner([x*y-1, y^2-x])
[x^2 - y, x*y - 1, y^2 - x]
```

Zero and duplicate generators are accepted. The zero ideal is represented by `[]`
and the unit ideal by `[1]`. All output polynomials are monic, and no monomial of
one output polynomial is divisible by another's leading monomial. For a fixed
variable precedence and monomial order, the result is independent of generator
order. Variable registration still follows first appearance in the session.

The core exposes `PolynomialRing::s_polynomial`, `PolynomialRing::buchberger`
(a monic basis that need not be reduced), and `PolynomialRing::groebner_basis`
(the reduced basis). S-polynomials require nonzero operands. Exponent overflow
is reported; failed REPL evaluations do not change the context. The implementation
favors readability and processes all critical pairs, so large problems may be slow
or require substantial memory.

## Core design decisions

- Coefficients in ℚ backed by arbitrary-precision integers.
- An explicit session context owns variables, variable precedence, and the monomial
  order. Polynomials do not store their own order; there is no mutable global state.
- Support lexicographic (`lex`), graded lexicographic (`grlex`), and graded reverse
  lexicographic (`grevlex`) orders. The initial order is `lex`.
- Register variables in order of appearance. Append each new variable at the end
  of the precedence without invalidating previous expressions.
- Missing exponents are zero; monomials omit trailing zeros. The constant monomial
  uses an empty vector.
- Changing the monomial order or existing variable precedence starts an empty session.
- Represent polynomials as sorted terms, with no duplicate monomials or zero
  coefficients. The zero polynomial uses an empty collection.
- Keep the algebraic core independent of the parser and I/O.

See [ROADMAP.md](ROADMAP.md) for completed milestones and future extensions.

The next phase starts with univariate and then multivariate polynomial GCD,
followed by normalized rational functions and gradual symbolic expression support.
Irrational and transcendental values will remain exact symbolic expressions rather
than floating-point approximations. Gaussian rational coefficients and exact linear
algebra are planned extensions; these capabilities are not implemented yet.

Dependency references: [num-bigint](https://docs.rs/num-bigint/),
[num-rational](https://docs.rs/num-rational/),
[num-traits](https://docs.rs/num-traits/).
