//! Interpreter values and rectangular polynomial matrices.

use crate::polynomial::{Polynomial, PolynomialRing};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Polynomial(Polynomial),
    Tuple(Vec<Value>),
    Matrix(Matrix),
}

/// A rectangular array stored in row-major order. Vectors are single-row or
/// single-column matrices; the only empty shape is 0 by 0.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix {
    rows: usize,
    columns: usize,
    entries: Vec<Polynomial>,
}

impl Matrix {
    pub fn new(
        rows: usize,
        columns: usize,
        entries: Vec<Polynomial>,
    ) -> Result<Self, &'static str> {
        if (rows == 0) != (columns == 0) || rows.checked_mul(columns) != Some(entries.len()) {
            return Err("matrix dimensions do not match its entries");
        }
        Ok(Self {
            rows,
            columns,
            entries,
        })
    }
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn columns(&self) -> usize {
        self.columns
    }
    pub fn entries(&self) -> &[Polynomial] {
        &self.entries
    }
}

impl Value {
    pub fn format(&self, ring: &PolynomialRing) -> String {
        match self {
            Self::Polynomial(value) => ring.format(value),
            Self::Tuple(values) => {
                let mut body = values
                    .iter()
                    .map(|v| v.format(ring))
                    .collect::<Vec<_>>()
                    .join(", ");
                if values.len() == 1 {
                    body.push(',');
                }
                format!("({body})")
            }
            Self::Matrix(matrix) => {
                if matrix.rows == 0 {
                    return "[]".into();
                }
                let rows = matrix
                    .entries
                    .chunks(matrix.columns)
                    .map(|row| {
                        row.iter()
                            .map(|p| ring.format(p))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("[{rows}]")
            }
        }
    }
}
