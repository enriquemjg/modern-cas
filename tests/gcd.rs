use modern_cas::parser::evaluate;
use modern_cas::polynomial::{MonomialOrder, PolynomialRing};
use num_traits::One;

#[test]
fn known_gcds_and_divisibility_under_every_order() {
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        let mut ring = PolynomialRing::new(order);
        evaluate("x+y+z", &mut ring).unwrap();
        for (a, b, expected) in [
            ("x^3-x", "x^2-1", "x^2 - 1"),
            ("(x-1)^3*(x+2)", "(x-1)^2*(x+3)", "x^2 - 2*x + 1"),
            ("1/2*y^2-1/2", "-3/4*y+3/4", "y - 1"),
            ("z^2+1", "z+1", "1"),
            ("0", "-2*z+4", "z - 2"),
            ("-2*z+4", "0", "z - 2"),
            ("0", "0", "0"),
            ("2/3", "4/5", "1"),
            ("x^2", "2", "1"),
        ] {
            let a = evaluate(a, &mut ring).unwrap();
            let b = evaluate(b, &mut ring).unwrap();
            let gcd = ring.gcd(&a, &b).unwrap();
            assert_eq!(ring.format(&gcd), expected);
            assert_eq!(gcd, ring.gcd(&b, &a).unwrap());
            if !gcd.is_zero() {
                assert!(gcd.leading_term().unwrap().coefficient.is_one());
                for value in [&a, &b] {
                    let q = ring.exact_quotient(value, &gcd).unwrap();
                    assert_eq!(ring.multiply(&q, &gcd).unwrap(), *value);
                }
            }
        }
    }
}

#[test]
fn exact_quotient_accepts_multivariate_inputs_and_rejects_remainders() {
    let mut ring = PolynomialRing::default();
    let f = evaluate("(x+y)*(x-y)", &mut ring).unwrap();
    let g = evaluate("x+y", &mut ring).unwrap();
    assert_eq!(ring.format(&ring.exact_quotient(&f, &g).unwrap()), "x - y");
    assert!(ring.exact_quotient(&g, &f).is_err());
    let zero = evaluate("0", &mut ring).unwrap();
    assert!(ring.exact_quotient(&zero, &zero).is_err());
    assert!(ring.exact_quotient(&zero, &g).unwrap().is_zero());
}

#[test]
fn invalid_gcd_calls_preserve_the_session() {
    let mut ring = PolynomialRing::default();
    for input in ["gcd([new],1)", "gcd(new,(1,))", "gcd(new)", "gcd(new,1,2)"] {
        assert!(evaluate(input, &mut ring).is_err(), "{input}");
        assert!(ring.variables().is_empty());
    }
}

#[test]
fn multivariate_gcd_matches_known_factors_under_all_orders() {
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        let mut ring = PolynomialRing::new(order);
        evaluate("x+y+z+w", &mut ring).unwrap();
        for (a, b, expected) in [
            ("(x+y)*(x+1)", "(x+y)*(y+1)", "x+y"),
            ("y*(x+1)", "y^2*(x+2)", "y"),
            ("(y+z)*(x+y)", "(y+z)*(x+z)", "y+z"),
            ("(x+y)^3*(z+1)", "(x+y)^2*(z+2)", "(x+y)^2"),
            ("2/3*(x+y)*(y*x+1)", "-5/7*(x+y)*(y*x+2)", "x+y"),
            ("x", "y", "1"),
            ("x+y", "0", "x+y"),
            ("x+y", "2", "1"),
            ("x*y", "x*z", "x"),
            ("(z+w)*(z+1)", "(z+w)*(w+1)", "z+w"),
            ("(x*y+z)*(x*z+y)", "(x*y+z)*(y*z+x)", "x*y+z"),
        ] {
            let a = evaluate(a, &mut ring).unwrap();
            let b = evaluate(b, &mut ring).unwrap();
            let expected = evaluate(expected, &mut ring).unwrap().monic();
            let gcd = ring.gcd(&a, &b).unwrap();
            assert_eq!(gcd, expected);
            assert_eq!(ring.gcd(&b, &a).unwrap(), gcd);
            for f in [&a, &b] {
                let q = ring.exact_quotient(f, &gcd).unwrap();
                assert_eq!(ring.multiply(&q, &gcd).unwrap(), *f);
            }
            let qa = ring.exact_quotient(&a, &gcd).unwrap();
            let qb = ring.exact_quotient(&b, &gcd).unwrap();
            assert_eq!(ring.format(&ring.gcd(&qa, &qb).unwrap()), "1");
        }
    }
}
