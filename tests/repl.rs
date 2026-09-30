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
        session("1/0\n(1+2\n2*(3+)\n2/3/4\n2^-1\n2^4294967296\n2^3^2\n@\n2 3\n:nope\n1+1\n");
    assert_eq!(out, "2\n");
    assert_eq!(err.matches("error:").count(), 10);
    assert!(err.contains("column 3: denominator cannot be zero\n1/0\n  ^"));
    assert!(err.contains("expected ')'"));
    assert!(err.contains("exponent must fit in u32"));
}

#[test]
fn commands_blank_lines_crlf_and_quit() {
    let (out, err) = session("\r\n:help\r\n:reset\n3\n:quit\n999\n");
    assert!(out.starts_with("Exact polynomial expressions:"));
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

#[test]
fn polynomial_expansion_and_normalization() {
    let (out, err) = session(
        "(x+y)*(x-y)\n1/2*x+1/3*x\nx-x\n(x+1)^3\n-x^2\n(-x)^2\n0*x\n-x+y-1\n(x+y)^0\n2*alpha_1-alpha_1\n",
    );
    assert_eq!(
        out,
        "x^2 - y^2\n5/6*x\n0\nx^3 + 3*x^2 + 3*x + 1\n-x^2\nx^2\n0\n-x + y - 1\n1\nalpha_1\n"
    );
    assert_eq!(err, "");
}

#[test]
fn all_orders_and_session_lifecycle() {
    let (out, err) = session(
        ":vars\nx+y+z\nx+y^2\nx^2*z+x*y^2\n:order grlex\n:vars\nx+y+z\nx+y^2\nx^2*z+x*y^2\n:order grevlex\nx+y+z\nx+y^2\nx^2*z+x*y^2\n:order grevlex\n:vars\n:reset\n:vars\ny+x\n:vars\n",
    );
    assert_eq!(
        out,
        "No variables.\nx + y + z\nx + y^2\nx^2*z + x*y^2\nOrder: grlex. Session reset.\nNo variables.\nx + y + z\ny^2 + x\nx^2*z + x*y^2\nOrder: grevlex. Session reset.\nx + y + z\ny^2 + x\nx*y^2 + x^2*z\nOrder: grevlex.\nx > y > z\nSession reset.\nNo variables.\ny + x\ny > x\n"
    );
    assert_eq!(err, "");
}

#[test]
fn failed_expressions_and_commands_preserve_variables() {
    let (out, err) = session(
        "x\nbad+\nnew^4294967295*new\n:order invalid\n:vars\ny\n:vars\n(x+y)/(x-y)\n2x\nx^4294967295\nx-x\n",
    );
    assert_eq!(out, "x\nx\ny\nx > y\nx^4294967295\n0\n");
    assert_eq!(err.matches("error:").count(), 5);
    assert!(err.contains("monomial exponent overflow"));
}

#[test]
fn compound_literals_through_the_binary() {
    let (out, err) = session(
        "()\n(x)\n(x,)\n(x,y,z,)\n[]\n[x]\n[x,y]\n[x;y]\n[x+x,(x+1)^2;0,1]\n([1,0],1)\n(x,(y,z))\n:vars\n",
    );
    assert_eq!(
        out,
        "()\nx\n(x,)\n(x, y, z)\n[]\n[x]\n[x, y]\n[x; y]\n[2*x, x^2 + 2*x + 1; 0, 1]\n([1, 0], 1)\n(x, (y, z))\nx > y > z\n"
    );
    assert_eq!(err, "");
}

#[test]
fn compound_errors_recover_without_registering_variables() {
    let (out, err) = session(
        "x\n[new,1;2]\n[new,(y,)]\n+[new]\n(new,)+1\n[new,]\n[;]\n{new}\n:vars\n([x],x+x)\n",
    );
    assert_eq!(out, "x\nx\n([x], 2*x)\n");
    assert_eq!(err.matches("error:").count(), 7);
    assert!(err.contains("matrix rows must have equal lengths"));
    assert!(err.contains("expected a polynomial"));
}

#[test]
fn division_returns_quotients_and_remainder() {
    let (out, err) = session(
        "div(x*y,[x*y-1,y-1])\ndiv(x*y,[y-1,x*y-1])\ndiv(x^2-1,[x-1])\ndiv(x*y,[y-1;x*y-1])\ndiv(x+1,[])\ndiv(0,[x,1])\ndiv(x+1,[2])\n(div(x,[x]),div(y,[]))\n:vars\n",
    );
    assert_eq!(
        out,
        "([1, 0], 1)\n([x, 0], x)\n([x + 1], 0)\n([x; 0], x)\n([], x + 1)\n([0, 0], 0)\n([1/2*x + 1/2], 0)\n(([1], 0), ([], y))\nx > y\n"
    );
    assert_eq!(err, "");
}

#[test]
fn division_uses_active_order_and_recovers_from_errors() {
    let (out, err) = session(
        "div(x^2,[x-y^2])\n:order grlex\ndiv(x^2,[x-y^2])\n:order grevlex\ndiv(x^2,[x-y^2])\ndiv(new,[0])\ndiv(new,[x,x;x,x])\ndiv()\nfoo(new)\n:vars\ndiv(x,[x])\n",
    );
    assert_eq!(
        out,
        "([x + y^2], y^4)\nOrder: grlex. Session reset.\n([0], x^2)\nOrder: grevlex. Session reset.\n([0], x^2)\nx > y\n([1], 0)\n"
    );
    assert_eq!(err.matches("error:").count(), 4);
    assert!(err.contains("divisor 1 is the zero polynomial"));
}

#[test]
fn groebner_bases_and_composed_division_through_the_binary() {
    let (out, err) = session(
        "groebner([x*y-1,y^2-x])\ndiv(x*y-1,groebner([x*y-1,y^2-x]))\n:order grlex\ngroebner([x*y-1;y^2-x])\n:order grevlex\ngroebner([x*y-1,y^2-x])\ngroebner([])\ngroebner([0,0])\ngroebner([x,x+1])\n",
    );
    assert_eq!(
        out,
        "[x - y^2, y^3 - 1]\n([y, 1], 0)\nOrder: grlex. Session reset.\n[x^2 - y, x*y - 1, y^2 - x]\nOrder: grevlex. Session reset.\n[x^2 - y, x*y - 1, y^2 - x]\n[]\n[]\n[1]\n"
    );
    assert_eq!(err, "");
}

#[test]
fn groebner_errors_recover_without_registering_variables() {
    let (out, err) = session(
        "x\ngroebner(new)\ngroebner([new,1;2,3])\ngroebner([new],[])\ngroebner([new])+1\n:vars\ngroebner([x])\n",
    );
    assert_eq!(out, "x\nx\n[x]\n");
    assert_eq!(err.matches("error:").count(), 4);
}

#[test]
fn univariate_gcd_is_exact_and_composable() {
    let (out, err) = session(
        "gcd(x^3-x,x^2-1)\ngcd(0,0)\ngcd(0,-2*x+4)\ngcd(2/3,4/5)\ngcd(x^2+1,x+1)\ndiv(x^3-x,[gcd(x^3-x,x^2-1)])\ngcd(x,y)\ngcd([new],1)\n:vars\ngcd(y-y+x,x^2)\n:order grevlex\ngcd(z^2-1,z-1)\n",
    );
    assert_eq!(
        out,
        "x^2 - 1\n0\nx - 2\n1\n1\n([x], 0)\nx\nx\nOrder: grevlex. Session reset.\nz - 1\n"
    );
    assert_eq!(err.matches("error:").count(), 2);
    assert!(err.contains("at most one variable"));
}
