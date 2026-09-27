//! Direct Buchberger completion followed by minimalization and interreduction.

use std::collections::VecDeque;

use crate::polynomial::{Polynomial, PolynomialRing, Term};

impl PolynomialRing {
    /// Cancel the leading terms of two nonzero polynomials.
    pub fn s_polynomial(&self, a: &Polynomial, b: &Polynomial) -> Result<Polynomial, &'static str> {
        let left = a
            .leading_term()
            .ok_or("S-polynomial requires nonzero polynomials")?;
        let right = b
            .leading_term()
            .ok_or("S-polynomial requires nonzero polynomials")?;
        let lcm = left.monomial.lcm(&right.monomial);
        let factor = |term: &Term| {
            self.normalize(vec![Term {
                monomial: lcm
                    .quotient(&term.monomial)
                    .expect("LCM is divisible by each monomial"),
                coefficient: term.coefficient.recip(),
            }])
        };
        Ok(self.subtract(
            &self.multiply(&factor(left), a)?,
            &self.multiply(&factor(right), b)?,
        ))
    }

    /// Complete the generators to a monic Gröbner basis using every critical pair.
    /// Zero generators are ignored. The resulting basis is not necessarily reduced.
    pub fn buchberger(&self, generators: &[Polynomial]) -> Result<Vec<Polynomial>, &'static str> {
        let mut basis = Vec::new();
        for generator in generators.iter().filter(|g| !g.is_zero()) {
            let generator = generator.monic();
            if generator.leading_term().unwrap().monomial.degree() == 0 {
                return Ok(vec![generator]);
            }
            if !basis.contains(&generator) {
                basis.push(generator);
            }
        }
        let mut pairs = VecDeque::new();
        for j in 0..basis.len() {
            for i in 0..j {
                pairs.push_back((i, j));
            }
        }
        while let Some((i, j)) = pairs.pop_front() {
            let s = self.s_polynomial(&basis[i], &basis[j])?;
            let remainder = self.divide(&s, &basis)?.remainder;
            if remainder.is_zero() {
                continue;
            }
            let remainder = remainder.monic();
            if remainder.leading_term().unwrap().monomial.degree() == 0 {
                return Ok(vec![remainder]);
            }
            let next = basis.len();
            for i in 0..next {
                pairs.push_back((i, next));
            }
            basis.push(remainder);
        }
        Ok(basis)
    }

    /// Compute the unique reduced basis, sorted by descending leading monomial.
    /// The zero ideal is represented by an empty basis; the unit ideal by [1].
    pub fn groebner_basis(
        &self,
        generators: &[Polynomial],
    ) -> Result<Vec<Polynomial>, &'static str> {
        let basis = self.buchberger(generators)?;
        // Keep one representative when leading monomials coincide; otherwise remove
        // every leading monomial divisible by another. This preserves a Gröbner basis.
        let minimal: Vec<_> = basis
            .iter()
            .enumerate()
            .filter(|(i, f)| {
                let lm = &f.leading_term().unwrap().monomial;
                !basis.iter().enumerate().any(|(j, g)| {
                    let other = &g.leading_term().unwrap().monomial;
                    j != *i && other.divides(lm) && (other != lm || j < *i)
                })
            })
            .map(|(_, f)| f.clone())
            .collect();
        let mut reduced = Vec::new();
        for (i, polynomial) in minimal.iter().enumerate() {
            let others: Vec<_> = minimal
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, g)| g.clone())
                .collect();
            reduced.push(self.divide(polynomial, &others)?.remainder.monic());
        }
        reduced.sort_by(|a, b| {
            self.order().compare(
                &b.leading_term().unwrap().monomial,
                &a.leading_term().unwrap().monomial,
            )
        });
        Ok(reduced)
    }
}
