//! Sparse polynomials over the rationals in an explicit ring context.

use crate::Rational;
use num_traits::{One, Signed, Zero};
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Monomial(Vec<u32>);

impl Monomial {
    pub fn new(mut exponents: Vec<u32>) -> Self {
        while exponents.last() == Some(&0) {
            exponents.pop();
        }
        Self(exponents)
    }
    pub fn exponents(&self) -> &[u32] {
        &self.0
    }
    pub fn degree(&self) -> u128 {
        self.0.iter().map(|&n| u128::from(n)).sum()
    }
    fn exponent(&self, index: usize) -> u32 {
        self.0.get(index).copied().unwrap_or(0)
    }
    pub fn multiply(&self, other: &Self) -> Result<Self, &'static str> {
        (0..self.0.len().max(other.0.len()))
            .map(|i| {
                self.exponent(i)
                    .checked_add(other.exponent(i))
                    .ok_or("monomial exponent overflow")
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Self::new)
    }

    /// Return self / divisor when every exponent of the divisor fits.
    pub fn quotient(&self, divisor: &Self) -> Option<Self> {
        (0..self.0.len().max(divisor.0.len()))
            .map(|i| self.exponent(i).checked_sub(divisor.exponent(i)))
            .collect::<Option<Vec<_>>>()
            .map(Self::new)
    }

    pub fn divides(&self, other: &Self) -> bool {
        (0..self.0.len()).all(|i| self.exponent(i) <= other.exponent(i))
    }

