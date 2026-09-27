use modern_cas::parser::{evaluate, evaluate_value};
use modern_cas::polynomial::{Monomial, MonomialOrder, PolynomialRing};

#[test]
fn monomial_division_handles_missing_exponents_and_constants() {
    let a = Monomial::new(vec![3, 2]);
    let b = Monomial::new(vec![1]);
    assert!(b.divides(&a));
    assert_eq!(a.quotient(&b), Some(Monomial::new(vec![2, 2])));
    assert!(!a.divides(&b));
    assert_eq!(b.quotient(&a), None);
    assert_eq!(a.quotient(&Monomial::new(vec![])), Some(a.clone()));
    assert_eq!(a.quotient(&a), Some(Monomial::new(vec![])));
    assert_eq!(a.quotient(&Monomial::new(vec![0, 0, 1])), None);
}

#[test]
fn division_reconstructs_dividends_and_fully_reduces_remainders() {
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        let mut ring = PolynomialRing::new(order);
        evaluate("x+y+z", &mut ring).unwrap();
        for source in [
            "x*y^2+1",
            "x^2*y+x*y^2+y^2",
            "x^3+y^3+z",
            "1/2*x^2-2/3*y+1",
            "0",
            "7",
        ] {
            let f = evaluate(source, &mut ring).unwrap();
            for generators in [
                vec!["x*y-1", "y^2-1"],
                vec!["y^2-1", "x*y-1"],
                vec!["2*x-y^2", "3*y-z"],
                vec!["2/3"],
                vec!["x", "x"],
                vec![],
            ] {
                let divisors: Vec<_> = generators
                    .iter()
                    .map(|g| evaluate(g, &mut ring).unwrap())
                    .collect();
                let result = ring.divide(&f, &divisors).unwrap();
                assert_eq!(result.quotients.len(), divisors.len());
                let mut reconstructed = result.remainder.clone();
                for (q, g) in result.quotients.iter().zip(&divisors) {
                    reconstructed = ring.add(&reconstructed, &ring.multiply(q, g).unwrap());
                    for term in result.remainder.terms() {
                        assert!(!g.leading_term().unwrap().monomial.divides(&term.monomial));
                    }
                }
                assert_eq!(reconstructed, f);
                assert_eq!(
                    ring.normalize(result.remainder.terms().to_vec()),
                    result.remainder
                );
            }
        }
    }
}

#[test]
fn division_errors_are_transactional_in_both_apis() {
    let mut ring = PolynomialRing::default();
    let zero = evaluate("0", &mut ring).unwrap();
    assert!(ring.divide(&zero, std::slice::from_ref(&zero)).is_err());
    for source in [
        "div(new, [0])",
        "div(new, [x-x])",
        "div(new, [x,1;2,3])",
        "div(new, (x,))",
        "div([new], [x])",
        "div(new)",
        "other(new)",
        "div(new,[x],)",
        "div(new,[x])+1",
        "div(x*y^4294967295, [x-y])",
    ] {
        assert!(evaluate_value(source, &mut ring).is_err(), "{source}");
        assert!(ring.variables().is_empty(), "{source}");
    }
    assert!(evaluate("div(new, [1])", &mut ring).is_err());
    assert!(ring.variables().is_empty());
}
