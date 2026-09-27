//! Line-oriented interface shared by terminal and piped sessions.

use std::io::{self, BufRead, Write};

use crate::parser;
use crate::polynomial::{MonomialOrder, PolynomialRing};

const HELP: &str = "Exact polynomial expressions: +, -, *, parentheses, and ^ with a nonnegative u32 integer literal.\nFractions: p/q (unsigned integers; use a leading sign). No polynomial division.\nParenthesize chained powers. 0^0 = 1. Variables: ASCII letters or underscore, followed by letters, digits, or underscores. Use explicit multiplication.\nCommands: :help, :vars, :reset, :order lex|grlex|grevlex, :quit\n";

pub fn run(
    mut input: impl BufRead,
    mut output: impl Write,
    mut diagnostics: impl Write,
    interactive: bool,
) -> io::Result<()> {
    let mut line = String::new();
    let mut ring = PolynomialRing::default();
    loop {
        if interactive {
            write!(output, "cas> ")?;
            output.flush()?;
        }
        line.clear();
        if input.read_line(&mut line)? == 0 {
            return Ok(());
        }
        let text = line.trim();
        match text {
            "" => continue,
            ":quit" => return Ok(()),
            ":help" => write!(output, "{HELP}")?,
            ":reset" => {
                ring = PolynomialRing::new(ring.order());
                writeln!(output, "Session reset.")?;
            }
            ":vars" => writeln!(
                output,
                "{}",
                if ring.variables().is_empty() {
                    "No variables.".to_owned()
                } else {
                    ring.variables().join(" > ")
                }
            )?,
            text if text.split_whitespace().next() == Some(":order") => {
                let parts: Vec<_> = text.split_whitespace().collect();
                let order = match parts.as_slice() {
                    [":order", "lex"] => Some(MonomialOrder::Lex),
                    [":order", "grlex"] => Some(MonomialOrder::GrLex),
                    [":order", "grevlex"] => Some(MonomialOrder::GrevLex),
                    _ => None,
                };
                if let Some(order) = order {
                    if ring.order() != order {
                        ring = PolynomialRing::new(order);
                        writeln!(output, "Order: {}. Session reset.", parts[1])?;
                    } else {
                        writeln!(output, "Order: {}.", parts[1])?;
                    }
                } else {
                    writeln!(diagnostics, "error: usage: :order lex|grlex|grevlex")?;
                }
            }
            text if text.starts_with(':') => {
                writeln!(diagnostics, "error: unknown command '{text}'; use :help")?
            }
            _ => match parser::evaluate(line.trim_end(), &mut ring) {
                Ok(value) => writeln!(output, "{}", ring.format(&value))?,
                Err(err) => {
                    writeln!(diagnostics, "error: {err}")?;
                    writeln!(diagnostics, "{}", line.trim_end())?;
                    writeln!(diagnostics, "{}^", " ".repeat(err.column - 1))?;
                }
            },
        }
        output.flush()?;
        diagnostics.flush()?;
    }
}