    pub fn lcm(&self, other: &Self) -> Self {
        Self::new(
            (0..self.0.len().max(other.0.len()))
                .map(|i| self.exponent(i).max(other.exponent(i)))
                .collect(),
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MonomialOrder {
    #[default]
    Lex,
    GrLex,
    GrevLex,
}

impl MonomialOrder {
    pub fn compare(self, a: &Monomial, b: &Monomial) -> Ordering {
        if self != Self::Lex {
            let degree = a.degree().cmp(&b.degree());
            if degree != Ordering::Equal {
                return degree;
            }
        }
        let indices = 0..a.0.len().max(b.0.len());
        if self == Self::GrevLex {
            indices
                .rev()
                .map(|i| b.exponent(i).cmp(&a.exponent(i)))
                .find(|&c| c != Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        } else {
            indices
                .map(|i| a.exponent(i).cmp(&b.exponent(i)))
                .find(|&c| c != Ordering::Equal)
                .unwrap_or(Ordering::Equal)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Term {
    pub monomial: Monomial,
    pub coefficient: Rational,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Polynomial {
    terms: Vec<Term>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DivisionResult {
    pub quotients: Vec<Polynomial>,
    pub remainder: Polynomial,
}

impl Polynomial {
    pub fn terms(&self) -> &[Term] {
        &self.terms
    }
    pub fn leading_term(&self) -> Option<&Term> {
        self.terms.first()
    }
    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }
    pub fn constant(value: Rational) -> Self {
        if value.is_zero() {
            Self::default()
        } else {
            Self {
                terms: vec![Term {
                    monomial: Monomial::new(vec![]),
                    coefficient: value,
                }],
            }
        }
    }
    pub fn negate(&self) -> Self {
        Self {
            terms: self
                .terms
                .iter()
                .map(|t| Term {
                    monomial: t.monomial.clone(),
                    coefficient: -&t.coefficient,
                })
                .collect(),
        }
    }

    /// Normalize the leading coefficient to one; zero remains zero.
    pub fn monic(&self) -> Self {
        let Some(leading) = self.leading_term() else {
            return self.clone();
        };
        Self {
            terms: self
                .terms
                .iter()
                .map(|term| Term {
                    monomial: term.monomial.clone(),
                    coefficient: &term.coefficient / &leading.coefficient,
                })
                .collect(),
        }
    }
}

/// Polynomials passed to a ring must use its variable registry and ordering.
/// Keep a ring fixed for a session; append variables without reordering existing ones.
#[derive(Clone, Debug, Default)]
pub struct PolynomialRing {
    variables: Vec<String>,
    order: MonomialOrder,
}

impl PolynomialRing {
    pub fn new(order: MonomialOrder) -> Self {
        Self {
            variables: vec![],
            order,
        }
    }
    pub fn order(&self) -> MonomialOrder {
        self.order
    }
    pub fn variables(&self) -> &[String] {
        &self.variables
    }
    pub fn variable(&mut self, name: &str) -> Polynomial {
        let index = self
            .variables
            .iter()
            .position(|v| v == name)
            .unwrap_or_else(|| {
                self.variables.push(name.to_owned());
                self.variables.len() - 1
            });
        let mut exponents = vec![0; index + 1];
        exponents[index] = 1;
        Polynomial {
            terms: vec![Term {
                monomial: Monomial::new(exponents),
                coefficient: Rational::one(),
            }],
        }
    }
    pub fn normalize(&self, mut terms: Vec<Term>) -> Polynomial {
        terms.sort_by(|a, b| self.order.compare(&b.monomial, &a.monomial));
        let mut result: Vec<Term> = Vec::new();
        for term in terms {
            if let Some(last) = result
                .last_mut()
                .filter(|last| last.monomial == term.monomial)
            {
                last.coefficient += term.coefficient;
            } else {
                result.push(term);
            }
        }
        result.retain(|term| !term.coefficient.is_zero());
        Polynomial { terms: result }
    }
    pub fn add(&self, a: &Polynomial, b: &Polynomial) -> Polynomial {
        self.normalize(a.terms.iter().chain(&b.terms).cloned().collect())
    }
    pub fn subtract(&self, a: &Polynomial, b: &Polynomial) -> Polynomial {
        self.add(a, &b.negate())
    }
    pub fn multiply(&self, a: &Polynomial, b: &Polynomial) -> Result<Polynomial, &'static str> {
        let mut terms = Vec::new();
        for left in &a.terms {
            for right in &b.terms {
                terms.push(Term {
                    monomial: left.monomial.multiply(&right.monomial)?,
                    coefficient: &left.coefficient * &right.coefficient,
                });
            }
        }
        Ok(self.normalize(terms))
    }
    pub fn power(&self, value: &Polynomial, mut exponent: u32) -> Result<Polynomial, &'static str> {
        let mut result = Polynomial::constant(Rational::one());
        let mut base = value.clone();
        while exponent > 0 {
            if exponent % 2 == 1 {
                result = self.multiply(&result, &base)?;
            }
            exponent /= 2;
            if exponent > 0 {
                base = self.multiply(&base, &base)?;
            }
        }
        Ok(result)
    }

    /// Divide by ordered, nonzero divisors using the first applicable leading term.
    /// No remainder monomial is divisible by any divisor's leading monomial.
    pub fn divide(
        &self,
        dividend: &Polynomial,
        divisors: &[Polynomial],
    ) -> Result<DivisionResult, &'static str> {
        if divisors.iter().any(Polynomial::is_zero) {
            return Err("division by a zero polynomial");
        }
        let mut pending = dividend.clone();
        let mut quotients = vec![Polynomial::default(); divisors.len()];
        let mut remainder = Polynomial::default();
        while let Some(leading) = pending.leading_term().cloned() {
            let mut reduced = false;
            for (index, divisor) in divisors.iter().enumerate() {
                let divisor_leading = divisor.leading_term().expect("zero divisors were rejected");
                if let Some(monomial) = leading.monomial.quotient(&divisor_leading.monomial) {
                    let factor = Polynomial {
                        terms: vec![Term {
                            monomial,
                            coefficient: &leading.coefficient / &divisor_leading.coefficient,
                        }],
                    };
                    let multiple = self.multiply(&factor, divisor)?;
                    quotients[index] = self.add(&quotients[index], &factor);
                    pending = self.subtract(&pending, &multiple);
                    reduced = true;
                    break;
                }
            }
            if !reduced {
                remainder.terms.push(leading);
                pending.terms.remove(0);
            }
        }
        Ok(DivisionResult {
            quotients,
            remainder,
        })
    }
    pub fn format(&self, value: &Polynomial) -> String {
        let mut result = String::new();
        for term in &value.terms {
            let negative = term.coefficient.is_negative();
            if result.is_empty() {
                if negative {
                    result.push('-');
                }
            } else {
                result.push_str(if negative { " - " } else { " + " });
            }
            let coefficient = term.coefficient.abs();
            let mut factors = Vec::new();
            if !coefficient.is_one() || term.monomial.0.is_empty() {
                factors.push(coefficient.to_string());
            }
            for (index, &exponent) in term.monomial.0.iter().enumerate() {
                if exponent != 0 {
                    let name = &self.variables[index];
                    factors.push(if exponent == 1 {
                        name.clone()
                    } else {
                        format!("{name}^{exponent}")
                    });
                }
            }
            result.push_str(&factors.join("*"));
        }
        if result.is_empty() {
            "0".into()
        } else {
            result
        }
    }
}
