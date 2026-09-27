use modern_cas::parser::{evaluate, evaluate_value};
use modern_cas::polynomial::{MonomialOrder, Polynomial, PolynomialRing};
use modern_cas::value::{Matrix, Value};

#[test]
fn formatting_preserves_compound_structure_and_polynomial_values() {
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        let mut ring = PolynomialRing::new(order);
        for (input, expected) in [
            ("(x)", "x"),
            ("()", "()"),
            ("(x,)", "(x,)"),
            ("(x,y,z,)", "(x, y, z)"),
            ("[x]", "[x]"),
            ("[]", "[]"),
            ("[x,y]", "[x, y]"),
            ("[x;y]", "[x; y]"),
            ("[x+x,(x+1)^2;0,1/2+1/3]", "[2*x, x^2 + 2*x + 1; 0, 5/6]"),
            ("([x,y], (x, (), (y,)), x)", "([x, y], (x, (), (y,)), x)"),
            ("((x+1))*2", "2*x + 2"),
        ] {
            let value = evaluate_value(input, &mut ring).unwrap();
            let formatted = value.format(&ring);
            assert_eq!(formatted, expected);
            assert_eq!(evaluate_value(&formatted, &mut ring).unwrap(), value);
        }
    }
}

#[test]
fn invalid_values_are_transactional() {
    let mut ring = PolynomialRing::default();
    evaluate("existing", &mut ring).unwrap();
    for input in [
        "[new,1;2]",
        "[new,]",
        "[new;]",
        "[;]",
        "[new 1]",
        "[new,,1]",
        "(new,,1)",
        "(new;1)",
        "[new,(x,)]",
        "[[new]]",
        "[new]+1",
        "-(new,)",
        "+[new]",
        "[new]^0",
        "(new,)*2",
        "{new}",
        "(new,1/0)",
        "(new,x^4294967295*x)",
        "[new)",
        "(new]",
        "new,1",
    ] {
        assert!(evaluate_value(input, &mut ring).is_err(), "{input}");
        assert_eq!(ring.variables(), &["existing"]);
    }
    for input in ["(new,)", "[new]"] {
        assert!(evaluate(input, &mut ring).is_err());
        assert_eq!(ring.variables(), &["existing"]);
    }
    let err = evaluate_value("x + [new]", &mut ring).unwrap_err();
    assert_eq!(err.column, 3);
    assert!(err.message.contains("matrix"));
    assert_eq!(
        evaluate_value("[new, (x,)]", &mut ring).unwrap_err().column,
        7
    );
}

#[test]
fn dimensions_and_registration_follow_row_major_order() {
    let mut ring = PolynomialRing::default();
    let Value::Matrix(matrix) = evaluate_value("[a,b;c,d]", &mut ring).unwrap() else {
        panic!()
    };
    assert_eq!(
        (matrix.rows(), matrix.columns(), matrix.entries().len()),
        (2, 2, 4)
    );
    evaluate_value("(e, [f;g], (h,))", &mut ring).unwrap();
    assert_eq!(ring.variables(), &["a", "b", "c", "d", "e", "f", "g", "h"]);
    assert!(Matrix::new(0, 0, vec![]).is_ok());
    assert!(Matrix::new(0, 1, vec![]).is_err());
    assert!(Matrix::new(1, 0, vec![]).is_err());
    assert!(Matrix::new(2, 2, vec![Polynomial::default()]).is_err());
    assert!(Matrix::new(usize::MAX, 2, vec![]).is_err());
}

#[test]
fn collections_obey_parser_limits() {
    let mut ring = PolynomialRing::default();
    for input in [
        format!("{}x{}", "[".repeat(140), "]".repeat(140)),
        format!("{}x{}", "(".repeat(140), ",)".repeat(140)),
    ] {
        assert!(
            evaluate_value(&input, &mut ring)
                .unwrap_err()
                .message
                .contains("nesting limit")
        );
    }
    let input = format!("({}1)", "1,".repeat(600));
    assert!(
        evaluate_value(&input, &mut ring)
            .unwrap_err()
            .message
            .contains("token limit")
    );
    assert!(ring.variables().is_empty());
}
