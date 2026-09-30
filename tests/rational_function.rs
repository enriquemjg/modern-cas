use modern_cas::parser::{evaluate, evaluate_value};
use modern_cas::polynomial::{MonomialOrder, PolynomialRing};
use modern_cas::rational_function::RationalFunction;
use modern_cas::value::Value;
use num_traits::One;

#[test]
fn canonical_fractions_equal_equivalent_expressions_in_each_order() {
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        let mut ring = PolynomialRing::new(order);
        evaluate("x+y+z", &mut ring).unwrap();
        for (a, b) in [
            ("(x^2-1)/(x-1)", "x+1"),
            ("1/x+1/y", "(x+y)/(x*y)"),
            ("(2*x)/(4*y)", "x/(2*y)"),
            ("0/x", "0"),
            ("x/x", "1"),
            ("(x/y)*(y/z)", "x/z"),
            ("(x/y)/(z/y)", "x/z"),
            ("(x/y)^-2", "y^2/x^2"),
            ("1/x-1/x", "0"),
            ("-(1/x)", "1/(-x)"),
            ("1/(x+y^2)", "2/(2*x+2*y^2)"),
            ("(x+y)/(x+y)^2", "1/(x+y)"),
            ("(1/x)^0", "1"),
            ("2/3/4", "1/6"),
            ("1/2^2", "1/4"),
            ("-x^-2", "-1/x^2"),
        ] {
            let a = evaluate_value(a, &mut ring).unwrap();
            let b = evaluate_value(b, &mut ring).unwrap();
            assert_eq!(a, b);
            assert_eq!(evaluate_value(&a.format(&ring), &mut ring).unwrap(), a);
            if let Value::RationalFunction(f) = a {
                assert!(f.denominator().leading_term().unwrap().coefficient.is_one());
                assert_eq!(
                    ring.format(&ring.gcd(f.numerator(), f.denominator()).unwrap()),
                    "1"
                );
            }
        }
    }
}

#[test]
fn fraction_field_laws() {
    let mut ring = PolynomialRing::default();
    let mut fraction = |n, d| {
        let n = evaluate(n, &mut ring).unwrap();
        let d = evaluate(d, &mut ring).unwrap();
        RationalFunction::new(&ring, n, d).unwrap()
    };
    let a = fraction("x+1", "y-1");
    let b = fraction("y", "x");
    let c = fraction("x-y", "x+y");
    assert_eq!(a.add(&b, &ring).unwrap(), b.add(&a, &ring).unwrap());
    assert_eq!(
        a.multiply(&b.add(&c, &ring).unwrap(), &ring).unwrap(),
        a.multiply(&b, &ring)
            .unwrap()
            .add(&a.multiply(&c, &ring).unwrap(), &ring)
            .unwrap()
    );
    assert_eq!(a.multiply(&b, &ring).unwrap().divide(&b, &ring).unwrap(), a);
    assert!(a.subtract(&a, &ring).unwrap().numerator().is_zero());
}

#[test]
fn undefined_and_nonpolynomial_inputs_do_not_change_the_context() {
    let mut ring = PolynomialRing::default();
    for text in [
        "new/(x-x)",
        "0/0",
        "0^-1",
        "(new,1/0)",
        "[1/new]",
        "gcd(1/new,x)",
        "div(1/new,[x])",
        "groebner([1/new])",
        "new^4294967296",
        "new^1/0",
        "[new]/1",
        "1/(new,)",
    ] {
        assert!(evaluate_value(text, &mut ring).is_err(), "{text}");
        assert!(ring.variables().is_empty());
    }
    assert!(evaluate("1/new", &mut ring).is_err());
    assert!(ring.variables().is_empty());
    assert_eq!(
        ring.format(&evaluate("new/new", &mut ring.clone()).unwrap()),
        "1"
    );
}
