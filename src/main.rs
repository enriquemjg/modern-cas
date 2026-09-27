use std::io::{self, IsTerminal};
use std::process::ExitCode;

fn main() -> ExitCode {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let interactive = stdin.is_terminal() && stdout.is_terminal();
    match modern_cas::repl::run(
        stdin.lock(),
        stdout.lock(),
        io::stderr().lock(),
        interactive,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("I/O error: {error}");
            ExitCode::FAILURE
        }
    }
}
