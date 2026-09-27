use modern_cas::polynomial::{Monomial, MonomialOrder, Polynomial, PolynomialRing, Term};
use modern_cas::{Rational, parser};
use std::cmp::Ordering;

#[test]
fn monomials_are_canonical_and_arithmetic_is_checked() {
    assert_eq!(Monomial::new(vec![2, 1, 0]), Monomial::new(vec![2, 1]));
    assert_eq!(Monomial::new(vec![0, 0]).exponents(), &[]);
    let a = Monomial::new(vec![2, 1]);
    let b = Monomial::new(vec![0, 2, 3]);
    assert_eq!(a.multiply(&b).unwrap(), Monomial::new(vec![2, 3, 3]));
    let large = Monomial::new(vec![u32::MAX, u32::MAX]);
    assert_eq!(large.degree(), 2 * u128::from(u32::MAX));
    assert!(large.multiply(&a).is_err());
}

#[test]
fn orders_are_total_and_preserved_by_multiplication() {
    let monomials: Vec<_> = (0..3)
        .flat_map(|x| (0..3).flat_map(move |y| (0..3).map(move |z| Monomial::new(vec![x, y, z]))))
        .collect();
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        let factor = Monomial::new(vec![2, 1, 3, 4]);
        for a in &monomials {
            for b in &monomials {
                let comparison = order.compare(a, b);
                assert_eq!(comparison == Ordering::Equal, a == b);
                assert_eq!(comparison, order.compare(b, a).reverse());
                assert_eq!(
                    comparison,
                    order.compare(&a.multiply(&factor).unwrap(), &b.multiply(&factor).unwrap())
                );
                for c in &monomials {
                    if comparison.is_le() && order.compare(b, c).is_le() {
                        assert!(order.compare(a, c).is_le());
                    }
                }
            }
        }
    }
}

#[test]
fn normalization_removes_cancellation_and_zero_has_no_leader() {
    let mut ring = PolynomialRing::default();
    let x = ring.variable("x");
    let term = x.leading_term().unwrap().clone();
    let negative = Term {
        monomial: term.monomial.clone(),
        coefficient: -term.coefficient.clone(),
    };
    assert!(ring.normalize(vec![term, negative]).is_zero());
    assert!(Polynomial::default().leading_term().is_none());
    assert!(Polynomial::constant(Rational::from_integer(0.into())).is_zero());
}

#[test]
fn ring_laws_and_existing_values_survive_new_variables() {
    for order in [
        MonomialOrder::Lex,
        MonomialOrder::GrLex,
        MonomialOrder::GrevLex,
    ] {
        let mut ring = PolynomialRing::new(order);
        let a = parser::evaluate("x+1/2", &mut ring).unwrap();
        let before = ring.format(&a);
        let b = parser::evaluate("y-x", &mut ring).unwrap();
        let c = parser::evaluate("z+2", &mut ring).unwrap();
        assert_eq!(ring.format(&a), before);
        assert_eq!(ring.add(&a, &b), ring.add(&b, &a));
        assert_eq!(
            ring.multiply(&a, &b).unwrap(),
            ring.multiply(&b, &a).unwrap()
        );
        assert_eq!(
            ring.multiply(&a, &ring.add(&b, &c)).unwrap(),
            ring.add(
                &ring.multiply(&a, &b).unwrap(),
                &ring.multiply(&a, &c).unwrap()
            )
        );
        assert!(ring.subtract(&a, &a).is_zero());
        assert_eq!(
            ring.multiply(&a, &Polynomial::constant(Rational::from_integer(1.into())))
                .unwrap(),
            a
        );
        assert!(ring.multiply(&a, &Polynomial::default()).unwrap().is_zero());
        let saved = ring.variables().to_vec();
        assert!(parser::evaluate("new^4294967295*new", &mut ring).is_err());
        assert_eq!(ring.variables(), saved);
    }
}
