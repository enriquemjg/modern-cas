# modern-cas

A small Computer Algebra System in Rust, focused on algorithms from
*Ideals, Varieties, and Algorithms*. We prioritize simple code, exact arithmetic,
and recognizable algorithms over premature optimization.

## Status

A working REPL for multivariate polynomials over exact rationals, with a lexer and Pratt parser
requiring no additional dependencies. Coefficients use
`num-rational::BigRational` and `num-bigint::BigInt`. Expressions are expanded
and normalized automatically; polynomial division is the next milestone.

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

See [ROADMAP.md](ROADMAP.md) for completed acceptance criteria and the next
milestones: multivariate division and Gröbner bases.

Dependency references: [num-bigint](https://docs.rs/num-bigint/),
[num-rational](https://docs.rs/num-rational/),
[num-traits](https://docs.rs/num-traits/).
