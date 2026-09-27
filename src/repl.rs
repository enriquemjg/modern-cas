//! Line-oriented interface shared by terminal and piped sessions.

use std::io::{self, BufRead, Write};

use crate::parser;

const HELP: &str = "Exact rational expressions: +, -, *, parentheses, and ^ with a nonnegative u32 integer literal.\nFractions: p/q (unsigned integers; use a leading sign). No polynomial division.\nParenthesize chained powers. 0^0 = 1. Variables are not supported yet.\nCommands: :help, :reset, :quit\n";

pub fn run(
    mut input: impl BufRead,
    mut output: impl Write,
    mut diagnostics: impl Write,
    interactive: bool,
) -> io::Result<()> {
    let mut line = String::new();
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
            ":reset" => writeln!(output, "Session reset.")?,
            text if text.starts_with(':') => {
                writeln!(diagnostics, "error: unknown command '{text}'; use :help")?
            }
            _ => match parser::evaluate(line.trim_end()) {
                Ok(value) => writeln!(output, "{value}")?,
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
