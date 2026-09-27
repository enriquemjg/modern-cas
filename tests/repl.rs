use std::io::Write;
use std::process::{Command, Stdio};

fn session(input: &str) -> (String, String) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_modern-cas"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    (
        String::from_utf8(result.stdout).unwrap(),
        String::from_utf8(result.stderr).unwrap(),
    )
}

#[test]
fn exact_arithmetic_and_precedence_through_the_binary() {
    let (out, err) = session(
        "1/2 + 1/3\n2+3*4\n(2+3)*4\n-2^2\n(-2)^2\n8-3-2\n(2/3)^3\n0^0\n999999999999999999999999999999+1\n",
    );
    assert_eq!(
        out,
        "5/6\n14\n20\n-4\n4\n3\n8/27\n1\n1000000000000000000000000000000\n"
    );
    assert_eq!(err, "");
}

#[test]
fn errors_do_not_end_the_session() {
    let (out, err) =
        session("1/0\n(1+2\n2*(3+)\n2/3/4\n2^-1\n2^4294967296\n2^3^2\nx\n2 3\n:nope\n1+1\n");
    assert_eq!(out, "2\n");
    assert_eq!(err.matches("error:").count(), 10);
    assert!(err.contains("column 3: denominator cannot be zero\n1/0\n  ^"));
    assert!(err.contains("expected ')'"));
    assert!(err.contains("exponent must fit in u32"));
}

#[test]
fn commands_blank_lines_crlf_and_quit() {
    let (out, err) = session("\r\n:help\r\n:reset\n3\n:quit\n999\n");
    assert!(out.starts_with("Exact rational expressions:"));
    assert!(out.ends_with("Session reset.\n3\n"));
    assert!(!out.contains("cas>"));
    assert_eq!(err, "");
}

#[test]
fn eof_accepts_a_final_line_without_newline() {
    assert_eq!(session("2*3"), ("6\n".into(), "".into()));
    assert_eq!(session(""), ("".into(), "".into()));
}

#[test]
fn excessive_nesting_is_reported_and_recovery_works() {
    let input = format!("{}1{}\n7\n", "(".repeat(140), ")".repeat(140));
    let (out, err) = session(&input);
    assert_eq!(out, "7\n");
    assert!(err.contains("nesting limit"));
}

#[test]
fn long_operator_chains_are_rejected_before_building_a_deep_tree() {
    let (out, err) = session(&format!("{}1\n7\n", "1+".repeat(600)));
    assert_eq!(out, "7\n");
    assert!(err.contains("token limit"));
}

#[test]
fn interactive_mode_shows_and_flushes_prompts() {
    let mut out = Vec::new();
    let mut err = Vec::new();
    modern_cas::repl::run(&b"1+1\n:quit\n"[..], &mut out, &mut err, true).unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "cas> 2\ncas> ");
    assert!(err.is_empty());
}
