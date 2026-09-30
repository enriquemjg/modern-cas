//! Recursive primitive remainder sequences over polynomial coefficient rings.

use crate::Rational;
use crate::polynomial::{Monomial, Polynomial, PolynomialRing, Term};
use num_traits::One;
use std::collections::BTreeMap;

fn exponent(term: &Term, variable: usize) -> u32 {
    term.monomial
        .exponents()
        .get(variable)
        .copied()
        .unwrap_or(0)
}

impl PolynomialRing {
    /// Decompose sparsely in one variable; coefficients contain no such variable.
    fn coefficients(&self, value: &Polynomial, variable: usize) -> BTreeMap<u32, Polynomial> {
        let mut groups: BTreeMap<u32, Vec<Term>> = BTreeMap::new();
        for term in value.terms() {
            let degree = exponent(term, variable);
            let mut powers = term.monomial.exponents().to_vec();
            if variable < powers.len() {
                powers[variable] = 0;
            }
            groups.entry(degree).or_default().push(Term {
                monomial: Monomial::new(powers),
                coefficient: term.coefficient.clone(),
            });
        }
        groups
            .into_iter()
            .map(|(degree, terms)| (degree, self.normalize(terms)))
            .collect()
    }

    fn content(&self, value: &Polynomial, variable: usize) -> Result<Polynomial, &'static str> {
        let mut content = Polynomial::default();
        for coefficient in self.coefficients(value, variable).values() {
            content = self.gcd(&content, coefficient)?;
            if content == Polynomial::constant(Rational::one()) {
                break;
            }
        }
        Ok(content)
    }

    fn primitive_part(
        &self,
        value: &Polynomial,
        variable: usize,
    ) -> Result<Polynomial, &'static str> {
        if value.is_zero() {
            return Ok(value.clone());
        }
        self.exact_quotient(value, &self.content(value, variable)?)
    }

    /// Each step multiplies by the divisor's leading coefficient before cancelling.
    /// Degrees are in the selected variable, independently of the monomial order.
    fn pseudo_remainder(
        &self,
        a: &Polynomial,
        b: &Polynomial,
        variable: usize,
    ) -> Result<Polynomial, &'static str> {
        let coefficients = self.coefficients(b, variable);
        let (&degree_b, leading_b) = coefficients.last_key_value().expect("nonzero divisor");
        let mut remainder = a.clone();
        while !remainder.is_zero() {
            let coefficients = self.coefficients(&remainder, variable);
            let (&degree_r, leading_r) = coefficients.last_key_value().unwrap();
            if degree_r < degree_b {
                break;
            }
            let mut powers = vec![0; variable + 1];
            powers[variable] = degree_r - degree_b;
            let shift = self.normalize(vec![Term {
                monomial: Monomial::new(powers),
                coefficient: Rational::one(),
            }]);
            let multiple = self.multiply(&self.multiply(leading_r, &shift)?, b)?;
            remainder = self.subtract(&self.multiply(leading_b, &remainder)?, &multiple);
        }
        Ok(remainder)
    }

    /// Monic GCD over Q[x1,...,xn], with gcd(0,0) = 0.
    pub fn gcd(&self, a: &Polynomial, b: &Polynomial) -> Result<Polynomial, &'static str> {
        if a.is_zero() {
            return Ok(b.monic());
        }
        if b.is_zero() {
            return Ok(a.monic());
        }
        let mut variables = std::collections::BTreeSet::new();
        for term in a.terms().iter().chain(b.terms()) {
            for (i, &power) in term.monomial.exponents().iter().enumerate() {
                if power != 0 {
                    variables.insert(i);
                }
            }
        }
        if variables.len() <= 1 {
            let (mut a, mut b) = (a.monic(), b.monic());
            while !b.is_zero() {
                let remainder = self.divide(&a, std::slice::from_ref(&b))?.remainder.monic();
                a = b;
                b = remainder;
            }
            return Ok(a);
        }
        let variable = *variables.first().unwrap();
        let content_a = self.content(a, variable)?;
        let content_b = self.content(b, variable)?;
        let content = self.gcd(&content_a, &content_b)?;
        let mut a = self.exact_quotient(a, &content_a)?;
        let mut b = self.exact_quotient(b, &content_b)?;
        while !b.is_zero() {
            let remainder = self.pseudo_remainder(&a, &b, variable)?;
            a = b;
            b = self.primitive_part(&remainder, variable)?.monic();
        }
        Ok(self.multiply(&content, &a)?.monic())
    }
}
