use modern_cas::parser::{evaluate, evaluate_value};
use modern_cas::polynomial::{Monomial, MonomialOrder, Polynomial, PolynomialRing};
use num_traits::One;

fn check_basis(ring: &PolynomialRing, basis: &[Polynomial]) {
    for (i, f) in basis.iter().enumerate() {
        assert!(f.leading_term().unwrap().coefficient.is_one());
        for g in &basis[..i] {
            let s = ring.s_polynomial(f, g).unwrap();
            assert!(ring.divide(&s, basis).unwrap().remainder.is_zero());
        }
    }
}

#[test]
fn lcm_and_s_polynomials_cancel_leading_terms() {
    let a = Monomial::new(vec![3, 1]);
    let b = Monomial::new(vec![1, 2, 4]);
    assert_eq!(a.lcm(&b), Monomial::new(vec![3, 2, 4]));
    assert_eq!(a.lcm(&Monomial::new(vec![])), a);
    let mut ring = PolynomialRing::default();
    let f = evaluate("2*x^2-y", &mut ring).unwrap();
    let g = evaluate("3*x*y-1", &mut ring).unwrap();
    assert_eq!(
        ring.format(&ring.s_polynomial(&f, &g).unwrap()),
        "1/3*x - 1/2*y^2"
    );
    assert!(ring.s_polynomial(&f, &f).unwrap().is_zero());
    assert!(ring.s_polynomial(&f, &Polynomial::default()).is_err());
    assert!(Polynomial::default().monic().is_zero());
}

#[test]
fn reduced_bases_satisfy_buchberger_and_reducedness_under_all_orders() {
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        for sources in [
            vec!["x*y-1", "y^2-x"],
            vec!["x^2-y", "x*y-1"],
            vec!["x*y-z", "y^2-x", "z^2-y"],
            vec!["x+y", "x-y"],
            vec!["x^2", "x", "2*x", "0"],
            vec!["x", "x+1"],
            vec!["0"],
            vec![],
            vec!["2/3*x^2-4/5*y"],
        ] {
            let mut ring = PolynomialRing::new(order);
            evaluate("x+y+z", &mut ring).unwrap();
            let generators: Vec<_> = sources
                .iter()
                .map(|s| evaluate(s, &mut ring).unwrap())
                .collect();
            let raw = ring.buchberger(&generators).unwrap();
            check_basis(&ring, &raw);
            let basis = ring.groebner_basis(&generators).unwrap();
            check_basis(&ring, &basis);
            for g in &generators {
                assert!(ring.divide(g, &basis).unwrap().remainder.is_zero());
            }
            for (i, f) in basis.iter().enumerate() {
                for (j, g) in basis.iter().enumerate() {
                    if i != j {
                        for term in f.terms() {
                            assert!(!g.leading_term().unwrap().monomial.divides(&term.monomial));
                        }
                    }
                }
            }
            let mut reversed = generators.clone();
            reversed.reverse();
            assert_eq!(ring.groebner_basis(&reversed).unwrap(), basis);
            assert_eq!(ring.groebner_basis(&basis).unwrap(), basis);
        }
    }
}

#[test]
fn known_bases_and_degenerate_ideals() {
    let mut ring = PolynomialRing::default();
    for (source, expected) in [
        ("groebner([x*y-1,y^2-x])", "[x - y^2, y^3 - 1]"),
        ("groebner([x+y,x-y])", "[x, y]"),
        ("groebner([x,x+1])", "[1]"),
        ("groebner([0,2/3, x])", "[1]"),
        ("groebner([x^2,x,2*x,0])", "[x]"),
        ("groebner([0,0])", "[]"),
        ("groebner([])", "[]"),
    ] {
        assert_eq!(
            evaluate_value(source, &mut ring).unwrap().format(&ring),
            expected
        );
    }
}

#[test]
fn errors_leave_both_apis_transactional() {
    let mut ring = PolynomialRing::default();
    for source in [
        "groebner(new)",
        "groebner([new,1;2,3])",
        "groebner([new],[])",
        "groebner()",
        "groebner([new])+1",
        "groebner([x*y^4294967295-1,x-y])",
    ] {
        assert!(evaluate_value(source, &mut ring).is_err(), "{source}");
        assert!(ring.variables().is_empty());
    }
    assert!(evaluate("groebner([new])", &mut ring).is_err());
    assert!(ring.variables().is_empty());
}
