//! Exact fraction-field arithmetic, normalized for a fixed polynomial ring.

use crate::Rational;
use crate::polynomial::{Polynomial, PolynomialRing};
use num_traits::One;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RationalFunction {
    numerator: Polynomial,
    denominator: Polynomial,
}

fn one() -> Polynomial {
    Polynomial::constant(Rational::one())
}

impl RationalFunction {
    /// Cancel common factors and make the denominator monic. Equality is equality
    /// in the fraction field; excluded points from input expressions are not stored.
    pub fn new(
        ring: &PolynomialRing,
        numerator: Polynomial,
        denominator: Polynomial,
    ) -> Result<Self, &'static str> {
        if denominator.is_zero() {
            return Err("denominator cannot be zero");
        }
        if numerator.is_zero() {
            return Ok(Self::from_polynomial(numerator));
        }
        if denominator == one() {
            return Ok(Self::from_polynomial(numerator));
        }
        let gcd = ring.gcd(&numerator, &denominator)?;
        let numerator = ring.exact_quotient(&numerator, &gcd)?;
        let denominator = ring.exact_quotient(&denominator, &gcd)?;
        let scale = Polynomial::constant(denominator.leading_term().unwrap().coefficient.recip());
        Ok(Self {
            numerator: ring.multiply(&numerator, &scale)?,
            denominator: denominator.monic(),
        })
    }
    pub fn from_polynomial(numerator: Polynomial) -> Self {
        Self {
            numerator,
            denominator: one(),
        }
    }
    pub fn numerator(&self) -> &Polynomial {
        &self.numerator
    }
    pub fn denominator(&self) -> &Polynomial {
        &self.denominator
    }
    pub fn is_polynomial(&self) -> bool {
        self.denominator == one()
    }
    pub fn negate(&self) -> Self {
        Self {
            numerator: self.numerator.negate(),
            denominator: self.denominator.clone(),
        }
    }
    pub fn add(&self, other: &Self, ring: &PolynomialRing) -> Result<Self, &'static str> {
        Self::new(
            ring,
            ring.add(
                &ring.multiply(&self.numerator, &other.denominator)?,
                &ring.multiply(&other.numerator, &self.denominator)?,
            ),
            ring.multiply(&self.denominator, &other.denominator)?,
        )
    }
    pub fn subtract(&self, other: &Self, ring: &PolynomialRing) -> Result<Self, &'static str> {
        self.add(&other.negate(), ring)
    }
    pub fn multiply(&self, other: &Self, ring: &PolynomialRing) -> Result<Self, &'static str> {
        Self::new(
            ring,
            ring.multiply(&self.numerator, &other.numerator)?,
            ring.multiply(&self.denominator, &other.denominator)?,
        )
    }
    pub fn divide(&self, other: &Self, ring: &PolynomialRing) -> Result<Self, &'static str> {
        if other.numerator.is_zero() {
            return Err("denominator cannot be zero");
        }
        Self::new(
            ring,
            ring.multiply(&self.numerator, &other.denominator)?,
            ring.multiply(&self.denominator, &other.numerator)?,
        )
    }
    /// The magnitude fits u32, retaining the polynomial exponent range.
    pub fn power(
        &self,
        magnitude: u32,
        negative: bool,
        ring: &PolynomialRing,
    ) -> Result<Self, &'static str> {
        if negative && magnitude != 0 && self.numerator.is_zero() {
            return Err("zero cannot have a negative power");
        }
        let (n, d) = if negative {
            (&self.denominator, &self.numerator)
        } else {
            (&self.numerator, &self.denominator)
        };
        Self::new(ring, ring.power(n, magnitude)?, ring.power(d, magnitude)?)
    }
    pub fn format(&self, ring: &PolynomialRing) -> String {
        if self.is_polynomial() {
            ring.format(&self.numerator)
        } else {
            format!(
                "({})/({})",
                ring.format(&self.numerator),
                ring.format(&self.denominator)
            )
        }
    }
}
